use core::ops::Drop;

use crate::forest::Forest;
use crate::geometry::Size;
use crate::id::{Allocator, Id, NodeId};
use crate::number::Number;
use crate::result::Layout;
use crate::style::Style;
use crate::sys::{new_map_with_capacity, Box, ChildrenVec, Map, Vec};
use crate::Error;

pub enum MeasureFunc {
    Raw(fn(Size<Number>) -> Size<f32>),
    #[cfg(any(feature = "std", feature = "alloc"))]
    Boxed(Box<dyn Fn(Size<Number>) -> Size<f32>>),
}

/// Global stretch instance id allocator.
static INSTANCE_ALLOCATOR: Allocator = Allocator::new();

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(not(any(feature = "std", feature = "alloc")), derive(hash32_derive::Hash32))]
pub struct Node {
    instance: Id,
    local: Id,
}

pub struct Stretch {
    id: Id,
    nodes: Allocator,
    nodes_to_ids: Map<Node, NodeId>,
    ids_to_nodes: Map<NodeId, Node>,
    forest: Forest,
}

impl Default for Stretch {
    fn default() -> Self {
        Self::with_capacity(16)
    }
}

impl Stretch {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            id: INSTANCE_ALLOCATOR.allocate(),
            nodes: Allocator::new(),
            nodes_to_ids: new_map_with_capacity(capacity),
            ids_to_nodes: new_map_with_capacity(capacity),
            forest: Forest::with_capacity(capacity),
        }
    }

    fn allocate_node(&mut self) -> Node {
        let local = self.nodes.allocate();
        Node { instance: self.id, local }
    }

    fn add_node(&mut self, node: Node, id: NodeId) {
        let _ = self.nodes_to_ids.insert(node, id);
        let _ = self.ids_to_nodes.insert(id, node);
    }

    // Find node in the forest.
    fn find_node(&self, node: Node) -> Result<NodeId, Error> {
        match self.nodes_to_ids.get(&node) {
            Some(id) => Ok(*id),
            None => Err(Error::InvalidNode(node)),
        }
    }

    pub fn new_leaf(&mut self, style: Style, measure: MeasureFunc) -> Result<Node, Error> {
        let node = self.allocate_node();
        let id = self.forest.new_leaf(style, measure);
        self.add_node(node, id);
        Ok(node)
    }

    pub fn new_node(&mut self, style: Style, children: &[Node]) -> Result<Node, Error> {
        let node = self.allocate_node();
        let children =
            children.iter().map(|child| self.find_node(*child)).collect::<Result<ChildrenVec<_>, Error>>()?;
        let id = self.forest.new_node(style, children);
        self.add_node(node, id);
        Ok(node)
    }

    /// Removes all nodes.
    ///
    /// All associated nodes will be invalid.
    pub fn clear(&mut self) {
        for node in self.nodes_to_ids.keys() {
            self.nodes.free(&[node.local]);
        }
        self.nodes_to_ids.clear();
        self.ids_to_nodes.clear();
        self.forest.clear();
    }

    /// Remove nodes.
    pub fn remove(&mut self, node: Node) {
        let id = if let Ok(id) = self.find_node(node) { id } else { return };

        self.nodes_to_ids.remove(&node);
        self.ids_to_nodes.remove(&id);

        if let Some(new_id) = self.forest.swap_remove(id) {
            let new = self.ids_to_nodes.remove(&new_id).unwrap();
            let _ = self.nodes_to_ids.insert(new, id);
            let _ = self.ids_to_nodes.insert(id, new);
        }
    }

    pub fn set_measure(&mut self, node: Node, measure: Option<MeasureFunc>) -> Result<(), Error> {
        let id = self.find_node(node)?;
        self.forest.nodes[id].measure = measure;
        self.forest.mark_dirty(id);
        Ok(())
    }

    pub fn add_child(&mut self, node: Node, child: Node) -> Result<(), Error> {
        let node_id = self.find_node(node)?;
        let child_id = self.find_node(child)?;

        self.forest.add_child(node_id, child_id);
        Ok(())
    }

    pub fn set_children(&mut self, node: Node, children: &[Node]) -> Result<(), Error> {
        let node_id = self.find_node(node)?;
        let children_id = children.iter().map(|child| self.find_node(*child)).collect::<Result<ChildrenVec<_>, _>>()?;

        // Remove node as parent from all its current children.
        for child in &self.forest.children[node_id] {
            self.forest.parents[*child].retain(|p| *p != node_id);
        }

        // Build up relation node <-> child
        for child in &children_id {
            self.forest.parents[*child].push(node_id);
        }
        self.forest.children[node_id] = children_id;

        self.forest.mark_dirty(node_id);
        Ok(())
    }

    pub fn remove_child(&mut self, node: Node, child: Node) -> Result<Node, Error> {
        let node_id = self.find_node(node)?;
        let child_id = self.find_node(child)?;

        let prev_id = self.forest.remove_child(node_id, child_id);
        Ok(self.ids_to_nodes[&prev_id])
    }

    pub fn remove_child_at_index(&mut self, node: Node, index: usize) -> Result<Node, Error> {
        let node_id = self.find_node(node)?;
        // TODO: index check

        let prev_id = self.forest.remove_child_at_index(node_id, index);
        Ok(self.ids_to_nodes[&prev_id])
    }

    pub fn replace_child_at_index(&mut self, node: Node, index: usize, child: Node) -> Result<Node, Error> {
        let node_id = self.find_node(node)?;
        let child_id = self.find_node(child)?;
        // TODO: index check

        self.forest.parents[child_id].push(node_id);
        let old_child = core::mem::replace(&mut self.forest.children[node_id][index], child_id);
        self.forest.parents[old_child].retain(|p| *p != node_id);

        self.forest.mark_dirty(node_id);

        Ok(self.ids_to_nodes[&old_child])
    }

    pub fn children(&self, node: Node) -> Result<Vec<Node>, Error> {
        let id = self.find_node(node)?;
        Ok(self.forest.children[id].iter().map(|child| self.ids_to_nodes[child]).collect())
    }

    pub fn child_at_index(&self, node: Node, index: usize) -> Result<Node, Error> {
        let id = self.find_node(node)?;
        Ok(self.ids_to_nodes[&self.forest.children[id][index]])
    }

    pub fn child_count(&self, node: Node) -> Result<usize, Error> {
        let id = self.find_node(node)?;
        Ok(self.forest.children[id].len())
    }

    pub fn set_style(&mut self, node: Node, style: Style) -> Result<(), Error> {
        let id = self.find_node(node)?;
        self.forest.nodes[id].style = style;
        self.forest.mark_dirty(id);
        Ok(())
    }

    pub fn style(&self, node: Node) -> Result<&Style, Error> {
        let id = self.find_node(node)?;
        Ok(&self.forest.nodes[id].style)
    }

    /// Return this node layout relative to its parent
    pub fn layout(&self, node: Node) -> Result<&Layout, Error> {
        let id = self.find_node(node)?;
        Ok(&self.forest.nodes[id].layout)
    }

    pub fn mark_dirty(&mut self, node: Node) -> Result<(), Error> {
        let id = self.find_node(node)?;
        self.forest.mark_dirty(id);
        Ok(())
    }

    pub fn dirty(&self, node: Node) -> Result<bool, Error> {
        let id = self.find_node(node)?;
        Ok(self.forest.nodes[id].is_dirty)
    }

    pub fn compute_layout(&mut self, node: Node, size: Size<Number>) -> Result<(), Error> {
        let id = self.find_node(node)?;
        self.forest.compute_layout(id, size);
        Ok(())
    }
}

impl Drop for Stretch {
    fn drop(&mut self) {
        INSTANCE_ALLOCATOR.free(&[self.id]);
    }
}

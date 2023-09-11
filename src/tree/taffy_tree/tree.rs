//! UI node types and related data structures.
//!
//! Layouts are composed of multiple nodes, which live in a tree-like data structure.
use core::cell::{RefCell, RefMut};

use slotmap::{DefaultKey, SlotMap, SparseSecondaryMap};

use crate::geometry::Size;
use crate::style::{AvailableSpace, Display, Style};
use crate::tree::{Cache, Layout, LayoutInput, LayoutOutput, LayoutTree, NodeData, NodeId, PartialLayoutTree, RunMode};
use crate::util::debug::{debug_log, debug_log_node};
use crate::util::sys::{new_vec_with_capacity, ChildrenVec, Vec};

#[cfg(feature = "block_layout")]
use crate::compute::compute_block_layout;
#[cfg(feature = "flexbox")]
use crate::compute::compute_flexbox_layout;
#[cfg(feature = "grid")]
use crate::compute::compute_grid_layout;
use crate::compute::{compute_cached_layout, compute_hidden_layout, compute_layout, compute_leaf_layout, round_layout};

use super::{TaffyError, TaffyResult};

/// Global configuration values for a Taffy instance
pub(crate) struct TaffyConfig {
    /// Whether to round layout values
    pub(crate) use_rounding: bool,
}

impl Default for TaffyConfig {
    fn default() -> Self {
        Self { use_rounding: true }
    }
}

/// Used to cache the resolved children of a node (taking into account `Display::contents`) during layout
/// so that repeated calls to the children method don't need to re-resolve the list.
struct ChildrenCache {
    /// The NodeId of the node whose children we are caching (the cache key)
    node_id: NodeId,
    /// The actual list of child ids
    children: Vec<NodeId>,
}
impl ChildrenCache {
    /// Create a new empty cache
    fn new() -> ChildrenCache {
        ChildrenCache { node_id: NodeId::new(0), children: Vec::new() }
    }
}

/// A tree of UI nodes suitable for UI layout
pub struct Taffy<NodeContext = ()> {
    /// The [`NodeData`] for each node stored in this tree
    pub(crate) nodes: SlotMap<DefaultKey, NodeData>,

    /// Functions/closures that compute the intrinsic size of leaf nodes
    pub(crate) node_context_data: SparseSecondaryMap<DefaultKey, NodeContext>,

    /// The children of each node
    ///
    /// The indexes in the outer vector correspond to the position of the parent [`NodeData`]
    pub(crate) children: SlotMap<DefaultKey, ChildrenVec<NodeId>>,

    /// The parents of each node
    ///
    /// The indexes in the outer vector correspond to the position of the child [`NodeData`]
    pub(crate) parents: SlotMap<DefaultKey, Option<NodeId>>,

    /// Layout mode configuration
    pub(crate) config: TaffyConfig,
}

impl Default for Taffy {
    fn default() -> Taffy<()> {
        Taffy::new()
    }
}

/// Iterator over the Vec in a RefMut<'a, Vec<NodeId>>
pub struct RefCellVecIter<'a> {
    /// The items to iterate over
    children: RefMut<'a, Vec<NodeId>>,
    /// The next index to return
    index: usize,
}
impl<'a> Iterator for RefCellVecIter<'a> {
    type Item = NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.children.get(self.index).copied();
        self.index += 1;
        item
    }
}

/// Iterates over children, checking the Display type of the node
/// If the node is `Display::Contents`, then we recurse it's children, else we simply push the `NodeId` into the list
fn find_children_recursive<NodeContext>(tree: &Taffy<NodeContext>, node: NodeId, out: &mut Vec<NodeId>) {
    for child_id in tree.children[node.into()].iter() {
        let child_key: DefaultKey = (*child_id).into();
        let display = tree.nodes[child_key].style.display;
        match display {
            Display::Contents => find_children_recursive(tree, *child_id, out),
            _ => out.push(*child_id),
        }
    }
}

/// View over the Taffy tree that holds the tree itself along with a reference to the context
/// and implements LayoutTree. This allows the context to be stored outside of the Taffy struct
/// which makes the lifetimes of the context much more flexible.
pub(crate) struct TaffyView<'t, NodeContext, MeasureFunction>
where
    MeasureFunction: FnMut(Size<Option<f32>>, Size<AvailableSpace>, NodeId, Option<&mut NodeContext>) -> Size<f32>,
{
    /// A reference to the Taffy tree
    pub(crate) taffy: &'t mut Taffy<NodeContext>,
    /// The context provided for passing to measure functions if layout is run over this struct
    pub(crate) measure_function: MeasureFunction,
    /// Used to cache the resolved children of a node (taking into account `Display::contents`) during layout
    /// so that repeated calls to the children method don't need to re-resolve the list.
    node_children_cache: RefCell<ChildrenCache>,
}

impl<'t, NodeContext, MeasureFunction> TaffyView<'t, NodeContext, MeasureFunction>
where
    MeasureFunction: FnMut(Size<Option<f32>>, Size<AvailableSpace>, NodeId, Option<&mut NodeContext>) -> Size<f32>,
{
    /// Create a new TaffyView from a Taffy and a measure function
    pub(crate) fn new(taffy: &'t mut Taffy<NodeContext>, measure_function: MeasureFunction) -> Self {
        TaffyView { taffy, measure_function, node_children_cache: RefCell::new(ChildrenCache::new()) }
    }

    /// Returns the resolved children, taking into account `Display::Contents`
    /// Will use cached result if available, else compute and cache.
    fn resolve_children(&self, node: NodeId) -> RefMut<'_, Vec<NodeId>> {
        let mut cache = self.node_children_cache.borrow_mut();

        // If the cache key does not match the requested node_id, then recompute the children for
        // the requested node and update the cache in-place.
        if cache.node_id != node {
            cache.node_id = node;
            cache.children.clear();
            find_children_recursive(self.taffy, node, &mut cache.children);
        }

        // In all cases, return a reference into the cache
        RefMut::map(cache, |c| &mut c.children)
    }
}

impl<'t, NodeContext, MeasureFunction> PartialLayoutTree for TaffyView<'t, NodeContext, MeasureFunction>
where
    MeasureFunction: FnMut(Size<Option<f32>>, Size<AvailableSpace>, NodeId, Option<&mut NodeContext>) -> Size<f32>,
{
    type ChildIter<'a> = RefCellVecIter<'a> where Self: 'a;

    #[inline(always)]
    fn child_ids(&self, node: NodeId) -> Self::ChildIter<'_> {
        RefCellVecIter { children: self.resolve_children(node), index: 0 }
    }

    #[inline(always)]
    fn child_count(&self, node: NodeId) -> usize {
        self.resolve_children(node).len()
    }

    #[inline(always)]
    fn get_child_id(&self, node: NodeId, id: usize) -> NodeId {
        self.resolve_children(node)[id]
    }

    #[inline(always)]
    fn get_style(&self, node: NodeId) -> &Style {
        &self.taffy.nodes[node.into()].style
    }

    #[inline(always)]
    fn get_cache_mut(&mut self, node: NodeId) -> &mut Cache {
        &mut self.taffy.nodes[node.into()].cache
    }

    #[inline(always)]
    fn get_unrounded_layout_mut(&mut self, node: NodeId) -> &mut Layout {
        &mut self.taffy.nodes[node.into()].unrounded_layout
    }

    #[inline(always)]
    fn compute_child_layout(&mut self, node: NodeId, inputs: LayoutInput) -> LayoutOutput {
        // If RunMode is PerformHiddenLayout then this indicates that an ancestor node is `Display::None`
        // and thus that we should lay out this node using hidden layout regardless of it's own display style.
        if inputs.run_mode == RunMode::PerformHiddenLayout {
            debug_log!("HIDDEN");
            return compute_hidden_layout(self, node);
        }

        // We run the following wrapped in "compute_cached_layout", which will check the cache for an entry matching the node and inputs and:
        //   - Return that entry if exists
        //   - Else call the passed closure (below) to compute the result
        //
        // If there was no cache match and a new result needs to be computed then that result will be added to the cache
        compute_cached_layout(self, node, inputs, |tree, node, inputs| {
            let display_mode = tree.get_style(node).display;
            let has_children = tree.child_count(node) > 0;

            debug_log!(display_mode);
            debug_log_node!(
                inputs.known_dimensions,
                inputs.parent_size,
                inputs.available_space,
                inputs.run_mode,
                inputs.sizing_mode
            );

            // Dispatch to a layout algorithm based on the node's display style and whether the node has children or not.
            match (display_mode, has_children) {
                (Display::None, _) => compute_hidden_layout(tree, node),
                (Display::Contents, _) => {
                    *tree.get_unrounded_layout_mut(node) = Layout::with_order(0);
                    tree.get_cache_mut(node).clear();
                    LayoutOutput::HIDDEN
                }
                #[cfg(feature = "block_layout")]
                (Display::Block, true) => compute_block_layout(tree, node, inputs),
                #[cfg(feature = "flexbox")]
                (Display::Flex, true) => compute_flexbox_layout(tree, node, inputs),
                #[cfg(feature = "grid")]
                (Display::Grid, true) => compute_grid_layout(tree, node, inputs),
                (_, false) => {
                    let node_key = node.into();
                    let style = &tree.taffy.nodes[node_key].style;
                    let needs_measure = tree.taffy.nodes[node_key].needs_measure;
                    if needs_measure {
                        let node_context = Some(&mut tree.taffy.node_context_data[node_key]);
                        compute_leaf_layout(
                            inputs,
                            style,
                            Some(|known_dimensions, available_space| {
                                (tree.measure_function)(known_dimensions, available_space, node, node_context)
                            }),
                        )
                    } else {
                        /// Type inference gets confused because we're just passing None here. So we give
                        /// it a concrete type to work with (even though we never construct the inner type)
                        type DummyMeasureFunction = Option<fn(Size<Option<f32>>, Size<AvailableSpace>) -> Size<f32>>;
                        let measure_function: DummyMeasureFunction = None;
                        compute_leaf_layout(inputs, style, measure_function)
                    }
                }
            }
        })
    }
}

impl<'t, NodeContext, MeasureFunction> LayoutTree for TaffyView<'t, NodeContext, MeasureFunction>
where
    MeasureFunction: FnMut(Size<Option<f32>>, Size<AvailableSpace>, NodeId, Option<&mut NodeContext>) -> Size<f32>,
{
    #[inline(always)]
    fn get_final_layout(&self, node: NodeId) -> &Layout {
        &self.taffy.nodes[node.into()].final_layout
    }

    #[inline(always)]
    fn get_final_layout_mut(&mut self, node: NodeId) -> &mut Layout {
        &mut self.taffy.nodes[node.into()].final_layout
    }
}

#[allow(clippy::iter_cloned_collect)] // due to no-std support, we need to use `iter_cloned` instead of `collect`
impl<NodeContext> Taffy<NodeContext> {
    /// Creates a new [`Taffy`]
    ///
    /// The default capacity of a [`Taffy`] is 16 nodes.
    #[must_use]
    pub fn new() -> Self {
        Self::with_capacity(16)
    }

    /// Creates a new [`Taffy`] that can store `capacity` nodes before reallocation
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Taffy {
            // TODO: make this method const upstream,
            // so constructors here can be const
            nodes: SlotMap::with_capacity(capacity),
            children: SlotMap::with_capacity(capacity),
            parents: SlotMap::with_capacity(capacity),
            node_context_data: SparseSecondaryMap::with_capacity(capacity),
            config: TaffyConfig::default(),
        }
    }

    /// Enable rounding of layout values. Rounding is enabled by default.
    pub fn enable_rounding(&mut self) {
        self.config.use_rounding = true;
    }

    /// Disable rounding of layout values. Rounding is enabled by default.
    pub fn disable_rounding(&mut self) {
        self.config.use_rounding = false;
    }

    /// Creates and adds a new unattached leaf node to the tree, and returns the node of the new node
    pub fn new_leaf(&mut self, layout: Style) -> TaffyResult<NodeId> {
        let id = self.nodes.insert(NodeData::new(layout));
        let _ = self.children.insert(new_vec_with_capacity(0));
        let _ = self.parents.insert(None);

        Ok(id.into())
    }

    /// Creates and adds a new unattached leaf node to the tree, and returns the [`NodeId`] of the new node
    ///
    /// Creates and adds a new leaf node with a supplied context
    pub fn new_leaf_with_context(&mut self, layout: Style, context: NodeContext) -> TaffyResult<NodeId> {
        let mut data = NodeData::new(layout);
        data.needs_measure = true;

        let id = self.nodes.insert(data);
        self.node_context_data.insert(id, context);

        let _ = self.children.insert(new_vec_with_capacity(0));
        let _ = self.parents.insert(None);

        Ok(id.into())
    }

    /// Creates and adds a new node, which may have any number of `children`
    pub fn new_with_children(&mut self, layout: Style, children: &[NodeId]) -> TaffyResult<NodeId> {
        let id = NodeId::from(self.nodes.insert(NodeData::new(layout)));

        for child in children {
            self.parents[(*child).into()] = Some(id);
        }

        let _ = self.children.insert(children.iter().copied().collect::<_>());
        let _ = self.parents.insert(None);

        Ok(id)
    }

    /// Drops all nodes in the tree
    pub fn clear(&mut self) {
        self.nodes.clear();
        self.children.clear();
        self.parents.clear();
    }

    /// Remove a specific node from the tree and drop it
    ///
    /// Returns the id of the node removed.
    pub fn remove(&mut self, node: NodeId) -> TaffyResult<NodeId> {
        let key = node.into();
        if let Some(parent) = self.parents[key] {
            if let Some(children) = self.children.get_mut(parent.into()) {
                children.retain(|f| *f != node);
            }
        }

        // Remove "parent" references to a node when removing that node
        if let Some(children) = self.children.get(key) {
            for child in children.iter().copied() {
                self.parents[child.into()] = None;
            }
        }

        let _ = self.children.remove(key);
        let _ = self.parents.remove(key);
        let _ = self.nodes.remove(key);

        Ok(node)
    }

    /// Sets the context data associated with the node
    pub fn set_node_context(&mut self, node: NodeId, measure: Option<NodeContext>) -> TaffyResult<()> {
        let key = node.into();
        if let Some(measure) = measure {
            self.nodes[key].needs_measure = true;
            self.node_context_data.insert(key, measure);
        } else {
            self.nodes[key].needs_measure = false;
            self.node_context_data.remove(key);
        }

        self.mark_dirty(node)?;

        Ok(())
    }

    /// Get's a mutable reference to the the context data associated with the node
    pub fn get_node_context_mut(&mut self, node: NodeId) -> Option<&mut NodeContext> {
        self.node_context_data.get_mut(node.into())
    }

    /// Adds a `child` node under the supplied `parent`
    pub fn add_child(&mut self, parent: NodeId, child: NodeId) -> TaffyResult<()> {
        let parent_key = parent.into();
        let child_key = child.into();
        self.parents[child_key] = Some(parent);
        self.children[parent_key].push(child);
        self.mark_dirty(parent)?;

        Ok(())
    }

    /// Inserts a `child` node at the given `child_index` under the supplied `parent`, shifting all children after it to the right.
    pub fn insert_child_at_index(&mut self, parent: NodeId, child_index: usize, child: NodeId) -> TaffyResult<()> {
        let parent_key = parent.into();

        let child_count = self.children[parent_key].len();
        if child_index > child_count {
            return Err(TaffyError::ChildIndexOutOfBounds { parent, child_index, child_count });
        }

        self.parents[child.into()] = Some(parent);
        self.children[parent_key].insert(child_index, child);
        self.mark_dirty(parent)?;

        Ok(())
    }

    /// Directly sets the `children` of the supplied `parent`
    pub fn set_children(&mut self, parent: NodeId, children: &[NodeId]) -> TaffyResult<()> {
        let parent_key = parent.into();

        // Remove node as parent from all its current children.
        for child in &self.children[parent_key] {
            self.parents[(*child).into()] = None;
        }

        // Build up relation node <-> child
        for child in children {
            self.parents[(*child).into()] = Some(parent);
        }

        let parent_children = &mut self.children[parent_key];
        parent_children.clear();
        children.iter().for_each(|child| parent_children.push(*child));

        self.mark_dirty(parent)?;

        Ok(())
    }

    /// Removes the `child` of the parent `node`
    ///
    /// The child is not removed from the tree entirely, it is simply no longer attached to its previous parent.
    pub fn remove_child(&mut self, parent: NodeId, child: NodeId) -> TaffyResult<NodeId> {
        let index = self.children[parent.into()].iter().position(|n| *n == child).unwrap();
        self.remove_child_at_index(parent, index)
    }

    /// Removes the child at the given `index` from the `parent`
    ///
    /// The child is not removed from the tree entirely, it is simply no longer attached to its previous parent.
    pub fn remove_child_at_index(&mut self, parent: NodeId, child_index: usize) -> TaffyResult<NodeId> {
        let parent_key = parent.into();
        let child_count = self.children[parent_key].len();
        if child_index >= child_count {
            return Err(TaffyError::ChildIndexOutOfBounds { parent, child_index, child_count });
        }

        let child = self.children[parent_key].remove(child_index);
        self.parents[child.into()] = None;

        self.mark_dirty(parent)?;

        Ok(child)
    }

    /// Replaces the child at the given `child_index` from the `parent` node with the new `child` node
    ///
    /// The child is not removed from the tree entirely, it is simply no longer attached to its previous parent.
    pub fn replace_child_at_index(
        &mut self,
        parent: NodeId,
        child_index: usize,
        new_child: NodeId,
    ) -> TaffyResult<NodeId> {
        let parent_key = parent.into();

        let child_count = self.children[parent_key].len();
        if child_index >= child_count {
            return Err(TaffyError::ChildIndexOutOfBounds { parent, child_index, child_count });
        }

        self.parents[new_child.into()] = Some(parent);
        let old_child = core::mem::replace(&mut self.children[parent_key][child_index], new_child);
        self.parents[old_child.into()] = None;

        self.mark_dirty(parent)?;

        Ok(old_child)
    }

    /// Returns the child node of the parent `node` at the provided `child_index`
    pub fn child_at_index(&self, parent: NodeId, child_index: usize) -> TaffyResult<NodeId> {
        let parent_key = parent.into();
        let child_count = self.children[parent_key].len();
        if child_index >= child_count {
            return Err(TaffyError::ChildIndexOutOfBounds { parent, child_index, child_count });
        }

        Ok(self.children[parent_key][child_index])
    }

    /// Returns the total number of nodes in the tree
    pub fn total_node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Returns the number of children of the `parent` node
    pub fn child_count(&self, parent: NodeId) -> TaffyResult<usize> {
        Ok(self.children[parent.into()].len())
    }

    /// Returns a list of children that belong to the parent node
    pub fn children(&self, parent: NodeId) -> TaffyResult<Vec<NodeId>> {
        Ok(self.children[parent.into()].iter().copied().collect::<_>())
    }

    /// Sets the [`Style`] of the provided `node`
    pub fn set_style(&mut self, node: NodeId, style: Style) -> TaffyResult<()> {
        self.nodes[node.into()].style = style;
        self.mark_dirty(node)?;
        Ok(())
    }

    /// Gets the [`Style`] of the provided `node`
    pub fn style(&self, node: NodeId) -> TaffyResult<&Style> {
        Ok(&self.nodes[node.into()].style)
    }

    /// Return this node layout relative to its parent
    pub fn layout(&self, node: NodeId) -> TaffyResult<&Layout> {
        Ok(&self.nodes[node.into()].final_layout)
    }

    /// Marks the layout computation of this node and its children as outdated
    ///
    /// Performs a recursive depth-first search up the tree until the root node is reached
    ///
    /// WARNING: this will stack-overflow if the tree contains a cycle
    pub fn mark_dirty(&mut self, node: NodeId) -> TaffyResult<()> {
        /// WARNING: this will stack-overflow if the tree contains a cycle
        fn mark_dirty_recursive(
            nodes: &mut SlotMap<DefaultKey, NodeData>,
            parents: &SlotMap<DefaultKey, Option<NodeId>>,
            node_key: DefaultKey,
        ) {
            nodes[node_key].mark_dirty();

            if let Some(Some(node)) = parents.get(node_key) {
                mark_dirty_recursive(nodes, parents, (*node).into());
            }
        }

        mark_dirty_recursive(&mut self.nodes, &self.parents, node.into());

        Ok(())
    }

    /// Indicates whether the layout of this node (and its children) need to be recomputed
    pub fn dirty(&self, node: NodeId) -> TaffyResult<bool> {
        Ok(self.nodes[node.into()].cache.is_empty())
    }

    /// Updates the stored layout of the provided `node` and its children
    pub fn compute_layout_with_measure<MeasureFunction>(
        &mut self,
        node_id: NodeId,
        available_space: Size<AvailableSpace>,
        measure_function: MeasureFunction,
    ) -> Result<(), TaffyError>
    where
        MeasureFunction: FnMut(Size<Option<f32>>, Size<AvailableSpace>, NodeId, Option<&mut NodeContext>) -> Size<f32>,
    {
        let use_rounding = self.config.use_rounding;
        let mut taffy_view = TaffyView::new(self, measure_function);
        compute_layout(&mut taffy_view, node_id, available_space);
        if use_rounding {
            round_layout(&mut taffy_view, node_id);
        }
        Ok(())
    }

    /// Updates the stored layout of the provided `node` and its children
    pub fn compute_layout(&mut self, node: NodeId, available_space: Size<AvailableSpace>) -> Result<(), TaffyError> {
        self.compute_layout_with_measure(node, available_space, |_, _, _, _| Size::ZERO)
    }

    /// Prints a debug representation of the tree's layout
    #[cfg(feature = "std")]
    pub fn print_tree(&mut self, root: NodeId) {
        let taffy_view = TaffyView::new(self, |_, _, _, _| Size::ZERO);
        crate::util::print_tree(&taffy_view, root)
    }

    /// Returns an instance of LayoutTree representing the Taffy
    #[cfg(test)]
    pub(crate) fn as_layout_tree(&mut self) -> impl LayoutTree + '_ {
        TaffyView::new(self, |_, _, _, _| Size::ZERO)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::bool_assert_comparison)]

    use super::*;
    use crate::style::{Dimension, Display, FlexDirection};
    use crate::style_helpers::*;
    use crate::util::sys;

    fn size_measure_function(
        known_dimensions: Size<Option<f32>>,
        _available_space: Size<AvailableSpace>,
        _node_id: NodeId,
        node_context: Option<&mut Size<f32>>,
    ) -> Size<f32> {
        known_dimensions.unwrap_or(node_context.cloned().unwrap_or(Size::ZERO))
    }

    #[test]
    fn new_should_allocate_default_capacity() {
        const DEFAULT_CAPACITY: usize = 16; // This is the capacity defined in the `impl Default`
        let taffy: Taffy<()> = Taffy::new();

        assert!(taffy.children.capacity() >= DEFAULT_CAPACITY);
        assert!(taffy.parents.capacity() >= DEFAULT_CAPACITY);
        assert!(taffy.nodes.capacity() >= DEFAULT_CAPACITY);
    }

    #[test]
    fn test_with_capacity() {
        const CAPACITY: usize = 8;
        let taffy: Taffy<()> = Taffy::with_capacity(CAPACITY);

        assert!(taffy.children.capacity() >= CAPACITY);
        assert!(taffy.parents.capacity() >= CAPACITY);
        assert!(taffy.nodes.capacity() >= CAPACITY);
    }

    #[test]
    fn test_new_leaf() {
        let mut taffy: Taffy<()> = Taffy::new();

        let res = taffy.new_leaf(Style::default());
        assert!(res.is_ok());
        let node = res.unwrap();

        // node should be in the taffy tree and have no children
        assert!(taffy.child_count(node).unwrap() == 0);
    }

    #[test]
    fn new_leaf_with_context() {
        let mut taffy: Taffy<Size<f32>> = Taffy::new();

        let res = taffy.new_leaf_with_context(Style::default(), Size::ZERO);
        assert!(res.is_ok());
        let node = res.unwrap();

        // node should be in the taffy tree and have no children
        assert!(taffy.child_count(node).unwrap() == 0);
    }

    /// Test that new_with_children works as expected
    #[test]
    fn test_new_with_children() {
        let mut taffy: Taffy<()> = Taffy::new();
        let child0 = taffy.new_leaf(Style::default()).unwrap();
        let child1 = taffy.new_leaf(Style::default()).unwrap();
        let node = taffy.new_with_children(Style::default(), &[child0, child1]).unwrap();

        // node should have two children
        assert_eq!(taffy.child_count(node).unwrap(), 2);
        assert_eq!(taffy.children(node).unwrap()[0], child0);
        assert_eq!(taffy.children(node).unwrap()[1], child1);
    }

    #[test]
    fn remove_node_should_remove() {
        let mut taffy: Taffy<()> = Taffy::new();

        let node = taffy.new_leaf(Style::default()).unwrap();

        let _ = taffy.remove(node).unwrap();
    }

    #[test]
    fn remove_node_should_detach_herarchy() {
        let mut taffy: Taffy<()> = Taffy::new();

        // Build a linear tree layout: <0> <- <1> <- <2>
        let node2 = taffy.new_leaf(Style::default()).unwrap();
        let node1 = taffy.new_with_children(Style::default(), &[node2]).unwrap();
        let node0 = taffy.new_with_children(Style::default(), &[node1]).unwrap();

        // Both node0 and node1 should have 1 child nodes
        assert_eq!(taffy.children(node0).unwrap().as_slice(), &[node1]);
        assert_eq!(taffy.children(node1).unwrap().as_slice(), &[node2]);

        // Disconnect the tree: <0> <2>
        let _ = taffy.remove(node1).unwrap();

        // Both remaining nodes should have no child nodes
        assert!(taffy.children(node0).unwrap().is_empty());
        assert!(taffy.children(node2).unwrap().is_empty());
    }

    #[test]
    fn remove_last_node() {
        let mut taffy: Taffy<()> = Taffy::new();

        let parent = taffy.new_leaf(Style::default()).unwrap();
        let child = taffy.new_leaf(Style::default()).unwrap();
        taffy.add_child(parent, child).unwrap();

        taffy.remove(child).unwrap();
        taffy.remove(parent).unwrap();
    }

    #[test]
    fn set_measure() {
        let mut taffy: Taffy<Size<f32>> = Taffy::new();
        let node = taffy.new_leaf_with_context(Style::default(), Size { width: 200.0, height: 200.0 }).unwrap();
        taffy.compute_layout_with_measure(node, Size::MAX_CONTENT, size_measure_function).unwrap();
        assert_eq!(taffy.layout(node).unwrap().size.width, 200.0);

        taffy.set_node_context(node, Some(Size { width: 100.0, height: 100.0 })).unwrap();
        taffy.compute_layout_with_measure(node, Size::MAX_CONTENT, size_measure_function).unwrap();
        assert_eq!(taffy.layout(node).unwrap().size.width, 100.0);
    }

    #[test]
    fn set_measure_of_previously_unmeasured_node() {
        let mut taffy: Taffy<Size<f32>> = Taffy::new();
        let node = taffy.new_leaf(Style::default()).unwrap();
        taffy.compute_layout_with_measure(node, Size::MAX_CONTENT, size_measure_function).unwrap();
        assert_eq!(taffy.layout(node).unwrap().size.width, 0.0);

        taffy.set_node_context(node, Some(Size { width: 100.0, height: 100.0 })).unwrap();
        taffy.compute_layout_with_measure(node, Size::MAX_CONTENT, size_measure_function).unwrap();
        assert_eq!(taffy.layout(node).unwrap().size.width, 100.0);
    }

    /// Test that adding `add_child()` works
    #[test]
    fn add_child() {
        let mut taffy: Taffy<()> = Taffy::new();
        let node = taffy.new_leaf(Style::default()).unwrap();
        assert_eq!(taffy.child_count(node).unwrap(), 0);

        let child0 = taffy.new_leaf(Style::default()).unwrap();
        taffy.add_child(node, child0).unwrap();
        assert_eq!(taffy.child_count(node).unwrap(), 1);

        let child1 = taffy.new_leaf(Style::default()).unwrap();
        taffy.add_child(node, child1).unwrap();
        assert_eq!(taffy.child_count(node).unwrap(), 2);
    }

    #[test]
    fn insert_child_at_index() {
        let mut taffy: Taffy<()> = Taffy::new();

        let child0 = taffy.new_leaf(Style::default()).unwrap();
        let child1 = taffy.new_leaf(Style::default()).unwrap();
        let child2 = taffy.new_leaf(Style::default()).unwrap();

        let node = taffy.new_leaf(Style::default()).unwrap();
        assert_eq!(taffy.child_count(node).unwrap(), 0);

        taffy.insert_child_at_index(node, 0, child0).unwrap();
        assert_eq!(taffy.child_count(node).unwrap(), 1);
        assert_eq!(taffy.children(node).unwrap()[0], child0);

        taffy.insert_child_at_index(node, 0, child1).unwrap();
        assert_eq!(taffy.child_count(node).unwrap(), 2);
        assert_eq!(taffy.children(node).unwrap()[0], child1);
        assert_eq!(taffy.children(node).unwrap()[1], child0);

        taffy.insert_child_at_index(node, 1, child2).unwrap();
        assert_eq!(taffy.child_count(node).unwrap(), 3);
        assert_eq!(taffy.children(node).unwrap()[0], child1);
        assert_eq!(taffy.children(node).unwrap()[1], child2);
        assert_eq!(taffy.children(node).unwrap()[2], child0);
    }

    #[test]
    fn set_children() {
        let mut taffy: Taffy<()> = Taffy::new();

        let child0 = taffy.new_leaf(Style::default()).unwrap();
        let child1 = taffy.new_leaf(Style::default()).unwrap();
        let node = taffy.new_with_children(Style::default(), &[child0, child1]).unwrap();

        assert_eq!(taffy.child_count(node).unwrap(), 2);
        assert_eq!(taffy.children(node).unwrap()[0], child0);
        assert_eq!(taffy.children(node).unwrap()[1], child1);

        let child2 = taffy.new_leaf(Style::default()).unwrap();
        let child3 = taffy.new_leaf(Style::default()).unwrap();
        taffy.set_children(node, &[child2, child3]).unwrap();

        assert_eq!(taffy.child_count(node).unwrap(), 2);
        assert_eq!(taffy.children(node).unwrap()[0], child2);
        assert_eq!(taffy.children(node).unwrap()[1], child3);
    }

    /// Test that removing a child works
    #[test]
    fn remove_child() {
        let mut taffy: Taffy<()> = Taffy::new();
        let child0 = taffy.new_leaf(Style::default()).unwrap();
        let child1 = taffy.new_leaf(Style::default()).unwrap();
        let node = taffy.new_with_children(Style::default(), &[child0, child1]).unwrap();

        assert_eq!(taffy.child_count(node).unwrap(), 2);

        taffy.remove_child(node, child0).unwrap();
        assert_eq!(taffy.child_count(node).unwrap(), 1);
        assert_eq!(taffy.children(node).unwrap()[0], child1);

        taffy.remove_child(node, child1).unwrap();
        assert_eq!(taffy.child_count(node).unwrap(), 0);
    }

    #[test]
    fn remove_child_at_index() {
        let mut taffy: Taffy<()> = Taffy::new();
        let child0 = taffy.new_leaf(Style::default()).unwrap();
        let child1 = taffy.new_leaf(Style::default()).unwrap();
        let node = taffy.new_with_children(Style::default(), &[child0, child1]).unwrap();

        assert_eq!(taffy.child_count(node).unwrap(), 2);

        taffy.remove_child_at_index(node, 0).unwrap();
        assert_eq!(taffy.child_count(node).unwrap(), 1);
        assert_eq!(taffy.children(node).unwrap()[0], child1);

        taffy.remove_child_at_index(node, 0).unwrap();
        assert_eq!(taffy.child_count(node).unwrap(), 0);
    }

    // Related to: https://github.com/DioxusLabs/taffy/issues/510
    #[test]
    fn remove_child_updates_parents() {
        let mut taffy: Taffy<()> = Taffy::new();

        let parent = taffy.new_leaf(Style::default()).unwrap();
        let child = taffy.new_leaf(Style::default()).unwrap();

        taffy.add_child(parent, child).unwrap();

        taffy.remove(parent).unwrap();

        // Once the parent is removed this shouldn't panic.
        assert!(taffy.set_children(child, &[]).is_ok());
    }

    #[test]
    fn replace_child_at_index() {
        let mut taffy: Taffy<()> = Taffy::new();

        let child0 = taffy.new_leaf(Style::default()).unwrap();
        let child1 = taffy.new_leaf(Style::default()).unwrap();

        let node = taffy.new_with_children(Style::default(), &[child0]).unwrap();
        assert_eq!(taffy.child_count(node).unwrap(), 1);
        assert_eq!(taffy.children(node).unwrap()[0], child0);

        taffy.replace_child_at_index(node, 0, child1).unwrap();
        assert_eq!(taffy.child_count(node).unwrap(), 1);
        assert_eq!(taffy.children(node).unwrap()[0], child1);
    }
    #[test]
    fn test_child_at_index() {
        let mut taffy: Taffy<()> = Taffy::new();
        let child0 = taffy.new_leaf(Style::default()).unwrap();
        let child1 = taffy.new_leaf(Style::default()).unwrap();
        let child2 = taffy.new_leaf(Style::default()).unwrap();
        let node = taffy.new_with_children(Style::default(), &[child0, child1, child2]).unwrap();

        assert!(if let Ok(result) = taffy.child_at_index(node, 0) { result == child0 } else { false });
        assert!(if let Ok(result) = taffy.child_at_index(node, 1) { result == child1 } else { false });
        assert!(if let Ok(result) = taffy.child_at_index(node, 2) { result == child2 } else { false });
    }
    #[test]
    fn test_child_count() {
        let mut taffy: Taffy<()> = Taffy::new();
        let child0 = taffy.new_leaf(Style::default()).unwrap();
        let child1 = taffy.new_leaf(Style::default()).unwrap();
        let node = taffy.new_with_children(Style::default(), &[child0, child1]).unwrap();

        assert!(if let Ok(count) = taffy.child_count(node) { count == 2 } else { false });
        assert!(if let Ok(count) = taffy.child_count(child0) { count == 0 } else { false });
        assert!(if let Ok(count) = taffy.child_count(child1) { count == 0 } else { false });
    }

    #[allow(clippy::vec_init_then_push)]
    #[test]
    fn test_children() {
        let mut taffy: Taffy<()> = Taffy::new();
        let child0 = taffy.new_leaf(Style::default()).unwrap();
        let child1 = taffy.new_leaf(Style::default()).unwrap();
        let node = taffy.new_with_children(Style::default(), &[child0, child1]).unwrap();

        let mut children = sys::Vec::new();
        children.push(child0);
        children.push(child1);

        let children_result = taffy.children(node).unwrap();
        assert_eq!(children_result, children);

        assert!(taffy.children(child0).unwrap().is_empty());
    }
    #[test]
    fn test_set_style() {
        let mut taffy: Taffy<()> = Taffy::new();

        let node = taffy.new_leaf(Style::default()).unwrap();
        assert_eq!(taffy.style(node).unwrap().display, Display::Flex);

        taffy.set_style(node, Style { display: Display::None, ..Style::default() }).unwrap();
        assert_eq!(taffy.style(node).unwrap().display, Display::None);
    }
    #[test]
    fn test_style() {
        let mut taffy: Taffy<()> = Taffy::new();

        let style = Style { display: Display::None, flex_direction: FlexDirection::RowReverse, ..Default::default() };

        let node = taffy.new_leaf(style.clone()).unwrap();

        let res = taffy.style(node);
        assert!(res.is_ok());
        assert!(res.unwrap() == &style);
    }
    #[test]
    fn test_layout() {
        let mut taffy: Taffy<()> = Taffy::new();
        let node = taffy.new_leaf(Style::default()).unwrap();

        // TODO: Improve this test?
        let res = taffy.layout(node);
        assert!(res.is_ok());
    }

    #[test]
    fn test_mark_dirty() {
        let mut taffy: Taffy<()> = Taffy::new();
        let child0 = taffy.new_leaf(Style::default()).unwrap();
        let child1 = taffy.new_leaf(Style::default()).unwrap();
        let node = taffy.new_with_children(Style::default(), &[child0, child1]).unwrap();

        taffy.compute_layout(node, Size::MAX_CONTENT).unwrap();

        assert_eq!(taffy.dirty(child0).unwrap(), false);
        assert_eq!(taffy.dirty(child1).unwrap(), false);
        assert_eq!(taffy.dirty(node).unwrap(), false);

        taffy.mark_dirty(node).unwrap();
        assert_eq!(taffy.dirty(child0).unwrap(), false);
        assert_eq!(taffy.dirty(child1).unwrap(), false);
        assert_eq!(taffy.dirty(node).unwrap(), true);

        taffy.compute_layout(node, Size::MAX_CONTENT).unwrap();
        taffy.mark_dirty(child0).unwrap();
        assert_eq!(taffy.dirty(child0).unwrap(), true);
        assert_eq!(taffy.dirty(child1).unwrap(), false);
        assert_eq!(taffy.dirty(node).unwrap(), true);
    }

    #[test]
    fn compute_layout_should_produce_valid_result() {
        let mut taffy: Taffy<()> = Taffy::new();
        let node_result = taffy.new_leaf(Style {
            size: Size { width: Dimension::Length(10f32), height: Dimension::Length(10f32) },
            ..Default::default()
        });
        assert!(node_result.is_ok());
        let node = node_result.unwrap();
        let layout_result = taffy.compute_layout(
            node,
            Size { width: AvailableSpace::Definite(100.), height: AvailableSpace::Definite(100.) },
        );
        assert!(layout_result.is_ok());
    }

    #[test]
    fn make_sure_layout_location_is_top_left() {
        use crate::prelude::Rect;

        let mut taffy: Taffy<()> = Taffy::new();

        let node = taffy
            .new_leaf(Style {
                size: Size { width: Dimension::Percent(1f32), height: Dimension::Percent(1f32) },
                ..Default::default()
            })
            .unwrap();

        let root = taffy
            .new_with_children(
                Style {
                    size: Size { width: Dimension::Length(100f32), height: Dimension::Length(100f32) },
                    padding: Rect {
                        left: length(10f32),
                        right: length(20f32),
                        top: length(30f32),
                        bottom: length(40f32),
                    },
                    ..Default::default()
                },
                &[node],
            )
            .unwrap();

        taffy.compute_layout(root, Size::MAX_CONTENT).unwrap();

        // If Layout::location represents top-left coord, 'node' location
        // must be (due applied 'root' padding): {x: 10, y: 30}.
        //
        // It's important, since result will be different for each other
        // coordinate space:
        // - bottom-left:  {x: 10, y: 40}
        // - top-right:    {x: 20, y: 30}
        // - bottom-right: {x: 20, y: 40}
        let layout = taffy.layout(node).unwrap();
        assert_eq!(layout.location.x, 10f32);
        assert_eq!(layout.location.y, 30f32);
    }
}

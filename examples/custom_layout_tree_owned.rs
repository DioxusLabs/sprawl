mod common {
    pub mod image;
    pub mod text;
}
use common::image::{image_measure_function, ImageContext};
use common::text::{text_measure_function, FontMetrics, TextContext, WritingMode, LOREM_IPSUM};
use taffy::tree::{Cache, PartialLayoutTree};
use taffy::util::print_tree;
use taffy::{
    compute_flexbox_layout, compute_grid_layout, compute_layout, compute_leaf_layout, prelude::*, round_layout,
};

#[derive(Debug, Copy, Clone)]
#[allow(dead_code)]
enum NodeKind {
    Flexbox,
    Grid,
    Text,
    Image,
}

struct Node {
    kind: NodeKind,
    style: Style,
    text_data: Option<TextContext>,
    image_data: Option<ImageContext>,
    cache: Cache,
    unrounded_layout: Layout,
    final_layout: Layout,
    children: Vec<Node>,
}

impl Default for Node {
    fn default() -> Self {
        Node {
            kind: NodeKind::Flexbox,
            style: Style::default(),
            text_data: None,
            image_data: None,
            cache: Cache::new(),
            unrounded_layout: Layout::with_order(0),
            final_layout: Layout::with_order(0),
            children: Vec::new(),
        }
    }
}

#[allow(dead_code)]
impl Node {
    pub fn new_row(style: Style) -> Node {
        Node {
            kind: NodeKind::Flexbox,
            style: Style { display: Display::Flex, flex_direction: FlexDirection::Row, ..style },
            ..Node::default()
        }
    }
    pub fn new_column(style: Style) -> Node {
        Node {
            kind: NodeKind::Flexbox,
            style: Style { display: Display::Flex, flex_direction: FlexDirection::Column, ..style },
            ..Node::default()
        }
    }
    pub fn new_grid(style: Style) -> Node {
        Node { kind: NodeKind::Grid, style: Style { display: Display::Grid, ..style }, ..Node::default() }
    }
    pub fn new_text(style: Style, text_data: TextContext) -> Node {
        Node { kind: NodeKind::Text, style, text_data: Some(text_data), ..Node::default() }
    }
    pub fn new_image(style: Style, image_data: ImageContext) -> Node {
        Node { kind: NodeKind::Image, style, image_data: Some(image_data), ..Node::default() }
    }
    pub fn append_child(&mut self, node: Node) {
        self.children.push(node);
    }

    unsafe fn as_id(&self) -> NodeId {
        NodeId::from(self as *const Node as usize)
    }

    pub fn compute_layout(&mut self, available_space: Size<AvailableSpace>, use_rounding: bool) {
        let root_node_id = unsafe { self.as_id() };
        compute_layout(&mut StatelessLayoutTree, root_node_id, available_space);
        if use_rounding {
            round_layout(&mut StatelessLayoutTree, root_node_id)
        }
    }

    pub fn print_tree(&mut self) {
        print_tree(&mut StatelessLayoutTree, unsafe { self.as_id() });
    }
}

struct ChildIter<'a>(std::slice::Iter<'a, Node>);
impl<'a> Iterator for ChildIter<'a> {
    type Item = NodeId;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|c| NodeId::from(c as *const Node as usize))
    }
}

#[inline(always)]
unsafe fn node_from_id<'a>(node_id: NodeId) -> &'a Node {
    &*(usize::from(node_id) as *const Node)
}

#[inline(always)]
unsafe fn node_from_id_mut<'a>(node_id: NodeId) -> &'a mut Node {
    &mut *(usize::from(node_id) as *mut Node)
}

struct StatelessLayoutTree;
impl PartialLayoutTree for StatelessLayoutTree {
    type ChildIter<'a> = ChildIter<'a>;

    fn child_ids(&self, node_id: NodeId) -> Self::ChildIter<'_> {
        unsafe { ChildIter(node_from_id(node_id).children.iter()) }
    }

    fn child_count(&self, node_id: NodeId) -> usize {
        unsafe { node_from_id(node_id).children.len() }
    }

    fn get_child_id(&self, node_id: NodeId, index: usize) -> NodeId {
        unsafe { node_from_id(node_id).children[index].as_id() }
    }

    fn get_style(&self, node_id: NodeId) -> &Style {
        unsafe { &node_from_id(node_id).style }
    }

    fn get_unrounded_layout_mut(&mut self, node_id: NodeId) -> &mut Layout {
        unsafe { &mut node_from_id_mut(node_id).unrounded_layout }
    }

    fn get_final_layout_mut(&mut self, node_id: NodeId) -> &mut Layout {
        unsafe { &mut node_from_id_mut(node_id).final_layout }
    }

    fn get_cache_mut(&mut self, node_id: NodeId) -> &mut Cache {
        unsafe { &mut node_from_id_mut(node_id).cache }
    }

    fn compute_child_layout(&mut self, node_id: NodeId, inputs: taffy::tree::LayoutInput) -> taffy::tree::LayoutOutput {
        let node = unsafe { node_from_id_mut(node_id) };
        let font_metrics = FontMetrics { char_width: 10.0, char_height: 10.0 };

        match node.kind {
            NodeKind::Flexbox => compute_flexbox_layout(self, node_id, inputs),
            NodeKind::Grid => compute_grid_layout(self, node_id, inputs),
            NodeKind::Text => compute_leaf_layout(
                inputs,
                &node.style,
                Some(|known_dimensions, available_space| {
                    text_measure_function(
                        known_dimensions,
                        available_space,
                        node.text_data.as_ref().unwrap(),
                        &font_metrics,
                    )
                }),
            ),
            NodeKind::Image => compute_leaf_layout(
                inputs,
                &node.style,
                Some(|known_dimensions, _available_space| {
                    image_measure_function(known_dimensions, node.image_data.as_ref().unwrap())
                }),
            ),
        }
    }
}

impl LayoutTree for StatelessLayoutTree {
    fn get_final_layout(&self, node_id: NodeId) -> &Layout {
        unsafe { &node_from_id(node_id).final_layout }
    }
}

fn main() -> Result<(), taffy::TaffyError> {
    let mut root = Node::new_column(Style::DEFAULT);

    let text_node = Node::new_text(
        Style::default(),
        TextContext { text_content: LOREM_IPSUM.into(), writing_mode: WritingMode::Horizontal },
    );
    root.append_child(text_node);

    let image_node = Node::new_image(Style::default(), ImageContext { width: 400.0, height: 300.0 });
    root.append_child(image_node);

    // Compute layout and print result
    root.compute_layout(Size::MAX_CONTENT, true);
    root.print_tree();

    Ok(())
}

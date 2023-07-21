use taffy::prelude::*;

#[test]
fn rounding_doesnt_leave_gaps() {
    // First create an instance of Taffy
    let mut taffy = Taffy::new();

    let w_square = Size { width: length(100.3), height: length(100.3) };
    let child_a = taffy.new_leaf(Style { size: w_square, ..Default::default() });
    let child_b = taffy.new_leaf(Style { size: w_square, ..Default::default() });

    let root_node = taffy.new_with_children(
        Style {
            size: Size { width: length(963.3333), height: length(1000.) },
            justify_content: Some(JustifyContent::Center),
            ..Default::default()
        },
        &[child_a, child_b],
    );

    taffy.compute_layout(root_node, Size::MAX_CONTENT);

    let layout_a = taffy.layout(child_a);
    let layout_b = taffy.layout(child_b);
    taffy::util::print_tree(&taffy, root_node);
    assert_eq!(layout_a.location.x + layout_a.size.width, layout_b.location.x);
}

#[test]
#[allow(non_snake_case)]
fn leaf_with_content_and_padding_border__border_box() {
    #[allow(unused_imports)]
    use taffy::{prelude::*, Layout};
    let mut taffy = crate::new_test_tree();
    let node = taffy
        .new_leaf_with_context(
            taffy::style::Style {
                padding: taffy::geometry::Rect {
                    left: length(8f32),
                    right: length(4f32),
                    top: length(2f32),
                    bottom: length(6f32),
                },
                border: taffy::geometry::Rect {
                    left: length(7f32),
                    right: length(3f32),
                    top: length(1f32),
                    bottom: length(5f32),
                },
                ..Default::default()
            },
            crate::TestNodeContext::ahem_text("HHHH", crate::WritingMode::Horizontal),
        )
        .unwrap();
    taffy.compute_layout_with_measure(node, taffy::geometry::Size::MAX_CONTENT, crate::test_measure_function).unwrap();
    println!("\nComputed tree:");
    taffy.print_tree(node);
    println!();
    let layout = taffy.layout(node).unwrap();
    let Layout { size, location, .. } = layout;
    assert_eq!(size.width, 62f32, "width of node {:?}. Expected {}. Actual {}", node, 62f32, size.width);
    assert_eq!(size.height, 24f32, "height of node {:?}. Expected {}. Actual {}", node, 24f32, size.height);
    assert_eq!(location.x, 0f32, "x of node {:?}. Expected {}. Actual {}", node, 0f32, location.x);
    assert_eq!(location.y, 0f32, "y of node {:?}. Expected {}. Actual {}", node, 0f32, location.y);
}

#[test]
#[allow(non_snake_case)]
fn leaf_with_content_and_padding_border__content_box() {
    #[allow(unused_imports)]
    use taffy::{prelude::*, Layout};
    let mut taffy = crate::new_test_tree();
    let node = taffy
        .new_leaf_with_context(
            taffy::style::Style {
                box_sizing: taffy::style::BoxSizing::ContentBox,
                padding: taffy::geometry::Rect {
                    left: length(8f32),
                    right: length(4f32),
                    top: length(2f32),
                    bottom: length(6f32),
                },
                border: taffy::geometry::Rect {
                    left: length(7f32),
                    right: length(3f32),
                    top: length(1f32),
                    bottom: length(5f32),
                },
                ..Default::default()
            },
            crate::TestNodeContext::ahem_text("HHHH", crate::WritingMode::Horizontal),
        )
        .unwrap();
    taffy.compute_layout_with_measure(node, taffy::geometry::Size::MAX_CONTENT, crate::test_measure_function).unwrap();
    println!("\nComputed tree:");
    taffy.print_tree(node);
    println!();
    let layout = taffy.layout(node).unwrap();
    let Layout { size, location, .. } = layout;
    assert_eq!(size.width, 62f32, "width of node {:?}. Expected {}. Actual {}", node, 62f32, size.width);
    assert_eq!(size.height, 24f32, "height of node {:?}. Expected {}. Actual {}", node, 24f32, size.height);
    assert_eq!(location.x, 0f32, "x of node {:?}. Expected {}. Actual {}", node, 0f32, location.x);
    assert_eq!(location.y, 0f32, "y of node {:?}. Expected {}. Actual {}", node, 0f32, location.y);
}

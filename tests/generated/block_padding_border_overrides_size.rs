#[test]
fn block_padding_border_overrides_size() {
    #[allow(unused_imports)]
    use taffy::{prelude::*, tree::Layout};
    let mut taffy = taffy::Taffy::new();
    let node00 = taffy.new_leaf(taffy::style::Style { ..Default::default() });
    let node0 = taffy.new_with_children(
        taffy::style::Style {
            display: taffy::style::Display::Block,
            size: taffy::geometry::Size {
                width: taffy::style::Dimension::Length(12f32),
                height: taffy::style::Dimension::Length(12f32),
            },
            padding: taffy::geometry::Rect {
                left: taffy::style::LengthPercentage::Length(8f32),
                right: taffy::style::LengthPercentage::Length(4f32),
                top: taffy::style::LengthPercentage::Length(2f32),
                bottom: taffy::style::LengthPercentage::Length(6f32),
            },
            border: taffy::geometry::Rect {
                left: taffy::style::LengthPercentage::Length(7f32),
                right: taffy::style::LengthPercentage::Length(3f32),
                top: taffy::style::LengthPercentage::Length(1f32),
                bottom: taffy::style::LengthPercentage::Length(5f32),
            },
            ..Default::default()
        },
        &[node00],
    );
    let node = taffy.new_with_children(
        taffy::style::Style { display: taffy::style::Display::Block, ..Default::default() },
        &[node0],
    );
    taffy.compute_layout(node, taffy::geometry::Size::MAX_CONTENT);
    println!("\nComputed tree:");
    taffy::util::print_tree(&taffy, node);
    println!();
    let Layout { size, location, .. } = taffy.layout(node);
    assert_eq!(size.width, 22f32, "width of node {:?}. Expected {}. Actual {}", node, 22f32, size.width);
    assert_eq!(size.height, 14f32, "height of node {:?}. Expected {}. Actual {}", node, 14f32, size.height);
    assert_eq!(location.x, 0f32, "x of node {:?}. Expected {}. Actual {}", node, 0f32, location.x);
    assert_eq!(location.y, 0f32, "y of node {:?}. Expected {}. Actual {}", node, 0f32, location.y);
    let Layout { size, location, .. } = taffy.layout(node0);
    assert_eq!(size.width, 22f32, "width of node {:?}. Expected {}. Actual {}", node0, 22f32, size.width);
    assert_eq!(size.height, 14f32, "height of node {:?}. Expected {}. Actual {}", node0, 14f32, size.height);
    assert_eq!(location.x, 0f32, "x of node {:?}. Expected {}. Actual {}", node0, 0f32, location.x);
    assert_eq!(location.y, 0f32, "y of node {:?}. Expected {}. Actual {}", node0, 0f32, location.y);
    let Layout { size, location, .. } = taffy.layout(node00);
    assert_eq!(size.width, 0f32, "width of node {:?}. Expected {}. Actual {}", node00, 0f32, size.width);
    assert_eq!(size.height, 0f32, "height of node {:?}. Expected {}. Actual {}", node00, 0f32, size.height);
    assert_eq!(location.x, 15f32, "x of node {:?}. Expected {}. Actual {}", node00, 15f32, location.x);
    assert_eq!(location.y, 3f32, "y of node {:?}. Expected {}. Actual {}", node00, 3f32, location.y);
}

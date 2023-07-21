#[test]
fn justify_content_min_width_with_padding_child_width_greater_than_parent() {
    #[allow(unused_imports)]
    use taffy::{prelude::*, tree::Layout};
    let mut taffy = taffy::Taffy::new();
    let node000 = taffy.new_leaf(taffy::style::Style {
        align_content: Some(taffy::style::AlignContent::Stretch),
        size: taffy::geometry::Size {
            width: taffy::style::Dimension::Length(300f32),
            height: taffy::style::Dimension::Length(100f32),
        },
        ..Default::default()
    });
    let node00 = taffy.new_with_children(
        taffy::style::Style {
            align_content: Some(taffy::style::AlignContent::Stretch),
            justify_content: Some(taffy::style::JustifyContent::Center),
            min_size: taffy::geometry::Size { width: taffy::style::Dimension::Length(400f32), height: auto() },
            padding: taffy::geometry::Rect {
                left: taffy::style::LengthPercentage::Length(100f32),
                right: taffy::style::LengthPercentage::Length(100f32),
                top: zero(),
                bottom: zero(),
            },
            ..Default::default()
        },
        &[node000],
    );
    let node0 = taffy.new_with_children(
        taffy::style::Style { align_content: Some(taffy::style::AlignContent::Stretch), ..Default::default() },
        &[node00],
    );
    let node = taffy.new_with_children(
        taffy::style::Style {
            flex_direction: taffy::style::FlexDirection::Column,
            align_content: Some(taffy::style::AlignContent::Stretch),
            size: taffy::geometry::Size {
                width: taffy::style::Dimension::Length(1000f32),
                height: taffy::style::Dimension::Length(1584f32),
            },
            ..Default::default()
        },
        &[node0],
    );
    taffy.compute_layout(node, taffy::geometry::Size::MAX_CONTENT);
    println!("\nComputed tree:");
    taffy::util::print_tree(&taffy, node);
    println!();
    let Layout { size, location, .. } = taffy.layout(node);
    assert_eq!(size.width, 1000f32, "width of node {:?}. Expected {}. Actual {}", node, 1000f32, size.width);
    assert_eq!(size.height, 1584f32, "height of node {:?}. Expected {}. Actual {}", node, 1584f32, size.height);
    assert_eq!(location.x, 0f32, "x of node {:?}. Expected {}. Actual {}", node, 0f32, location.x);
    assert_eq!(location.y, 0f32, "y of node {:?}. Expected {}. Actual {}", node, 0f32, location.y);
    let Layout { size, location, .. } = taffy.layout(node0);
    assert_eq!(size.width, 1000f32, "width of node {:?}. Expected {}. Actual {}", node0, 1000f32, size.width);
    assert_eq!(size.height, 100f32, "height of node {:?}. Expected {}. Actual {}", node0, 100f32, size.height);
    assert_eq!(location.x, 0f32, "x of node {:?}. Expected {}. Actual {}", node0, 0f32, location.x);
    assert_eq!(location.y, 0f32, "y of node {:?}. Expected {}. Actual {}", node0, 0f32, location.y);
    let Layout { size, location, .. } = taffy.layout(node00);
    assert_eq!(size.width, 500f32, "width of node {:?}. Expected {}. Actual {}", node00, 500f32, size.width);
    assert_eq!(size.height, 100f32, "height of node {:?}. Expected {}. Actual {}", node00, 100f32, size.height);
    assert_eq!(location.x, 0f32, "x of node {:?}. Expected {}. Actual {}", node00, 0f32, location.x);
    assert_eq!(location.y, 0f32, "y of node {:?}. Expected {}. Actual {}", node00, 0f32, location.y);
    let Layout { size, location, .. } = taffy.layout(node000);
    assert_eq!(size.width, 300f32, "width of node {:?}. Expected {}. Actual {}", node000, 300f32, size.width);
    assert_eq!(size.height, 100f32, "height of node {:?}. Expected {}. Actual {}", node000, 100f32, size.height);
    assert_eq!(location.x, 100f32, "x of node {:?}. Expected {}. Actual {}", node000, 100f32, location.x);
    assert_eq!(location.y, 0f32, "y of node {:?}. Expected {}. Actual {}", node000, 0f32, location.y);
}

#[test]
fn wrapped_row_within_align_items_center() {
    #[allow(unused_imports)]
    use taffy::{prelude::*, tree::Layout};
    let mut taffy = taffy::Taffy::new();
    let node00 = taffy.new_leaf(taffy::style::Style {
        size: taffy::geometry::Size {
            width: taffy::style::Dimension::Length(150f32),
            height: taffy::style::Dimension::Length(80f32),
        },
        ..Default::default()
    });
    let node01 = taffy.new_leaf(taffy::style::Style {
        size: taffy::geometry::Size {
            width: taffy::style::Dimension::Length(80f32),
            height: taffy::style::Dimension::Length(80f32),
        },
        ..Default::default()
    });
    let node0 = taffy.new_with_children(
        taffy::style::Style { flex_wrap: taffy::style::FlexWrap::Wrap, ..Default::default() },
        &[node00, node01],
    );
    let node = taffy.new_with_children(
        taffy::style::Style {
            flex_direction: taffy::style::FlexDirection::Column,
            align_items: Some(taffy::style::AlignItems::Center),
            size: taffy::geometry::Size {
                width: taffy::style::Dimension::Length(200f32),
                height: taffy::style::Dimension::Length(200f32),
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
    assert_eq!(size.width, 200f32, "width of node {:?}. Expected {}. Actual {}", node, 200f32, size.width);
    assert_eq!(size.height, 200f32, "height of node {:?}. Expected {}. Actual {}", node, 200f32, size.height);
    assert_eq!(location.x, 0f32, "x of node {:?}. Expected {}. Actual {}", node, 0f32, location.x);
    assert_eq!(location.y, 0f32, "y of node {:?}. Expected {}. Actual {}", node, 0f32, location.y);
    let Layout { size, location, .. } = taffy.layout(node0);
    assert_eq!(size.width, 200f32, "width of node {:?}. Expected {}. Actual {}", node0, 200f32, size.width);
    assert_eq!(size.height, 160f32, "height of node {:?}. Expected {}. Actual {}", node0, 160f32, size.height);
    assert_eq!(location.x, 0f32, "x of node {:?}. Expected {}. Actual {}", node0, 0f32, location.x);
    assert_eq!(location.y, 0f32, "y of node {:?}. Expected {}. Actual {}", node0, 0f32, location.y);
    let Layout { size, location, .. } = taffy.layout(node00);
    assert_eq!(size.width, 150f32, "width of node {:?}. Expected {}. Actual {}", node00, 150f32, size.width);
    assert_eq!(size.height, 80f32, "height of node {:?}. Expected {}. Actual {}", node00, 80f32, size.height);
    assert_eq!(location.x, 0f32, "x of node {:?}. Expected {}. Actual {}", node00, 0f32, location.x);
    assert_eq!(location.y, 0f32, "y of node {:?}. Expected {}. Actual {}", node00, 0f32, location.y);
    let Layout { size, location, .. } = taffy.layout(node01);
    assert_eq!(size.width, 80f32, "width of node {:?}. Expected {}. Actual {}", node01, 80f32, size.width);
    assert_eq!(size.height, 80f32, "height of node {:?}. Expected {}. Actual {}", node01, 80f32, size.height);
    assert_eq!(location.x, 0f32, "x of node {:?}. Expected {}. Actual {}", node01, 0f32, location.x);
    assert_eq!(location.y, 80f32, "y of node {:?}. Expected {}. Actual {}", node01, 80f32, location.y);
}

#[test]
fn block_absolute_child_with_margin_y() {
    #[allow(unused_imports)]
    use taffy::{prelude::*, tree::Layout};
    let mut taffy = taffy::Taffy::new();
    let node0 = taffy.new_leaf(taffy::style::Style {
        position: taffy::style::Position::Absolute,
        size: taffy::geometry::Size {
            width: taffy::style::Dimension::Length(9f32),
            height: taffy::style::Dimension::Length(9f32),
        },
        margin: taffy::geometry::Rect {
            left: zero(),
            right: zero(),
            top: taffy::style::LengthPercentageAuto::Length(7f32),
            bottom: zero(),
        },
        ..Default::default()
    });
    let node1 = taffy.new_leaf(taffy::style::Style {
        position: taffy::style::Position::Absolute,
        size: taffy::geometry::Size {
            width: taffy::style::Dimension::Length(9f32),
            height: taffy::style::Dimension::Length(9f32),
        },
        margin: taffy::geometry::Rect {
            left: zero(),
            right: zero(),
            top: zero(),
            bottom: taffy::style::LengthPercentageAuto::Length(7f32),
        },
        ..Default::default()
    });
    let node2 = taffy.new_leaf(taffy::style::Style {
        position: taffy::style::Position::Absolute,
        size: taffy::geometry::Size {
            width: taffy::style::Dimension::Length(9f32),
            height: taffy::style::Dimension::Length(9f32),
        },
        margin: taffy::geometry::Rect {
            left: zero(),
            right: zero(),
            top: taffy::style::LengthPercentageAuto::Length(10f32),
            bottom: taffy::style::LengthPercentageAuto::Length(5f32),
        },
        ..Default::default()
    });
    let node = taffy.new_with_children(
        taffy::style::Style {
            display: taffy::style::Display::Block,
            size: taffy::geometry::Size {
                width: taffy::style::Dimension::Length(20f32),
                height: taffy::style::Dimension::Length(37f32),
            },
            ..Default::default()
        },
        &[node0, node1, node2],
    );
    taffy.compute_layout(node, taffy::geometry::Size::MAX_CONTENT);
    println!("\nComputed tree:");
    taffy::util::print_tree(&taffy, node);
    println!();
    let Layout { size, location, .. } = taffy.layout(node);
    assert_eq!(size.width, 20f32, "width of node {:?}. Expected {}. Actual {}", node, 20f32, size.width);
    assert_eq!(size.height, 37f32, "height of node {:?}. Expected {}. Actual {}", node, 37f32, size.height);
    assert_eq!(location.x, 0f32, "x of node {:?}. Expected {}. Actual {}", node, 0f32, location.x);
    assert_eq!(location.y, 0f32, "y of node {:?}. Expected {}. Actual {}", node, 0f32, location.y);
    let Layout { size, location, .. } = taffy.layout(node0);
    assert_eq!(size.width, 9f32, "width of node {:?}. Expected {}. Actual {}", node0, 9f32, size.width);
    assert_eq!(size.height, 9f32, "height of node {:?}. Expected {}. Actual {}", node0, 9f32, size.height);
    assert_eq!(location.x, 0f32, "x of node {:?}. Expected {}. Actual {}", node0, 0f32, location.x);
    assert_eq!(location.y, 7f32, "y of node {:?}. Expected {}. Actual {}", node0, 7f32, location.y);
    let Layout { size, location, .. } = taffy.layout(node1);
    assert_eq!(size.width, 9f32, "width of node {:?}. Expected {}. Actual {}", node1, 9f32, size.width);
    assert_eq!(size.height, 9f32, "height of node {:?}. Expected {}. Actual {}", node1, 9f32, size.height);
    assert_eq!(location.x, 0f32, "x of node {:?}. Expected {}. Actual {}", node1, 0f32, location.x);
    assert_eq!(location.y, 0f32, "y of node {:?}. Expected {}. Actual {}", node1, 0f32, location.y);
    let Layout { size, location, .. } = taffy.layout(node2);
    assert_eq!(size.width, 9f32, "width of node {:?}. Expected {}. Actual {}", node2, 9f32, size.width);
    assert_eq!(size.height, 9f32, "height of node {:?}. Expected {}. Actual {}", node2, 9f32, size.height);
    assert_eq!(location.x, 0f32, "x of node {:?}. Expected {}. Actual {}", node2, 0f32, location.x);
    assert_eq!(location.y, 10f32, "y of node {:?}. Expected {}. Actual {}", node2, 10f32, location.y);
}

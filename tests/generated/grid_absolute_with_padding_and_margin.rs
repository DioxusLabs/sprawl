#[test]
fn grid_absolute_with_padding_and_margin() {
    #[allow(unused_imports)]
    use taffy::{prelude::*, tree::Layout};
    let mut taffy = taffy::Taffy::new();
    let node0 = taffy.new_leaf(taffy::style::Style {
        position: taffy::style::Position::Absolute,
        margin: taffy::geometry::Rect {
            left: taffy::style::LengthPercentageAuto::Length(4f32),
            right: taffy::style::LengthPercentageAuto::Length(2f32),
            top: taffy::style::LengthPercentageAuto::Length(1f32),
            bottom: taffy::style::LengthPercentageAuto::Length(3f32),
        },
        inset: taffy::geometry::Rect {
            left: auto(),
            right: taffy::style::LengthPercentageAuto::Length(0f32),
            top: taffy::style::LengthPercentageAuto::Length(0f32),
            bottom: auto(),
        },
        ..Default::default()
    });
    let node1 = taffy.new_leaf(taffy::style::Style {
        position: taffy::style::Position::Absolute,
        margin: taffy::geometry::Rect {
            left: taffy::style::LengthPercentageAuto::Length(4f32),
            right: taffy::style::LengthPercentageAuto::Length(2f32),
            top: taffy::style::LengthPercentageAuto::Length(1f32),
            bottom: taffy::style::LengthPercentageAuto::Length(3f32),
        },
        inset: taffy::geometry::Rect {
            left: taffy::style::LengthPercentageAuto::Length(10f32),
            right: auto(),
            top: auto(),
            bottom: taffy::style::LengthPercentageAuto::Length(10f32),
        },
        ..Default::default()
    });
    let node2 = taffy.new_leaf(taffy::style::Style { ..Default::default() });
    let node3 = taffy.new_leaf(taffy::style::Style { ..Default::default() });
    let node4 = taffy.new_leaf(taffy::style::Style { ..Default::default() });
    let node5 = taffy.new_leaf(taffy::style::Style { ..Default::default() });
    let node6 = taffy.new_leaf(taffy::style::Style { ..Default::default() });
    let node7 = taffy.new_leaf(taffy::style::Style { ..Default::default() });
    let node8 = taffy.new_leaf(taffy::style::Style { ..Default::default() });
    let node = taffy.new_with_children(
        taffy::style::Style {
            display: taffy::style::Display::Grid,
            grid_template_rows: vec![length(40f32), length(40f32), length(40f32)],
            grid_template_columns: vec![length(40f32), length(40f32), length(40f32)],
            padding: taffy::geometry::Rect {
                left: taffy::style::LengthPercentage::Length(40f32),
                right: taffy::style::LengthPercentage::Length(20f32),
                top: taffy::style::LengthPercentage::Length(10f32),
                bottom: taffy::style::LengthPercentage::Length(30f32),
            },
            ..Default::default()
        },
        &[node0, node1, node2, node3, node4, node5, node6, node7, node8],
    );
    taffy.compute_layout(node, taffy::geometry::Size::MAX_CONTENT);
    println!("\nComputed tree:");
    taffy::util::print_tree(&taffy, node);
    println!();
    let Layout { size, location, .. } = taffy.layout(node);
    assert_eq!(size.width, 180f32, "width of node {:?}. Expected {}. Actual {}", node, 180f32, size.width);
    assert_eq!(size.height, 160f32, "height of node {:?}. Expected {}. Actual {}", node, 160f32, size.height);
    assert_eq!(location.x, 0f32, "x of node {:?}. Expected {}. Actual {}", node, 0f32, location.x);
    assert_eq!(location.y, 0f32, "y of node {:?}. Expected {}. Actual {}", node, 0f32, location.y);
    let Layout { size, location, .. } = taffy.layout(node0);
    assert_eq!(size.width, 0f32, "width of node {:?}. Expected {}. Actual {}", node0, 0f32, size.width);
    assert_eq!(size.height, 0f32, "height of node {:?}. Expected {}. Actual {}", node0, 0f32, size.height);
    assert_eq!(location.x, 178f32, "x of node {:?}. Expected {}. Actual {}", node0, 178f32, location.x);
    assert_eq!(location.y, 1f32, "y of node {:?}. Expected {}. Actual {}", node0, 1f32, location.y);
    let Layout { size, location, .. } = taffy.layout(node1);
    assert_eq!(size.width, 0f32, "width of node {:?}. Expected {}. Actual {}", node1, 0f32, size.width);
    assert_eq!(size.height, 0f32, "height of node {:?}. Expected {}. Actual {}", node1, 0f32, size.height);
    assert_eq!(location.x, 14f32, "x of node {:?}. Expected {}. Actual {}", node1, 14f32, location.x);
    assert_eq!(location.y, 147f32, "y of node {:?}. Expected {}. Actual {}", node1, 147f32, location.y);
    let Layout { size, location, .. } = taffy.layout(node2);
    assert_eq!(size.width, 40f32, "width of node {:?}. Expected {}. Actual {}", node2, 40f32, size.width);
    assert_eq!(size.height, 40f32, "height of node {:?}. Expected {}. Actual {}", node2, 40f32, size.height);
    assert_eq!(location.x, 40f32, "x of node {:?}. Expected {}. Actual {}", node2, 40f32, location.x);
    assert_eq!(location.y, 10f32, "y of node {:?}. Expected {}. Actual {}", node2, 10f32, location.y);
    let Layout { size, location, .. } = taffy.layout(node3);
    assert_eq!(size.width, 40f32, "width of node {:?}. Expected {}. Actual {}", node3, 40f32, size.width);
    assert_eq!(size.height, 40f32, "height of node {:?}. Expected {}. Actual {}", node3, 40f32, size.height);
    assert_eq!(location.x, 80f32, "x of node {:?}. Expected {}. Actual {}", node3, 80f32, location.x);
    assert_eq!(location.y, 10f32, "y of node {:?}. Expected {}. Actual {}", node3, 10f32, location.y);
    let Layout { size, location, .. } = taffy.layout(node4);
    assert_eq!(size.width, 40f32, "width of node {:?}. Expected {}. Actual {}", node4, 40f32, size.width);
    assert_eq!(size.height, 40f32, "height of node {:?}. Expected {}. Actual {}", node4, 40f32, size.height);
    assert_eq!(location.x, 120f32, "x of node {:?}. Expected {}. Actual {}", node4, 120f32, location.x);
    assert_eq!(location.y, 10f32, "y of node {:?}. Expected {}. Actual {}", node4, 10f32, location.y);
    let Layout { size, location, .. } = taffy.layout(node5);
    assert_eq!(size.width, 40f32, "width of node {:?}. Expected {}. Actual {}", node5, 40f32, size.width);
    assert_eq!(size.height, 40f32, "height of node {:?}. Expected {}. Actual {}", node5, 40f32, size.height);
    assert_eq!(location.x, 40f32, "x of node {:?}. Expected {}. Actual {}", node5, 40f32, location.x);
    assert_eq!(location.y, 50f32, "y of node {:?}. Expected {}. Actual {}", node5, 50f32, location.y);
    let Layout { size, location, .. } = taffy.layout(node6);
    assert_eq!(size.width, 40f32, "width of node {:?}. Expected {}. Actual {}", node6, 40f32, size.width);
    assert_eq!(size.height, 40f32, "height of node {:?}. Expected {}. Actual {}", node6, 40f32, size.height);
    assert_eq!(location.x, 80f32, "x of node {:?}. Expected {}. Actual {}", node6, 80f32, location.x);
    assert_eq!(location.y, 50f32, "y of node {:?}. Expected {}. Actual {}", node6, 50f32, location.y);
    let Layout { size, location, .. } = taffy.layout(node7);
    assert_eq!(size.width, 40f32, "width of node {:?}. Expected {}. Actual {}", node7, 40f32, size.width);
    assert_eq!(size.height, 40f32, "height of node {:?}. Expected {}. Actual {}", node7, 40f32, size.height);
    assert_eq!(location.x, 120f32, "x of node {:?}. Expected {}. Actual {}", node7, 120f32, location.x);
    assert_eq!(location.y, 50f32, "y of node {:?}. Expected {}. Actual {}", node7, 50f32, location.y);
    let Layout { size, location, .. } = taffy.layout(node8);
    assert_eq!(size.width, 40f32, "width of node {:?}. Expected {}. Actual {}", node8, 40f32, size.width);
    assert_eq!(size.height, 40f32, "height of node {:?}. Expected {}. Actual {}", node8, 40f32, size.height);
    assert_eq!(location.x, 40f32, "x of node {:?}. Expected {}. Actual {}", node8, 40f32, location.x);
    assert_eq!(location.y, 90f32, "y of node {:?}. Expected {}. Actual {}", node8, 90f32, location.y);
}

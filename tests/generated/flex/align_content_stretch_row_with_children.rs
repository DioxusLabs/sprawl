#[test]
#[allow(non_snake_case)]
fn align_content_stretch_row_with_children__border_box() {
    #[allow(unused_imports)]
    use taffy::{prelude::*, tree::Layout, TaffyTree};
    let mut taffy: TaffyTree<crate::TextMeasure> = TaffyTree::new();
    let node00 = taffy
        .new_leaf(taffy::style::Style {
            flex_grow: 1f32,
            flex_shrink: 1f32,
            flex_basis: taffy::style::Dimension::Percent(0f32),
            ..Default::default()
        })
        .unwrap();
    let node0 = taffy
        .new_with_children(
            taffy::style::Style {
                flex_direction: taffy::style::FlexDirection::Column,
                size: taffy::geometry::Size { width: taffy::style::Dimension::Length(50f32), height: auto() },
                ..Default::default()
            },
            &[node00],
        )
        .unwrap();
    let node1 = taffy
        .new_leaf(taffy::style::Style {
            size: taffy::geometry::Size { width: taffy::style::Dimension::Length(50f32), height: auto() },
            ..Default::default()
        })
        .unwrap();
    let node2 = taffy
        .new_leaf(taffy::style::Style {
            size: taffy::geometry::Size { width: taffy::style::Dimension::Length(50f32), height: auto() },
            ..Default::default()
        })
        .unwrap();
    let node3 = taffy
        .new_leaf(taffy::style::Style {
            size: taffy::geometry::Size { width: taffy::style::Dimension::Length(50f32), height: auto() },
            ..Default::default()
        })
        .unwrap();
    let node4 = taffy
        .new_leaf(taffy::style::Style {
            size: taffy::geometry::Size { width: taffy::style::Dimension::Length(50f32), height: auto() },
            ..Default::default()
        })
        .unwrap();
    let node = taffy
        .new_with_children(
            taffy::style::Style {
                flex_wrap: taffy::style::FlexWrap::Wrap,
                align_content: Some(taffy::style::AlignContent::Stretch),
                size: taffy::geometry::Size {
                    width: taffy::style::Dimension::Length(150f32),
                    height: taffy::style::Dimension::Length(100f32),
                },
                ..Default::default()
            },
            &[node0, node1, node2, node3, node4],
        )
        .unwrap();
    taffy.compute_layout_with_measure(node, taffy::geometry::Size::MAX_CONTENT, crate::test_measure_function).unwrap();
    println!("\nComputed tree:");
    taffy.print_tree(node);
    println!();
    let layout = taffy.layout(node).unwrap();
    let Layout { size, location, .. } = layout;
    assert_eq!(size.width, 150f32, "width of node {:?}. Expected {}. Actual {}", node, 150f32, size.width);
    assert_eq!(size.height, 100f32, "height of node {:?}. Expected {}. Actual {}", node, 100f32, size.height);
    assert_eq!(location.x, 0f32, "x of node {:?}. Expected {}. Actual {}", node, 0f32, location.x);
    assert_eq!(location.y, 0f32, "y of node {:?}. Expected {}. Actual {}", node, 0f32, location.y);
    let layout = taffy.layout(node0).unwrap();
    let Layout { size, location, .. } = layout;
    assert_eq!(size.width, 50f32, "width of node {:?}. Expected {}. Actual {}", node0, 50f32, size.width);
    assert_eq!(size.height, 50f32, "height of node {:?}. Expected {}. Actual {}", node0, 50f32, size.height);
    assert_eq!(location.x, 0f32, "x of node {:?}. Expected {}. Actual {}", node0, 0f32, location.x);
    assert_eq!(location.y, 0f32, "y of node {:?}. Expected {}. Actual {}", node0, 0f32, location.y);
    let layout = taffy.layout(node00).unwrap();
    let Layout { size, location, .. } = layout;
    assert_eq!(size.width, 50f32, "width of node {:?}. Expected {}. Actual {}", node00, 50f32, size.width);
    assert_eq!(size.height, 50f32, "height of node {:?}. Expected {}. Actual {}", node00, 50f32, size.height);
    assert_eq!(location.x, 0f32, "x of node {:?}. Expected {}. Actual {}", node00, 0f32, location.x);
    assert_eq!(location.y, 0f32, "y of node {:?}. Expected {}. Actual {}", node00, 0f32, location.y);
    let layout = taffy.layout(node1).unwrap();
    let Layout { size, location, .. } = layout;
    assert_eq!(size.width, 50f32, "width of node {:?}. Expected {}. Actual {}", node1, 50f32, size.width);
    assert_eq!(size.height, 50f32, "height of node {:?}. Expected {}. Actual {}", node1, 50f32, size.height);
    assert_eq!(location.x, 50f32, "x of node {:?}. Expected {}. Actual {}", node1, 50f32, location.x);
    assert_eq!(location.y, 0f32, "y of node {:?}. Expected {}. Actual {}", node1, 0f32, location.y);
    let layout = taffy.layout(node2).unwrap();
    let Layout { size, location, .. } = layout;
    assert_eq!(size.width, 50f32, "width of node {:?}. Expected {}. Actual {}", node2, 50f32, size.width);
    assert_eq!(size.height, 50f32, "height of node {:?}. Expected {}. Actual {}", node2, 50f32, size.height);
    assert_eq!(location.x, 100f32, "x of node {:?}. Expected {}. Actual {}", node2, 100f32, location.x);
    assert_eq!(location.y, 0f32, "y of node {:?}. Expected {}. Actual {}", node2, 0f32, location.y);
    let layout = taffy.layout(node3).unwrap();
    let Layout { size, location, .. } = layout;
    assert_eq!(size.width, 50f32, "width of node {:?}. Expected {}. Actual {}", node3, 50f32, size.width);
    assert_eq!(size.height, 50f32, "height of node {:?}. Expected {}. Actual {}", node3, 50f32, size.height);
    assert_eq!(location.x, 0f32, "x of node {:?}. Expected {}. Actual {}", node3, 0f32, location.x);
    assert_eq!(location.y, 50f32, "y of node {:?}. Expected {}. Actual {}", node3, 50f32, location.y);
    let layout = taffy.layout(node4).unwrap();
    let Layout { size, location, .. } = layout;
    assert_eq!(size.width, 50f32, "width of node {:?}. Expected {}. Actual {}", node4, 50f32, size.width);
    assert_eq!(size.height, 50f32, "height of node {:?}. Expected {}. Actual {}", node4, 50f32, size.height);
    assert_eq!(location.x, 50f32, "x of node {:?}. Expected {}. Actual {}", node4, 50f32, location.x);
    assert_eq!(location.y, 50f32, "y of node {:?}. Expected {}. Actual {}", node4, 50f32, location.y);
}

#[test]
#[allow(non_snake_case)]
fn align_content_stretch_row_with_children__content_box() {
    #[allow(unused_imports)]
    use taffy::{prelude::*, tree::Layout, TaffyTree};
    let mut taffy: TaffyTree<crate::TextMeasure> = TaffyTree::new();
    let node00 = taffy
        .new_leaf(taffy::style::Style {
            box_sizing: taffy::style::BoxSizing::ContentBox,
            flex_grow: 1f32,
            flex_shrink: 1f32,
            flex_basis: taffy::style::Dimension::Percent(0f32),
            ..Default::default()
        })
        .unwrap();
    let node0 = taffy
        .new_with_children(
            taffy::style::Style {
                box_sizing: taffy::style::BoxSizing::ContentBox,
                flex_direction: taffy::style::FlexDirection::Column,
                size: taffy::geometry::Size { width: taffy::style::Dimension::Length(50f32), height: auto() },
                ..Default::default()
            },
            &[node00],
        )
        .unwrap();
    let node1 = taffy
        .new_leaf(taffy::style::Style {
            box_sizing: taffy::style::BoxSizing::ContentBox,
            size: taffy::geometry::Size { width: taffy::style::Dimension::Length(50f32), height: auto() },
            ..Default::default()
        })
        .unwrap();
    let node2 = taffy
        .new_leaf(taffy::style::Style {
            box_sizing: taffy::style::BoxSizing::ContentBox,
            size: taffy::geometry::Size { width: taffy::style::Dimension::Length(50f32), height: auto() },
            ..Default::default()
        })
        .unwrap();
    let node3 = taffy
        .new_leaf(taffy::style::Style {
            box_sizing: taffy::style::BoxSizing::ContentBox,
            size: taffy::geometry::Size { width: taffy::style::Dimension::Length(50f32), height: auto() },
            ..Default::default()
        })
        .unwrap();
    let node4 = taffy
        .new_leaf(taffy::style::Style {
            box_sizing: taffy::style::BoxSizing::ContentBox,
            size: taffy::geometry::Size { width: taffy::style::Dimension::Length(50f32), height: auto() },
            ..Default::default()
        })
        .unwrap();
    let node = taffy
        .new_with_children(
            taffy::style::Style {
                box_sizing: taffy::style::BoxSizing::ContentBox,
                flex_wrap: taffy::style::FlexWrap::Wrap,
                align_content: Some(taffy::style::AlignContent::Stretch),
                size: taffy::geometry::Size {
                    width: taffy::style::Dimension::Length(150f32),
                    height: taffy::style::Dimension::Length(100f32),
                },
                ..Default::default()
            },
            &[node0, node1, node2, node3, node4],
        )
        .unwrap();
    taffy.compute_layout_with_measure(node, taffy::geometry::Size::MAX_CONTENT, crate::test_measure_function).unwrap();
    println!("\nComputed tree:");
    taffy.print_tree(node);
    println!();
    let layout = taffy.layout(node).unwrap();
    let Layout { size, location, .. } = layout;
    assert_eq!(size.width, 150f32, "width of node {:?}. Expected {}. Actual {}", node, 150f32, size.width);
    assert_eq!(size.height, 100f32, "height of node {:?}. Expected {}. Actual {}", node, 100f32, size.height);
    assert_eq!(location.x, 0f32, "x of node {:?}. Expected {}. Actual {}", node, 0f32, location.x);
    assert_eq!(location.y, 0f32, "y of node {:?}. Expected {}. Actual {}", node, 0f32, location.y);
    let layout = taffy.layout(node0).unwrap();
    let Layout { size, location, .. } = layout;
    assert_eq!(size.width, 50f32, "width of node {:?}. Expected {}. Actual {}", node0, 50f32, size.width);
    assert_eq!(size.height, 50f32, "height of node {:?}. Expected {}. Actual {}", node0, 50f32, size.height);
    assert_eq!(location.x, 0f32, "x of node {:?}. Expected {}. Actual {}", node0, 0f32, location.x);
    assert_eq!(location.y, 0f32, "y of node {:?}. Expected {}. Actual {}", node0, 0f32, location.y);
    let layout = taffy.layout(node00).unwrap();
    let Layout { size, location, .. } = layout;
    assert_eq!(size.width, 50f32, "width of node {:?}. Expected {}. Actual {}", node00, 50f32, size.width);
    assert_eq!(size.height, 50f32, "height of node {:?}. Expected {}. Actual {}", node00, 50f32, size.height);
    assert_eq!(location.x, 0f32, "x of node {:?}. Expected {}. Actual {}", node00, 0f32, location.x);
    assert_eq!(location.y, 0f32, "y of node {:?}. Expected {}. Actual {}", node00, 0f32, location.y);
    let layout = taffy.layout(node1).unwrap();
    let Layout { size, location, .. } = layout;
    assert_eq!(size.width, 50f32, "width of node {:?}. Expected {}. Actual {}", node1, 50f32, size.width);
    assert_eq!(size.height, 50f32, "height of node {:?}. Expected {}. Actual {}", node1, 50f32, size.height);
    assert_eq!(location.x, 50f32, "x of node {:?}. Expected {}. Actual {}", node1, 50f32, location.x);
    assert_eq!(location.y, 0f32, "y of node {:?}. Expected {}. Actual {}", node1, 0f32, location.y);
    let layout = taffy.layout(node2).unwrap();
    let Layout { size, location, .. } = layout;
    assert_eq!(size.width, 50f32, "width of node {:?}. Expected {}. Actual {}", node2, 50f32, size.width);
    assert_eq!(size.height, 50f32, "height of node {:?}. Expected {}. Actual {}", node2, 50f32, size.height);
    assert_eq!(location.x, 100f32, "x of node {:?}. Expected {}. Actual {}", node2, 100f32, location.x);
    assert_eq!(location.y, 0f32, "y of node {:?}. Expected {}. Actual {}", node2, 0f32, location.y);
    let layout = taffy.layout(node3).unwrap();
    let Layout { size, location, .. } = layout;
    assert_eq!(size.width, 50f32, "width of node {:?}. Expected {}. Actual {}", node3, 50f32, size.width);
    assert_eq!(size.height, 50f32, "height of node {:?}. Expected {}. Actual {}", node3, 50f32, size.height);
    assert_eq!(location.x, 0f32, "x of node {:?}. Expected {}. Actual {}", node3, 0f32, location.x);
    assert_eq!(location.y, 50f32, "y of node {:?}. Expected {}. Actual {}", node3, 50f32, location.y);
    let layout = taffy.layout(node4).unwrap();
    let Layout { size, location, .. } = layout;
    assert_eq!(size.width, 50f32, "width of node {:?}. Expected {}. Actual {}", node4, 50f32, size.width);
    assert_eq!(size.height, 50f32, "height of node {:?}. Expected {}. Actual {}", node4, 50f32, size.height);
    assert_eq!(location.x, 50f32, "x of node {:?}. Expected {}. Actual {}", node4, 50f32, location.x);
    assert_eq!(location.y, 50f32, "y of node {:?}. Expected {}. Actual {}", node4, 50f32, location.y);
}

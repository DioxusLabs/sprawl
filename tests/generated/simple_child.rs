#[test]
fn simple_child() {
    #[allow(unused_imports)]
    use taffy::{prelude::*, tree::Layout};
    let mut taffy = taffy::Taffy::new();
    let node000 = taffy.new_leaf(taffy::style::Style {
        size: taffy::geometry::Size {
            width: taffy::style::Dimension::Length(10f32),
            height: taffy::style::Dimension::Length(10f32),
        },
        ..Default::default()
    });
    let node00 = taffy.new_with_children(
        taffy::style::Style {
            size: taffy::geometry::Size {
                width: taffy::style::Dimension::Length(10f32),
                height: taffy::style::Dimension::Length(10f32),
            },
            ..Default::default()
        },
        &[node000],
    );
    let node010 = taffy.new_leaf(taffy::style::Style {
        align_self: Some(taffy::style::AlignSelf::Center),
        size: taffy::geometry::Size {
            width: taffy::style::Dimension::Length(10f32),
            height: taffy::style::Dimension::Length(10f32),
        },
        ..Default::default()
    });
    let node011 = taffy.new_leaf(taffy::style::Style {
        align_self: Some(taffy::style::AlignSelf::Center),
        size: taffy::geometry::Size {
            width: taffy::style::Dimension::Length(10f32),
            height: taffy::style::Dimension::Length(10f32),
        },
        ..Default::default()
    });
    let node01 = taffy.new_with_children(taffy::style::Style { ..Default::default() }, &[node010, node011]);
    let node0 = taffy.new_with_children(
        taffy::style::Style {
            size: taffy::geometry::Size {
                width: taffy::style::Dimension::Percent(1f32),
                height: taffy::style::Dimension::Percent(1f32),
            },
            ..Default::default()
        },
        &[node00, node01],
    );
    let node = taffy.new_with_children(
        taffy::style::Style {
            size: taffy::geometry::Size {
                width: taffy::style::Dimension::Length(100f32),
                height: taffy::style::Dimension::Length(100f32),
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
    assert_eq!(size.width, 100f32, "width of node {:?}. Expected {}. Actual {}", node, 100f32, size.width);
    assert_eq!(size.height, 100f32, "height of node {:?}. Expected {}. Actual {}", node, 100f32, size.height);
    assert_eq!(location.x, 0f32, "x of node {:?}. Expected {}. Actual {}", node, 0f32, location.x);
    assert_eq!(location.y, 0f32, "y of node {:?}. Expected {}. Actual {}", node, 0f32, location.y);
    let Layout { size, location, .. } = taffy.layout(node0);
    assert_eq!(size.width, 100f32, "width of node {:?}. Expected {}. Actual {}", node0, 100f32, size.width);
    assert_eq!(size.height, 100f32, "height of node {:?}. Expected {}. Actual {}", node0, 100f32, size.height);
    assert_eq!(location.x, 0f32, "x of node {:?}. Expected {}. Actual {}", node0, 0f32, location.x);
    assert_eq!(location.y, 0f32, "y of node {:?}. Expected {}. Actual {}", node0, 0f32, location.y);
    let Layout { size, location, .. } = taffy.layout(node00);
    assert_eq!(size.width, 10f32, "width of node {:?}. Expected {}. Actual {}", node00, 10f32, size.width);
    assert_eq!(size.height, 10f32, "height of node {:?}. Expected {}. Actual {}", node00, 10f32, size.height);
    assert_eq!(location.x, 0f32, "x of node {:?}. Expected {}. Actual {}", node00, 0f32, location.x);
    assert_eq!(location.y, 0f32, "y of node {:?}. Expected {}. Actual {}", node00, 0f32, location.y);
    let Layout { size, location, .. } = taffy.layout(node000);
    assert_eq!(size.width, 10f32, "width of node {:?}. Expected {}. Actual {}", node000, 10f32, size.width);
    assert_eq!(size.height, 10f32, "height of node {:?}. Expected {}. Actual {}", node000, 10f32, size.height);
    assert_eq!(location.x, 0f32, "x of node {:?}. Expected {}. Actual {}", node000, 0f32, location.x);
    assert_eq!(location.y, 0f32, "y of node {:?}. Expected {}. Actual {}", node000, 0f32, location.y);
    let Layout { size, location, .. } = taffy.layout(node01);
    assert_eq!(size.width, 20f32, "width of node {:?}. Expected {}. Actual {}", node01, 20f32, size.width);
    assert_eq!(size.height, 100f32, "height of node {:?}. Expected {}. Actual {}", node01, 100f32, size.height);
    assert_eq!(location.x, 10f32, "x of node {:?}. Expected {}. Actual {}", node01, 10f32, location.x);
    assert_eq!(location.y, 0f32, "y of node {:?}. Expected {}. Actual {}", node01, 0f32, location.y);
    let Layout { size, location, .. } = taffy.layout(node010);
    assert_eq!(size.width, 10f32, "width of node {:?}. Expected {}. Actual {}", node010, 10f32, size.width);
    assert_eq!(size.height, 10f32, "height of node {:?}. Expected {}. Actual {}", node010, 10f32, size.height);
    assert_eq!(location.x, 0f32, "x of node {:?}. Expected {}. Actual {}", node010, 0f32, location.x);
    assert_eq!(location.y, 45f32, "y of node {:?}. Expected {}. Actual {}", node010, 45f32, location.y);
    let Layout { size, location, .. } = taffy.layout(node011);
    assert_eq!(size.width, 10f32, "width of node {:?}. Expected {}. Actual {}", node011, 10f32, size.width);
    assert_eq!(size.height, 10f32, "height of node {:?}. Expected {}. Actual {}", node011, 10f32, size.height);
    assert_eq!(location.x, 10f32, "x of node {:?}. Expected {}. Actual {}", node011, 10f32, location.x);
    assert_eq!(location.y, 45f32, "y of node {:?}. Expected {}. Actual {}", node011, 45f32, location.y);
}

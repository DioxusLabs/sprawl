#[test]
fn absolute_layout_align_items_and_justify_content_flex_end() {
    let mut stretch = stretch::Stretch::new();
    let node0 = stretch
        .new_node(
            stretch::style::Style {
                position_type: stretch::style::PositionType::Absolute,
                size: stretch::geometry::Size {
                    width: stretch::style::Dimension::Points(60f32),
                    height: stretch::style::Dimension::Points(40f32),
                    ..Default::default()
                },
                ..Default::default()
            },
            vec![],
        )
        .unwrap();
    let node = stretch
        .new_node(
            stretch::style::Style {
                align_items: stretch::style::AlignItems::FlexEnd,
                justify_content: stretch::style::JustifyContent::FlexEnd,
                size: stretch::geometry::Size {
                    width: stretch::style::Dimension::Points(110f32),
                    height: stretch::style::Dimension::Points(100f32),
                    ..Default::default()
                },
                ..Default::default()
            },
            vec![node0],
        )
        .unwrap();
    stretch.compute_layout(node, stretch::geometry::Size::undefined()).unwrap();
    assert_eq!(stretch.layout(node).unwrap().size.width, 110f32);
    assert_eq!(stretch.layout(node).unwrap().size.height, 100f32);
    assert_eq!(stretch.layout(node).unwrap().location.x, 0f32);
    assert_eq!(stretch.layout(node).unwrap().location.y, 0f32);
    assert_eq!(stretch.layout(node0).unwrap().size.width, 60f32);
    assert_eq!(stretch.layout(node0).unwrap().size.height, 40f32);
    assert_eq!(stretch.layout(node0).unwrap().location.x, 50f32);
    assert_eq!(stretch.layout(node0).unwrap().location.y, 60f32);
}

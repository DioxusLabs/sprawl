pub fn compute() {
    #[allow(unused_imports)]
    use taffy::prelude::*;
    let mut taffy = taffy::Taffy::new();
    let node0 = taffy.new_leaf(taffy::style::Style { ..Default::default() }).unwrap();
    let node1 = taffy.new_leaf(taffy::style::Style { ..Default::default() }).unwrap();
    let node2 = taffy.new_leaf(taffy::style::Style { ..Default::default() }).unwrap();
    let node3 = taffy.new_leaf(taffy::style::Style { ..Default::default() }).unwrap();
    let node4 = taffy.new_leaf(taffy::style::Style { ..Default::default() }).unwrap();
    let node5 = taffy.new_leaf(taffy::style::Style { ..Default::default() }).unwrap();
    let node6 = taffy.new_leaf(taffy::style::Style { ..Default::default() }).unwrap();
    let node7 = taffy.new_leaf(taffy::style::Style { ..Default::default() }).unwrap();
    let node8 = taffy.new_leaf(taffy::style::Style { ..Default::default() }).unwrap();
    let node = taffy
        .new_with_children(
            taffy::style::Style {
                display: taffy::style::Display::Grid,
                grid_template_rows: vec![
                    taffy::style::TrackSizingFunction::Fixed(taffy::style::Dimension::Points(40f32)),
                    taffy::style::TrackSizingFunction::Fixed(taffy::style::Dimension::Points(40f32)),
                    taffy::style::TrackSizingFunction::Fixed(taffy::style::Dimension::Points(40f32)),
                ],
                grid_template_columns: vec![
                    taffy::style::TrackSizingFunction::Fixed(taffy::style::Dimension::Points(40f32)),
                    taffy::style::TrackSizingFunction::MinMax {
                        min: taffy::style::MinTrackSizingFunction::Fixed(taffy::style::Dimension::Points(20f32)),
                        max: taffy::style::MaxTrackSizingFunction::Fixed(taffy::style::Dimension::Points(40f32)),
                    },
                    taffy::style::TrackSizingFunction::Fixed(taffy::style::Dimension::Points(40f32)),
                ],
                size: taffy::geometry::Size { width: taffy::style::Dimension::Points(140f32), ..Size::auto() },
                ..Default::default()
            },
            &[node0, node1, node2, node3, node4, node5, node6, node7, node8],
        )
        .unwrap();
    taffy.compute_layout(node, taffy::geometry::Size::MAX_CONTENT).unwrap();
}

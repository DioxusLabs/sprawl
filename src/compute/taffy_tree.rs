//! Computation specific for the default `Taffy` tree implementation

use crate::compute::{leaf, HiddenAlgorithm};
use crate::geometry::{Point, Size};
use crate::style::{AvailableSpace, Display};
use crate::tree::{Layout, LayoutTree, NodeId, RunMode, SizeAndBaselines, SizingMode, Taffy, TaffyError};
use crate::util::sys::round;

#[cfg(feature = "flexbox")]
use super::FlexboxAlgorithm;

#[cfg(feature = "grid")]
use super::CssGridAlgorithm;

/// Updates the stored layout of the provided `node` and its children
pub(crate) fn compute_layout(
    taffy: &mut Taffy,
    root: NodeId,
    available_space: Size<AvailableSpace>,
) -> Result<(), TaffyError> {
    // Recursively compute node layout
    let size_and_baselines = perform_node_layout(
        taffy,
        root,
        Size::NONE,
        available_space.into_options(),
        available_space,
        SizingMode::InherentSize,
    );

    let layout = Layout { order: 0, size: size_and_baselines.size, location: Point::ZERO };
    *taffy.layout_mut(root) = layout;

    // If rounding is enabled, recursively round the layout's of this node and all children
    if taffy.config.use_rounding {
        round_layout(taffy, root, 0.0, 0.0);
    }

    Ok(())
}

/// Perform full layout on a node. Chooses which algorithm to use based on the `display` property.
pub(crate) fn perform_node_layout(
    tree: &mut Taffy,
    node: NodeId,
    known_dimensions: Size<Option<f32>>,
    parent_size: Size<Option<f32>>,
    available_space: Size<AvailableSpace>,
    sizing_mode: SizingMode,
) -> SizeAndBaselines {
    compute_node_layout(tree, node, known_dimensions, parent_size, available_space, RunMode::PeformLayout, sizing_mode)
}

/// Measure a node's size. Chooses which algorithm to use based on the `display` property.
pub(crate) fn measure_node_size(
    tree: &mut Taffy,
    node: NodeId,
    known_dimensions: Size<Option<f32>>,
    parent_size: Size<Option<f32>>,
    available_space: Size<AvailableSpace>,
    sizing_mode: SizingMode,
) -> Size<f32> {
    compute_node_layout(tree, node, known_dimensions, parent_size, available_space, RunMode::ComputeSize, sizing_mode)
        .size
}

/// Updates the stored layout of the provided `node` and its children
fn compute_node_layout(
    tree: &mut Taffy,
    node: NodeId,
    known_dimensions: Size<Option<f32>>,
    parent_size: Size<Option<f32>>,
    available_space: Size<AvailableSpace>,
    run_mode: RunMode,
    sizing_mode: SizingMode,
) -> SizeAndBaselines {
    #[cfg(any(feature = "debug", feature = "profile"))]
    NODE_LOGGER.push_node(node);
    #[cfg(feature = "debug")]
    println!();

    let node_key = node.into();
    let has_children = !tree.children[node_key].is_empty();

    // First we check if we have a cached result for the given input
    let cache_run_mode = if !has_children { RunMode::PeformLayout } else { run_mode };
    if let Some(cached_size_and_baselines) =
        tree.nodes[node_key].cache.get(known_dimensions, available_space, cache_run_mode)
    {
        #[cfg(feature = "debug")]
        NODE_LOGGER.labelled_debug_log("CACHE", cached_size_and_baselines.size);
        #[cfg(feature = "debug")]
        debug_log_node(known_dimensions, parent_size, available_space, run_mode, sizing_mode);
        #[cfg(any(feature = "debug", feature = "profile"))]
        NODE_LOGGER.pop_node();
        return cached_size_and_baselines;
    }

    #[cfg(feature = "debug")]
    debug_log_node(known_dimensions, parent_size, available_space, run_mode, sizing_mode);

    /// Inlined function generic over the LayoutAlgorithm to reduce code duplication
    #[inline(always)]
    fn perform_computations<Algorithm: LayoutAlgorithm>(
        tree: &mut impl LayoutTree,
        node: NodeId,
        known_dimensions: Size<Option<f32>>,
        parent_size: Size<Option<f32>>,
        available_space: Size<AvailableSpace>,
        run_mode: RunMode,
        sizing_mode: SizingMode,
    ) -> SizeAndBaselines {
        #[cfg(feature = "debug")]
        NODE_LOGGER.log(Algorithm::NAME);

        match run_mode {
            RunMode::PeformLayout => {
                Algorithm::perform_layout(tree, node, known_dimensions, parent_size, available_space, sizing_mode)
            }
            RunMode::ComputeSize => {
                Algorithm::measure_size(tree, node, known_dimensions, parent_size, available_space, sizing_mode).into()
            }
        }
    }

    let display_mode = tree.nodes[node_key].style.display;
    let computed_size_and_baselines = match (display_mode, has_children) {
        (Display::None, _) => perform_computations::<HiddenAlgorithm>(
            tree,
            node,
            known_dimensions,
            parent_size,
            available_space,
            run_mode,
            sizing_mode,
        ),
        #[cfg(feature = "flexbox")]
        (Display::Flex, true) => perform_computations::<FlexboxAlgorithm>(
            tree,
            node,
            known_dimensions,
            parent_size,
            available_space,
            run_mode,
            sizing_mode,
        ),
        #[cfg(feature = "grid")]
        (Display::Grid, true) => perform_computations::<CssGridAlgorithm>(
            tree,
            node,
            known_dimensions,
            parent_size,
            available_space,
            run_mode,
            sizing_mode,
        ),
        (_, false) => match run_mode {
            RunMode::PeformLayout => leaf::perform_layout(
                &tree.nodes[node_key].style,
                tree.nodes[node_key].needs_measure.then(|| &tree.measure_funcs[node_key]),
                known_dimensions,
                parent_size,
                available_space,
                sizing_mode,
            ),
            RunMode::ComputeSize => leaf::measure_size(
                &tree.nodes[node_key].style,
                tree.nodes[node_key].needs_measure.then(|| &tree.measure_funcs[node_key]),
                known_dimensions,
                parent_size,
                available_space,
                sizing_mode,
            )
            .into(),
        },
    };

    // Cache result
    tree.nodes[node_key].cache.store(known_dimensions, available_space, cache_run_mode, computed_size_and_baselines);

    #[cfg(feature = "debug")]
    NODE_LOGGER.labelled_debug_log("RESULT", computed_size_and_baselines.size);
    #[cfg(any(feature = "debug", feature = "profile"))]
    NODE_LOGGER.pop_node();

    computed_size_and_baselines
}

/// Rounds the calculated [`Layout`] to exact pixel values
/// In order to ensure that no gaps in the layout are introduced we:
///   - Always round based on the absolute coordinates rather than parent-relative coordinates
///   - Compute width/height by first rounding the top/bottom/left/right and then computing the difference
///     rather than rounding the width/height directly
///
/// See <https://github.com/facebook/yoga/commit/aa5b296ac78f7a22e1aeaf4891243c6bb76488e2> for more context
fn round_layout(tree: &mut impl LayoutTree, node: NodeId, abs_x: f32, abs_y: f32) {
    let layout = tree.layout_mut(node);
    let abs_x = abs_x + layout.location.x;
    let abs_y = abs_y + layout.location.y;

    layout.location.x = round(layout.location.x);
    layout.location.y = round(layout.location.y);
    layout.size.width = round(abs_x + layout.size.width) - round(abs_x);
    layout.size.height = round(abs_y + layout.size.height) - round(abs_y);

    let child_count = tree.child_count(node);
    for index in 0..child_count {
        let child = tree.child(node, index);
        round_layout(tree, child, abs_x, abs_y);
    }
}

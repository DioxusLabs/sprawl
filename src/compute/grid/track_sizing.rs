use super::placement::TrackCounts;
use crate::geometry::{Line, Size};
use crate::layout::{AvailableSpace, RunMode, SizingMode};
use crate::math::MaybeMath;
use crate::node::Node;
use crate::prelude::LayoutTree;
use crate::resolve::{MaybeResolve, ResolveOrZero};
use crate::style::{AlignContent, Dimension, MaxTrackSizingFunction, MinTrackSizingFunction, Style};
// use super::AbsoluteAxis;
use super::types::{GridAxis, GridItem, GridTrack};
use crate::sys::{f32_max, f32_min};
use core::cmp::Ordering;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(in super::super) enum AvailableSpaceMode {
    Estimates,
    OtherAxisSizes,
}

/// Takes an axis, and a list of grid items sorted firstly by whether they cross a flex track
/// in the specified axis (items that don't cross a flex track first) and then by the number
/// of tracks they cross in specified axis (ascending order).
struct ItemBatcher<'a> {
    remaining_items: &'a [GridItem],
    axis: GridAxis,
    current_span: u16,
    current_is_flex: bool,
}

impl<'a> ItemBatcher<'a> {
    fn new(items: &'a [GridItem], axis: GridAxis) -> Self {
        ItemBatcher { remaining_items: items, axis, current_span: 0, current_is_flex: false }
    }
}

impl<'a> Iterator for ItemBatcher<'a> {
    type Item = (&'a [GridItem], bool);

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_is_flex {
            return None;
        }

        let next_index_offset = self.remaining_items.iter().position(|item: &GridItem| {
            item.crosses_flexible_track(self.axis) || item.span(self.axis) > self.current_span
        })?;

        let item = &self.remaining_items[next_index_offset];
        self.current_span = item.span(self.axis);
        self.current_is_flex = item.crosses_flexible_track(self.axis);

        let (batch, rest) = self.remaining_items.split_at(next_index_offset);
        self.remaining_items = batch;

        Some((rest, self.current_is_flex))
    }
}

/// To make track sizing efficient we want to order tracks
/// Here a placement is either a Line<i16> representing a row-start/row-end or a column-start/column-end
#[inline(always)]
pub(in super::super) fn cmp_by_cross_flex_then_span_then_start(
    axis: GridAxis,
) -> impl FnMut(&GridItem, &GridItem) -> Ordering {
    move |item_a: &GridItem, item_b: &GridItem| -> Ordering {
        match (item_a.crosses_flexible_track(axis), item_b.crosses_flexible_track(axis)) {
            (false, true) => Ordering::Less,
            (true, false) => Ordering::Greater,
            _ => {
                let placement_a = item_a.placement(axis);
                let placement_b = item_b.placement(axis);
                let a_span = placement_a.end - placement_a.start;
                let b_span = placement_b.end - placement_b.start;
                match a_span.cmp(&b_span) {
                    Ordering::Less => Ordering::Less,
                    Ordering::Greater => Ordering::Greater,
                    Ordering::Equal => placement_a.start.cmp(&placement_b.start),
                }
            }
        }
    }
}

pub(in super::super) fn compute_alignment_gutter_adjustment(
    alignment: AlignContent,
    available_space: AvailableSpace,
    get_track_size_estimate: impl Fn(&GridTrack, AvailableSpace) -> Option<f32>,
    tracks: &[GridTrack],
) -> f32 {
    if alignment.inner_gutter_weight() > 0 && available_space.is_definite() && tracks.len() > 1 {
        let inner_available_space = tracks
            .iter()
            .map(|track| get_track_size_estimate(track, available_space))
            .sum::<Option<f32>>()
            .map(|track_size_sum| f32_max(0.0, available_space.unwrap() - track_size_sum))
            .unwrap_or(0.0);

        let weighted_track_count = (((tracks.len() - 3) / 2) * alignment.inner_gutter_weight() as usize)
            + (2 * alignment.outer_gutter_weight() as usize);

        (inner_available_space / weighted_track_count as f32) * alignment.inner_gutter_weight() as f32
    } else {
        0.0
    }
}

/// Convert origin-zero coordinates track placement in grid track vector indexes
pub(in super::super) fn resolve_item_track_indexes(
    items: &mut Vec<GridItem>,
    column_counts: TrackCounts,
    row_counts: TrackCounts,
) {
    for item in items {
        item.column_indexes = item.column.map(|oz_index| column_counts.oz_line_to_grid_track_vec_index(oz_index));
        item.row_indexes = item.row.map(|oz_index| row_counts.oz_line_to_grid_track_vec_index(oz_index));
    }
}

/// Determine (in each axis) whether the item crosses any flexible tracks
pub(in super::super) fn determine_if_item_crosses_flexible_tracks(
    items: &mut Vec<GridItem>,
    columns: &Vec<GridTrack>,
    rows: &Vec<GridTrack>,
) {
    for item in items {
        item.crosses_flexible_column =
            (item.column_indexes.start..=item.column_indexes.end).any(|i| columns[i as usize].is_flexible());
        item.crosses_flexible_row =
            (item.row_indexes.start..=item.row_indexes.end).any(|i| rows[i as usize].is_flexible());
    }
}

pub(in super::super) fn track_sizing_algorithm<Tree, MeasureFunc>(
    tree: &mut Tree,
    available_space: Size<AvailableSpace>,
    available_grid_space: Size<AvailableSpace>,
    available_space_mode: AvailableSpaceMode,
    axis: GridAxis,
    columns: &mut [GridTrack],
    container_style: &Style,
    rows: &mut [GridTrack],
    items: &mut [GridItem],
    measure_node: MeasureFunc,
) where
    Tree: LayoutTree,
    MeasureFunc: Fn(&mut Tree, Node, Size<Option<f32>>, Size<AvailableSpace>, RunMode, SizingMode) -> Size<f32>,
{
    let get_track_size_estimate = match available_space_mode {
        AvailableSpaceMode::Estimates => |track: &GridTrack, available_space: AvailableSpace| {
            track.max_track_sizing_function.definite_value(available_space)
        },
        AvailableSpaceMode::OtherAxisSizes => |track: &GridTrack, _| Some(track.base_size),
    };

    #[inline(always)]
    fn get_column_placement(item: &GridItem) -> Line<u16> {
        item.column_indexes
    }
    #[inline(always)]
    fn get_row_placement(item: &GridItem) -> Line<u16> {
        item.row_indexes
    }

    // The track sizing algorithm is generic over which axis it operates over, but it is performance sensitive
    // we don't want to perform a dynamic lookup every time we access a property, so we instead pass in getter functions
    // under the assumption that the inner function will be monomorphised, and they'll be inlined
    match axis {
        GridAxis::Inline => {
            #[inline(always)]
            fn get_column_cross_flex_track(item: &GridItem) -> bool {
                item.crosses_flexible_column
            }
            track_sizing_algorithm_inner(
                tree,
                axis,
                available_space,
                available_grid_space,
                container_style.min_size.width.get_absolute(),
                container_style.max_size.width.get_absolute(),
                columns,
                rows,
                items,
                container_style,
                get_track_size_estimate,
                get_column_placement,
                get_row_placement,
                get_column_cross_flex_track,
                measure_node,
            );
        }
        GridAxis::Block => {
            #[inline(always)]
            fn get_row_crosses_flex_track(item: &GridItem) -> bool {
                item.crosses_flexible_row
            }
            track_sizing_algorithm_inner(
                tree,
                axis,
                available_space,
                available_grid_space,
                container_style.min_size.height.get_absolute(),
                container_style.max_size.height.get_absolute(),
                rows,
                columns,
                items,
                container_style,
                get_track_size_estimate,
                get_row_placement,
                get_column_placement,
                get_row_crosses_flex_track,
                measure_node,
            );
        }
    }
}

/// Track sizing algorithm
/// Note: Gutters are treated as empty fixed-size tracks for the purpose of the track sizing algorithm.
pub(in super::super) fn track_sizing_algorithm_inner<Tree, MeasureFunc>(
    tree: &mut Tree,
    axis: GridAxis,
    available_space: Size<AvailableSpace>,
    available_grid_space: Size<AvailableSpace>,
    axis_min_size: Option<f32>,
    axis_max_size: Option<f32>,
    axis_tracks: &mut [GridTrack],
    other_axis_tracks: &mut [GridTrack],
    items: &mut [GridItem],
    style: &Style,
    get_track_size_estimate: impl Fn(&GridTrack, AvailableSpace) -> Option<f32>,
    get_item_placement: impl Fn(&GridItem) -> Line<u16>,
    get_other_axis_placement: impl Fn(&GridItem) -> Line<u16>,
    get_crosses_flex_track: impl Fn(&GridItem) -> bool,
    measure_node: MeasureFunc,
) where
    Tree: LayoutTree,
    MeasureFunc: Fn(&mut Tree, Node, Size<Option<f32>>, Size<AvailableSpace>, RunMode, SizingMode) -> Size<f32>,
{
    // 11.4 Initialise Track sizes
    // Initialize each track’s base size and growth limit.

    let last_track_idx = axis_tracks.len() - 1;

    // First and last grid lines are always zero-sized.
    axis_tracks[0].base_size = 0.0;
    axis_tracks[0].growth_limit = 0.0;
    axis_tracks[last_track_idx].base_size = 0.0;
    axis_tracks[last_track_idx].growth_limit = 0.0;

    let all_but_first_and_last = 1..last_track_idx;
    for track in axis_tracks[all_but_first_and_last].iter_mut() {
        // For each track, if the track’s min track sizing function is:
        // - A fixed sizing function
        //     Resolve to an absolute length and use that size as the track’s initial base size.
        //     Note: Indefinite lengths cannot occur, as they’re treated as auto.
        // - An intrinsic sizing function
        //     Use an initial base size of zero.
        track.base_size = track.min_track_sizing_function.definite_value(available_space.get(axis)).unwrap_or(0.0);

        // For each track, if the track’s max track sizing function is:
        // - A fixed sizing function
        //     Resolve to an absolute length and use that size as the track’s initial growth limit.
        // - An intrinsic sizing function
        //     Use an initial growth limit of infinity.
        // - A flexible sizing function
        //     Use an initial growth limit of infinity.
        track.growth_limit =
            track.max_track_sizing_function.definite_value(available_space.get(axis)).unwrap_or(f32::INFINITY);

        // In all cases, if the growth limit is less than the base size, increase the growth limit to match the base size.
        if track.growth_limit < track.base_size {
            track.growth_limit = track.base_size;
        }
    }

    // If all tracks have base_size = growth_limit, then skip the rest of this function.
    // Note: this can only happen both track sizing function have the same fixed track sizing function
    if axis_tracks.iter().all(|track| track.base_size == track.growth_limit) {
        return;
    }

    // Pre-computations for 11.5 Resolve Intrinsic Track Sizes

    // The track sizing algorithm requires us to iterate through the items in ascendeding order of the number of
    // tracks they span (first items that span 1 track, then items that span 2 tracks, etc).
    // To avoid having to do multiple iterations of the items, we pre-sort them into this order.
    items.sort_by(cmp_by_cross_flex_then_span_then_start(axis));

    // Compute an additional amount to add to each spanned gutter when computing item's estimated size in the
    // in the opposite axis based on the alignment, container size, and estimated track sizes in that axis
    let gutter_alignment_adjustment = compute_alignment_gutter_adjustment(
        style.grid_align_content(axis.other()),
        available_space.get(axis.other()),
        &get_track_size_estimate,
        &other_axis_tracks,
    );

    // 11.5b Resolve Intrinsic Track Sizes

    // Step 1. Shim baseline-aligned items so their intrinsic size contributions reflect their baseline alignment.
    // TODO: we do not yet support baseline alignment for CSS Grid

    // Step 2. We skip Step 2 as it is noted that:
    //
    //    This step is a simplification of the steps below for handling spanning items, and should yield
    //    the same behavior as running those instructions on items with a span of 1.
    //
    // We choose this alternative of running Step 3 on items with a span of 1 as we need to write the code for this anyway.

    // Step 3 and Step 4
    // 3. Iterate over items that don't cross a flex track. Items should have already been sorted in ascending order
    // of the number of tracks they cross.
    // 4. Next, repeat the previous step instead considering (together, rather than grouped by span size) all items
    // that do span a track with a flexible sizing function while

    // TODO: be smarter about only computing these when they are required
    // let compute_item_sizes = |item: &GridItem| {
    //     let item_other_axis_size: Option<f32> = {
    //         (&other_axis_tracks)[item.track_range_excluding_lines(axis)]
    //             .iter()
    //             .map(|track| get_track_size_estimate(track, available_space.get(axis.other())))
    //             .sum::<Option<f32>>()
    //     };

    //     let min_content_size = measure_node(
    //         tree,
    //         item.node,
    //         Size { width: None, height: item_other_axis_size },
    //         Size::MIN_CONTENT,
    //         RunMode::ComputeSize,
    //         SizingMode::ContentSize,
    //     );

    //     // TODO: resolve styles here
    //     let style = tree.style(item.node);
    //     let minimum_contributions = style.size.width;

    //     let max_content_size = measure_node(
    //         tree,
    //         item.node,
    //         Size { width: None, height: item_other_axis_size },
    //         Size::MAX_CONTENT,
    //         RunMode::ComputeSize,
    //         SizingMode::ContentSize,
    //     );

    //     (min_content_size, max_content_size)
    // };

    // let batched_item_iterator = ItemBatcher::new(items, axis);
    // for (items, is_flex) in batched_item_iterator {
    //     for item in items.iter() {
    //         // let placement = get_item_placement(item);

    //         // distribute_item_space_to_base_size(
    //         //     min_content_size.get(axis),
    //         //     &mut axis_tracks[item.track_range_excluding_lines(axis)],
    //         //     move |track| track.min_track_sizing_function.definite_value(available_space.get(axis)).is_none(),
    //         //     IntrinsicContributionType::Minimum,
    //         // );
    //     }
    // }

    // Step 5.
    // If any track still has an infinite growth limit (because, for example, it had no items placed in it or it is a flexible track),
    // set its growth limit to its base size. (NOTE: this step is super-important to ensure that the "Maximise Tracks" step doesn't affect flexible tracks
    axis_tracks
        .iter_mut()
        .filter(|track| track.growth_limit == f32::INFINITY)
        .for_each(|track| track.growth_limit = track.base_size);

    // 11.6 Maximise Tracks
    // Distributes free space (if any) to tracks with FINITE growth limits, up to their limits.
    let used_space: f32 = axis_tracks.iter().map(|track| track.base_size).sum();
    let free_space = available_grid_space.get(axis).compute_free_space(used_space);
    if free_space == f32::INFINITY {
        axis_tracks.iter_mut().for_each(|track| track.base_size = track.growth_limit);
    } else if free_space > 0.0 {
        distribute_space_up_to_limits(free_space, axis_tracks, |_| true);
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum IntrinsicContributionType {
    Minimum,
    // MinContent,
    Maximum,
    // MaxContent,
}

fn flush_planned_increases(tracks: &mut [GridTrack]) {
    for track in tracks {
        if track.base_size_planned_increase > track.base_size {
            track.base_size = track.base_size + track.base_size_planned_increase;
        }
    }
}

fn distribute_space_up_to_limits(
    space_to_distribute: f32,
    tracks: &mut [GridTrack],
    track_is_affected: impl Fn(&GridTrack) -> bool,
) {
    // Define a small constant to avoid infinite loops due to rounding errors. Rather than stopping distributing
    // extra space when it gets to exactly zero, we will stop when it falls below this amount
    const THRESHOLD: f32 = 0.000001;

    let mut space_to_distribute = space_to_distribute;
    while space_to_distribute > THRESHOLD {
        let number_of_growable_tracks = tracks
            .iter()
            .filter(|track| track.base_size < track.growth_limit)
            .filter(|track| track_is_affected(track))
            .count();
        if number_of_growable_tracks == 0 {
            break;
        }

        // Compute item-incurred increase for this iteration
        let min_increase_limit = tracks
            .iter()
            .filter(|track| track.base_size < track.growth_limit)
            .filter(|track| track_is_affected(track))
            .map(|track| track.growth_limit - track.base_size)
            .min_by(|a, b| a.total_cmp(b))
            .unwrap(); // We will never pass an empty track list to this function
        let item_incurred_increase =
            f32_min(min_increase_limit, space_to_distribute / number_of_growable_tracks as f32);

        for track in tracks
            .iter_mut()
            .filter(|track| track_is_affected(track))
            .filter(|track| track.base_size < track.growth_limit)
        {
            track.base_size += item_incurred_increase;
        }

        space_to_distribute -= item_incurred_increase * number_of_growable_tracks as f32;
    }
}

/// 11.5.1. Distributing Extra Space Across Spanned Tracks
/// https://www.w3.org/TR/css-grid-1/#extra-space
// TODO: Actually add planned increase to base size
fn distribute_item_space_to_base_size(
    space: f32,
    axis: GridAxis,
    tracks: &mut [GridTrack],
    track_is_affected: impl Fn(&GridTrack) -> bool,
    intrinsic_contribution_type: IntrinsicContributionType,
) {
    // Skip this distribution if there are no affected tracks to distribute space to.
    if tracks.iter().filter(|track| track_is_affected(track)).count() == 0 {
        return;
    }

    // 1. Find the space to distribute
    let track_sizes: f32 = tracks.iter().map(|track| track.base_size).sum();
    let mut extra_space: f32 = f32_max(0.0, space - track_sizes);

    // 2. Distribute space up to limits:
    // Note: there are two exit conditions to this loop:
    //   - We run out of space to distribute (extra_space falls below THRESHOLD)
    //   - We run out of growable tracks to distribute to

    // Define a small constant to avoid infinite loops due to rounding errors. Rather than stopping distributing
    // extra space when it gets to exactly zero, we will stop when it falls below this amount
    const THRESHOLD: f32 = 0.000001;

    while extra_space > THRESHOLD {
        let number_of_growable_tracks = tracks
            .iter()
            .filter(|track| track.base_size /*+ track.base_size_planned_increase*/ < track.growth_limit)
            .filter(|track| track_is_affected(track))
            .count();
        if number_of_growable_tracks == 0 {
            break;
        }

        // Compute item-incurred increase for this iteration
        let min_increase_limit = tracks
            .iter()
            .filter(|track| track_is_affected(track))
            .map(|track| track.growth_limit - track.base_size)
            .min_by(|a, b| a.total_cmp(b))
            .unwrap(); // We will never pass an empty track list to this function
        let item_incurred_increase = f32_min(min_increase_limit, extra_space / number_of_growable_tracks as f32);

        // for track in tracks.iter().filter(|track| track.base_size + /*track.base_size_planned_increase*/ < track.growth_limit) {
        //     if item_incurred_increase > track.base_size_planned_increase {
        //         track.base_size_planned_increase += item_incurred_increase;
        //     }
        // }
        for track in tracks
            .iter_mut()
            .filter(|track| track_is_affected(track))
            .filter(|track| track.base_size < track.growth_limit)
        {
            track.base_size += item_incurred_increase;
        }

        extra_space -= item_incurred_increase * number_of_growable_tracks as f32;
    }

    // 3. Distribute remaining span beyond limits (if any)
    if extra_space > THRESHOLD {
        // When accommodating minimum contributions or accommodating min-content contributions:
        //   - any affected track that happens to also have an intrinsic max track sizing function;
        // When accommodating max-content contributions:
        //   - any affected track that happens to also have a max-content max track sizing function
        let mut filter = match intrinsic_contribution_type {
            IntrinsicContributionType::Minimum => {
                (|track: &GridTrack| track.max_track_sizing_function.is_intrinsic()) as fn(&GridTrack) -> bool
            }
            IntrinsicContributionType::Maximum => {
                (|track: &GridTrack| track.max_track_sizing_function.is_max_content()) as fn(&GridTrack) -> bool
            }
        };

        // If there are no such tracks (matching filter above), then use all affected tracks.
        let mut number_of_tracks =
            tracks.iter().filter(|track| track_is_affected(track)).filter(|track| filter(track)).count();
        if number_of_tracks == 0 {
            filter = (|_| true) as fn(&GridTrack) -> bool;
            number_of_tracks = tracks.len();
        }

        // Distribute remaining space
        let item_incurred_increase = extra_space / number_of_tracks as f32;
        for track in tracks.iter_mut().filter(|track| track_is_affected(track)).filter(|track| filter(track)) {
            track.base_size += item_incurred_increase;
        }
    }
}

/// 11.5.1. Distributing Extra Space Across Spanned Tracks
/// This is simplified (and faster) version of the algorithm for growth limits
/// https://www.w3.org/TR/css-grid-1/#extra-space
// TODO: Actually add planned increase to growth limit
fn distribute_space_to_growth_limit(
    space: f32,
    tracks: &mut [GridTrack],
    track_is_affected: impl Fn(&GridTrack) -> bool,
) {
    // 1. Find the space to distribute
    let track_sizes: f32 = tracks
        .iter()
        .map(|track| if track.growth_limit == f32::INFINITY { track.base_size } else { track.growth_limit })
        .sum();
    let extra_space: f32 = f32_max(0.0, space - track_sizes);

    // 2. Distribute space up to limits:
    // 3. Distribute space beyond limits
    // If space remains after all tracks are frozen, unfreeze and continue to distribute space to the item-incurred increase
    // when handling any intrinsic growth limit: all affected tracks.
    let number_of_growable_tracks =
        tracks.iter().filter(|track| track_is_affected(track)).filter(|track| track.infinitely_growable).count();
    if number_of_growable_tracks > 0 {
        let item_incurred_increase = extra_space / number_of_growable_tracks as f32;
        for track in
            tracks.iter_mut().filter(|track| track_is_affected(track)).filter(|track| track.infinitely_growable)
        {
            track.growth_limit = if track.growth_limit == f32::INFINITY {
                track.base_size + item_incurred_increase
            } else {
                track.growth_limit + item_incurred_increase
            }
        }
    } else {
        let item_incurred_increase = extra_space / tracks.len() as f32;
        for track in tracks {
            track.growth_limit = if track.growth_limit == f32::INFINITY {
                track.base_size + item_incurred_increase
            } else {
                track.growth_limit + item_incurred_increase
            }
        }
    };
}

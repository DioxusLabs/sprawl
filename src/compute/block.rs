//! Computes the [flexbox](https://css-tricks.com/snippets/css/a-guide-to-flexbox/) layout algorithm on [`Taffy`](crate::Taffy) according to the [spec](https://www.w3.org/TR/css-flexbox-1/)
use crate::compute::LayoutAlgorithm;
use crate::geometry::{Line, Point, Rect, Size};
use crate::style::{AvailableSpace, Display, LengthPercentageAuto, Overflow, Position};
use crate::style_helpers::TaffyMaxContent;
use crate::tree::{CollapsibleMarginSet, Layout, RunMode, SizeBaselinesAndMargins, SizingMode};
use crate::tree::{LayoutTree, NodeId};
use crate::util::sys::f32_max;
use crate::util::sys::Vec;
use crate::util::MaybeMath;
use crate::util::{MaybeResolve, ResolveOrZero};

#[cfg(feature = "debug")]
use crate::util::debug::NODE_LOGGER;

/// The public interface to Taffy's Flexbox algorithm implementation
pub struct BlockAlgorithm;
impl LayoutAlgorithm for BlockAlgorithm {
    const NAME: &'static str = "FLEXBOX";

    fn perform_layout(
        tree: &mut impl LayoutTree,
        node: NodeId,
        known_dimensions: Size<Option<f32>>,
        parent_size: Size<Option<f32>>,
        available_space: Size<AvailableSpace>,
        _sizing_mode: SizingMode,
        vertical_margins_are_collapsible: Line<bool>,
    ) -> SizeBaselinesAndMargins {
        compute(
            tree,
            node,
            known_dimensions,
            parent_size,
            available_space,
            RunMode::PeformLayout,
            vertical_margins_are_collapsible,
        )
    }

    fn measure_size(
        tree: &mut impl LayoutTree,
        node: NodeId,
        known_dimensions: Size<Option<f32>>,
        parent_size: Size<Option<f32>>,
        available_space: Size<AvailableSpace>,
        _sizing_mode: SizingMode,
        vertical_margins_are_collapsible: Line<bool>,
    ) -> Size<f32> {
        compute(
            tree,
            node,
            known_dimensions,
            parent_size,
            available_space,
            RunMode::ComputeSize,
            vertical_margins_are_collapsible,
        )
        .size
    }
}

/// The intermediate results of a flexbox calculation for a single item
struct BlockItem {
    /// The identifier for the associated node
    node_id: NodeId,

    /// The order of the item
    order: u32,

    /// The base size of this item
    size: Size<Option<f32>>,
    /// The minimum allowable size of this item
    min_size: Size<Option<f32>>,
    /// The maximum allowable size of this item
    max_size: Size<Option<f32>>,

    /// The position style of the item
    position: Position,
    /// The final offset of this item
    inset: Rect<LengthPercentageAuto>,
    /// The margin of this item
    margin: Rect<LengthPercentageAuto>,

    /// The computed border box size of this item
    computed_size: Size<f32>,
    /// The computed "static position" of this item. The static position is the position
    /// taking into account padding, border, margins, and scrollbar_gutters but not inset
    static_position: Point<f32>,
    /// Whether margins can be collapsed through this item
    can_be_collapsed_through: bool,

    /// The position of the bottom edge of this item
    baseline: Option<f32>,
}

/// Computes the layout of [`LayoutTree`] according to the block layout algorithm
pub fn compute(
    tree: &mut impl LayoutTree,
    node_id: NodeId,
    known_dimensions: Size<Option<f32>>,
    parent_size: Size<Option<f32>>,
    available_space: Size<AvailableSpace>,
    run_mode: RunMode,
    vertical_margins_are_collapsible: Line<bool>,
) -> SizeBaselinesAndMargins {
    let style = tree.style(node_id);

    // Pull these out earlier to avoid borrowing issues
    let aspect_ratio = style.aspect_ratio;
    let margin = style.margin.resolve_or_zero(parent_size.width);
    let min_size = style.min_size.maybe_resolve(parent_size).maybe_apply_aspect_ratio(aspect_ratio);
    let max_size = style.max_size.maybe_resolve(parent_size).maybe_apply_aspect_ratio(aspect_ratio);
    let clamped_style_size =
        style.size.maybe_resolve(parent_size).maybe_apply_aspect_ratio(aspect_ratio).maybe_clamp(min_size, max_size);

    // If both min and max in a given axis are set and max <= min then this determines the size in that axis
    let min_max_definite_size = min_size.zip_map(max_size, |min, max| match (min, max) {
        (Some(min), Some(max)) if max <= min => Some(min),
        _ => None,
    });

    // Block nodes automatically stretch fit their width to fit available space if available space is definite
    let available_space_based_size =
        Size { width: available_space.width.into_option().maybe_sub(margin.horizontal_axis_sum()), height: None };

    let styled_based_known_dimensions =
        known_dimensions.or(min_max_definite_size).or(clamped_style_size).or(available_space_based_size);

    // Short-circuit layout if the container's size is fully determined by the container's size and the run mode
    // is ComputeSize (and thus the container's size is all that we're interested in)
    if run_mode == RunMode::ComputeSize {
        if let Size { width: Some(width), height: Some(height) } = styled_based_known_dimensions {
            return Size { width, height }.into();
        }
    }

    #[cfg(feature = "debug")]
    NODE_LOGGER.log("BLOCK");
    compute_inner(
        tree,
        node_id,
        styled_based_known_dimensions,
        parent_size,
        available_space,
        run_mode,
        vertical_margins_are_collapsible,
    )
}

/// Computes the layout of [`LayoutTree`] according to the block layout algorithm
fn compute_inner(
    tree: &mut impl LayoutTree,
    node_id: NodeId,
    known_dimensions: Size<Option<f32>>,
    parent_size: Size<Option<f32>>,
    available_space: Size<AvailableSpace>,
    run_mode: RunMode,
    vertical_margins_are_collapsible: Line<bool>,
) -> SizeBaselinesAndMargins {
    let style = tree.style(node_id);
    let raw_padding = style.padding;
    let raw_border = style.border;
    let raw_margin = style.margin;
    let aspect_ratio = style.aspect_ratio;
    let size = style.size.maybe_resolve(parent_size).maybe_apply_aspect_ratio(aspect_ratio);
    let min_size = style.min_size.maybe_resolve(parent_size).maybe_apply_aspect_ratio(aspect_ratio);
    let max_size = style.max_size.maybe_resolve(parent_size).maybe_apply_aspect_ratio(aspect_ratio);
    let padding = style.padding.resolve_or_zero(parent_size.width);
    let border = style.border.resolve_or_zero(parent_size.width);

    // Scrollbar gutters are reserved when the `overflow` property is set to `Overflow::Scroll`.
    // However, the axis are switched (transposed) because a node that scrolls vertically needs
    // *horizontal* space to be reserved for a scrollbar
    let scrollbar_gutter = {
        let offsets = style.overflow.transpose().map(|overflow| match overflow {
            Overflow::Scroll => style.scrollbar_width,
            _ => 0.0,
        });
        // TODO: make side configurable based on the `direction` property
        Rect { top: 0.0, left: 0.0, right: offsets.x, bottom: offsets.y }
    };
    let content_box_inset = padding + border + scrollbar_gutter;
    let container_content_box_size = known_dimensions.maybe_sub(content_box_inset.sum_axes());

    // Determine margin collapsing behaviour
    let own_margins_collapse_with_children = Line {
        start: vertical_margins_are_collapsible.start
            && !style.overflow.y.is_scroll_container()
            && style.position == Position::Relative
            && padding.top == 0.0
            && border.top == 0.0,
        end: vertical_margins_are_collapsible.end
            && !style.overflow.y.is_scroll_container()
            && style.position == Position::Relative
            && padding.bottom == 0.0
            && border.bottom == 0.0
            && size.height.is_none(),
    };
    let has_styles_preventing_being_collapsed_through = style.display != Display::Block
        || style.overflow.y.is_scroll_container()
        || style.position == Position::Absolute
        || padding.top > 0.0
        || padding.bottom > 0.0
        || border.top > 0.0
        || border.bottom > 0.0;

    // 1. Generate items
    let mut items = generate_item_list(tree, node_id, container_content_box_size);

    // 2. Compute container width
    let container_outer_width = known_dimensions.width.unwrap_or_else(|| {
        let available_width = available_space.width.maybe_sub(content_box_inset.horizontal_axis_sum());
        let intrinsic_width = determine_content_based_container_width(tree, &items, available_width)
            + content_box_inset.horizontal_axis_sum();
        intrinsic_width.maybe_clamp(min_size.width, max_size.width)
    });

    // Short-circuit if computing size and both dimensions known
    if let (RunMode::ComputeSize, Some(container_outer_height)) = (run_mode, known_dimensions.height) {
        return Size { width: container_outer_width, height: container_outer_height }.into();
    }

    // 3. Perform final item layout and return content height
    let resolved_padding = raw_padding.resolve_or_zero(Some(container_outer_width));
    let resolved_border = raw_border.resolve_or_zero(Some(container_outer_width));
    let resolved_content_box_inset = resolved_padding + resolved_border + scrollbar_gutter;
    let (intrinsic_outer_height, first_child_top_margin_set, last_child_bottom_margin_set) =
        perform_final_layout_on_in_flow_children(
            tree,
            &mut items,
            container_outer_width,
            content_box_inset,
            resolved_content_box_inset,
            own_margins_collapse_with_children,
        );
    let container_outer_height =
        known_dimensions.height.unwrap_or(intrinsic_outer_height.maybe_clamp(min_size.height, max_size.height));
    let final_outer_size = Size { width: container_outer_width, height: container_outer_height };

    // Short-circuit if computing size
    if run_mode == RunMode::ComputeSize {
        return final_outer_size.into();
    }

    // 4. Layout absolutely positioned children
    let absolute_position_inset = resolved_border + scrollbar_gutter;
    let absolute_position_area = final_outer_size - absolute_position_inset.sum_axes();
    let absolute_position_offset = Point { x: absolute_position_inset.left, y: absolute_position_inset.top };
    perform_absolute_layout_on_absolute_children(tree, &mut items, absolute_position_area, absolute_position_offset);

    // 5. Perform hidden layout on hidden children
    let len = tree.child_count(node_id);
    for order in 0..len {
        let child = tree.child(node_id, order);
        if tree.style(child).display == Display::None {
            *tree.layout_mut(child) = Layout::with_order(order as u32);
            tree.perform_child_layout(
                child,
                Size::NONE,
                Size::NONE,
                Size::MAX_CONTENT,
                SizingMode::InherentSize,
                Line::FALSE,
            );
        }
    }

    // 6. Calculate the container's first vertical baseline
    let first_vertical_baseline = items
        .iter()
        .find(|item| item.position != Position::Absolute)
        .map(|child| child.static_position.y + child.baseline.unwrap_or(child.computed_size.height));

    // 7. Determine whether this node can be collapsed through
    let all_in_flow_children_can_be_collapsed_through =
        items.iter().all(|item| item.position == Position::Absolute || item.can_be_collapsed_through);
    let can_be_collapsed_through =
        !has_styles_preventing_being_collapsed_through && all_in_flow_children_can_be_collapsed_through;

    SizeBaselinesAndMargins {
        size: final_outer_size,
        first_baselines: Point { x: None, y: first_vertical_baseline },
        top_margin: if own_margins_collapse_with_children.start {
            first_child_top_margin_set
        } else {
            let margin_top = raw_margin.top.resolve_or_zero(parent_size.width);
            CollapsibleMarginSet::from_margin(margin_top)
        },
        bottom_margin: if own_margins_collapse_with_children.end {
            last_child_bottom_margin_set
        } else {
            let margin_bottom = raw_margin.bottom.resolve_or_zero(parent_size.width);
            CollapsibleMarginSet::from_margin(margin_bottom)
        },
        margins_can_collapse_through: can_be_collapsed_through,
    }
}

/// Create a `Vec` of `BlockItem` structs where each item in the `Vec` represents a child of the current node
#[inline]
fn generate_item_list(tree: &impl LayoutTree, node: NodeId, node_inner_size: Size<Option<f32>>) -> Vec<BlockItem> {
    tree.children(node)
        .map(|child_node_id| (child_node_id, tree.style(child_node_id)))
        .filter(|(_, style)| style.display != Display::None)
        .enumerate()
        .map(|(order, (child_node_id, child_style))| {
            let aspect_ratio = child_style.aspect_ratio;
            BlockItem {
                node_id: child_node_id,
                order: order as u32,

                size: child_style.size.maybe_resolve(node_inner_size).maybe_apply_aspect_ratio(aspect_ratio),
                min_size: child_style.min_size.maybe_resolve(node_inner_size).maybe_apply_aspect_ratio(aspect_ratio),
                max_size: child_style.max_size.maybe_resolve(node_inner_size).maybe_apply_aspect_ratio(aspect_ratio),
                position: child_style.position,
                inset: child_style.inset,
                margin: child_style.margin,

                // Fields to be computed later (for now we initialise with dummy values)
                computed_size: Size::zero(),
                static_position: Point::zero(),
                can_be_collapsed_through: false,
                baseline: None,
            }
        })
        .collect()
}

/// Compute the content-based width in the case that the width of the container is not known
#[inline]
fn determine_content_based_container_width(
    tree: &mut impl LayoutTree,
    items: &[BlockItem],
    available_width: AvailableSpace,
) -> f32 {
    let available_space = Size { width: available_width, height: AvailableSpace::MinContent };

    let mut max_child_width = 0.0;
    for item in items.iter().filter(|item| item.position != Position::Absolute) {
        let known_dimensions = item.size.maybe_clamp(item.min_size, item.max_size);
        let item_x_margin_sum = item.margin.resolve_or_zero(available_space.width.into_option()).horizontal_axis_sum();

        let width = known_dimensions.width.unwrap_or_else(|| {
            let size_and_baselines = tree.perform_child_layout(
                item.node_id,
                known_dimensions,
                Size::NONE,
                available_space.map_width(|w| w.maybe_sub(item_x_margin_sum)),
                SizingMode::InherentSize,
                Line::TRUE,
            );

            size_and_baselines.size.width + item_x_margin_sum
        });

        max_child_width = f32_max(max_child_width, width);
    }

    max_child_width
}

/// Compute each child's final size and position
#[inline]
fn perform_final_layout_on_in_flow_children(
    tree: &mut impl LayoutTree,
    items: &mut [BlockItem],
    container_outer_width: f32,
    content_box_inset: Rect<f32>,
    resolved_content_box_inset: Rect<f32>,
    own_margins_collapse_with_children: Line<bool>,
) -> (f32, CollapsibleMarginSet, CollapsibleMarginSet) {
    // Resolve container_inner_width for sizing child nodes using intial content_box_inset
    let container_inner_width = container_outer_width - content_box_inset.horizontal_axis_sum();
    let parent_size = Size { width: Some(container_outer_width), height: None };
    let available_space =
        Size { width: AvailableSpace::Definite(container_inner_width), height: AvailableSpace::MinContent };

    let mut committed_y_offset = resolved_content_box_inset.top;
    let mut first_child_top_margin_set = CollapsibleMarginSet::ZERO;
    let mut active_collapsible_margin_set = CollapsibleMarginSet::ZERO;
    let mut is_collapsing_with_first_margin_set = true;
    for item in items.iter_mut() {
        if item.position == Position::Absolute {
            item.static_position.y = committed_y_offset;
        } else {
            let item_margin = item.margin.map(|margin| margin.resolve_to_option(container_outer_width));
            let item_non_auto_margin = item_margin.map(|m| m.unwrap_or(0.0));
            let item_non_auto_x_margin_sum = item_non_auto_margin.horizontal_axis_sum();
            let known_dimensions = item
                .size
                .maybe_clamp(item.min_size, item.max_size)
                .map_width(|width| Some(width.unwrap_or(container_inner_width - item_non_auto_x_margin_sum)));

            let item_layout = tree.perform_child_layout(
                item.node_id,
                known_dimensions,
                parent_size,
                available_space.map_width(|w| w.maybe_sub(item_non_auto_x_margin_sum)),
                SizingMode::InherentSize,
                Line::TRUE,
            );
            let final_size = item_layout.size;

            let top_margin_set = item_layout.top_margin.collapse_with_margin(item_margin.top.unwrap_or(0.0));
            let bottom_margin_set = item_layout.bottom_margin.collapse_with_margin(item_margin.bottom.unwrap_or(0.0));

            // Expand auto margins to fill available space
            // Note: Vertical auto-margins for relatively positioned block items simply resolve to 0.
            // See: https://www.w3.org/TR/CSS21/visudet.html#abs-non-replaced-width
            let free_x_space = f32_max(0.0, container_inner_width - final_size.width - item_non_auto_x_margin_sum);
            let x_axis_auto_margin_size = {
                let auto_margin_count = item_margin.left.is_none() as u8 + item_margin.right.is_none() as u8;
                if auto_margin_count == 2 && item.size.width.is_none() {
                    0.0
                } else if auto_margin_count > 0 {
                    free_x_space / auto_margin_count as f32
                } else {
                    0.0
                }
            };
            let resolved_margin = Rect {
                left: item_margin.left.unwrap_or(x_axis_auto_margin_size),
                right: item_margin.right.unwrap_or(x_axis_auto_margin_size),
                top: top_margin_set.resolve(),
                bottom: bottom_margin_set.resolve(),
            };

            // Resolve item inset
            let inset =
                item.inset.zip_size(Size { width: container_inner_width, height: 0.0 }, |p, s| p.maybe_resolve(s));
            let inset_offset = Point {
                x: inset.left.or(inset.right.map(|x| -x)).unwrap_or(0.0),
                y: inset.top.or(inset.bottom.map(|x| -x)).unwrap_or(0.0),
            };

            let y_margin_offset = if is_collapsing_with_first_margin_set && own_margins_collapse_with_children.start {
                0.0
            } else {
                active_collapsible_margin_set.collapse_with_margin(resolved_margin.top).resolve()
            };

            item.computed_size = item_layout.size;
            item.baseline = item_layout.first_baselines.y;
            item.can_be_collapsed_through = item_layout.margins_can_collapse_through;
            item.static_position = Point {
                x: resolved_content_box_inset.left,
                y: committed_y_offset + active_collapsible_margin_set.resolve(),
            };

            *tree.layout_mut(item.node_id) = Layout {
                order: item.order,
                size: item_layout.size,
                location: Point {
                    x: resolved_content_box_inset.left + inset_offset.x + resolved_margin.left,
                    y: committed_y_offset + inset_offset.y + y_margin_offset,
                },
            };

            // Update first_child_top_margin_set
            if is_collapsing_with_first_margin_set {
                if item.can_be_collapsed_through {
                    first_child_top_margin_set = first_child_top_margin_set
                        .collapse_with_set(top_margin_set)
                        .collapse_with_set(bottom_margin_set);
                } else {
                    first_child_top_margin_set = first_child_top_margin_set.collapse_with_set(top_margin_set);
                    is_collapsing_with_first_margin_set = false;
                }
            }

            // Update active_collapsible_margin_set
            if item.can_be_collapsed_through {
                active_collapsible_margin_set = active_collapsible_margin_set
                    .collapse_with_set(top_margin_set)
                    .collapse_with_set(bottom_margin_set);
            } else {
                committed_y_offset += item_layout.size.height + y_margin_offset;
                active_collapsible_margin_set = bottom_margin_set;
            }
        }
    }

    let last_child_bottom_margin_set = active_collapsible_margin_set;
    let bottom_y_margin_offset =
        if own_margins_collapse_with_children.end { 0.0 } else { last_child_bottom_margin_set.resolve() };

    committed_y_offset += resolved_content_box_inset.bottom + bottom_y_margin_offset;
    let content_height = f32_max(0.0, committed_y_offset);
    (content_height, first_child_top_margin_set, last_child_bottom_margin_set)
}

/// Perform absolute layout on all absolutely positioned children.
#[inline]
fn perform_absolute_layout_on_absolute_children(
    tree: &mut impl LayoutTree,
    items: &mut [BlockItem],
    area_size: Size<f32>,
    area_offset: Point<f32>,
) {
    let area_width = area_size.width;
    let area_height = area_size.height;

    for item in items.iter().filter(|item| item.position == Position::Absolute) {
        let child_style = tree.style(item.node_id);

        // Skip items that are display:none or are not position:absolute
        if child_style.display == Display::None || child_style.position != Position::Absolute {
            continue;
        }

        let aspect_ratio = child_style.aspect_ratio;
        let margin = child_style.margin.map(|margin| margin.resolve_to_option(area_width));
        let padding = child_style.padding.resolve_or_zero(Some(area_width));
        let border = child_style.border.resolve_or_zero(Some(area_width));
        let padding_border_sum = (padding + border).sum_axes();

        // Resolve inset
        let left = child_style.inset.left.maybe_resolve(area_width);
        let right = child_style.inset.right.maybe_resolve(area_width);
        let top = child_style.inset.top.maybe_resolve(area_height);
        let bottom = child_style.inset.bottom.maybe_resolve(area_height);

        // Compute known dimensions from min/max/inherent size styles
        let style_size = child_style.size.maybe_resolve(area_size).maybe_apply_aspect_ratio(aspect_ratio);
        let min_size = child_style
            .min_size
            .maybe_resolve(area_size)
            .maybe_apply_aspect_ratio(aspect_ratio)
            .or(padding_border_sum.map(Some))
            .maybe_max(padding_border_sum);
        let max_size = child_style.max_size.maybe_resolve(area_size).maybe_apply_aspect_ratio(aspect_ratio);
        let mut known_dimensions = style_size.maybe_clamp(min_size, max_size);

        // Fill in width from left/right and reapply aspect ratio if:
        //   - Width is not already known
        //   - Item has both left and right inset properties set
        if let (None, Some(left), Some(right)) = (known_dimensions.width, left, right) {
            let new_width_raw = area_width.maybe_sub(margin.left).maybe_sub(margin.right) - left - right;
            known_dimensions.width = Some(f32_max(new_width_raw, 0.0));
            known_dimensions = known_dimensions.maybe_apply_aspect_ratio(aspect_ratio).maybe_clamp(min_size, max_size);
        }

        // Fill in height from top/bottom and reapply aspect ratio if:
        //   - Height is not already known
        //   - Item has both top and bottom inset properties set
        if let (None, Some(top), Some(bottom)) = (known_dimensions.height, top, bottom) {
            let new_height_raw = area_height.maybe_sub(margin.top).maybe_sub(margin.bottom) - top - bottom;
            known_dimensions.height = Some(f32_max(new_height_raw, 0.0));
            known_dimensions = known_dimensions.maybe_apply_aspect_ratio(aspect_ratio).maybe_clamp(min_size, max_size);
        }

        let measured_size_and_baselines = tree.perform_child_layout(
            item.node_id,
            known_dimensions,
            area_size.map(Some),
            Size {
                width: AvailableSpace::Definite(area_width.maybe_clamp(min_size.width, max_size.width)),
                height: AvailableSpace::Definite(area_height.maybe_clamp(min_size.height, max_size.height)),
            },
            SizingMode::ContentSize,
            Line::FALSE,
        );
        let measured_size = measured_size_and_baselines.size;
        let final_size = known_dimensions.unwrap_or(measured_size).maybe_clamp(min_size, max_size);

        let non_auto_margin = Rect {
            left: if left.is_some() { margin.left.unwrap_or(0.0) } else { 0.0 },
            right: if right.is_some() { margin.right.unwrap_or(0.0) } else { 0.0 },
            top: if top.is_some() { margin.top.unwrap_or(0.0) } else { 0.0 },
            bottom: if bottom.is_some() { margin.left.unwrap_or(0.0) } else { 0.0 },
        };

        // Expand auto margins to fill available space
        // https://www.w3.org/TR/CSS21/visudet.html#abs-non-replaced-width
        let auto_margin = {
            // Auto margins for absolutely positioned elements in block containers only resolve
            // if inset is set. Otherwise they resolve to 0.
            let absolute_auto_margin_space = Point {
                x: right.map(|right| area_size.width - right - left.unwrap_or(0.0)).unwrap_or(final_size.width),
                y: bottom.map(|bottom| area_size.height - bottom - top.unwrap_or(0.0)).unwrap_or(final_size.height),
            };
            let free_space = Size {
                width: absolute_auto_margin_space.x - final_size.width - non_auto_margin.horizontal_axis_sum(),
                height: absolute_auto_margin_space.y - final_size.height - non_auto_margin.vertical_axis_sum(),
            };

            let auto_margin_size = Size {
                // If all three of 'left', 'width', and 'right' are 'auto': First set any 'auto' values for 'margin-left' and 'margin-right' to 0.
                // Then, if the 'direction' property of the element establishing the static-position containing block is 'ltr' set 'left' to the
                // static position and apply rule number three below; otherwise, set 'right' to the static position and apply rule number one below.
                //
                // If none of the three is 'auto': If both 'margin-left' and 'margin-right' are 'auto', solve the equation under the extra constraint
                // that the two margins get equal values, unless this would make them negative, in which case when direction of the containing block is
                // 'ltr' ('rtl'), set 'margin-left' ('margin-right') to zero and solve for 'margin-right' ('margin-left'). If one of 'margin-left' or
                // 'margin-right' is 'auto', solve the equation for that value. If the values are over-constrained, ignore the value for 'left' (in case
                // the 'direction' property of the containing block is 'rtl') or 'right' (in case 'direction' is 'ltr') and solve for that value.
                width: {
                    let auto_margin_count = margin.left.is_none() as u8 + margin.right.is_none() as u8;
                    if auto_margin_count == 2
                        && (style_size.width.is_none() || style_size.width.unwrap() >= free_space.width)
                    {
                        0.0
                    } else if auto_margin_count > 0 {
                        free_space.width / auto_margin_count as f32
                    } else {
                        0.0
                    }
                },
                height: {
                    let auto_margin_count = margin.top.is_none() as u8 + margin.bottom.is_none() as u8;
                    if auto_margin_count == 2
                        && (style_size.height.is_none() || style_size.height.unwrap() >= free_space.height)
                    {
                        0.0
                    } else if auto_margin_count > 0 {
                        free_space.height / auto_margin_count as f32
                    } else {
                        0.0
                    }
                },
            };

            Rect {
                left: margin.left.map(|_| 0.0).unwrap_or(auto_margin_size.width),
                right: margin.right.map(|_| 0.0).unwrap_or(auto_margin_size.width),
                top: margin.top.map(|_| 0.0).unwrap_or(auto_margin_size.height),
                bottom: margin.bottom.map(|_| 0.0).unwrap_or(auto_margin_size.height),
            }
        };

        let resolved_margin = Rect {
            left: margin.left.unwrap_or(auto_margin.left),
            right: margin.right.unwrap_or(auto_margin.right),
            top: margin.top.unwrap_or(auto_margin.top),
            bottom: margin.bottom.unwrap_or(auto_margin.bottom),
        };

        let item_offset = Point {
            x: left
                .map(|left| left + resolved_margin.left)
                .or(right.map(|right| area_size.width - final_size.width - right - resolved_margin.right))
                .unwrap_or(resolved_margin.left),
            y: top
                .map(|top| top + resolved_margin.top)
                .or(bottom.map(|bottom| area_size.height - final_size.height - bottom - resolved_margin.bottom))
                .unwrap_or(item.static_position.y + resolved_margin.top),
        };

        *tree.layout_mut(item.node_id) =
            Layout { order: item.order, size: final_size, location: area_offset + item_offset };
    }
}

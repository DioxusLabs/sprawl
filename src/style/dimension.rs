//! Style types for representing lengths / sizes

use std::ops::Neg;
use std::sync::Arc;
use num_traits::{Signed, Zero};
use crate::geometry::{Rect, Size};
use crate::style_helpers::{FromLength, FromPercent, TaffyAuto, TaffyMaxContent, TaffyMinContent, TaffyZero};
use crate::util::sys::abs;

/// A unit of linear measurement
///
/// This is commonly combined with [`Rect`], [`Point`](crate::geometry::Point) and [`Size<T>`].
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum LengthPercentage {
    /// An absolute length in some abstract units. Users of Taffy may define what they correspond
    /// to in their application (pixels, logical pixels, mm, etc) as they see fit.
    Length(f32),
    /// The dimension is stored in percentage relative to the parent item.
    Percent(f32),
    /// todo
    Calculation(Calc),
}
impl LengthPercentage {
    pub fn resolve(&self, percentage_length: f32) -> f32 {
        match self {
            LengthPercentage::Length(length) => *length,
            LengthPercentage::Percent(fraction) => fraction * percentage_length,
            LengthPercentage::Calculation(calc) => calc.resolve(percentage_length),
        }
    }
}
impl TaffyZero for LengthPercentage {
    const ZERO: Self = Self::Length(0.0);
}
impl FromLength for LengthPercentage {
    fn from_length<Input: Into<f32> + Copy>(value: Input) -> Self {
        Self::Length(value.into())
    }
}
impl FromPercent for LengthPercentage {
    fn from_percent<Input: Into<f32> + Copy>(percent: Input) -> Self {
        Self::Percent(percent.into())
    }
}

/// A unit of linear measurement
///
/// This is commonly combined with [`Rect`], [`Point`](crate::geometry::Point) and [`Size<T>`].
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum LengthPercentageAuto {
    /// An absolute length in some abstract units. Users of Taffy may define what they correspond
    /// to in their application (pixels, logical pixels, mm, etc) as they see fit.
    Length(f32),
    /// The dimension is stored in percentage relative to the parent item.
    Percent(f32),
    /// todo
    Calculation(Calc),
    /// The dimension should be automatically computed
    Auto,
}
impl TaffyZero for LengthPercentageAuto {
    const ZERO: Self = Self::Length(0.0);
}
impl TaffyAuto for LengthPercentageAuto {
    const AUTO: Self = Self::Auto;
}
impl FromLength for LengthPercentageAuto {
    fn from_length<Input: Into<f32> + Copy>(value: Input) -> Self {
        Self::Length(value.into())
    }
}
impl FromPercent for LengthPercentageAuto {
    fn from_percent<Input: Into<f32> + Copy>(percent: Input) -> Self {
        Self::Percent(percent.into())
    }
}

impl From<LengthPercentage> for LengthPercentageAuto {
    fn from(input: LengthPercentage) -> Self {
        match input {
            LengthPercentage::Length(value) => Self::Length(value),
            LengthPercentage::Percent(value) => Self::Percent(value),
            LengthPercentage::Calculation(calc) => Self::Calculation(calc),
        }
    }
}

impl LengthPercentageAuto {
    /// Returns:
    ///   - Some(length) for Length variants
    ///   - Some(resolved) using the provided context for Percent variants
    ///   - Some(calculation) todo
    ///   - None for Auto variants
    #[inline(always)]
    pub fn resolve_to_option(self, context: f32) -> Option<f32> {
        match self {
            Self::Length(length) => Some(length),
            Self::Percent(percent) => Some(context * percent),
            Self::Calculation(calc) => Some(calc.resolve(context)),
            Self::Auto => None,
        }
    }

    /// Returns true if value is LengthPercentageAuto::Auto
    #[inline(always)]
    pub fn is_auto(self) -> bool {
        self == Self::Auto
    }
}

/// A unit of linear measurement
///
/// This is commonly combined with [`Rect`], [`Point`](crate::geometry::Point) and [`Size<T>`].
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Dimension {
    /// An absolute length in some abstract units. Users of Taffy may define what they correspond
    /// to in their application (pixels, logical pixels, mm, etc) as they see fit.
    Length(f32),
    /// The dimension is stored in percentage relative to the parent item.
    Percent(f32),
    /// todo
    Calculation(Calc),
    /// The dimension should be automatically computed
    Auto,
}
impl TaffyZero for Dimension {
    const ZERO: Self = Self::Length(0.0);
}
impl TaffyAuto for Dimension {
    const AUTO: Self = Self::Auto;
}
impl FromLength for Dimension {
    fn from_length<Input: Into<f32> + Copy>(value: Input) -> Self {
        Self::Length(value.into())
    }
}
impl FromPercent for Dimension {
    fn from_percent<Input: Into<f32> + Copy>(percent: Input) -> Self {
        Self::Percent(percent.into())
    }
}

impl From<LengthPercentage> for Dimension {
    fn from(input: LengthPercentage) -> Self {
        match input {
            LengthPercentage::Length(value) => Self::Length(value),
            LengthPercentage::Percent(value) => Self::Percent(value),
            LengthPercentage::Calculation(value) => Self::Calculation(value),
        }
    }
}

impl From<LengthPercentageAuto> for Dimension {
    fn from(input: LengthPercentageAuto) -> Self {
        match input {
            LengthPercentageAuto::Length(value) => Self::Length(value),
            LengthPercentageAuto::Percent(value) => Self::Percent(value),
            LengthPercentageAuto::Calculation(value) => Self::Calculation(value),
            LengthPercentageAuto::Auto => Self::Auto,
        }
    }
}

impl Dimension {
    /// Get Length value if value is Length variant
    #[cfg(feature = "grid")]
    pub fn into_option(self) -> Option<f32> {
        match self {
            Dimension::Length(value) => Some(value),
            _ => None,
        }
    }
}

impl Rect<Dimension> {
    /// Create a new Rect with [`Dimension::Length`]
    #[must_use]
    pub const fn from_length(start: f32, end: f32, top: f32, bottom: f32) -> Self {
        Rect {
            left: Dimension::Length(start),
            right: Dimension::Length(end),
            top: Dimension::Length(top),
            bottom: Dimension::Length(bottom),
        }
    }

    /// Create a new Rect with [`Dimension::Percent`]
    #[must_use]
    pub const fn from_percent(start: f32, end: f32, top: f32, bottom: f32) -> Self {
        Rect {
            left: Dimension::Percent(start),
            right: Dimension::Percent(end),
            top: Dimension::Percent(top),
            bottom: Dimension::Percent(bottom),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Calc(Arc<CalcNode>);
impl Calc {
    pub fn resolve(&self, percentage_length: f32) -> f32 {
        self.0.resolve(percentage_length)
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum CalcNode {
    Leaf(LengthPercentage),

    Sum(Box<CalcNode>, Box<CalcNode>),
    Difference(Box<CalcNode>, Box<CalcNode>),
    Product(Box<CalcNode>, Box<CalcNode>),
    Quotient(Box<CalcNode>, Box<CalcNode>),

    Negate(Box<CalcNode>),
    Min(Vec<CalcNode>),
    Max(Vec<CalcNode>),

    Clamp { min: Box<CalcNode>, center: Box<CalcNode>, max: Box<CalcNode> },
    Round {
        strategy: RoundingStrategy,
        value: Box<CalcNode>,
        interval: Box<CalcNode>
    }
}
impl CalcNode {
    fn resolve(&self, percentage_length: f32) -> f32 {
        match self {
            CalcNode::Leaf(leaf) => leaf.resolve(percentage_length),
            CalcNode::Sum(lhs, rhs) => lhs.resolve(percentage_length) + rhs.resolve(percentage_length),
            CalcNode::Difference(lhs, rhs) => lhs.resolve(percentage_length) - rhs.resolve(percentage_length),
            CalcNode::Product(lhs, rhs) => lhs.resolve(percentage_length) * rhs.resolve(percentage_length),
            CalcNode::Quotient(lhs, rhs) => lhs.resolve(percentage_length) / rhs.resolve(percentage_length),
            CalcNode::Negate(node) => node.resolve(percentage_length).neg(),
            CalcNode::Min(nodes) => {
                nodes.iter().map(|node| node.resolve(percentage_length)).reduce(f32::min).unwrap_or_default()
            }
            CalcNode::Max(nodes) => {
                nodes.iter().map(|node| node.resolve(percentage_length)).reduce(f32::max).unwrap_or_default()
            }
            CalcNode::Clamp { min, center: value, max } => {
                let min = min.resolve(percentage_length);
                let value = value.resolve(percentage_length);
                let max = max.resolve(percentage_length);

                let max = value.max(max);
                let min = min.min(max);
                min
            }

            CalcNode::Round { strategy, value, interval } => {
                // https://developer.mozilla.org/en-US/docs/Web/CSS/round#return_value
                // https://drafts.csswg.org/css-values/#funcdef-round
                
                let value = value.resolve(percentage_length);
                let interval = interval.resolve(percentage_length);
                
                // todo: The argument calculations can resolve to <number>, <dimension>, or 
                // <percentage>, but must have the same type, or else the function is invalid; the 
                // result will have the same type as the arguments.
                
                if interval.is_zero() {
                    return f32::NAN;
                }
                if value.is_infinite() {
                    if interval.is_infinite() {
                        return value
                    }
                    return f32::NAN;
                }
                
                if interval.is_infinite() {
                    return match strategy {
                        RoundingStrategy::Up => {
                            if value.is_positive() && !value.is_zero() {
                                f32::INFINITY
                            } else if value.is_positive() {
                                0.0
                            } else {
                                -0.0
                            }
                        }
                        RoundingStrategy::Down => {
                            if value.is_negative() && !value.is_zero() {
                                f32::NEG_INFINITY
                            } else if value.is_negative() {
                                -0.0
                            } else {
                                0.0
                            }
                        }
                        RoundingStrategy::Nearest |
                        RoundingStrategy::ToZero => {
                            if value.is_positive() { 0.0 } else { -0.0 }
                        }
                    }
                }

                let div = value / interval;
                let lower_bound = div.floor() * interval;
                let upper_bound = div.ceil() * interval;

                match strategy {
                    RoundingStrategy::Up => upper_bound,
                    RoundingStrategy::Down => lower_bound,
                    RoundingStrategy::Nearest => {
                        if value - lower_bound < upper_bound - value {
                            lower_bound
                        } else {
                            upper_bound
                        }
                    },
                    RoundingStrategy::ToZero => {
                        if lower_bound.abs() < upper_bound.abs() {
                            lower_bound
                        } else {
                            upper_bound
                        }
                    },
                }
            }
        }
    }

    fn into_calc(self) -> Calc {
        Calc(Arc::new(self))
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum RoundingStrategy {
    Up,
    Down,
    #[default]
    Nearest,
    ToZero
}

/// The amount of space available to a node in a given axis
/// <https://www.w3.org/TR/css-sizing-3/#available>
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum AvailableSpace {
    /// The amount of space available is the specified number of pixels
    Definite(f32),
    /// The amount of space available is indefinite and the node should be laid out under a min-content constraint
    MinContent,
    /// The amount of space available is indefinite and the node should be laid out under a max-content constraint
    MaxContent,
}
impl TaffyZero for AvailableSpace {
    const ZERO: Self = Self::Definite(0.0);
}
impl TaffyMaxContent for AvailableSpace {
    const MAX_CONTENT: Self = Self::MaxContent;
}
impl TaffyMinContent for AvailableSpace {
    const MIN_CONTENT: Self = Self::MinContent;
}
impl FromLength for AvailableSpace {
    fn from_length<Input: Into<f32> + Copy>(value: Input) -> Self {
        Self::Definite(value.into())
    }
}

impl AvailableSpace {
    /// Returns true for definite values, else false
    pub fn is_definite(self) -> bool {
        matches!(self, AvailableSpace::Definite(_))
    }

    /// Convert to Option
    /// Definite values become Some(value). Constraints become None.
    pub fn into_option(self) -> Option<f32> {
        match self {
            AvailableSpace::Definite(value) => Some(value),
            _ => None,
        }
    }

    /// Return the definite value or a default value
    pub fn unwrap_or(self, default: f32) -> f32 {
        self.into_option().unwrap_or(default)
    }

    /// Return the definite value. Panic is the value is not definite.
    #[track_caller]
    pub fn unwrap(self) -> f32 {
        self.into_option().unwrap()
    }

    /// Return self if definite or a default value
    pub fn or(self, default: AvailableSpace) -> AvailableSpace {
        match self {
            AvailableSpace::Definite(_) => self,
            _ => default,
        }
    }

    /// Return self if definite or a the result of the default value callback
    pub fn or_else(self, default_cb: impl FnOnce() -> AvailableSpace) -> AvailableSpace {
        match self {
            AvailableSpace::Definite(_) => self,
            _ => default_cb(),
        }
    }

    /// Return the definite value or the result of the default value callback
    pub fn unwrap_or_else(self, default_cb: impl FnOnce() -> f32) -> f32 {
        self.into_option().unwrap_or_else(default_cb)
    }

    /// If passed value is Some then return AvailableSpace::Definite containing that value, else return self
    pub fn maybe_set(self, value: Option<f32>) -> AvailableSpace {
        match value {
            Some(value) => AvailableSpace::Definite(value),
            None => self,
        }
    }

    /// If passed value is Some then return AvailableSpace::Definite containing that value, else return self
    pub fn map_definite_value(self, map_function: impl FnOnce(f32) -> f32) -> AvailableSpace {
        match self {
            AvailableSpace::Definite(value) => AvailableSpace::Definite(map_function(value)),
            _ => self,
        }
    }

    /// Compute free_space given the passed used_space
    pub fn compute_free_space(&self, used_space: f32) -> f32 {
        match self {
            AvailableSpace::MaxContent => f32::INFINITY,
            AvailableSpace::MinContent => 0.0,
            AvailableSpace::Definite(available_space) => available_space - used_space,
        }
    }

    /// Compare equality with another AvailableSpace, treating definite values
    /// that are within f32::EPSILON of each other as equal
    pub fn is_roughly_equal(self, other: AvailableSpace) -> bool {
        use AvailableSpace::*;
        match (self, other) {
            (Definite(a), Definite(b)) => abs(a - b) < f32::EPSILON,
            (MinContent, MinContent) => true,
            (MaxContent, MaxContent) => true,
            _ => false,
        }
    }
}

impl From<f32> for AvailableSpace {
    fn from(value: f32) -> Self {
        Self::Definite(value)
    }
}

impl From<Option<f32>> for AvailableSpace {
    fn from(option: Option<f32>) -> Self {
        match option {
            Some(value) => Self::Definite(value),
            None => Self::MaxContent,
        }
    }
}

impl Size<AvailableSpace> {
    /// Convert `Size<AvailableSpace>` into `Size<Option<f32>>`
    pub fn into_options(self) -> Size<Option<f32>> {
        Size { width: self.width.into_option(), height: self.height.into_option() }
    }

    /// If passed value is Some then return AvailableSpace::Definite containing that value, else return self
    pub fn maybe_set(self, value: Size<Option<f32>>) -> Size<AvailableSpace> {
        Size { width: self.width.maybe_set(value.width), height: self.height.maybe_set(value.height) }
    }
}

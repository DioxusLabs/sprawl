//! Style types for representing lengths / sizes
use super::CompactLength;
use crate::geometry::Rect;
use crate::style_helpers::{FromLength, FromPercent, TaffyAuto, TaffyZero};

/// A enum representing the tag of `LengthPercentage`
#[derive(Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum LengthPercentageTag {
    /// An absolute length in some abstract units.
    Length,
    /// A percentage length relative to the size of the containing block.
    Percentage,
    /// A `calc()` value.
    Calc
}

/// A enum representing the tag of a `LengthPercentage`
#[derive(Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ExpandedLengthPercentage {
    /// An absolute length in some abstract units. Users of Taffy may define what they correspond
    /// to in their application (pixels, logical pixels, mm, etc) as they see fit.
    Length(f32),
    /// A percentage length relative to the size of the containing block.
    ///
    /// **NOTE: percentages are represented as a f32 value in the range [0.0, 1.0] NOT the range [0.0, 100.0]**
    Percentage(f32),
    /// A `calc()` value. The value passed here is treated as an opaque handle to
    /// the actual calc representation and may be a pointer, index, etc.
    ///
    /// The low 3 bits are used as a tag value and will be returned as 0.
    Calc(*const ())
}

/// A unit of linear measurement
///
/// This is commonly combined with [`Rect`], [`Point`](crate::geometry::Point) and [`Size<T>`](crate::geometry::Size).
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct LengthPercentage(pub(crate) CompactLength);
impl TaffyZero for LengthPercentage {
    const ZERO: Self = Self(CompactLength::ZERO);
}
impl FromLength for LengthPercentage {
    fn from_length<Input: Into<f32> + Copy>(value: Input) -> Self {
        Self::length(value.into())
    }
}
impl FromPercent for LengthPercentage {
    fn from_percent<Input: Into<f32> + Copy>(value: Input) -> Self {
        Self::percent(value.into())
    }
}
impl LengthPercentage {
    /// An absolute length in some abstract units. Users of Taffy may define what they correspond
    /// to in their application (pixels, logical pixels, mm, etc) as they see fit.
    #[inline(always)]
    pub const fn length(val: f32) -> Self {
        Self(CompactLength::length(val))
    }

    /// A percentage length relative to the size of the containing block.
    ///
    /// **NOTE: percentages are represented as a f32 value in the range [0.0, 1.0] NOT the range [0.0, 100.0]**
    #[inline(always)]
    pub const fn percent(val: f32) -> Self {
        Self(CompactLength::percent(val))
    }

    /// A `calc()` value. The value passed here is treated as an opaque handle to
    /// the actual calc representation and may be a pointer, index, etc.
    ///
    /// The low 3 bits are used as a tag value and will be returned as 0.
    #[inline(always)]
    pub fn calc(ptr: *const ()) -> Self {
        Self(CompactLength::calc(ptr))
    }

    /// Create a LengthPercentage from a raw `CompactLength`.
    /// # Safety
    /// CompactLength must represent a valid variant for LengthPercentage
    #[allow(unsafe_code)]
    pub unsafe fn from_raw(val: CompactLength) -> Self {
        Self(val)
    }

    /// Get the underlying `CompactLength` representation of the value
    pub fn into_raw(self) -> CompactLength {
        self.0
    }

    /// Get the tag of the `LengthPercentage`
    pub fn tag(self) -> LengthPercentageTag {
        match self.0.tag() {
            CompactLength::LENGTH_TAG => LengthPercentageTag::Length,
            CompactLength::PERCENT_TAG => LengthPercentageTag::Percentage,
            _ => LengthPercentageTag::Calc
        }
    }

    /// Expand the compact `LengthPercentage` into a regular enum
    pub fn expanded(self) -> ExpandedLengthPercentage {
        match self.0.tag() {
            CompactLength::LENGTH_TAG => ExpandedLengthPercentage::Length(self.0.value()),
            CompactLength::PERCENT_TAG => ExpandedLengthPercentage::Percentage(self.0.value()),
            _ => ExpandedLengthPercentage::Calc(self.0.calc_value()),
        }
    }
}

/// A unit of linear measurement
///
/// This is commonly combined with [`Rect`], [`Point`](crate::geometry::Point) and [`Size<T>`](crate::geometry::Size).
#[derive(Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct LengthPercentageAuto(pub(crate) CompactLength);
impl TaffyZero for LengthPercentageAuto {
    const ZERO: Self = Self(CompactLength::ZERO);
}
impl TaffyAuto for LengthPercentageAuto {
    const AUTO: Self = Self(CompactLength::AUTO);
}
impl FromLength for LengthPercentageAuto {
    fn from_length<Input: Into<f32> + Copy>(value: Input) -> Self {
        Self::length(value.into())
    }
}
impl FromPercent for LengthPercentageAuto {
    fn from_percent<Input: Into<f32> + Copy>(value: Input) -> Self {
        Self::percent(value.into())
    }
}
impl From<LengthPercentage> for LengthPercentageAuto {
    fn from(input: LengthPercentage) -> Self {
        Self(input.0)
    }
}

impl LengthPercentageAuto {
    /// An absolute length in some abstract units. Users of Taffy may define what they correspond
    /// to in their application (pixels, logical pixels, mm, etc) as they see fit.
    #[inline(always)]
    pub const fn length(val: f32) -> Self {
        Self(CompactLength::length(val))
    }

    /// A percentage length relative to the size of the containing block.
    ///
    /// **NOTE: percentages are represented as a f32 value in the range [0.0, 1.0] NOT the range [0.0, 100.0]**
    #[inline(always)]
    pub const fn percent(val: f32) -> Self {
        Self(CompactLength::percent(val))
    }

    /// The dimension should be automatically computed according to algorithm-specific rules
    /// regarding the default size of boxes.
    #[inline(always)]
    pub const fn auto() -> Self {
        Self(CompactLength::auto())
    }

    /// A `calc()` value. The value passed here is treated as an opaque handle to
    /// the actual calc representation and may be a pointer, index, etc.
    ///
    /// The low 3 bits are used as a tag value and will be returned as 0.
    #[inline]
    pub fn calc(ptr: *const ()) -> Self {
        Self(CompactLength::calc(ptr))
    }

    /// Create a LengthPercentageAuto from a raw `CompactLength`.
    /// # Safety
    /// CompactLength must represent a valid variant for LengthPercentageAuto
    #[allow(unsafe_code)]
    pub unsafe fn from_raw(val: CompactLength) -> Self {
        Self(val)
    }

    /// Get the underlying `CompactLength` representation of the value
    pub fn into_raw(self) -> CompactLength {
        self.0
    }

    /// Returns:
    ///   - Some(length) for Length variants
    ///   - Some(resolved) using the provided context for Percent variants
    ///   - None for Auto variants
    #[inline(always)]
    pub fn resolve_to_option(self, context: f32, calc_resolver: impl Fn(*const (), f32) -> f32) -> Option<f32> {
        match self.0.tag() {
            CompactLength::LENGTH_TAG => Some(self.0.value()),
            CompactLength::PERCENT_TAG => Some(context * self.0.value()),
            CompactLength::AUTO_TAG => None,
            _ if self.0.is_calc() => Some(calc_resolver(self.0.calc_value(), context)),
            _ => unreachable!("LengthPercentageAuto values cannot be constructed with other tags"),
        }
    }

    /// Returns true if value is LengthPercentageAuto::Auto
    #[inline(always)]
    pub fn is_auto(self) -> bool {
        self.0.is_auto()
    }
}

/// A unit of linear measurement
///
/// This is commonly combined with [`Rect`], [`Point`](crate::geometry::Point) and [`Size<T>`](crate::geometry::Size).
#[derive(Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Dimension(pub(crate) CompactLength);
impl TaffyZero for Dimension {
    const ZERO: Self = Self(CompactLength::ZERO);
}
impl TaffyAuto for Dimension {
    const AUTO: Self = Self(CompactLength::AUTO);
}
impl FromLength for Dimension {
    fn from_length<Input: Into<f32> + Copy>(value: Input) -> Self {
        Self::length(value.into())
    }
}
impl FromPercent for Dimension {
    fn from_percent<Input: Into<f32> + Copy>(value: Input) -> Self {
        Self::percent(value.into())
    }
}
impl From<LengthPercentage> for Dimension {
    fn from(input: LengthPercentage) -> Self {
        Self(input.0)
    }
}
impl From<LengthPercentageAuto> for Dimension {
    fn from(input: LengthPercentageAuto) -> Self {
        Self(input.0)
    }
}

impl Dimension {
    /// An absolute length in some abstract units. Users of Taffy may define what they correspond
    /// to in their application (pixels, logical pixels, mm, etc) as they see fit.
    #[inline(always)]
    pub const fn length(val: f32) -> Self {
        Self(CompactLength::length(val))
    }

    /// A percentage length relative to the size of the containing block.
    ///
    /// **NOTE: percentages are represented as a f32 value in the range [0.0, 1.0] NOT the range [0.0, 100.0]**
    #[inline(always)]
    pub const fn percent(val: f32) -> Self {
        Self(CompactLength::percent(val))
    }

    /// The dimension should be automatically computed according to algorithm-specific rules
    /// regarding the default size of boxes.
    #[inline(always)]
    pub const fn auto() -> Self {
        Self(CompactLength::auto())
    }

    /// A `calc()` value. The value passed here is treated as an opaque handle to
    /// the actual calc representation and may be a pointer, index, etc.
    ///
    /// The low 3 bits are used as a tag value and will be returned as 0.
    #[inline]
    pub fn calc(ptr: *const ()) -> Self {
        Self(CompactLength::calc(ptr))
    }

    /// Create a LengthPercentageAuto from a raw `CompactLength`.
    /// # Safety
    /// CompactLength must represent a valid variant for LengthPercentageAuto
    #[allow(unsafe_code)]
    pub unsafe fn from_raw(val: CompactLength) -> Self {
        Self(val)
    }

    /// Get the underlying `CompactLength` representation of the value
    pub fn into_raw(self) -> CompactLength {
        self.0
    }

    /// Get Length value if value is Length variant
    #[cfg(feature = "grid")]
    pub fn into_option(self) -> Option<f32> {
        match self.0.tag() {
            CompactLength::LENGTH_TAG => Some(self.0.value()),
            _ => None,
        }
    }
    /// Returns true if value is Auto
    #[inline(always)]
    pub fn is_auto(self) -> bool {
        self.0.is_auto()
    }

    /// Get the raw `CompactLength` tag
    pub fn tag(self) -> usize {
        self.0.tag()
    }

    /// Get the raw `CompactLength` value for non-calc variants that have a numeric parameter
    pub fn value(self) -> f32 {
        self.0.value()
    }
}

impl Rect<Dimension> {
    /// Create a new Rect with length values
    #[must_use]
    pub const fn from_length(start: f32, end: f32, top: f32, bottom: f32) -> Self {
        Rect {
            left: Dimension(CompactLength::length(start)),
            right: Dimension(CompactLength::length(end)),
            top: Dimension(CompactLength::length(top)),
            bottom: Dimension(CompactLength::length(bottom)),
        }
    }

    /// Create a new Rect with percentage values
    #[must_use]
    pub const fn from_percent(start: f32, end: f32, top: f32, bottom: f32) -> Self {
        Rect {
            left: Dimension(CompactLength::percent(start)),
            right: Dimension(CompactLength::percent(end)),
            top: Dimension(CompactLength::percent(top)),
            bottom: Dimension(CompactLength::percent(bottom)),
        }
    }
}

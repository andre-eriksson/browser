use css_cssom::{ComponentValue, CssToken, CssTokenKind, NumericValue};
use strum::EnumString;

use crate::{CSSParsable, error::CssValueError};

/// Length units as defined in CSS Values and Units Module Level 4
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, EnumString)]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
pub enum LengthUnit {
    /// Equal to the "cap height" (nominal height of capital letters) of the element's font.
    Cap,

    /// Represents the width or, more precisely, the advance measure of the glyph `0` (zero, the Unicode character U+0030)
    /// in the element's font. In cases where determining the measure of the `0` glyph is impossible or impractical,
    /// it must be assumed to be `0.5em` wide by `1em` tall.
    Ch,

    /// Represents the calculated font-size of the element. If used on the font-size property itself,
    /// it represents the inherited font-size of the element.
    Em,

    /// Equal to the x-height of the element's font. In fonts with the `x` letter, this is generally the
    /// height of lowercase letters in the font; `1ex ≈ 0.5em` in many fonts.
    Ex,

    /// Represents the used advance measure of the "水" glyph (CJK water ideograph, U+6C34), found in the font used to render it.
    Ic,

    /// Equal to the computed value of the line-height property of the element on which it is used, converted to an absolute length.
    /// This unit enables length calculations based on the theoretical size of an ideal empty line. However, the size of actual
    /// line boxes may differ based on their content.
    Lh,

    /// Equal to the "cap height" (nominal height of capital letters) of the root element's font.
    Rcap,

    /// Equal to the width or the advance measure of the glyph 0 (zero, the Unicode character U+0030) in the root element's font.
    Rch,

    /// Represents the font-size of the root element (typically `<html>`). When used within the root element font-size,
    /// it represents its initial value. The default is `16px` (for this browser), but user-defined preferences may modify this.
    Rem,

    /// Equal to the x-height of the root element's font.
    Rex,

    /// Equal to the value of `ic` unit on the root element's font.
    Ric,

    /// Equal to the value of `lh` unit on the root element's font.
    /// This unit enables length calculations based on the theoretical size of an ideal empty line.
    /// However, the size of actual line boxes may differ based on their content.
    Rlh,

    /// Represents a percentage of the height of the viewport's initial containing block.
    /// `1vh` is 1% of the viewport height. For example, if the viewport height is `300px`,
    /// then a value of `70vh` on a property will be `210px`.
    Vw,

    /// Represents a percentage of the width of the viewport's initial containing block.
    /// `1vw` is 1% of the viewport width. For example, if the viewport width is `800px`,
    /// then a value of `50vw` on a property will be `400px`.
    Vh,

    /// Represents in percentage the largest of `vw` and `vh`.
    Vmax,

    /// Represents in percentage the smallest of `vw` and `vh`.
    Vmin,

    /// Represents the percentage of the size of the initial containing block, in the direction of the root element's block axis.
    Vb,

    /// Represents a percentage of the size of the initial containing block, in the direction of the root element's inline axis.
    Vi,

    /// The small viewport height variant, see `vh` for details.
    Svh,

    /// The small viewport width variant, see `vw` for details.
    Svw,

    /// The small viewport larger dimension variant, see `vmax` for details.
    Svmax,

    /// The small viewport smaller dimension variant, see `vmin` for details.
    Svmin,

    /// The small viewport block size variant, see `vb` for details.
    Svb,

    /// The small viewport inline size variant, see `vi` for details.
    Svi,

    /// The large viewport height variant, see `vh` for details.
    Lvh,

    /// The large viewport width variant, see `vw` for details.
    Lvw,

    /// The large viewport larger dimension variant, see `vmax` for details.
    Lvmax,

    /// The large viewport smaller dimension variant, see `vmin` for details.
    Lvmin,

    /// The large viewport block size variant, see `vb` for details.
    Lvb,

    /// The large viewport inline size variant, see `vi` for details.
    Lvi,

    /// The dynamic viewport height variant, see `vh` for details.
    Dvh,

    /// The dynamic viewport width variant, see `vw` for details.
    Dvw,

    /// The dynamic viewport larger dimension variant, see `vmax` for details.
    Dvmax,

    /// The dynamic viewport smaller dimension variant, see `vmin` for details.
    Dvmin,

    /// The dynamic viewport block size variant, see `vb` for details.
    Dvb,

    /// The dynamic viewport inline size variant, see `vi` for details.
    Dvi,

    /// Represents a percentage of the width of the query container.
    /// `1cqw` is 1% of the query container's width. For example, if the query container's width is `800px`,
    /// then a value of `50cqw` on a property will be `400px`.
    Cqw,

    /// Represents a percentage of the height of the query container.
    /// `1cqh` is 1% of the query container's height. For example, if the query container's height is `300px`,
    /// then a value of `10cqh` on a property will be `30px`.
    Cqh,

    /// Represents a percentage of the inline size of the query container.
    /// `1cqi` is 1% of the query container's inline size. For example, if the query container's inline size is `800px`,
    /// then a value of `50cqi` on a property will be `400px`.
    Cqi,

    /// Represents a percentage of the block size of the query container.
    /// `1cqb` is 1% of the query container's block size. For example, if the query container's block size is `300px`,
    /// then a value of `10cqb` on a property will be `30px`.
    Cqb,

    /// Represents a percentage of the smaller value of either the query container's inline size or block size.
    /// `1cqmin` is 1% of the smaller value of either the query container's inline size or block size. For example,
    /// if the query container's inline size is `800px` and its block size is `300px`, then a value of `50cqmin` on a
    /// property will be `150px`.
    Cqmin,

    /// Represents a percentage of the larger value of either the query container's inline size or block size.
    /// `1cqmax` is 1% of the larger value of either the query container's inline size or block size. For example,
    /// if the query container's inline size is `800px` and its block size is `300px`, then a value of `50cqmax` on a
    /// property will be `400px`.
    Cqmax,

    /// One pixel. For screen displays, it traditionally represents one device pixel (dot).
    /// However, for printers and high-resolution screens, one CSS pixel implies multiple device pixels.
    /// `1px` = `1in / 96`.
    #[default]
    Px,

    /// One centimeter. `1cm` = `96px / 2.54`.
    Cm,

    /// One millimeter. `1mm` = `1cm / 10`.
    Mm,

    /// One quarter of a millimeter. `1Q` = `1cm / 40`.
    Q,

    /// One inch. `1in` = `2.54cm = 96px`.
    In,

    /// One pica. `1pc` = `12pt = 1in / 6`.
    Pc,

    /// One point. `1pt` = `1in / 72`.
    Pt,
}

/// Represents a CSS length value with a numeric value and a unit.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Length {
    /// The numeric value of the length.
    value: f64,

    /// The unit of the length.
    unit: LengthUnit,
}

impl Length {
    /// Creates a new Length with the given value and unit.
    #[must_use]
    pub const fn new(value: f64, unit: LengthUnit) -> Self {
        Self { value, unit }
    }

    /// Returns the numeric value of the length.
    #[must_use]
    pub const fn value(&self) -> f64 {
        self.value
    }

    /// Returns the unit of the length.
    #[must_use]
    pub const fn unit(&self) -> LengthUnit {
        self.unit
    }

    /// Constructs a Length of zero, for convenience.
    #[must_use]
    pub const fn zero() -> Self {
        Self {
            value: 0.0,
            unit: LengthUnit::Px,
        }
    }

    /// Constructs a Length in pixels, for convenience.
    #[must_use]
    pub const fn px(value: f64) -> Self {
        Self {
            value,
            unit: LengthUnit::Px,
        }
    }
}

impl TryFrom<&CssToken> for Length {
    type Error = CssValueError;

    fn try_from(token: &CssToken) -> Result<Self, Self::Error> {
        match &token.kind {
            CssTokenKind::Dimension { value, unit } => {
                let unit = LengthUnit::try_from(unit.as_str()).map_err(|_| CssValueError::InvalidUnit(unit.clone()))?;

                Ok(Self::new(value.to_f64(), unit))
            }
            _ => Err(CssValueError::InvalidToken(token.kind.clone())),
        }
    }
}

impl TryFrom<&ComponentValue> for Length {
    type Error = CssValueError;

    fn try_from(cv: &ComponentValue) -> Result<Self, Self::Error> {
        cv.as_token()
            .map_or_else(|| Err(CssValueError::InvalidComponentValue(cv.clone())), Self::try_from)
    }
}

impl CSSParsable for Length {
    fn parse(stream: &mut css_cssom::ComponentValueStream) -> Result<Self, CssValueError> {
        stream
            .next_non_whitespace()
            .map_or(Err(CssValueError::UnexpectedEndOfInput), Self::try_from)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, EnumString)]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
pub enum AngleUnit {
    Deg,
    Rad,
    Grad,
    Turn,
}

/// Angle representation for CSS properties that accept angles, such as hue in HSL colors or rotation in transforms
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Angle {
    /// The numeric value of the angle
    value: f64,

    /// The unit of the angle
    unit: AngleUnit,
}

impl Angle {
    /// Creates a new Angle with the given value and unit.
    #[must_use]
    pub const fn new(value: f64, unit: AngleUnit) -> Self {
        Self { value, unit }
    }

    /// A shorthand constructor for creating an Angle with the degrees unit, for convenience.
    #[must_use]
    pub const fn deg(value: f64) -> Self {
        Self {
            value,
            unit: AngleUnit::Deg,
        }
    }

    /// A shorthand constructor for creating an Angle with the radians unit, for convenience.
    #[must_use]
    pub const fn rad(value: f64) -> Self {
        Self {
            value,
            unit: AngleUnit::Rad,
        }
    }

    /// A shorthand constructor for creating an Angle with the gradians unit, for convenience.
    #[must_use]
    pub const fn grad(value: f64) -> Self {
        Self {
            value,
            unit: AngleUnit::Grad,
        }
    }

    /// A shorthand constructor for creating an Angle with the turns unit, for convenience.
    #[must_use]
    pub const fn turn(value: f64) -> Self {
        Self {
            value,
            unit: AngleUnit::Turn,
        }
    }

    /// Convert the angle to degrees
    #[must_use]
    pub fn to_degrees(self) -> f64 {
        match self.unit {
            AngleUnit::Deg => self.value,
            AngleUnit::Rad => self.value.to_degrees(),
            AngleUnit::Grad => self.value * 0.9,
            AngleUnit::Turn => self.value * 360.0,
        }
    }

    /// Convert the angle to radians
    #[must_use]
    pub fn to_radians(self) -> f64 {
        match self.unit {
            AngleUnit::Deg => self.value.to_radians(),
            AngleUnit::Rad => self.value,
            AngleUnit::Grad => (self.value * 0.9).to_radians(),
            AngleUnit::Turn => self.value * 2.0 * std::f64::consts::PI,
        }
    }

    /// Convert f32 degrees to an Angle, normalizing it to the [0, 360) range
    #[must_use]
    pub fn from_degrees(deg: f64) -> Self {
        Self {
            value: (deg / 360.0).fract() * 360.0,
            unit: AngleUnit::Deg,
        }
    }

    /// Convert f32 radians to an Angle, normalizing it to the [0, 2π) range
    #[must_use]
    pub fn from_radians(rad: f64) -> Self {
        Self {
            value: (rad / (2.0 * std::f64::consts::PI)).fract() * (2.0 * std::f64::consts::PI),
            unit: AngleUnit::Rad,
        }
    }

    /// Convert f32 gradians to an Angle, normalizing it to the [0, 400) range
    #[must_use]
    pub fn from_gradians(grad: f64) -> Self {
        Self {
            value: (grad / 400.0).fract() * 400.0,
            unit: AngleUnit::Grad,
        }
    }

    /// Convert f32 turns to an Angle, normalizing it to the [0, 1) range
    #[must_use]
    pub const fn from_turns(turn: f64) -> Self {
        Self {
            value: turn.fract(),
            unit: AngleUnit::Turn,
        }
    }
}

impl TryFrom<&CssToken> for Angle {
    type Error = CssValueError;

    fn try_from(token: &CssToken) -> Result<Self, Self::Error> {
        match &token.kind {
            CssTokenKind::Dimension { value, unit } => {
                let angle_unit =
                    AngleUnit::try_from(unit.as_str()).map_err(|_| CssValueError::InvalidUnit(unit.clone()))?;

                Ok(Self {
                    value: value.to_f64(),
                    unit: angle_unit,
                })
            }
            CssTokenKind::Number(value) => Ok(Self::from_degrees(value.to_f64())),
            _ => Err(CssValueError::InvalidToken(token.kind.clone())),
        }
    }
}

impl CSSParsable for Angle {
    fn parse(stream: &mut css_cssom::ComponentValueStream) -> Result<Self, CssValueError> {
        stream
            .next_non_whitespace()
            .map_or(Err(CssValueError::UnexpectedEndOfInput), |cv| {
                cv.as_token()
                    .map_or_else(|| Err(CssValueError::InvalidComponentValue(cv.clone())), Self::try_from)
            })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, EnumString)]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
pub enum TimeUnit {
    S,
    Ms,
}

/// The <time> CSS data type represents a time value, which can be specified in seconds or milliseconds. It is commonly used in properties like `animation-duration` and `transition-delay`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Time {
    /// The numeric value of the time
    value: f64,

    /// The unit of the time
    unit: TimeUnit,
}

impl Time {
    /// Creates a new Time with the given value and unit.
    #[must_use]
    pub const fn new(value: f64, unit: TimeUnit) -> Self {
        Self { value, unit }
    }

    /// A shorthand constructor for creating a Time with the seconds unit, for convenience.
    #[must_use]
    pub const fn s(value: f64) -> Self {
        Self {
            value,
            unit: TimeUnit::S,
        }
    }

    /// A shorthand constructor for creating a Time with the milliseconds unit, for convenience.
    #[must_use]
    pub const fn ms(value: f64) -> Self {
        Self {
            value,
            unit: TimeUnit::Ms,
        }
    }

    /// Convert the time to seconds
    #[must_use]
    pub const fn to_seconds(self) -> f64 {
        match self.unit {
            TimeUnit::S => self.value,
            TimeUnit::Ms => self.value / 1000.0,
        }
    }

    /// Convert the time to milliseconds
    #[must_use]
    pub const fn to_milliseconds(self) -> f64 {
        match self.unit {
            TimeUnit::S => self.value * 1000.0,
            TimeUnit::Ms => self.value,
        }
    }
}

impl TryFrom<&CssToken> for Time {
    type Error = CssValueError;

    fn try_from(token: &CssToken) -> Result<Self, Self::Error> {
        match &token.kind {
            CssTokenKind::Dimension { value, unit } => {
                let time_unit =
                    TimeUnit::try_from(unit.as_str()).map_err(|_| CssValueError::InvalidUnit(unit.clone()))?;

                Ok(Self {
                    value: value.to_f64(),
                    unit: time_unit,
                })
            }
            _ => Err(CssValueError::InvalidToken(token.kind.clone())),
        }
    }
}

impl CSSParsable for Time {
    fn parse(stream: &mut css_cssom::ComponentValueStream) -> Result<Self, CssValueError> {
        stream
            .next_non_whitespace()
            .map_or(Err(CssValueError::UnexpectedEndOfInput), |cv| {
                cv.as_token()
                    .map_or_else(|| Err(CssValueError::InvalidComponentValue(cv.clone())), Self::try_from)
            })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, EnumString)]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
pub enum FrequencyUnit {
    Hz,
    KHz,
}

/// The <frequency> CSS data type represents a frequency value, which can be specified in hertz or kilohertz.
/// It is currently unused.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Frequency {
    /// The numeric value of the frequency
    value: f64,

    /// The unit of the frequency
    unit: FrequencyUnit,
}

impl Frequency {
    /// Creates a new Frequency with the given value and unit.
    #[must_use]
    pub const fn new(value: f64, unit: FrequencyUnit) -> Self {
        Self { value, unit }
    }

    /// A shorthand constructor for creating a Frequency with the hertz unit, for convenience.
    #[must_use]
    pub const fn hz(value: f64) -> Self {
        Self {
            value,
            unit: FrequencyUnit::Hz,
        }
    }

    /// A shorthand constructor for creating a Frequency with the kilohertz unit, for convenience.
    #[must_use]
    pub const fn khz(value: f64) -> Self {
        Self {
            value,
            unit: FrequencyUnit::KHz,
        }
    }

    /// Convert the frequency to hertz
    #[must_use]
    pub const fn to_hertz(self) -> f64 {
        match self.unit {
            FrequencyUnit::Hz => self.value,
            FrequencyUnit::KHz => self.value * 1000.0,
        }
    }

    /// Convert the frequency to kilohertz
    #[must_use]
    pub const fn to_kilohertz(self) -> f64 {
        match self.unit {
            FrequencyUnit::Hz => self.value / 1000.0,
            FrequencyUnit::KHz => self.value,
        }
    }
}

impl TryFrom<&CssToken> for Frequency {
    type Error = CssValueError;

    fn try_from(token: &CssToken) -> Result<Self, Self::Error> {
        match &token.kind {
            CssTokenKind::Dimension { value, unit } => {
                let frequency_unit =
                    FrequencyUnit::try_from(unit.as_str()).map_err(|_| CssValueError::InvalidUnit(unit.clone()))?;

                Ok(Self {
                    value: value.to_f64(),
                    unit: frequency_unit,
                })
            }
            _ => Err(CssValueError::InvalidToken(token.kind.clone())),
        }
    }
}

impl CSSParsable for Frequency {
    fn parse(stream: &mut css_cssom::ComponentValueStream) -> Result<Self, CssValueError> {
        stream
            .next_non_whitespace()
            .map_or(Err(CssValueError::UnexpectedEndOfInput), |cv| {
                cv.as_token()
                    .map_or_else(|| Err(CssValueError::InvalidComponentValue(cv.clone())), Self::try_from)
            })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, EnumString)]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
pub enum ResolutionUnit {
    Dpi,
    Dpcm,
    Dppx,
}

/// The <resolution> CSS data type
///
/// Represents a resolution value, which can be specified in dots per inch, dots per centimeter, or dots per pixel.
/// It is commonly used in media queries to specify the resolution of the output device.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Resolution {
    /// The numeric value of the resolution
    value: f64,

    /// The unit of the resolution
    unit: ResolutionUnit,
}

impl Resolution {
    /// Creates a new Resolution with the given value and unit.
    #[must_use]
    pub const fn new(value: f64, unit: ResolutionUnit) -> Self {
        Self { value, unit }
    }

    /// A shorthand constructor for creating a Resolution with the dots per inch unit, for convenience.
    #[must_use]
    pub const fn dpi(value: f64) -> Self {
        Self {
            value,
            unit: ResolutionUnit::Dpi,
        }
    }

    /// A shorthand constructor for creating a Resolution with the dots per centimeter unit, for convenience.
    #[must_use]
    pub const fn dpcm(value: f64) -> Self {
        Self {
            value,
            unit: ResolutionUnit::Dpcm,
        }
    }

    /// A shorthand constructor for creating a Resolution with the dots per pixel unit, for convenience.
    #[must_use]
    pub const fn dppx(value: f64) -> Self {
        Self {
            value,
            unit: ResolutionUnit::Dppx,
        }
    }

    /// Convert the resolution to dots per inch
    #[must_use]
    pub const fn to_dpi(self) -> f64 {
        match self.unit {
            ResolutionUnit::Dpi => self.value,
            ResolutionUnit::Dpcm => self.value * 2.54,
            ResolutionUnit::Dppx => self.value * 96.0,
        }
    }

    /// Convert the resolution to dots per centimeter
    #[must_use]
    pub const fn to_dpcm(self) -> f64 {
        match self.unit {
            ResolutionUnit::Dpi => self.value / 2.54,
            ResolutionUnit::Dpcm => self.value,
            ResolutionUnit::Dppx => self.value * (96.0 / 2.54),
        }
    }

    /// Convert the resolution to dots per pixel
    #[must_use]
    pub const fn to_dppx(self) -> f64 {
        match self.unit {
            ResolutionUnit::Dpi => self.value / 96.0,
            ResolutionUnit::Dpcm => self.value / (96.0 / 2.54),
            ResolutionUnit::Dppx => self.value,
        }
    }
}

impl TryFrom<&CssToken> for Resolution {
    type Error = CssValueError;

    fn try_from(token: &CssToken) -> Result<Self, Self::Error> {
        match &token.kind {
            CssTokenKind::Dimension { value, unit } => {
                let resolution_unit =
                    ResolutionUnit::try_from(unit.as_str()).map_err(|_| CssValueError::InvalidUnit(unit.clone()))?;

                Ok(Self {
                    value: value.to_f64(),
                    unit: resolution_unit,
                })
            }
            _ => Err(CssValueError::InvalidToken(token.kind.clone())),
        }
    }
}

impl CSSParsable for Resolution {
    fn parse(stream: &mut css_cssom::ComponentValueStream) -> Result<Self, CssValueError> {
        stream
            .next_non_whitespace()
            .map_or(Err(CssValueError::UnexpectedEndOfInput), |cv| {
                cv.as_token()
                    .map_or_else(|| Err(CssValueError::InvalidComponentValue(cv.clone())), Self::try_from)
            })
    }
}

/// The `Dimension` enum represents a CSS dimension value, which can be a length, angle, time, frequency, or resolution.
///
/// <https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Values/dimension>
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Dimension {
    Length(Length),
    Angle(Angle),
    Time(Time),
    Frequency(Frequency),
    Resolution(Resolution),
}

impl Dimension {
    pub fn parse(value: &NumericValue, unit: &str) -> Result<Self, CssValueError> {
        if let Ok(length_unit) = LengthUnit::try_from(unit) {
            Ok(Self::Length(Length::new(value.to_f64(), length_unit)))
        } else if let Ok(angle_unit) = AngleUnit::try_from(unit) {
            Ok(Self::Angle(Angle {
                value: value.to_f64(),
                unit: angle_unit,
            }))
        } else if let Ok(time_unit) = TimeUnit::try_from(unit) {
            Ok(Self::Time(Time {
                value: value.to_f64(),
                unit: time_unit,
            }))
        } else if let Ok(frequency_unit) = FrequencyUnit::try_from(unit) {
            Ok(Self::Frequency(Frequency {
                value: value.to_f64(),
                unit: frequency_unit,
            }))
        } else if let Ok(resolution_unit) = ResolutionUnit::try_from(unit) {
            Ok(Self::Resolution(Resolution {
                value: value.to_f64(),
                unit: resolution_unit,
            }))
        } else {
            Err(CssValueError::InvalidUnit(unit.to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_degrees() {
        let angle_deg = Angle::deg(90.0);
        assert_eq!(angle_deg.to_degrees(), 90.0);

        let angle_rad = Angle::rad(std::f64::consts::PI / 2.0);
        assert!((angle_rad.to_degrees() - 90.0).abs() < 1e-6);

        let angle_grad = Angle::grad(100.0);
        assert_eq!(angle_grad.to_degrees(), 90.0);

        let angle_turn = Angle::turn(0.25);
        assert_eq!(angle_turn.to_degrees(), 90.0);
    }
}

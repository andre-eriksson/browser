use css_cssom::{CssToken, CssTokenKind};
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
    value: f32,

    /// The unit of the length.
    unit: LengthUnit,
}

impl Length {
    /// Creates a new Length with the given value and unit.
    pub const fn new(value: f32, unit: LengthUnit) -> Self {
        Self { value, unit }
    }

    /// Returns the numeric value of the length.
    pub const fn value(&self) -> f32 {
        self.value
    }

    /// Returns the unit of the length.
    pub const fn unit(&self) -> LengthUnit {
        self.unit
    }

    /// Constructs a Length of zero, for convenience.
    pub const fn zero() -> Self {
        Self {
            value: 0.0,
            unit: LengthUnit::Px,
        }
    }

    /// Constructs a Length in pixels, for convenience.
    pub const fn px(value: f32) -> Self {
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

                Ok(Self::new(value.to_f64() as f32, unit))
            }
            _ => Err(CssValueError::InvalidToken(token.kind.clone())),
        }
    }
}

impl CSSParsable for Length {
    fn parse(stream: &mut css_cssom::ComponentValueStream) -> Result<Self, CssValueError> {
        stream
            .next_non_whitespace()
            .map_or(Err(CssValueError::UnexpectedEndOfInput), |cv| {
                cv.as_token()
                    .map_or_else(|| Err(CssValueError::InvalidComponentValue(cv.clone())), Self::try_from)
            })
    }
}

/// Angle representation for CSS properties that accept angles, such as hue in HSL colors or rotation in transforms
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Angle {
    /// Degrees (e.g., "45deg")
    ///
    /// Note: The value is normalized to the [0, 360) range, so "450deg" would be treated as "90deg".
    Deg(f32),

    /// Radians (e.g., "3.14rad")
    ///
    /// Note: The value is normalized to the [0, 2π) range, so "7.28rad" would be treated as "0.28rad".
    Rad(f32),

    /// Gradians (e.g., "100grad")
    ///
    /// Note: The value is normalized to the [0, 400) range, so "450grad" would be treated as "50grad".
    Grad(f32),

    /// Turns (e.g., "0.5turn")
    ///
    /// Note: The value is normalized to the [0, 1) range, so "1.5turn" would be treated as "0.5turn".
    Turn(f32),
}

impl Angle {
    /// Convert the angle to degrees
    pub fn to_degrees(self) -> f32 {
        match self {
            Self::Deg(v) => v,
            Self::Rad(v) => v.to_degrees(),
            Self::Grad(v) => v * 0.9,
            Self::Turn(v) => v * 360.0,
        }
    }

    /// Convert the angle to radians
    pub fn to_radians(self) -> f32 {
        match self {
            Self::Deg(v) => v.to_radians(),
            Self::Rad(v) => v,
            Self::Grad(v) => (v * 0.9).to_radians(),
            Self::Turn(v) => v * 2.0 * std::f32::consts::PI,
        }
    }

    /// Convert f32 degrees to an Angle, normalizing it to the [0, 360) range
    pub fn from_degrees(deg: f32) -> Self {
        Self::Deg((deg / 360.0).fract() * 360.0)
    }

    /// Convert f32 radians to an Angle, normalizing it to the [0, 2π) range
    pub fn from_radians(rad: f32) -> Self {
        Self::Rad((rad / (2.0 * std::f32::consts::PI)).fract() * (2.0 * std::f32::consts::PI))
    }

    /// Convert f32 gradians to an Angle, normalizing it to the [0, 400) range
    pub fn from_gradians(grad: f32) -> Self {
        Self::Grad((grad / 400.0).fract() * 400.0)
    }

    /// Convert f32 turns to an Angle, normalizing it to the [0, 1) range
    pub const fn from_turns(turn: f32) -> Self {
        Self::Turn((turn).fract())
    }
}

impl TryFrom<&CssToken> for Angle {
    type Error = CssValueError;

    fn try_from(token: &CssToken) -> Result<Self, Self::Error> {
        match &token.kind {
            CssTokenKind::Dimension { value, unit } => {
                let unit_str = unit.to_ascii_lowercase();
                match unit_str.as_str() {
                    "deg" => Ok(Self::from_degrees(value.to_f64() as f32)),
                    "rad" => Ok(Self::from_radians(value.to_f64() as f32)),
                    "grad" => Ok(Self::from_gradians(value.to_f64() as f32)),
                    "turn" => Ok(Self::from_turns(value.to_f64() as f32)),
                    _ => Ok(Self::from_degrees(value.to_f64() as f32)),
                }
            }
            CssTokenKind::Number(value) => Ok(Self::from_degrees(value.to_f64() as f32)),
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

/// The <time> CSS data type represents a time value, which can be specified in seconds or milliseconds. It is commonly used in properties like `animation-duration` and `transition-delay`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Time {
    /// Seconds (e.g., "2s")
    ///
    /// Note: The value is normalized to a non-negative number, so "-1s" would be treated as "0s".
    Seconds(f32),

    /// Milliseconds (e.g., "500ms")
    ///
    /// Note: The value is normalized to a non-negative number, so "-500ms" would be treated as "0ms".
    Milliseconds(f32),
}

impl TryFrom<&CssToken> for Time {
    type Error = CssValueError;

    fn try_from(token: &CssToken) -> Result<Self, Self::Error> {
        match &token.kind {
            CssTokenKind::Dimension { value, unit } => {
                if unit.eq_ignore_ascii_case("s") {
                    Ok(Self::Seconds(value.to_f64() as f32))
                } else if unit.eq_ignore_ascii_case("ms") {
                    Ok(Self::Milliseconds(value.to_f64() as f32))
                } else {
                    Err(CssValueError::InvalidUnit(unit.clone()))
                }
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

/// The <frequency> CSS data type represents a frequency value, which can be specified in hertz or kilohertz.
/// It is currently unused.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Frequency {
    /// Hertz (e.g., "440Hz")
    ///
    /// Note: The value is normalized to a non-negative number, so "-440Hz" would be treated as "0Hz".
    Hz(f32),

    /// Kilohertz (e.g., "1.5kHz")
    ///
    /// Note: The value is normalized to a non-negative number, so "-1.5kHz" would be treated as "0kHz".
    KHz(f32),
}

impl TryFrom<&CssToken> for Frequency {
    type Error = CssValueError;

    fn try_from(token: &CssToken) -> Result<Self, Self::Error> {
        match &token.kind {
            CssTokenKind::Dimension { value, unit } => {
                if unit.eq_ignore_ascii_case("hz") {
                    Ok(Self::Hz(value.to_f64() as f32))
                } else if unit.eq_ignore_ascii_case("khz") {
                    Ok(Self::KHz(value.to_f64() as f32))
                } else {
                    Err(CssValueError::InvalidUnit(unit.clone()))
                }
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

/// The <resolution> CSS data type
///
/// Represents a resolution value, which can be specified in dots per inch, dots per centimeter, or dots per pixel.
/// It is commonly used in media queries to specify the resolution of the output device.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Resolution {
    /// Dots per inch (e.g., "300dpi")
    ///
    /// Note: The value is normalized to a non-negative number, so "-300dpi" would be treated as "0dpi".
    Dpi(f32),

    /// Dots per centimeter (e.g., "118dpcm")
    ///
    /// Note: The value is normalized to a non-negative number, so "-118dpcm" would be treated as "0dpcm".
    Dpcm(f32),

    /// Dots per pixel (e.g., "2dppx")
    ///
    /// Note: The value is normalized to a non-negative number, so "-2dppx" would be treated as "0dppx".
    Dppx(f32),
}

impl TryFrom<&CssToken> for Resolution {
    type Error = CssValueError;

    fn try_from(token: &CssToken) -> Result<Self, Self::Error> {
        match &token.kind {
            CssTokenKind::Dimension { value, unit } => {
                if unit.eq_ignore_ascii_case("dpi") {
                    Ok(Self::Dpi(value.to_f64() as f32))
                } else if unit.eq_ignore_ascii_case("dpcm") {
                    Ok(Self::Dpcm(value.to_f64() as f32))
                } else if unit.eq_ignore_ascii_case("dppx") {
                    Ok(Self::Dppx(value.to_f64() as f32))
                } else {
                    Err(CssValueError::InvalidUnit(unit.clone()))
                }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_degrees() {
        let angle_deg = Angle::Deg(90.0);
        assert_eq!(angle_deg.to_degrees(), 90.0);

        let angle_rad = Angle::Rad(std::f32::consts::PI / 2.0);
        assert!((angle_rad.to_degrees() - 90.0).abs() < 1e-6);

        let angle_grad = Angle::Grad(100.0);
        assert_eq!(angle_grad.to_degrees(), 90.0);

        let angle_turn = Angle::Turn(0.25);
        assert_eq!(angle_turn.to_degrees(), 90.0);
    }
}

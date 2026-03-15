use std::ops::RangeInclusive;

use css_cssom::{ComponentValue, ComponentValueStream, CssTokenKind};

use crate::{
    CSSParsable,
    color::{
        base::{ColorBase, HexColor},
        function::ColorFunction,
        named::NamedColor,
        system::SystemColor,
    },
    error::CssValueError,
    numeric::Percentage,
};

pub mod base;
pub mod function;
pub mod named;
pub mod system;

/// Alpha can be specified as a number (e.g., "0.5"), a percentage (e.g., "50%"), or "none" (which is treated as 1.0).
///
/// Always in the range [0.0, 1.0]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Alpha(f32);

impl Alpha {
    /// Create a new Alpha value from a floating-point number, which is clamped to the range [0.0, 1.0].
    pub fn new(value: f32) -> Self {
        Alpha(value.clamp(0.0, 1.0))
    }

    /// Get the alpha value as a floating-point number in the range [0.0, 1.0].
    pub fn value(&self) -> f32 {
        self.0.clamp(0.0, 1.0)
    }
}

impl From<Percentage> for Alpha {
    fn from(value: Percentage) -> Self {
        Self((value.as_fraction()).clamp(0.0, 1.0))
    }
}

/// The hue component of a color can be specified as an angle (e.g., "120deg", "2.094rad", "133.33grad", "0.333turn") or as a number (e.g., "120"), which is treated as degrees.
///
/// Is represented as a floating-point number in degrees, normalized to the range [0, 360).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hue(f32);

impl Hue {
    /// Get the hue value as a floating-point number in degrees, normalized to the range [0, 360).
    pub fn value(&self) -> f32 {
        self.0.rem_euclid(360.0)
    }
}

impl From<ColorValue> for Hue {
    fn from(value: ColorValue) -> Self {
        match value {
            ColorValue::Number(n) => Hue(n),
            ColorValue::Percentage(p) => Hue(p.as_fraction() * 360.0),
        }
    }
}

/// Indicates how to interpret percentage values when converting them to numbers for color components.
pub enum Fraction {
    /// Treat percentage values as unsigned fractions, where 0% corresponds to the start of the range and 100% corresponds to the end of the range.
    Unsigned,

    /// Treat percentage values as signed fractions, where 0% corresponds to the middle of the range, 100% corresponds to the end of the range,
    /// and -100% corresponds to the start of the range.
    Signed,
}

/// A color component value can be specified as a number (e.g., "255") or a percentage (e.g., "100%").
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorValue {
    /// A number, which is clamped to the specified range (e.g., 0-255 for RGB components).
    Number(f32),

    /// A percentage, which is converted to a number based on the specified range (e.g., 100% would be 255 for RGB components).
    Percentage(Percentage),
}

impl ColorValue {
    /// Convert the ColorValue to a number based on the specified range and fraction type.
    ///
    /// For Number, the value is clamped to the range.
    /// For Percentage, the value is converted to a fraction and then linearly interpolated within the range.
    /// If the fraction type is Signed, the percentage is treated as a signed value where 0% corresponds to the start of the range,
    /// 100% corresponds to the end of the range, and -100% corresponds to the start of the range.
    ///
    /// # Example
    /// ```rust
    /// use css_values::{color::{ColorValue, Fraction}, numeric::Percentage};
    ///
    /// let percentage = ColorValue::Percentage(Percentage::new(50.0));
    /// assert_eq!(percentage.value(0.0..=255.0, Fraction::Unsigned), 127.5);
    /// assert_eq!(percentage.value(0.0..=255.0, Fraction::Signed), 191.25);
    /// ```
    pub fn value(&self, range: RangeInclusive<f32>, fraction: Fraction) -> f32 {
        match self {
            ColorValue::Number(n) => n.clamp(*range.start(), *range.end()),
            ColorValue::Percentage(p) => match fraction {
                Fraction::Unsigned => Self::lerp(p.as_fraction(), range),
                Fraction::Signed => Self::signed_lerp(p.as_fraction(), range),
            },
        }
    }

    /// Linearly interpolate a fraction (0.0 to 1.0) within the specified range.
    fn lerp(fraction: f32, range: RangeInclusive<f32>) -> f32 {
        let t = fraction.clamp(0.0, 1.0);
        *range.start() + t * (*range.end() - *range.start())
    }

    /// Linearly interpolate a signed fraction (-1.0 to 1.0) within the specified range, where 0.0 corresponds to the start of the range,
    /// 1.0 corresponds to the end of the range, and -1.0 corresponds to the start of the range.
    fn signed_lerp(fraction: f32, range: RangeInclusive<f32>) -> f32 {
        let t = (fraction.clamp(-1.0, 1.0) + 1.0) / 2.0;
        range.start() + t * (range.end() - range.start())
    }
}

impl From<f32> for ColorValue {
    fn from(value: f32) -> Self {
        ColorValue::Number(value)
    }
}

/// Represents the <color> data type in CSS, which can be specified using various formats such as named colors,
/// hexadecimal colors, functional notations (e.g., rgb(), hsl()), system colors, and the currentColor keyword.
#[derive(Debug, Clone, PartialEq)]
pub enum Color {
    Base(ColorBase),
    /// The 'currentColor' keyword represents the current value of the 'color' property.
    Current,
    System(SystemColor),
    /// The light-dark() function allows authors to specify two colors: one for light mode and one for dark mode.
    /// The user agent will use the appropriate color based on the user's preferred color scheme, currently the app theme.
    LightDark(Box<Self>, Box<Self>),
    // TODO: contrast-color()
    // TODO: device-cmyk()
}

impl Color {
    pub const BLACK: Self = Self::Base(ColorBase::Named(NamedColor::Black));
}

impl Default for Color {
    fn default() -> Self {
        Color::Base(ColorBase::Named(NamedColor::Black))
    }
}

impl TryFrom<&ComponentValue> for Color {
    type Error = CssValueError;

    fn try_from(value: &ComponentValue) -> Result<Self, Self::Error> {
        let mut stream = ComponentValueStream::new(std::slice::from_ref(value));
        Self::parse(&mut stream)
    }
}

impl CSSParsable for Color {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, CssValueError> {
        let color = if let Some(cv) = stream.next_non_whitespace() {
            match cv {
                ComponentValue::Token(token) => match &token.kind {
                    CssTokenKind::Ident(ident) => {
                        if ident.eq_ignore_ascii_case("currentColor") {
                            Ok(Self::Current)
                        } else if ident.eq_ignore_ascii_case("transparent") {
                            Ok(Self::Base(ColorBase::Transparent))
                        } else if let Ok(system_color) = ident.parse() {
                            Ok(Self::System(system_color))
                        } else if let Some(named_color) = NamedColor::from_str_insensitive(ident) {
                            Ok(Self::Base(ColorBase::Named(named_color)))
                        } else {
                            Err(CssValueError::InvalidValue(format!("Unrecognized color identifier: {}", ident)))
                        }
                    }
                    CssTokenKind::Hash { .. } => {
                        let hex_color = HexColor::try_from(token)
                            .map_err(|e| CssValueError::InvalidValue(format!("Invalid hex color: {}", e)))?;
                        Ok(Self::Base(ColorBase::Hex(hex_color)))
                    }
                    _ => Err(CssValueError::InvalidToken(token.kind.clone())),
                },
                ComponentValue::Function(function) => {
                    if let Ok(color_function) = ColorFunction::try_from(function) {
                        Ok(Self::Base(ColorBase::Function(color_function)))
                    } else if function.name.eq_ignore_ascii_case("light-dark") {
                        if let Some(pos) = function.value.iter().position(
                            |c| matches!(c, ComponentValue::Token(token) if token.kind == CssTokenKind::Comma),
                        ) {
                            let (light_values, dark_values) = function.value.split_at(pos);
                            let dark_values = &dark_values[1..];

                            let light_color = Color::parse(&mut light_values.into())?;
                            let dark_color = Color::parse(&mut dark_values.into())?;

                            Ok(Self::LightDark(Box::new(light_color), Box::new(dark_color)))
                        } else {
                            Err(CssValueError::InvalidValue(
                                "light-dark() function requires two color arguments separated by a comma".into(),
                            ))
                        }
                    } else {
                        Err(CssValueError::InvalidFunction(function.name.clone()))
                    }
                }
                cvs => Err(CssValueError::InvalidComponentValue(cvs.clone())),
            }
        } else {
            Err(CssValueError::ExpectedComponentValue)
        };

        stream.skip_whitespace();

        if stream.peek().is_some() {
            Err(CssValueError::UnexpectedRemainingInput)
        } else {
            color
        }
    }
}

//! This module defines the `BorderWidth` and `BorderStyle` types, which represent the width and style of CSS borders, respectively.
//! These types can be constructed from CSS component values and can be converted to pixel values for rendering.

use css_cssom::{ComponentValue, ComponentValueStream, CssTokenKind};
use strum::EnumString;

use crate::{
    functions::math::{MathExpression, is_math_function},
    length::LengthUnit,
    primitives::length::Length,
    properties::{AbsoluteContext, CSSParsable, RelativeContext},
};

/// Represents the width of a CSS border, which can be a length, a calc expression, or one of the keywords thin, medium, or thick.
///
/// <https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/border-width>
#[derive(Debug, Clone, PartialEq)]
pub enum BorderWidth {
    Length(Length),
    Math(MathExpression),
    Thin,
    Medium,
    Thick,
}

impl Default for BorderWidth {
    fn default() -> Self {
        BorderWidth::Length(Length::px(0.0))
    }
}

impl TryFrom<&ComponentValue> for BorderWidth {
    type Error = String;

    fn try_from(value: &ComponentValue) -> Result<Self, Self::Error> {
        let mut stream = ComponentValueStream::new(std::slice::from_ref(value));
        Self::parse(&mut stream)
    }
}

impl CSSParsable for BorderWidth {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, String> {
        stream.skip_whitespace();

        let width = if let Some(cv) = stream.peek() {
            match cv {
                ComponentValue::Token(token) => match &token.kind {
                    CssTokenKind::Ident(ident) => {
                        if ident.eq_ignore_ascii_case("thin") {
                            Ok(Self::Thin)
                        } else if ident.eq_ignore_ascii_case("medium") {
                            Ok(Self::Medium)
                        } else if ident.eq_ignore_ascii_case("thick") {
                            Ok(Self::Thick)
                        } else {
                            Err(format!("Invalid border width keyword: {}", ident))
                        }
                    }
                    CssTokenKind::Dimension { value, unit } => {
                        let len_unit = unit
                            .parse::<LengthUnit>()
                            .map_err(|_| format!("Invalid length unit: {}", unit))?;
                        Ok(Self::Length(Length::new(value.to_f64() as f32, len_unit)))
                    }
                    CssTokenKind::Number(num) => Ok(Self::Length(Length::px(num.to_f64() as f32))),
                    _ => Err("Expected a valid border width value".to_string()),
                },
                ComponentValue::Function(func) if is_math_function(&func.name) => {
                    let calc_expr = MathExpression::parse_math_function(&func.name, &func.value)?;
                    Ok(Self::Math(calc_expr))
                }
                _ => Err("Expected a valid border width value".to_string()),
            }
        } else {
            Err("Unexpected end of input while parsing border width".to_string())
        }?;

        stream.next_cv();
        Ok(width)
    }
}

impl BorderWidth {
    /// Create a BorderWidth from a pixel value.
    pub(crate) fn px(value: f32) -> Self {
        BorderWidth::Length(Length::px(value))
    }

    /// Create a BorderWidth with a value of zero.
    pub(crate) fn zero() -> Self {
        BorderWidth::Length(Length::px(0.0))
    }

    /// Convert the BorderWidth to pixels, using the provided contexts for relative units and calc expressions.
    pub(crate) fn to_px(&self, rel_ctx: &RelativeContext, abs_ctx: &AbsoluteContext) -> f32 {
        match self {
            BorderWidth::Length(len) => len.to_px(rel_ctx, abs_ctx),
            BorderWidth::Math(calc) => calc.to_px(None, rel_ctx, abs_ctx),
            BorderWidth::Thin => 1.0,
            BorderWidth::Medium => 3.0,
            BorderWidth::Thick => 5.0,
        }
    }
}

/// Represents the style of a CSS border, which can be one of the keywords none, hidden, dotted, dashed, solid, double, groove, ridge, inset, or outset.
///
/// <https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/border-style>
#[derive(Debug, Clone, Default, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, EnumString)]
#[strum(serialize_all = "lowercase", ascii_case_insensitive, parse_err_ty = String, parse_err_fn = String::from)]
pub enum BorderStyle {
    /// Like the `hidden` keyword, displays no border. Unless a `background-image` is set, the computed value of the same side's `border-width` will be `0`,
    /// even if the specified value is something else. In the case of table cell and border collapsing, the `none` value has the lowest priority:
    /// if any other conflicting border is set, it will be displayed.
    #[default]
    None,

    /// Like the none keyword, displays no border. Unless a `background-image` is set, the computed value of the same side's `border-width` will be `0`,
    /// even if the specified value is something else. In the case of table cell and border collapsing, the `hidden` value has the highest priority:
    /// if any other conflicting border is set, it won't be displayed.
    Hidden,

    /// Displays a series of rounded dots. The spacing of the dots is not defined by the specification and is implementation-specific.
    /// The radius of the dots is half the computed value of the same side's `border-width`.
    Dotted,

    /// Displays a series of short square-ended dashes or line segments. The exact size and length of the segments are not defined by the
    /// specification and are implementation-specific.
    Dashed,

    /// Displays a single, straight, solid line.
    Solid,

    /// Displays two straight lines that add up to the pixel size defined by `border-width`.
    Double,

    /// Displays a border with a carved appearance. It is the opposite of `ridge`.
    Groove,

    /// Displays a border with an extruded appearance. It is the opposite of `groove`.
    Ridge,

    /// Displays a border that makes the element appear embedded. It is the opposite of `outset`. When applied to a table cell with `border-collapse`
    /// set to `collapsed`, this value behaves like `ridge`.
    Inset,

    /// Displays a border that makes the element appear embossed. It is the opposite of `inset`. When applied to a table cell with `border-collapse`
    /// set to `collapsed`, this value behaves like `groove`.
    Outset,
}

impl CSSParsable for BorderStyle {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, String> {
        stream.skip_whitespace();

        if let Some(cv) = stream.peek() {
            if let ComponentValue::Token(token) = cv
                && let CssTokenKind::Ident(ident) = &token.kind
                && let Ok(style) = ident.parse()
            {
                Ok(style)
            } else {
                Err("Expected a valid border style keyword".to_string())
            }
        } else {
            Err("Unexpected end of input while parsing border style".to_string())
        }
    }
}

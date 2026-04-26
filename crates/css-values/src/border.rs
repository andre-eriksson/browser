use css_cssom::{ComponentValue, ComponentValueStream, CssTokenKind};
use strum::EnumString;

use crate::{
    CSSParsable,
    calc::{CalcDomain, CalcExpression, is_math_function},
    error::CssValueError,
    quantity::{Length, LengthUnit},
};

/// Represents the style of a CSS border, which can be one of the keywords none, hidden, dotted, dashed, solid, double, groove, ridge, inset, or outset.
///
/// <https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/border-style>
#[derive(Debug, Clone, Default, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, EnumString)]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
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
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, CssValueError> {
        stream.skip_whitespace();

        stream
            .peek()
            .map_or(Err(CssValueError::ExpectedComponentValue), |cv| {
                if let ComponentValue::Token(token) = cv
                    && let CssTokenKind::Ident(ident) = &token.kind
                    && let Ok(style) = ident.parse()
                {
                    Ok(style)
                } else {
                    Err(CssValueError::InvalidComponentValue(cv.clone()))
                }
            })
    }
}

/// Represents the width of a CSS border, which can be a length, a calc expression, or one of the keywords thin, medium, or thick.
///
/// <https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/border-width>
#[derive(Debug, Clone, PartialEq)]
pub enum BorderWidth {
    Length(Length),
    Calc(CalcExpression),
    Thin,
    Medium,
    Thick,
}

impl BorderWidth {
    /// Create a `BorderWidth` from a pixel value.
    #[must_use]
    pub const fn px(value: f64) -> Self {
        Self::Length(Length::px(value))
    }

    /// Create a `BorderWidth` with a value of zero.
    #[must_use]
    pub const fn zero() -> Self {
        Self::Length(Length::px(0.0))
    }
}

impl Default for BorderWidth {
    fn default() -> Self {
        Self::zero()
    }
}

impl TryFrom<&ComponentValue> for BorderWidth {
    type Error = CssValueError;

    fn try_from(value: &ComponentValue) -> Result<Self, Self::Error> {
        let mut stream = ComponentValueStream::new(std::slice::from_ref(value));
        Self::parse(&mut stream)
    }
}

impl CSSParsable for BorderWidth {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, CssValueError> {
        stream.skip_whitespace();

        let width = if let Some(cv) = stream.peek() {
            match cv {
                ComponentValue::Function(func) if is_math_function(&func.name) => {
                    let expr = CalcExpression::parse_math_function(&func.name, &func.value)?;
                    let domain = expr.resolve_domain()?;

                    if !matches!(domain, CalcDomain::Length) {
                        return Err(CssValueError::InvalidCalcDomain {
                            expected: vec![crate::calc::CalcDomain::Length],
                            found: domain,
                        });
                    }

                    Ok(Self::Calc(expr))
                }
                ComponentValue::Token(token) => match &token.kind {
                    CssTokenKind::Ident(ident) => {
                        if ident.eq_ignore_ascii_case("thin") {
                            Ok(Self::Thin)
                        } else if ident.eq_ignore_ascii_case("medium") {
                            Ok(Self::Medium)
                        } else if ident.eq_ignore_ascii_case("thick") {
                            Ok(Self::Thick)
                        } else {
                            Err(CssValueError::InvalidValue(format!("Invalid border width keyword: {ident}")))
                        }
                    }
                    CssTokenKind::Dimension { value, unit } => {
                        let len_unit = unit
                            .parse::<LengthUnit>()
                            .map_err(|_| CssValueError::InvalidUnit(unit.clone()))?;
                        Ok(Self::Length(Length::new(value.to_f64(), len_unit)))
                    }
                    CssTokenKind::Number(num) => Ok(Self::Length(Length::px(num.to_f64()))),
                    _ => Err(CssValueError::InvalidToken(token.kind.clone())),
                },
                cvs => Err(CssValueError::InvalidComponentValue(cvs.clone())),
            }
        } else {
            Err(CssValueError::ExpectedComponentValue)
        }?;

        stream.next_cv();
        Ok(width)
    }
}

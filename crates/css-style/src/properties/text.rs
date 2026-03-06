//! Properties related to text layout and formatting, such as `writing-mode`, `text-align`, `white-space`, and `line-height`.

use css_cssom::{ComponentValue, ComponentValueStream, CssTokenKind};
use strum::EnumString;

use crate::{
    ComputedStyle, RelativeType,
    functions::math::{MathExpression, is_math_function},
    length::LengthUnit,
    primitives::{length::Length, percentage::Percentage},
    properties::{AbsoluteContext, CSSParsable, RelativeContext},
};

/// The `writing-mode` property defines whether lines of text are laid out horizontally or vertically, and the direction in which blocks progress.
/// It also affects the orientation of certain characters and the behavior of text alignment and justification.
///
/// <https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/writing-mode>
#[derive(Debug, Clone, Default, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, EnumString)]
#[strum(serialize_all = "kebab_case", ascii_case_insensitive, parse_err_ty = String, parse_err_fn = String::from)]
pub enum WritingMode {
    /// For `ltr` scripts, content flows horizontally from left to right. For `rtl` scripts, content flows horizontally from right to left. The next horizontal
    /// line is positioned below the previous line.
    #[default]
    HorizontalTb,

    /// For `ltr` scripts, content flows vertically from top to bottom, and the next vertical line is positioned to the left of the previous line. For `rtl` scripts,
    /// content flows vertically from bottom to top, and the next vertical line is positioned to the right of the previous line.
    VerticalRl,

    /// For `ltr` scripts, content flows vertically from top to bottom, and the next vertical line is positioned to the right of the previous line. For `rtl` scripts,
    /// content flows vertically from bottom to top, and the next vertical line is positioned to the left of the previous line.
    VerticalLr,

    /// For `ltr` scripts, content flows vertically from top to bottom. For `rtl` scripts, content flows vertically from bottom to top. All the glyphs, even those in
    /// vertical scripts, are set sideways toward the right.
    SidewaysRl,

    /// For `ltr` scripts, content flows vertically from bottom to top. For `rtl` scripts, content flows vertically from top to bottom. All the glyphs, even those in
    /// vertical scripts, are set sideways toward the left.
    SidewaysLr,
}

impl CSSParsable for WritingMode {
    fn parse(stream: &mut css_cssom::ComponentValueStream) -> Result<Self, String> {
        stream.skip_whitespace();

        if let Some(cv) = stream.peek() {
            match cv {
                ComponentValue::Token(token) => match &token.kind {
                    CssTokenKind::Ident(ident) => ident
                        .parse()
                        .map_err(|_| format!("Invalid writing-mode value: {}", ident)),
                    _ => Err("Expected an identifier for writing-mode value".to_string()),
                },
                _ => Err("Expected a token for writing-mode value".to_string()),
            }
        } else {
            Err("Unexpected end of input while parsing writing-mode value".to_string())
        }
    }
}

/// The `text-align` property describes how inline content of a block element is aligned. It has no effect on non-block elements or when `display: table-cell` is used.
///
/// <https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/text-align>
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PartialOrd, Ord, EnumString)]
#[strum(serialize_all = "kebab_case", ascii_case_insensitive, parse_err_ty = String, parse_err_fn = String::from)]
pub enum TextAlign {
    /// The same as `left` if direction is left-to-right and `right` if direction is right-to-left.
    Start,

    /// The same as `right` if direction is left-to-right and `left` if direction is right-to-left.
    End,

    /// The inline contents are aligned to the left edge of the line box.
    #[default]
    Left,

    /// The inline contents are aligned to the right edge of the line box.
    Right,

    /// The inline contents are centered within the line box.
    Center,

    /// The inline contents are justified. Spaces out the content to line up its left and right edges to the left and right edges of the line box, except for the last line.
    Justify,

    /// Similar to `inherit`, but the values `start` and `end` are calculated according to the parent's `direction` and are replaced by the appropriate `left` or `right` value.
    MatchParent,
}

impl CSSParsable for TextAlign {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, String> {
        stream.skip_whitespace();

        if let Some(cv) = stream.peek() {
            match cv {
                ComponentValue::Token(token) => match &token.kind {
                    CssTokenKind::Ident(ident) => ident
                        .parse()
                        .map_err(|_| format!("Invalid text-align value: {}", ident)),
                    _ => Err("Expected an identifier for text-align value".to_string()),
                },
                _ => Err("Expected a token for text-align value".to_string()),
            }
        } else {
            Err("Unexpected end of input while parsing text-align value".to_string())
        }
    }
}

/// The `white-space` property describes how whitespace inside an element is handled. It can be used to control whether and how whitespace is collapsed,
/// and whether lines are broken at newline characters in the source code or at soft wrap opportunities.
///
/// <https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/white-space>
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PartialOrd, Ord, EnumString)]
#[strum(serialize_all = "kebab_case", ascii_case_insensitive, parse_err_ty = String, parse_err_fn = String::from)]
pub enum Whitespace {
    /// Sequences of white space are collapsed. Newline characters in the source are handled the same as other white spaces.
    /// Lines are broken as necessary to fill line boxes. Equivalent to `collapse wrap`.
    #[default]
    Normal,

    /// Sequences of white space are preserved. Lines are only broken at newline characters in the source and at `<br>` elements.
    /// Equivalent to `preserve nowrap`.
    Pre,

    /// Sequences of white space are preserved. Lines are broken at newline characters, at `<br>`, and as necessary to fill line boxes.
    /// Equivalent to `preserve wrap`.
    PreWrap,

    /// Sequences of white space are collapsed. Lines are broken at newline characters, at `<br>`, and as necessary to fill line boxes.
    /// Equivalent to `preserve-breaks wrap`.
    PreLine,

    // Shorthands, TODO: expand these into their full equivalents when parsing
    Nowrap,
    Wrap,
    BreakSpaces,
    Collapse,
}

impl CSSParsable for Whitespace {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, String> {
        stream.skip_whitespace();

        if let Some(cv) = stream.peek() {
            match cv {
                ComponentValue::Token(token) => match &token.kind {
                    CssTokenKind::Ident(ident) => ident
                        .parse()
                        .map_err(|_| format!("Invalid white-space value: {}", ident)),
                    _ => Err("Expected an identifier for white-space value".to_string()),
                },
                _ => Err("Expected a token for white-space value".to_string()),
            }
        } else {
            Err("Unexpected end of input while parsing white-space value".to_string())
        }
    }
}

/// The `line-height` property sets the height of a line box. It's commonly used to set the distance between lines of text.
///
/// <https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/line-height>
#[derive(Debug, Clone, Default, PartialEq)]
pub enum LineHeight {
    #[default]
    Normal,
    Number(f32),
    Length(Length),
    Percentage(Percentage),
    Math(MathExpression),
}

impl LineHeight {
    pub fn px(value: f32) -> Self {
        LineHeight::Length(Length::px(value))
    }

    /// Converts the `line-height` value to pixels based on the provided context and font size. This is used for layout calculations.
    pub fn to_px(&self, abs_ctx: &AbsoluteContext, font_size_px: f32) -> f32 {
        let rel_ctx = RelativeContext {
            parent: ComputedStyle {
                font_size: font_size_px,
                ..Default::default()
            }
            .into(),
        };

        match self {
            LineHeight::Normal => font_size_px * 1.2,
            LineHeight::Number(num) => font_size_px * num,
            LineHeight::Length(len) => len.to_px(&rel_ctx, abs_ctx),
            LineHeight::Percentage(pct) => pct.as_fraction() * font_size_px,
            LineHeight::Math(calc) => calc.to_px(Some(RelativeType::FontSize), &rel_ctx, abs_ctx),
        }
    }
}

impl CSSParsable for LineHeight {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, String> {
        stream.skip_whitespace();

        if let Some(cv) = stream.peek() {
            match cv {
                ComponentValue::Function(func) if is_math_function(&func.name) => {
                    Ok(LineHeight::Math(MathExpression::parse_math_function(&func.name, func.value.as_slice())?))
                }
                ComponentValue::Token(token) => match &token.kind {
                    CssTokenKind::Ident(ident) if ident.eq_ignore_ascii_case("normal") => Ok(LineHeight::Normal),
                    CssTokenKind::Number(num) => Ok(LineHeight::Number(num.to_f64() as f32)),
                    CssTokenKind::Dimension { value, unit } => {
                        let len_unit = unit
                            .parse::<LengthUnit>()
                            .map_err(|_| format!("Invalid length unit: {}", unit))?;
                        Ok(LineHeight::Length(Length::new(value.to_f64() as f32, len_unit)))
                    }
                    CssTokenKind::Percentage(pct) => Ok(LineHeight::Percentage(Percentage::new(pct.to_f64() as f32))),
                    _ => Err("Expected a valid line-height value".to_string()),
                },
                _ => Err("Expected a valid line-height value".to_string()),
            }
        } else {
            Err("Unexpected end of input while parsing line-height value".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_text_align() {
        assert_eq!("start".parse(), Ok(TextAlign::Start));
        assert_eq!("end".parse(), Ok(TextAlign::End));
        assert_eq!("left".parse(), Ok(TextAlign::Left));
        assert_eq!("right".parse(), Ok(TextAlign::Right));
        assert_eq!("center".parse(), Ok(TextAlign::Center));
        assert_eq!("justify".parse(), Ok(TextAlign::Justify));
        assert_eq!("match-parent".parse(), Ok(TextAlign::MatchParent));
        assert!("unknown".parse::<TextAlign>().is_err());
    }
}

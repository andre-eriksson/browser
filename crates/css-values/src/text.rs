//! This module defines font primitives, including absolute and relative font sizes, as well as generic font family names.
//! The absolute font sizes are mapped to specific pixel values, while the relative font sizes adjust based on the parent element's font size.
//! The generic font family names represent common categories of fonts that can be used in CSS.

use css_cssom::{ComponentValue, ComponentValueStream, CssTokenKind};
use strum::EnumString;

use crate::{
    CSSParsable,
    calc::{CalcExpression, is_math_function},
    error::CssValueError,
    numeric::Percentage,
    quantity::{Length, LengthUnit},
};

/// CSS defines several keywords for font sizes, which are mapped to specific pixel values.
/// These keywords are used to specify the size of text in a way that is relative to the user's preferred font size (medium) or to other font sizes.
/// The absolute size keywords are mapped to specific pixel values, while the relative size keywords adjust the font size based on the parent element's font size.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, EnumString)]
#[strum(serialize_all = "kebab_case", ascii_case_insensitive)]
pub enum AbsoluteSize {
    /// An absolute size 60% the size of medium. Mapped to the deprecated size="1".
    XxSmall,

    /// An absolute size 75% the size of medium.
    XSmall,

    /// An absolute size 89% the size of medium. Mapped to the deprecated size="2".
    Small,

    /// A user's preferred font size. This value is used as the reference middle value. Mapped to size="3".
    Medium,

    /// An absolute size 20% larger than medium. Mapped to the deprecated size="4".
    Large,

    /// An absolute size 50% larger than medium. Mapped to the deprecated size="5".
    XLarge,

    /// An absolute size twice the size of medium. Mapped to the deprecated size="6".
    XxLarge,

    /// An absolute size three times the size of medium. Mapped to the deprecated size="7".
    XxxLarge,
}

/// Represents a font family name, which can be either a generic family (serif, sans-serif, etc.) or a specific font name.
///
/// <https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/font-family>
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FontFamilyName {
    Generic(GenericName),
    Specific(String),
}

impl Default for FontFamilyName {
    fn default() -> Self {
        FontFamilyName::Generic(GenericName::SansSerif)
    }
}

/// Represents the font-size property, which can be specified using absolute-size keywords (e.g., small, medium),
/// relative-size keywords (e.g., larger, smaller), length units (e.g., 16px, 1.5em), percentage, or a calc() expression.
///
/// <https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/font-size>
#[derive(Debug, Clone, PartialEq)]
pub enum FontSize {
    Absolute(AbsoluteSize),
    Relative(RelativeSize),
    Length(Length),
    Percentage(Percentage),
    Calc(CalcExpression),
}

impl FontSize {
    /// Create a FontSize from a length value in pixels.
    pub fn px(value: f32) -> Self {
        Self::Length(Length::px(value))
    }
}

impl Default for FontSize {
    fn default() -> Self {
        FontSize::Absolute(AbsoluteSize::Medium)
    }
}

impl CSSParsable for FontSize {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, CssValueError> {
        stream.skip_whitespace();

        let font_size = if let Some(cv) = stream.peek() {
            match cv {
                ComponentValue::Function(func) => {
                    if is_math_function(&func.name) {
                        Ok(Self::Calc(CalcExpression::parse_math_function(&func.name, func.value.as_slice())?))
                    } else {
                        Err(CssValueError::InvalidFunction(func.name.clone()))
                    }
                }
                ComponentValue::Token(token) => match &token.kind {
                    CssTokenKind::Ident(ident) => {
                        if let Ok(abs_size) = ident.parse() {
                            Ok(Self::Absolute(abs_size))
                        } else if let Ok(rel_size) = ident.parse() {
                            Ok(Self::Relative(rel_size))
                        } else {
                            Err(CssValueError::InvalidValue(format!("Invalid font size keyword: {}", ident)))
                        }
                    }
                    CssTokenKind::Dimension { value, unit } => {
                        let len_unit = unit
                            .parse::<LengthUnit>()
                            .map_err(|_| CssValueError::InvalidUnit(unit.clone()))?;
                        Ok(Self::Length(Length::new(value.to_f64() as f32, len_unit)))
                    }
                    CssTokenKind::Percentage(num) => Ok(Self::Percentage(Percentage::new(num.to_f64() as f32))),
                    _ => Err(CssValueError::InvalidToken(token.kind.clone())),
                },
                cvs => Err(CssValueError::InvalidComponentValue(cvs.clone())),
            }
        } else {
            Err(CssValueError::ExpectedComponentValue)
        }?;

        stream.next_cv();
        Ok(font_size)
    }
}

/// Represents the font weight property, which can be a keyword (normal, bold) or a numeric value (100-900).
///
/// <https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/font-weight>
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FontWeight {
    Thin = 100,
    ExtraLight = 200,
    Light = 300,
    #[default]
    Normal = 400,
    Medium = 500,
    SemiBold = 600,
    Bold = 700,
    ExtraBold = 800,
    Black = 900,
}

impl From<u16> for FontWeight {
    fn from(value: u16) -> Self {
        match value {
            100 => FontWeight::Thin,
            200 => FontWeight::ExtraLight,
            300 => FontWeight::Light,
            400 => FontWeight::Normal,
            500 => FontWeight::Medium,
            600 => FontWeight::SemiBold,
            700 => FontWeight::Bold,
            800 => FontWeight::ExtraBold,
            900 => FontWeight::Black,
            // TODO: Once we support variable font weights, we need to clamp between 1 and 1000 and drop the rounding logic.
            _ => Self::from(((value.saturating_add(50)) / 100 * 100).clamp(100, 900)),
        }
    }
}

impl CSSParsable for FontWeight {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, CssValueError> {
        stream.skip_whitespace();

        if let Some(cv) = stream.peek() {
            match cv {
                ComponentValue::Token(token) => match &token.kind {
                    CssTokenKind::Ident(ident) => {
                        if ident.eq_ignore_ascii_case("normal") {
                            Ok(Self::Normal)
                        } else if ident.eq_ignore_ascii_case("bold") {
                            Ok(Self::Bold)
                        } else {
                            Err(CssValueError::InvalidValue(format!("Invalid font weight keyword: {}", ident)))
                        }
                    }
                    CssTokenKind::Number(num) => Ok(Self::from(num.to_f64() as u16)),
                    _ => Err(CssValueError::InvalidToken(token.kind.clone())),
                },
                cvs => Err(CssValueError::InvalidComponentValue(cvs.clone())),
            }
        } else {
            Err(CssValueError::ExpectedComponentValue)
        }
    }
}

/// Defines the generic font family names that can be used in CSS to specify a general category of font. These names are not specific font families,
/// but rather represent a group of fonts that share certain characteristics. When a generic font family is specified, the browser will use the
/// best available font that matches the specified category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, EnumString)]
#[strum(serialize_all = "kebab_case", ascii_case_insensitive)]
pub enum GenericName {
    /// A serif is a small line or stroke attached to the end of a larger stroke in a letter. In serif fonts, glyphs have finishing strokes,
    /// flared or tapering ends. Examples include Lucida Bright, Lucida Fax, Palatino, Palatino Linotype, Palladio, and URW Palladio.
    Serif,

    /// A font without serifs; glyphs have plain stroke endings, without ornamentation.
    /// Example sans-serif fonts include Open Sans, Fira Sans, Lucida Sans, Lucida Sans Unicode, Trebuchet MS, Liberation Sans, and Nimbus Sans L.
    SansSerif,

    /// All glyphs have the same fixed width. Example monospace fonts include Fira Mono, DejaVu Sans Mono, Menlo, Consolas, Liberation Mono,
    /// Monaco, and Lucida Console.
    Monospace,

    /// Glyphs in cursive fonts generally use a cursive script or other handwriting style, and the result looks more like handwritten pen or brush
    /// writing than printed typesetting. CSS uses the term "cursive" to apply to a font for any script, including those that do not have joining
    /// strokes. Example cursive fonts include Brush Script MT, Brush Script Std, Lucida Calligraphy, Lucida Handwriting, and Apple Chancery.
    Cursive,

    /// Fantasy fonts are primarily decorative fonts that contain playful representations of characters.
    ///  Example fantasy fonts include Papyrus, Herculanum, Party LET, Curlz MT, Harrington, and Comic Sans MS.
    Fantasy,

    /// Glyphs are taken from the default user interface font on a given platform. Because typographic traditions vary widely across the world,
    /// this generic family is provided for typefaces that don't map cleanly into the others.
    SystemUi,

    /// The default user interface serif font.
    UiSerif,

    /// The default user interface sans-serif font.
    UiSansSerif,

    /// The default user interface monospace font.
    UiMonospace,

    /// The default user interface font that has rounded features.
    UiRounded,

    /// Fonts for displaying mathematical expressions, for example superscript and subscript, brackets that cross several lines, nesting expressions,
    /// and double-struck glyphs with distinct meanings.
    Math,

    /// A particular style of Chinese characters that are between serif-style Song and cursive-style Kai forms. This style is often used for government documents.
    FangSong,
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
    Calc(CalcExpression),
}

impl LineHeight {
    pub fn px(value: f32) -> Self {
        LineHeight::Length(Length::px(value))
    }
}

impl CSSParsable for LineHeight {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, CssValueError> {
        stream.skip_whitespace();

        if let Some(cv) = stream.peek() {
            match cv {
                ComponentValue::Function(func) if is_math_function(&func.name) => {
                    Ok(LineHeight::Calc(CalcExpression::parse_math_function(&func.name, func.value.as_slice())?))
                }
                ComponentValue::Token(token) => match &token.kind {
                    CssTokenKind::Ident(ident) if ident.eq_ignore_ascii_case("normal") => Ok(LineHeight::Normal),
                    CssTokenKind::Number(num) => Ok(LineHeight::Number(num.to_f64() as f32)),
                    CssTokenKind::Dimension { value, unit } => {
                        let len_unit = unit
                            .parse::<LengthUnit>()
                            .map_err(|_| CssValueError::InvalidUnit(unit.clone()))?;
                        Ok(LineHeight::Length(Length::new(value.to_f64() as f32, len_unit)))
                    }
                    CssTokenKind::Percentage(pct) => Ok(LineHeight::Percentage(Percentage::new(pct.to_f64() as f32))),
                    _ => Err(CssValueError::InvalidToken(token.kind.clone())),
                },
                cvs => Err(CssValueError::InvalidComponentValue(cvs.clone())),
            }
        } else {
            Err(CssValueError::ExpectedComponentValue)
        }
    }
}

/// CSS defines two relative font size keywords, "smaller" and "larger", which adjust the font size based on the parent element's font size.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, EnumString)]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
pub enum RelativeSize {
    /// A relative size one size smaller than the inherited size.
    Smaller,

    /// A relative size one size larger than the inherited size.
    Larger,
}

/// The `text-align` property describes how inline content of a block element is aligned. It has no effect on non-block elements or when `display: table-cell` is used.
///
/// <https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/text-align>
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PartialOrd, Ord, EnumString)]
#[strum(serialize_all = "kebab_case", ascii_case_insensitive, parse_err_ty = String, parse_err_fn = String::from)]
pub enum TextAlign {
    /// The same as `left` if direction is left-to-right and `right` if direction is right-to-left.
    #[default]
    Start,

    /// The same as `right` if direction is left-to-right and `left` if direction is right-to-left.
    End,

    /// The inline contents are aligned to the left edge of the line box.
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
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, CssValueError> {
        stream.skip_whitespace();

        if let Some(cv) = stream.peek() {
            match cv {
                ComponentValue::Token(token) => match &token.kind {
                    CssTokenKind::Ident(ident) => ident
                        .parse()
                        .map_err(|_| CssValueError::InvalidValue(format!("Invalid text-align value: {}", ident))),
                    _ => Err(CssValueError::InvalidToken(token.kind.clone())),
                },
                cvs => Err(CssValueError::InvalidComponentValue(cvs.clone())),
            }
        } else {
            Err(CssValueError::ExpectedComponentValue)
        }
    }
}

/// The `white-space` property describes how whitespace inside an element is handled. It can be used to control whether and how whitespace is collapsed,
/// and whether lines are broken at newline characters in the source code or at soft wrap opportunities.
///
/// <https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/white-space>
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PartialOrd, Ord, EnumString)]
#[strum(serialize_all = "kebab_case", ascii_case_insensitive)]
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
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, CssValueError> {
        stream.skip_whitespace();

        if let Some(cv) = stream.peek() {
            match cv {
                ComponentValue::Token(token) => match &token.kind {
                    CssTokenKind::Ident(ident) => ident
                        .parse()
                        .map_err(|_| CssValueError::InvalidValue(format!("Invalid white-space value: {}", ident))),
                    _ => Err(CssValueError::InvalidToken(token.kind.clone())),
                },
                cvs => Err(CssValueError::InvalidComponentValue(cvs.clone())),
            }
        } else {
            Err(CssValueError::ExpectedComponentValue)
        }
    }
}

/// The `writing-mode` property defines whether lines of text are laid out horizontally or vertically, and the direction in which blocks progress.
/// It also affects the orientation of certain characters and the behavior of text alignment and justification.
///
/// <https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/writing-mode>
#[derive(Debug, Clone, Default, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, EnumString)]
#[strum(serialize_all = "kebab_case", ascii_case_insensitive)]
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
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, CssValueError> {
        stream.skip_whitespace();

        if let Some(cv) = stream.peek() {
            match cv {
                ComponentValue::Token(token) => match &token.kind {
                    CssTokenKind::Ident(ident) => ident
                        .parse()
                        .map_err(|_| CssValueError::InvalidValue(format!("Invalid writing-mode value: {}", ident))),
                    _ => Err(CssValueError::InvalidToken(token.kind.clone())),
                },
                cvs => Err(CssValueError::InvalidComponentValue(cvs.clone())),
            }
        } else {
            Err(CssValueError::ExpectedComponentValue)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generic_name_parse() {
        assert_eq!("serif".parse(), Ok(GenericName::Serif));
        assert_eq!("sans-serif".parse(), Ok(GenericName::SansSerif));
        assert_eq!("monospace".parse(), Ok(GenericName::Monospace));
    }

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

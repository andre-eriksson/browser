//! This module defines font primitives, including absolute and relative font sizes, as well as generic font family names.
//! The absolute font sizes are mapped to specific pixel values, while the relative font sizes adjust based on the parent element's font size.
//! The generic font family names represent common categories of fonts that can be used in CSS.

use strum::EnumString;

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

impl AbsoluteSize {
    pub fn to_px(self) -> f32 {
        match self {
            AbsoluteSize::XxSmall => 9.0,
            AbsoluteSize::XSmall => 10.0,
            AbsoluteSize::Small => 13.0,
            AbsoluteSize::Medium => 16.0,
            AbsoluteSize::Large => 18.0,
            AbsoluteSize::XLarge => 24.0,
            AbsoluteSize::XxLarge => 32.0,
            AbsoluteSize::XxxLarge => 48.0,
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

impl RelativeSize {
    /// Convert the relative size to an absolute pixel value based on the parent element's font size in pixels.
    pub fn to_px(self, parent_px: f32) -> f32 {
        match self {
            RelativeSize::Smaller => parent_px * 0.833,
            RelativeSize::Larger => parent_px * 1.2,
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
}

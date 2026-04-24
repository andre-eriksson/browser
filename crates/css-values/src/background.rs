use strum::EnumString;

use crate::combination::LengthPercentage;

#[derive(Debug, Clone, Default, Copy, PartialEq, Eq)]
pub enum Attachment {
    /// The background scrolls along with the element's content. This is the default value for the `background-attachment` property.
    #[default]
    Scroll,

    /// The background is fixed with regard to the viewport. It does not move when the content of the element is scrolled.
    Fixed,

    /// The background scrolls along with the element's content, but only within the bounds of the element itself. When the content of the element is
    /// scrolled, the background will move, but it will not scroll outside of the element's area.
    Local,
}

/// The `background-blend-mode` property specifies the blending mode for each background layer (color and image) of an element.
///
/// It determines how the background layers are blended together and with the content of the element.
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString)]
#[strum(serialize_all = "kebab-case", ascii_case_insensitive)]
pub enum BlendMode {
    Normal,
    Multiply,
    Screen,
    Overlay,
    Darken,
    Lighten,
    ColorDodge,
    ColorBurn,
    HardLight,
    SoftLight,
    Difference,
    Exclusion,
    Hue,
    Saturation,
    Color,
    Luminosity,
}

/// The `background-origin` property specifies the background painting area for an element.
///
/// It determines where the background image or color is applied in relation to the content, padding, and border of the element.
#[derive(Debug, Clone, Default, Copy, PartialEq, Eq)]
pub enum VisualBox {
    /// The background is painted within the content box, which is the area where the content of the element is displayed.
    Content,

    /// The background is painted within the padding box, which includes the content box and the padding area around it.
    /// The background will extend into the padding area but not into the border area.
    /// This is the default value for the `background-origin` property, meaning that if no value is specified,
    /// the background will be painted within the padding box.
    Padding,

    /// The background is painted within the border box, which includes the content box, padding box, and border area.
    /// The background will extend into both the padding and border areas.
    #[default]
    Border,
}

/// The `background-clip` property specifies the area within which the background image or color is visible.
///
/// It determines how the background is clipped in relation to the content, padding, and border of the element.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Clip {
    Text,
    BorderArea,
}

/// The `background-clip` property
///
/// Can take one of two forms: it can specify a single visual box (content, padding, or border)
/// that defines the area where the background is visible, or it can specify a combination of clipping areas using the `Clip`
/// enum. When using the `Clip` enum, the first value specifies the primary clipping area (e.g., text or border), and the optional
/// second value specifies an additional clipping area (e.g., border area) that further restricts the visibility of the background.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BgClip {
    Visual(VisualBox),
    Clip(Clip, Option<Clip>),
}

impl Default for BgClip {
    fn default() -> Self {
        Self::Visual(VisualBox::default())
    }
}

/// The `background-repeat` property
///
/// Specifies how background images are repeated (tiled) across the background of an element. It determines
/// how the background image is repeated in the horizontal and vertical directions, and whether it is stretched to fit the element's background
/// area or not.
#[derive(Debug, Clone, Copy, EnumString, PartialEq, Eq)]
#[strum(serialize_all = "kebab-case", ascii_case_insensitive)]
pub enum RepeatStyle {
    Repeat,
    Space,
    Round,
    NoRepeat,
}

/// The `background-size` property
///
/// Specifies the size of the background image. It determines how the background image is scaled and sized within the
/// background area of an element. The property can take various values, including keywords like `cover` and `contain`, as well as specific width and height values.
#[derive(Debug, Clone, PartialEq)]
pub enum WidthHeightSize {
    Auto,
    Length(LengthPercentage),
}

/// The `background-size` property
///
/// Can take one of three forms: it can be set to `cover`, which scales the background image to cover the entire background area while
/// maintaining its aspect ratio; it can be set to `contain`, which scales the background image to fit within the background area while maintaining its aspect ratio;
/// or it can be set to specific width and height values, where the first value specifies the width and the optional second value specifies the height. If only one
/// value is provided, it is used for both width and height.
#[derive(Debug, Clone, PartialEq)]
pub enum Size {
    Cover,
    Contain,
    WidthHeight(WidthHeightSize, Option<WidthHeightSize>),
}

impl Default for Size {
    fn default() -> Self {
        Self::WidthHeight(WidthHeightSize::Auto, Some(WidthHeightSize::Auto))
    }
}

//! SRGBA color representations: rgb(), rgba(), hsl(), hsla(), and hwb() functions

use crate::{
    color::{Alpha, ColorValue, Hue},
    percentage::Percentage,
};

/// SRGBA color representations as defined in CSS Color Module Level 4
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SRGBAColor {
    /// rgb() and rgba() functions with R, G, B components and optional alpha
    ///
    /// * R, G, B: 0 to 255 or 0% to 100%
    /// * alpha: Opacity (0.0 to 1.0)
    Rgb(ColorValue, ColorValue, ColorValue, Alpha),

    /// hsl() and hsla() functions with H, S, L components and optional alpha
    ///
    /// * H: Hue angle in degrees (0 to 360 as degrees)
    /// * S: Saturation (0% to 100%)
    /// * L: Lightness (0% to 100%)
    /// * alpha: Opacity (0.0 to 1.0)
    Hsl(Hue, Percentage, Percentage, Alpha),

    /// hwb() function with H, W, B components and optional alpha
    ///
    /// * H: Hue angle in degrees (0 to 360 as degrees)
    /// * W: Whiteness (0% to 100%)
    /// * B: Blackness (0% to 100%)
    /// * alpha: Opacity (0.0 to 1.0)
    Hwb(Hue, Percentage, Percentage, Alpha),
}

//! SRGBA color representations: rgb(), rgba(), hsl(), hsla(), and hwb() functions

use css_cssom::ComponentValue;

use crate::{
    color::{Alpha, ColorValue, FunctionColor, Hue},
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

impl TryFrom<&[ComponentValue]> for SRGBAColor {
    type Error = String;

    fn try_from(value: &[ComponentValue]) -> Result<Self, Self::Error> {
        for val in value {
            match val {
                ComponentValue::Function(func) => {
                    if func.name.eq_ignore_ascii_case("rgb")
                        || func.name.eq_ignore_ascii_case("rgba")
                    {
                        let raw = FunctionColor::parse_color_components(&func.value)?;

                        return match raw.channels {
                            [Some(r), Some(g), Some(b)] => Ok(SRGBAColor::Rgb(r, g, b, raw.alpha)),
                            _ => Err("Missing components in rgb()".to_string()),
                        };
                    } else if func.name.eq_ignore_ascii_case("hsl")
                        || func.name.eq_ignore_ascii_case("hsla")
                    {
                        let raw = FunctionColor::parse_color_components(&func.value)?;

                        return match raw.channels {
                            [Some(h), Some(s), Some(l)] => Ok(SRGBAColor::Hsl(
                                Hue::from(h),
                                Percentage::from(s),
                                Percentage::from(l),
                                raw.alpha,
                            )),
                            _ => Err("Missing components in hsl()".to_string()),
                        };
                    } else if func.name.eq_ignore_ascii_case("hwb") {
                        let raw = FunctionColor::parse_color_components(&func.value)?;

                        return match raw.channels {
                            [Some(h), Some(w), Some(b)] => Ok(SRGBAColor::Hwb(
                                Hue::from(h),
                                Percentage::from(w),
                                Percentage::from(b),
                                raw.alpha,
                            )),
                            _ => Err("Missing components in hwb()".to_string()),
                        };
                    } else {
                        continue;
                    }
                }
                _ => continue,
            }
        }

        Err("No valid SRGBA color found in component values".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}

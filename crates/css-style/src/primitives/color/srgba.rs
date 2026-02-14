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
    use css_cssom::{CssToken, CssTokenKind, Function, NumberType, NumericValue};

    use crate::css_color_fn;

    use super::*;

    #[test]
    fn test_rgb_parse() {
        let color = css_color_fn!("rgb", "200", "150", "200", "none");
        let rgb = SRGBAColor::try_from(color.as_slice()).unwrap();
        assert_eq!(
            rgb,
            SRGBAColor::Rgb(
                ColorValue::Number(200.0),
                ColorValue::Number(150.0),
                ColorValue::Number(200.0),
                Alpha(1.0)
            )
        );

        let color = css_color_fn!("rgb", "200", "100", "255", "50%");
        let rgb = SRGBAColor::try_from(color.as_slice()).unwrap();
        assert_eq!(
            rgb,
            SRGBAColor::Rgb(
                ColorValue::Number(200.0),
                ColorValue::Number(100.0),
                ColorValue::Number(255.0),
                Alpha(0.5)
            )
        );
    }

    #[test]
    fn test_rbga_parse() {
        let color = vec![ComponentValue::Function(Function {
            name: "rgba".to_string(),
            value: vec![
                ComponentValue::Token(CssToken {
                    kind: CssTokenKind::Number(NumericValue {
                        value: 200.0,
                        int_value: None,
                        type_flag: NumberType::Integer,
                        repr: String::new(),
                    }),
                    position: None,
                }),
                ComponentValue::Token(CssToken {
                    kind: CssTokenKind::Comma,
                    position: None,
                }),
                ComponentValue::Token(CssToken {
                    kind: CssTokenKind::Number(NumericValue {
                        value: 100.0,
                        int_value: None,
                        type_flag: NumberType::Integer,
                        repr: String::new(),
                    }),
                    position: None,
                }),
                ComponentValue::Token(CssToken {
                    kind: CssTokenKind::Comma,
                    position: None,
                }),
                ComponentValue::Token(CssToken {
                    kind: CssTokenKind::Number(NumericValue {
                        value: 255.0,
                        int_value: None,
                        type_flag: NumberType::Integer,
                        repr: String::new(),
                    }),
                    position: None,
                }),
                ComponentValue::Token(CssToken {
                    kind: CssTokenKind::Comma,
                    position: None,
                }),
                ComponentValue::Token(CssToken {
                    kind: CssTokenKind::Number(NumericValue {
                        value: 0.7,
                        int_value: None,
                        type_flag: NumberType::Number,
                        repr: String::new(),
                    }),
                    position: None,
                }),
            ],
        })];

        let rgba = SRGBAColor::try_from(color.as_slice()).unwrap();
        assert_eq!(
            rgba,
            SRGBAColor::Rgb(
                ColorValue::Number(200.0),
                ColorValue::Number(100.0),
                ColorValue::Number(255.0),
                Alpha(0.7)
            )
        );
    }

    #[test]
    fn test_hsl_parse() {
        let color = css_color_fn!("hsl", "120", "50%", "50%", "none");
        let hsl = SRGBAColor::try_from(color.as_slice()).unwrap();
        assert_eq!(
            hsl,
            SRGBAColor::Hsl(
                Hue::from(120.0),
                Percentage::new(50.0),
                Percentage::new(50.0),
                Alpha(1.0)
            )
        );

        let color = css_color_fn!("hsl", "120", "50%", "50%", "0.3");
        let hsl = SRGBAColor::try_from(color.as_slice()).unwrap();
        assert_eq!(
            hsl,
            SRGBAColor::Hsl(
                Hue::from(120.0),
                Percentage::new(50.0),
                Percentage::new(50.0),
                Alpha(0.3)
            )
        );
    }

    #[test]
    fn test_hsla_parse() {
        let color = vec![ComponentValue::Function(Function {
            name: "hsla".to_string(),
            value: vec![
                ComponentValue::Token(CssToken {
                    kind: CssTokenKind::Number(NumericValue {
                        value: 120.0,
                        int_value: None,
                        type_flag: NumberType::Integer,
                        repr: String::new(),
                    }),
                    position: None,
                }),
                ComponentValue::Token(CssToken {
                    kind: CssTokenKind::Comma,
                    position: None,
                }),
                ComponentValue::Token(CssToken {
                    kind: CssTokenKind::Percentage(NumericValue {
                        value: 50.0,
                        int_value: None,
                        type_flag: NumberType::Integer,
                        repr: String::new(),
                    }),
                    position: None,
                }),
                ComponentValue::Token(CssToken {
                    kind: CssTokenKind::Comma,
                    position: None,
                }),
                ComponentValue::Token(CssToken {
                    kind: CssTokenKind::Percentage(NumericValue {
                        value: 50.0,
                        int_value: None,
                        type_flag: NumberType::Integer,
                        repr: String::new(),
                    }),
                    position: None,
                }),
                ComponentValue::Token(CssToken {
                    kind: CssTokenKind::Comma,
                    position: None,
                }),
                ComponentValue::Token(CssToken {
                    kind: CssTokenKind::Number(NumericValue {
                        value: 0.3,
                        int_value: None,
                        type_flag: NumberType::Number,
                        repr: String::new(),
                    }),
                    position: None,
                }),
            ],
        })];

        let hsla = SRGBAColor::try_from(color.as_slice()).unwrap();
        assert_eq!(
            hsla,
            SRGBAColor::Hsl(
                Hue::from(120.0),
                Percentage::new(50.0),
                Percentage::new(50.0),
                Alpha(0.3)
            )
        );
    }

    #[test]
    fn test_hwb_parse() {
        let color = css_color_fn!("hwb", "240", "0%", "0%", "none");
        let hwb = SRGBAColor::try_from(color.as_slice()).unwrap();
        assert_eq!(
            hwb,
            SRGBAColor::Hwb(
                Hue::from(240.0),
                Percentage::new(0.0),
                Percentage::new(0.0),
                Alpha(1.0)
            )
        );

        let color = css_color_fn!("hwb", "240", "0%", "0%", "0.5");
        let hwb = SRGBAColor::try_from(color.as_slice()).unwrap();
        assert_eq!(
            hwb,
            SRGBAColor::Hwb(
                Hue::from(240.0),
                Percentage::new(0.0),
                Percentage::new(0.0),
                Alpha(0.5)
            )
        );
    }
}

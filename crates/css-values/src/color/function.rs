use css_cssom::{ComponentValue, CssTokenKind, Function};

use crate::{
    color::{Alpha, ColorValue, Hue},
    numeric::Percentage,
    quantity::Angle,
};

/// Represents a color specified using functional notation, which can be in the form of srgba() functions (e.g., rgb(), rgba(), hsl(), hsla(), hwb()) or color() functions (e.g., lab(), oklab()).
#[derive(Debug, Clone, PartialEq)]
pub enum ColorFunction {
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

    /// lab() function with L, a, b components and optional alpha
    ///
    /// * L: Lightness (0 to 100) or (0% to 100%)
    /// * a: Green-Red component (-125 to 125) or (-100% to 100%)
    /// * b: Blue-Yellow component (-125 to 125) or (-100% to 100%)
    /// * alpha: Opacity (0.0 to 1.0)
    Lab(ColorValue, ColorValue, ColorValue, Alpha),

    /// lch() function with L, C, H components and optional alpha
    ///
    /// * L: Lightness (0 to 100) or (0% to 100%)
    /// * C: Chroma (0 to 150) or (0% to 100%)
    /// * H: Hue angle in degrees
    /// * alpha: Opacity (0.0 to 1.0)
    Lch(ColorValue, ColorValue, Hue, Alpha),

    /// oklab() function with L, a, b components and optional alpha
    ///
    /// * L: Lightness (0 to 1) or (0% to 100%)
    /// * a: Green-Red component (-0.4 to 0.4) or (-100% to 100%)
    /// * b: Blue-Yellow component (-0.4 to 0.4) or (-100% to 100%)
    /// * alpha: Opacity (0.0 to 1.0)
    Oklab(ColorValue, ColorValue, ColorValue, Alpha),

    /// oklch() function with L, C, H components and optional alpha
    ///
    /// * L: Lightness (0 to 1) or (0% to 100%)
    /// * C: Chroma (0 to 0.4) or (0% to 100%)
    /// * H: Hue angle in degrees
    /// * alpha: Opacity (0.0 to 1.0)
    Oklch(ColorValue, ColorValue, Hue, Alpha),
    //
    // TODO: ictcp()
    //       jzazbz()
    //       jzczhz()
    //       alpha()
    //       color()
    //       hdr-color()
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct RawColorComponents {
    channels: [Option<ColorValue>; 3],
    alpha: Alpha,
}

impl ColorFunction {
    // TODO: Relative color syntax `color-function(from <origin> channel1 channel2 channel3)`

    fn parse_color_components(values: &[ComponentValue]) -> Result<RawColorComponents, String> {
        let mut channels = [None, None, None];
        let mut channel_idx = 0;
        let mut alpha = None;
        let mut parsing_alpha = false;

        for cv in values {
            match cv {
                ComponentValue::Token(token) => match &token.kind {
                    CssTokenKind::Ident(ident) => {
                        if ident.eq_ignore_ascii_case("none") {
                            if parsing_alpha {
                                alpha = Some(Alpha(1.0));
                            } else if channel_idx < 3 {
                                channels[channel_idx] = Some(ColorValue::Number(0.0));
                                channel_idx += 1;
                            } else {
                                return Err("Too many components in color function".to_string());
                            }
                        } else {
                            return Err(format!("Invalid token in color function: '{}'", ident));
                        }
                    }
                    CssTokenKind::Delim('/') => {
                        parsing_alpha = true;
                    }
                    CssTokenKind::Percentage(pct) => {
                        if parsing_alpha {
                            alpha = Some(Alpha::from(Percentage::new(pct.to_f64() as f32)));
                        } else if channel_idx < 3 {
                            channels[channel_idx] = Some(ColorValue::Percentage(Percentage::new(pct.to_f64() as f32)));
                            channel_idx += 1;
                        } else if alpha.is_none() {
                            alpha = Some(Alpha::from(Percentage::new(pct.to_f64() as f32)));
                        } else {
                            return Err("Too many percentage components in color function".to_string());
                        }
                    }
                    CssTokenKind::Number(num) => {
                        if parsing_alpha {
                            alpha = Some(Alpha::new(num.to_f64() as f32));
                        } else if channel_idx < 3 {
                            channels[channel_idx] = Some(ColorValue::Number(num.to_f64() as f32));
                            channel_idx += 1;
                        } else if alpha.is_none() {
                            alpha = Some(Alpha::new(num.to_f64() as f32));
                        } else {
                            return Err("Too many number components in color function".to_string());
                        }
                    }
                    CssTokenKind::Dimension { .. } => {
                        if parsing_alpha {
                            return Err("Dimension tokens are not allowed in alpha component".to_string());
                        } else if channel_idx < 3 {
                            if let Ok(angle) = Angle::try_from(token) {
                                channels[channel_idx] = Some(ColorValue::Number(angle.to_degrees()));
                                channel_idx += 1;
                            } else {
                                return Err("Invalid angle value in color function".to_string());
                            }
                        } else {
                            return Err("Too many components in color function".to_string());
                        }
                    }
                    _ => continue,
                },
                _ => continue,
            }
        }

        Ok(RawColorComponents {
            channels,
            alpha: alpha.unwrap_or(Alpha(1.0)),
        })
    }
}

impl TryFrom<&Function> for ColorFunction {
    type Error = String;

    fn try_from(func: &Function) -> Result<Self, Self::Error> {
        if func.name.eq_ignore_ascii_case("rgb") || func.name.eq_ignore_ascii_case("rgba") {
            let raw = Self::parse_color_components(&func.value)?;

            match raw.channels {
                [Some(r), Some(g), Some(b)] => Ok(Self::Rgb(r, g, b, raw.alpha)),
                _ => Err("Missing components in rgb() or rgba()".to_string()),
            }
        } else if func.name.eq_ignore_ascii_case("hsl") || func.name.eq_ignore_ascii_case("hsla") {
            let raw = Self::parse_color_components(&func.value)?;

            match raw.channels {
                [Some(h), Some(s), Some(l)] => {
                    Ok(Self::Hsl(Hue::from(h), Percentage::from(s), Percentage::from(l), raw.alpha))
                }
                _ => Err("Missing components in hsl() or hsla()".to_string()),
            }
        } else if func.name.eq_ignore_ascii_case("hwb") {
            let raw = Self::parse_color_components(&func.value)?;

            match raw.channels {
                [Some(h), Some(w), Some(b)] => {
                    Ok(Self::Hwb(Hue::from(h), Percentage::from(w), Percentage::from(b), raw.alpha))
                }
                _ => Err("Missing components in hwb()".to_string()),
            }
        } else if func.name.eq_ignore_ascii_case("lab") {
            let raw = Self::parse_color_components(&func.value)?;

            match raw.channels {
                [Some(l), Some(a), Some(b)] => Ok(Self::Lab(l, a, b, raw.alpha)),
                _ => Err("Missing components in lab() or lch()".to_string()),
            }
        } else if func.name.eq_ignore_ascii_case("lch") {
            let raw = Self::parse_color_components(&func.value)?;

            match raw.channels {
                [Some(l), Some(c), Some(h)] => Ok(Self::Lch(l, c, Hue::from(h), raw.alpha)),
                _ => Err("Missing components in lab() or lch()".to_string()),
            }
        } else if func.name.eq_ignore_ascii_case("oklab") {
            let raw = Self::parse_color_components(&func.value)?;

            match raw.channels {
                [Some(l), Some(a), Some(b)] => Ok(Self::Oklab(l, a, b, raw.alpha)),
                _ => Err("Missing components in oklab() or oklch()".to_string()),
            }
        } else if func.name.eq_ignore_ascii_case("oklch") {
            let raw = Self::parse_color_components(&func.value)?;

            match raw.channels {
                [Some(l), Some(c), Some(h)] => Ok(Self::Oklch(l, c, Hue::from(h), raw.alpha)),
                _ => Err("Missing components in oklab() or oklch()".to_string()),
            }
        } else {
            Err(format!("Unsupported color function: '{}'", func.name))
        }
    }
}

#[cfg(test)]
pub mod macros {
    #[macro_export]
    macro_rules! css_color_fn {
        ($func:expr, $ch1:expr, $ch2:expr, $ch3:expr, $alpha:expr) => {{
            use css_cssom::{CssToken, CssTokenKind, Function, NumericValue};
            macro_rules! css_value_token {
                ($val:expr) => {{
                    let s = $val.to_string();
                    if s == "none" {
                        CssToken {
                            kind: CssTokenKind::Ident("none".to_string()),
                            position: None,
                        }
                    } else if s.contains('%') {
                        CssToken {
                            kind: CssTokenKind::Percentage(NumericValue::from(
                                s.replace('%', "").parse::<f64>().unwrap_or(0.0),
                            )),
                            position: None,
                        }
                    } else {
                        CssToken {
                            kind: CssTokenKind::Number(NumericValue::from(s.parse::<f64>().unwrap_or(0.0))),
                            position: None,
                        }
                    }
                }};
            }

            let channel1_token = css_value_token!($ch1);
            let channel2_token = css_value_token!($ch2);
            let channel3_token = css_value_token!($ch3);

            let alpha_token = match $alpha {
                a if (a.to_string().eq("none")) => CssToken {
                    kind: CssTokenKind::Ident("none".to_string()),
                    position: None,
                },
                a if (a.to_string().contains('%')) => CssToken {
                    kind: CssTokenKind::Percentage(NumericValue::from(
                        a.to_string().replace('%', "").parse::<f64>().unwrap_or(0.0),
                    )),
                    position: None,
                },
                a if (0.0..=1.0).contains(&(a.to_string().parse::<f32>().unwrap_or(-1.0))) => CssToken {
                    kind: CssTokenKind::Number(NumericValue::from(a.to_string().parse::<f64>().unwrap_or(0.0))),
                    position: None,
                },
                _ => panic!("Invalid alpha value for css_color_fn_alpha! macro"),
            };

            if alpha_token.kind == CssTokenKind::Ident("none".to_string()) {
                vec![ComponentValue::Function(Function {
                    name: $func.to_string(),
                    value: vec![
                        ComponentValue::Token(channel1_token),
                        ComponentValue::Token(CssToken {
                            kind: CssTokenKind::Whitespace,
                            position: None,
                        }),
                        ComponentValue::Token(channel2_token),
                        ComponentValue::Token(CssToken {
                            kind: CssTokenKind::Whitespace,
                            position: None,
                        }),
                        ComponentValue::Token(channel3_token),
                    ],
                })]
            } else {
                vec![ComponentValue::Function(Function {
                    name: $func.to_string(),
                    value: vec![
                        ComponentValue::Token(channel1_token),
                        ComponentValue::Token(CssToken {
                            kind: CssTokenKind::Whitespace,
                            position: None,
                        }),
                        ComponentValue::Token(channel2_token),
                        ComponentValue::Token(CssToken {
                            kind: CssTokenKind::Whitespace,
                            position: None,
                        }),
                        ComponentValue::Token(channel3_token),
                        ComponentValue::Token(CssToken {
                            kind: CssTokenKind::Whitespace,
                            position: None,
                        }),
                        ComponentValue::Token(CssToken {
                            kind: CssTokenKind::Delim('/'),
                            position: None,
                        }),
                        ComponentValue::Token(CssToken {
                            kind: CssTokenKind::Whitespace,
                            position: None,
                        }),
                        ComponentValue::Token(alpha_token),
                    ],
                })]
            }
        }};
    }
}

#[cfg(test)]
mod tests {
    use css_cssom::{CssToken, NumericValue};

    use crate::{
        CSSParsable,
        color::{Color, base::ColorBase},
        css_color_fn,
    };

    use super::*;

    #[test]
    fn test_parse_color_components() {
        let components = vec![
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Number(NumericValue::from(255.0)),
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Percentage(NumericValue::from(50.0)),
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("none".to_string()),
                position: None,
            }),
        ];

        let result = ColorFunction::parse_color_components(&components).unwrap();
        assert_eq!(result.channels[0], Some(ColorValue::Number(255.0)));
        assert_eq!(result.channels[1], Some(ColorValue::Percentage(Percentage::new(50.0))));
        assert_eq!(result.channels[2], Some(ColorValue::Number(0.0)));
        assert_eq!(result.alpha, Alpha(1.0));
    }

    #[test]
    fn test_rgb_parsing() {
        let rgb = css_color_fn!("rgb", 255, 0, 128, "none");
        let parsed = Color::parse(&mut rgb.as_slice().into()).unwrap();
        assert_eq!(
            parsed,
            Color::Base(ColorBase::Function(ColorFunction::Rgb(
                ColorValue::Number(255.0),
                ColorValue::Number(0.0),
                ColorValue::Number(128.0),
                Alpha(1.0)
            )))
        );
    }

    #[test]
    fn test_rgba_parsing() {
        let rgba = css_color_fn!("rgba", 255, 0, 128, 0.5);
        let parsed = Color::parse(&mut rgba.as_slice().into()).unwrap();
        assert_eq!(
            parsed,
            Color::Base(ColorBase::Function(ColorFunction::Rgb(
                ColorValue::Number(255.0),
                ColorValue::Number(0.0),
                ColorValue::Number(128.0),
                Alpha(0.5)
            )))
        );
    }

    #[test]
    fn test_hsl_parsing() {
        let hsl = css_color_fn!("hsl", 120, "100%", "50%", "none");
        let parsed = Color::parse(&mut hsl.as_slice().into()).unwrap();
        assert_eq!(
            parsed,
            Color::Base(ColorBase::Function(ColorFunction::Hsl(
                Hue(120.0),
                Percentage::new(100.0),
                Percentage::new(50.0),
                Alpha(1.0)
            )))
        );
    }

    #[test]
    fn test_hsla_parsing() {
        let hsla = css_color_fn!("hsla", 120, "100%", "50%", 0.5);
        let parsed = Color::parse(&mut hsla.as_slice().into()).unwrap();
        assert_eq!(
            parsed,
            Color::Base(ColorBase::Function(ColorFunction::Hsl(
                Hue(120.0),
                Percentage::new(100.0),
                Percentage::new(50.0),
                Alpha(0.5)
            )))
        );
    }

    #[test]
    fn test_hwb_parsing() {
        let hwb = css_color_fn!("hwb", 120, "100%", "50%", "none");
        let parsed = Color::parse(&mut hwb.as_slice().into()).unwrap();
        assert_eq!(
            parsed,
            Color::Base(ColorBase::Function(ColorFunction::Hwb(
                Hue(120.0),
                Percentage::new(100.0),
                Percentage::new(50.0),
                Alpha(1.0)
            )))
        );
    }

    #[test]
    fn test_lab_parsing() {
        let lab = css_color_fn!("lab", 50, 20, -30, "none");
        let parsed = Color::parse(&mut lab.as_slice().into()).unwrap();
        assert_eq!(
            parsed,
            Color::Base(ColorBase::Function(ColorFunction::Lab(
                ColorValue::Number(50.0),
                ColorValue::Number(20.0),
                ColorValue::Number(-30.0),
                Alpha(1.0)
            )))
        );
    }

    #[test]
    fn test_oklab_parsing() {
        let oklab = css_color_fn!("oklab", 0.5, 0.1, -0.1, "none");
        let parsed = Color::parse(&mut oklab.as_slice().into()).unwrap();
        assert_eq!(
            parsed,
            Color::Base(ColorBase::Function(ColorFunction::Oklab(
                ColorValue::Number(0.5),
                ColorValue::Number(0.1),
                ColorValue::Number(-0.1),
                Alpha(1.0)
            )))
        );
    }

    #[test]
    fn test_oklch_parsing() {
        let oklch = css_color_fn!("oklch", 0.5, 0.1, 120, "none");
        let parsed = Color::parse(&mut oklch.as_slice().into()).unwrap();
        assert_eq!(
            parsed,
            Color::Base(ColorBase::Function(ColorFunction::Oklch(
                ColorValue::Number(0.5),
                ColorValue::Number(0.1),
                Hue(120.0),
                Alpha(1.0)
            )))
        );
    }
}

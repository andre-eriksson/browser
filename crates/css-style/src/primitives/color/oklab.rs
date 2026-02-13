//! Oklab color function with L, a, b components, e.g., oklab(0.5, 0.1, -0.1) or oklch(0.5, 0.1, 30)

use std::str::FromStr;

use css_cssom::ComponentValue;

use crate::color::{Alpha, ColorValue, FunctionColor, Hue};

/// Oklab and Oklch color representations as defined in CSS Color Module Level 4
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Oklab {
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
}

impl TryFrom<&[ComponentValue]> for Oklab {
    type Error = String;

    fn try_from(value: &[ComponentValue]) -> Result<Self, Self::Error> {
        for val in value {
            match val {
                ComponentValue::Function(func) => {
                    if func.name.eq_ignore_ascii_case("oklab") {
                        let raw = FunctionColor::parse_color_components(&func.value)?;
                        return match raw.channels {
                            [Some(l), Some(a), Some(b)] => Ok(Oklab::Oklab(l, a, b, raw.alpha)),
                            _ => Err("Missing components in lab()".to_string()),
                        };
                    } else if func.name.eq_ignore_ascii_case("oklch") {
                        let raw = FunctionColor::parse_color_components(&func.value)?;
                        return match raw.channels {
                            [Some(l), Some(c), Some(h)] => {
                                Ok(Oklab::Oklch(l, c, Hue::from(h), raw.alpha))
                            }
                            _ => Err("Missing components in lch()".to_string()),
                        };
                    }
                }
                _ => continue,
            }
        }
        Err("No valid oklab() or oklch() function found".to_string())
    }
}

impl FromStr for Oklab {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains("from") {
            return Err("Relative color syntax not supported yet".to_string());
        } else if s.contains("calc(") {
            return Err("CSS functions in color values not supported yet".to_string());
        }

        let parts = FunctionColor::tokenize_color(s, "oklch(")
            .or_else(|| FunctionColor::tokenize_color(s, "oklab("))
            .ok_or_else(|| format!("Invalid SRGBA color: {}", s))?;

        if s.starts_with("oklab") {
            match parts.as_slice() {
                [l, a, b] => {
                    let l = l
                        .parse::<ColorValue>()
                        .map_err(|_| format!("Invalid L value: {}", l))?;
                    let a = a
                        .parse::<ColorValue>()
                        .map_err(|_| format!("Invalid a value: {}", a))?;
                    let b = b
                        .parse::<ColorValue>()
                        .map_err(|_| format!("Invalid b value: {}", b))?;
                    Ok(Oklab::Oklab(l, a, b, Alpha(1.0)))
                }
                [l, a, b, alpha] => {
                    let l = l
                        .parse::<ColorValue>()
                        .map_err(|_| format!("Invalid L value: {}", l))?;
                    let a = a
                        .parse::<ColorValue>()
                        .map_err(|_| format!("Invalid a value: {}", a))?;
                    let b = b
                        .parse::<ColorValue>()
                        .map_err(|_| format!("Invalid b value: {}", b))?;
                    let alpha = alpha
                        .parse::<Alpha>()
                        .map_err(|_| format!("Invalid alpha value: {}", alpha))?;
                    Ok(Oklab::Oklab(l, a, b, alpha))
                }
                _ => Err(format!("Invalid number of components for oklab: {}", s)),
            }
        } else if s.starts_with("oklch") {
            match parts.as_slice() {
                [l, c, h] => {
                    let l = l
                        .parse::<ColorValue>()
                        .map_err(|_| format!("Invalid L value: {}", l))?;
                    let c = c
                        .parse::<ColorValue>()
                        .map_err(|_| format!("Invalid C value: {}", c))?;
                    let h = h
                        .parse::<Hue>()
                        .map_err(|_| format!("Invalid H value: {}", h))?;
                    Ok(Oklab::Oklch(l, c, h, Alpha(1.0)))
                }
                [l, c, h, alpha] => {
                    let l = l
                        .parse::<ColorValue>()
                        .map_err(|_| format!("Invalid L value: {}", l))?;
                    let c = c
                        .parse::<ColorValue>()
                        .map_err(|_| format!("Invalid C value: {}", c))?;
                    let h = h
                        .parse::<Hue>()
                        .map_err(|_| format!("Invalid H value: {}", h))?;
                    let alpha = alpha
                        .parse::<Alpha>()
                        .map_err(|_| format!("Invalid alpha value: {}", alpha))?;
                    Ok(Oklab::Oklch(l, c, h, alpha))
                }
                _ => Err(format!("Invalid number of components for oklch: {}", s)),
            }
        } else {
            Err(format!("Invalid Oklab color: {}", s))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oklab_parsing() {
        let color = "oklab(0.5, 0.1, -0.1)".parse::<Oklab>().unwrap();
        assert_eq!(
            color,
            Oklab::Oklab(
                ColorValue::Number(0.5),
                ColorValue::Number(0.1),
                ColorValue::Number(-0.1),
                Alpha(1.0)
            )
        );
        let color = "oklch(0.5, 0.2, 120)".parse::<Oklab>().unwrap();
        assert_eq!(
            color,
            Oklab::Oklch(
                ColorValue::Number(0.5),
                ColorValue::Number(0.2),
                Hue(120.0),
                Alpha(1.0)
            )
        );
    }
}

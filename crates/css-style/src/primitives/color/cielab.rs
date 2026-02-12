//! CIELAB color function with L, a, b components, e.g., lab(50, 20, -30) or lch(50, 20, 30)

use std::str::FromStr;

use crate::color::{Alpha, ColorValue, FunctionColor, Hue};

/// CIELAB and CIELCH color representations as defined in CSS Color Module Level 4
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Cielab {
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
}

impl FromStr for Cielab {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains("from") {
            return Err("Relative color syntax not supported yet".to_string());
        } else if s.contains("calc(") {
            return Err("CSS functions in color values not supported yet".to_string());
        }

        let parts = FunctionColor::tokenize_color(s, "lch(")
            .or_else(|| FunctionColor::tokenize_color(s, "lab("))
            .ok_or_else(|| format!("Invalid SRGBA color: {}", s))?;

        if s.starts_with("lab") {
            match parts.as_slice() {
                [l, a, b] => {
                    let l = l
                        .parse::<ColorValue>()
                        .map_err(|_| format!("Invalid 'L' value: {}", l))?;
                    let a = a
                        .parse::<ColorValue>()
                        .map_err(|_| format!("Invalid 'a' value: {}", a))?;
                    let b = b
                        .parse::<ColorValue>()
                        .map_err(|_| format!("Invalid 'b' value: {}", b))?;
                    Ok(Cielab::Lab(l, a, b, Alpha(1.0)))
                }
                [l, a, b, alpha] => {
                    let l = l
                        .parse::<ColorValue>()
                        .map_err(|_| format!("Invalid 'L' value: {}", l))?;
                    let a = a
                        .parse::<ColorValue>()
                        .map_err(|_| format!("Invalid 'a' value: {}", a))?;
                    let b = b
                        .parse::<ColorValue>()
                        .map_err(|_| format!("Invalid 'b' value: {}", b))?;
                    let alpha = alpha
                        .parse::<Alpha>()
                        .map_err(|_| format!("Invalid 'alpha' value: {}", alpha))?;
                    Ok(Cielab::Lab(l, a, b, alpha))
                }
                _ => Err(format!("Invalid number of components for lab: {}", s)),
            }
        } else if s.starts_with("lch") {
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
                    Ok(Cielab::Lch(l, c, h, Alpha(1.0)))
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
                    Ok(Cielab::Lch(l, c, h, alpha))
                }
                _ => Err(format!("Invalid number of components for lch: {}", s)),
            }
        } else {
            Err(format!("Invalid CIELAB color: {}", s))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cielab_parsing() {
        let color = "lab(50, 20, -30)".parse::<Cielab>().unwrap();
        assert_eq!(
            color,
            Cielab::Lab(
                ColorValue::Number(50.0),
                ColorValue::Number(20.0),
                ColorValue::Number(-30.0),
                Alpha(1.0)
            )
        );
        let color = "lab(90 0 10 / 1)".parse::<Cielab>().unwrap();
        assert_eq!(
            color,
            Cielab::Lab(
                ColorValue::Number(90.0),
                ColorValue::Number(0.0),
                ColorValue::Number(10.0),
                Alpha(1.0)
            )
        );

        let color = "lab(70, -10, 15, 0.5)".parse::<Cielab>().unwrap();
        assert_eq!(
            color,
            Cielab::Lab(
                ColorValue::Number(70.0),
                ColorValue::Number(-10.0),
                ColorValue::Number(15.0),
                Alpha(0.5)
            )
        );
        let color = "lch(60, 30, 120)".parse::<Cielab>().unwrap();
        assert_eq!(
            color,
            Cielab::Lch(
                ColorValue::Number(60.0),
                ColorValue::Number(30.0),
                Hue(120.0),
                Alpha(1.0)
            )
        );
        let color = "lch(80, 40, 240, 0.75)".parse::<Cielab>().unwrap();
        assert_eq!(
            color,
            Cielab::Lch(
                ColorValue::Number(80.0),
                ColorValue::Number(40.0),
                Hue(240.0),
                Alpha(0.75)
            )
        );
    }
}

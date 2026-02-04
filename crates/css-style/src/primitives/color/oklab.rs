use std::str::FromStr;

use crate::color::{Alpha, ColorValue, FunctionColor, Hue};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Oklab {
    Oklab(ColorValue<f32>, ColorValue<f32>, ColorValue<f32>, Alpha),
    Oklch(ColorValue<f32>, ColorValue<f32>, Hue, Alpha),
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
                        .parse::<ColorValue<f32>>()
                        .map_err(|_| format!("Invalid L value: {}", l))?;
                    let a = a
                        .parse::<ColorValue<f32>>()
                        .map_err(|_| format!("Invalid a value: {}", a))?;
                    let b = b
                        .parse::<ColorValue<f32>>()
                        .map_err(|_| format!("Invalid b value: {}", b))?;
                    Ok(Oklab::Oklab(l, a, b, Alpha::Number(1.0)))
                }
                [l, a, b, alpha] => {
                    let l = l
                        .parse::<ColorValue<f32>>()
                        .map_err(|_| format!("Invalid L value: {}", l))?;
                    let a = a
                        .parse::<ColorValue<f32>>()
                        .map_err(|_| format!("Invalid a value: {}", a))?;
                    let b = b
                        .parse::<ColorValue<f32>>()
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
                        .parse::<ColorValue<f32>>()
                        .map_err(|_| format!("Invalid L value: {}", l))?;
                    let c = c
                        .parse::<ColorValue<f32>>()
                        .map_err(|_| format!("Invalid C value: {}", c))?;
                    let h = h
                        .parse::<Hue>()
                        .map_err(|_| format!("Invalid H value: {}", h))?;
                    Ok(Oklab::Oklch(l, c, h, Alpha::Number(1.0)))
                }
                [l, c, h, alpha] => {
                    let l = l
                        .parse::<ColorValue<f32>>()
                        .map_err(|_| format!("Invalid L value: {}", l))?;
                    let c = c
                        .parse::<ColorValue<f32>>()
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
                Alpha::Number(1.0)
            )
        );
        let color = "oklch(0.5, 0.2, 120)".parse::<Oklab>().unwrap();
        assert_eq!(
            color,
            Oklab::Oklch(
                ColorValue::Number(0.5),
                ColorValue::Number(0.2),
                Hue::Number(120.0),
                Alpha::Number(1.0)
            )
        );
    }
}

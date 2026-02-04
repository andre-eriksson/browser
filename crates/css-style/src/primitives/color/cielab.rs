use std::str::FromStr;

use crate::color::{Alpha, ColorValue, FunctionColor, Hue};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Cielab {
    Lab(ColorValue<u8>, ColorValue<i8>, ColorValue<i8>, Alpha),
    Lch(ColorValue<u8>, ColorValue<u8>, Hue, Alpha),
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
                        .parse::<ColorValue<u8>>()
                        .map_err(|_| format!("Invalid L value: {}", l))?;
                    let a = a
                        .parse::<ColorValue<i8>>()
                        .map_err(|_| format!("Invalid a value: {}", a))?;
                    let b = b
                        .parse::<ColorValue<i8>>()
                        .map_err(|_| format!("Invalid b value: {}", b))?;
                    Ok(Cielab::Lab(l, a, b, Alpha::Number(1.0)))
                }
                [l, a, b, alpha] => {
                    let l = l
                        .parse::<ColorValue<u8>>()
                        .map_err(|_| format!("Invalid L value: {}", l))?;
                    let a = a
                        .parse::<ColorValue<i8>>()
                        .map_err(|_| format!("Invalid a value: {}", a))?;
                    let b = b
                        .parse::<ColorValue<i8>>()
                        .map_err(|_| format!("Invalid b value: {}", b))?;
                    let alpha = alpha
                        .parse::<Alpha>()
                        .map_err(|_| format!("Invalid alpha value: {}", alpha))?;
                    Ok(Cielab::Lab(l, a, b, alpha))
                }
                _ => Err(format!("Invalid number of components for lab: {}", s)),
            }
        } else if s.starts_with("lch") {
            match parts.as_slice() {
                [l, c, h] => {
                    let l = l
                        .parse::<ColorValue<u8>>()
                        .map_err(|_| format!("Invalid L value: {}", l))?;
                    let c = c
                        .parse::<ColorValue<u8>>()
                        .map_err(|_| format!("Invalid C value: {}", c))?;
                    let h = h
                        .parse::<Hue>()
                        .map_err(|_| format!("Invalid H value: {}", h))?;
                    Ok(Cielab::Lch(l, c, h, Alpha::Number(1.0)))
                }
                [l, c, h, alpha] => {
                    let l = l
                        .parse::<ColorValue<u8>>()
                        .map_err(|_| format!("Invalid L value: {}", l))?;
                    let c = c
                        .parse::<ColorValue<u8>>()
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
                ColorValue::Number(50),
                ColorValue::Number(20),
                ColorValue::Number(-30),
                Alpha::Number(1.0)
            )
        );
        let color = "lab(90 0 10 / 1)".parse::<Cielab>().unwrap();
        assert_eq!(
            color,
            Cielab::Lab(
                ColorValue::Number(90),
                ColorValue::Number(0),
                ColorValue::Number(10),
                Alpha::Number(1.0)
            )
        );

        let color = "lab(70, -10, 15, 0.5)".parse::<Cielab>().unwrap();
        assert_eq!(
            color,
            Cielab::Lab(
                ColorValue::Number(70),
                ColorValue::Number(-10),
                ColorValue::Number(15),
                Alpha::Number(0.5)
            )
        );
        let color = "lch(60, 30, 120)".parse::<Cielab>().unwrap();
        assert_eq!(
            color,
            Cielab::Lch(
                ColorValue::Number(60),
                ColorValue::Number(30),
                Hue::Number(120.0),
                Alpha::Number(1.0)
            )
        );
        let color = "lch(80, 40, 240, 0.75)".parse::<Cielab>().unwrap();
        assert_eq!(
            color,
            Cielab::Lch(
                ColorValue::Number(80),
                ColorValue::Number(40),
                Hue::Number(240.0),
                Alpha::Number(0.75)
            )
        );
    }
}

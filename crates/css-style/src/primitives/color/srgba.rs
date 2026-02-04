use std::str::FromStr;

use crate::{
    color::{Alpha, ColorValue, FunctionColor, Hue},
    percentage::Percentage,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SRGBAColor {
    Rgb(ColorValue<u8>, ColorValue<u8>, ColorValue<u8>, Alpha),
    Hsl(Hue, Percentage, Percentage, Alpha),
    Hwb(Hue, Percentage, Percentage, Alpha),
}

impl FromStr for SRGBAColor {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains("from") {
            return Err("Relative color syntax not supported yet".to_string());
        } else if s.contains("calc(") {
            return Err("CSS functions in color values not supported yet".to_string());
        }

        let parts = FunctionColor::tokenize_color(s, "rgb(")
            .or_else(|| FunctionColor::tokenize_color(s, "rgba("))
            .or_else(|| FunctionColor::tokenize_color(s, "hsl("))
            .or_else(|| FunctionColor::tokenize_color(s, "hsla("))
            .or_else(|| FunctionColor::tokenize_color(s, "hwb("))
            .ok_or_else(|| format!("Invalid SRGBA color: {}", s))?;

        if s.starts_with("rgb") {
            match parts.as_slice() {
                [r, g, b] => {
                    let r = r
                        .parse::<ColorValue<u8>>()
                        .map_err(|_| format!("Invalid red value: {}", r))?;
                    let g = g
                        .parse::<ColorValue<u8>>()
                        .map_err(|_| format!("Invalid green value: {}", g))?;
                    let b = b
                        .parse::<ColorValue<u8>>()
                        .map_err(|_| format!("Invalid blue value: {}", b))?;
                    Ok(SRGBAColor::Rgb(r, g, b, Alpha::Number(1.0)))
                }
                [r, g, b, a] => {
                    let r = r
                        .parse::<ColorValue<u8>>()
                        .map_err(|_| format!("Invalid red value: {}", r))?;
                    let g = g
                        .parse::<ColorValue<u8>>()
                        .map_err(|_| format!("Invalid green value: {}", g))?;
                    let b = b
                        .parse::<ColorValue<u8>>()
                        .map_err(|_| format!("Invalid blue value: {}", b))?;
                    let a = a
                        .parse::<Alpha>()
                        .map_err(|_| format!("Invalid alpha value: {}", a))?;
                    Ok(SRGBAColor::Rgb(r, g, b, a))
                }
                _ => Err(format!("Invalid RGB(A) color: {}", s)),
            }
        } else if s.starts_with("hsl") {
            match parts.as_slice() {
                [h, s, l] => {
                    let h = h
                        .parse::<Hue>()
                        .map_err(|_| format!("Invalid hue value: {}", h))?;
                    let s = s
                        .parse::<Percentage>()
                        .map_err(|_| format!("Invalid saturation value: {}", s))?;
                    let l = l
                        .parse::<Percentage>()
                        .map_err(|_| format!("Invalid lightness value: {}", l))?;
                    Ok(SRGBAColor::Hsl(h, s, l, Alpha::Number(1.0)))
                }
                [h, s, l, a] => {
                    let h = h
                        .parse::<Hue>()
                        .map_err(|_| format!("Invalid hue value: {}", h))?;
                    let s = s
                        .parse::<Percentage>()
                        .map_err(|_| format!("Invalid saturation value: {}", s))?;
                    let l = l
                        .parse::<Percentage>()
                        .map_err(|_| format!("Invalid lightness value: {}", l))?;
                    let a = a
                        .parse::<Alpha>()
                        .map_err(|_| format!("Invalid alpha value: {}", a))?;
                    Ok(SRGBAColor::Hsl(h, s, l, a))
                }
                _ => Err(format!("Invalid HSL(A) color: {}", s)),
            }
        } else if s.starts_with("hwb") {
            match parts.as_slice() {
                [h, w, b] => {
                    let h = h
                        .parse::<Hue>()
                        .map_err(|_| format!("Invalid hue value: {}", h))?;
                    let w = w
                        .parse::<Percentage>()
                        .map_err(|_| format!("Invalid whiteness value: {}", w))?;
                    let b = b
                        .parse::<Percentage>()
                        .map_err(|_| format!("Invalid blackness value: {}", b))?;
                    Ok(SRGBAColor::Hwb(h, w, b, Alpha::Number(1.0)))
                }
                [h, w, b, a] => {
                    let h = h
                        .parse::<Hue>()
                        .map_err(|_| format!("Invalid hue value: {}", h))?;
                    let w = w
                        .parse::<Percentage>()
                        .map_err(|_| format!("Invalid whiteness value: {}", w))?;
                    let b = b
                        .parse::<Percentage>()
                        .map_err(|_| format!("Invalid blackness value: {}", b))?;
                    let a = a
                        .parse::<Alpha>()
                        .map_err(|_| format!("Invalid alpha value: {}", a))?;
                    Ok(SRGBAColor::Hwb(h, w, b, a))
                }
                _ => Err(format!("Invalid HWB(A) color: {}", s)),
            }
        } else {
            Err(format!("Invalid SRGBA color: {}", s))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_srgba_color_parsing() {
        let color = "rgb(255, 0, 0)".parse::<SRGBAColor>().unwrap();
        assert_eq!(
            color,
            SRGBAColor::Rgb(
                ColorValue::from(255),
                ColorValue::from(0),
                ColorValue::from(0),
                Alpha::Number(1.0)
            )
        );

        let color = "rgba(255, 0, 0, 0.5)".parse::<SRGBAColor>().unwrap();
        assert_eq!(
            color,
            SRGBAColor::Rgb(
                ColorValue::from(255),
                ColorValue::from(0),
                ColorValue::from(0),
                Alpha::Number(0.5)
            )
        );

        let color = "hsl(120, 100%, 50%)".parse::<SRGBAColor>().unwrap();
        assert_eq!(
            color,
            SRGBAColor::Hsl(
                Hue::from(120.0),
                Percentage::from_percent(100.0),
                Percentage::from_percent(50.0),
                Alpha::Number(1.0)
            )
        );

        let color = "hsla(120, 100%, 50%, 0.3)".parse::<SRGBAColor>().unwrap();
        assert_eq!(
            color,
            SRGBAColor::Hsl(
                Hue::from(120.0),
                Percentage::from_percent(100.0),
                Percentage::from_percent(50.0),
                Alpha::Number(0.3)
            )
        );

        let color = "hwb(240, 50%, 25%)".parse::<SRGBAColor>().unwrap();
        assert_eq!(
            color,
            SRGBAColor::Hwb(
                Hue::from(240.0),
                Percentage::from_percent(50.0),
                Percentage::from_percent(25.0),
                Alpha::Number(1.0)
            )
        );

        let color = "hwb(240, 50%, 25%, 0.7)".parse::<SRGBAColor>().unwrap();
        assert_eq!(
            color,
            SRGBAColor::Hwb(
                Hue::from(240.0),
                Percentage::from_percent(50.0),
                Percentage::from_percent(25.0),
                Alpha::Number(0.7)
            )
        );
    }
}

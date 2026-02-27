//! CIELAB color function with L, a, b components, e.g., lab(50, 20, -30) or lch(50, 20, 30)

use css_cssom::ComponentValue;

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

impl TryFrom<&[ComponentValue]> for Cielab {
    type Error = String;

    fn try_from(value: &[ComponentValue]) -> Result<Self, Self::Error> {
        for val in value {
            match val {
                ComponentValue::Function(func) => {
                    if func.name.eq_ignore_ascii_case("lab") {
                        let raw = FunctionColor::parse_color_components(&func.value)?;
                        return match raw.channels {
                            [Some(l), Some(a), Some(b)] => Ok(Cielab::Lab(l, a, b, raw.alpha)),
                            _ => Err("Missing components in lab()".to_string()),
                        };
                    } else if func.name.eq_ignore_ascii_case("lch") {
                        let raw = FunctionColor::parse_color_components(&func.value)?;
                        return match raw.channels {
                            [Some(l), Some(c), Some(h)] => Ok(Cielab::Lch(l, c, Hue::from(h), raw.alpha)),
                            _ => Err("Missing components in lch()".to_string()),
                        };
                    }
                }
                _ => continue,
            }
        }
        Err("No valid lab() or lch() function found".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{css_color_fn, percentage::Percentage};

    #[test]
    fn test_lab_parse() {
        let color = css_color_fn!("lab", "20.0", "-30.0", "50.0", "none");
        let lab = Cielab::try_from(color.as_slice()).unwrap();
        assert_eq!(
            lab,
            Cielab::Lab(ColorValue::Number(20.0), ColorValue::Number(-30.0), ColorValue::Number(50.0), Alpha(1.0))
        );

        let color = css_color_fn!("lab", "20.0%", "-30.0%", "50.0%", 0.5);
        let lab = Cielab::try_from(color.as_slice()).unwrap();
        assert_eq!(
            lab,
            Cielab::Lab(
                ColorValue::Percentage(Percentage::new(20.0)),
                ColorValue::Percentage(Percentage::new(-30.0)),
                ColorValue::Percentage(Percentage::new(50.0)),
                Alpha(0.5)
            )
        );
    }

    #[test]
    fn test_lch_parse() {
        let color = css_color_fn!("lch", "20.0", "30.0", "60.0", "none");
        let lch = Cielab::try_from(color.as_slice()).unwrap();
        assert_eq!(lch, Cielab::Lch(ColorValue::Number(20.0), ColorValue::Number(30.0), Hue(60.0), Alpha(1.0)));

        let color = css_color_fn!("lch", "20.0%", "30.0%", "60.0", "0.5");
        let lch = Cielab::try_from(color.as_slice()).unwrap();
        assert_eq!(
            lch,
            Cielab::Lch(
                ColorValue::Percentage(Percentage::new(20.0)),
                ColorValue::Percentage(Percentage::new(30.0)),
                Hue(60.0),
                Alpha(0.5)
            )
        );
    }
}

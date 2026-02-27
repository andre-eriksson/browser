//! Oklab color function with L, a, b components, e.g., oklab(0.5, 0.1, -0.1) or oklch(0.5, 0.1, 30)

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
                            [Some(l), Some(c), Some(h)] => Ok(Oklab::Oklch(l, c, Hue::from(h), raw.alpha)),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{css_color_fn, percentage::Percentage};

    #[test]
    fn test_oklab_parsing() {
        let color = css_color_fn!("oklab", "0.3", "0.1", "-0.1", "0.5");
        let oklab = Oklab::try_from(color.as_slice()).unwrap();
        assert_eq!(
            oklab,
            Oklab::Oklab(ColorValue::Number(0.3), ColorValue::Number(0.1), ColorValue::Number(-0.1), Alpha(0.5))
        );

        let color = css_color_fn!("oklab", "50%", "0.2", "0.4", "0.8");
        let oklch = Oklab::try_from(color.as_slice()).unwrap();
        assert_eq!(
            oklch,
            Oklab::Oklab(
                ColorValue::Percentage(Percentage::new(50.0)),
                ColorValue::Number(0.2),
                ColorValue::Number(0.4),
                Alpha(0.8)
            )
        );
    }

    #[test]
    fn test_oklch_parsing() {
        let color = css_color_fn!("oklch", "0.5", "0.1", "30.0", "0.7");
        let oklch = Oklab::try_from(color.as_slice()).unwrap();
        assert_eq!(oklch, Oklab::Oklch(ColorValue::Number(0.5), ColorValue::Number(0.1), Hue(30.0), Alpha(0.7)));

        let color = css_color_fn!("oklch", "50%", "0.2", "120", "0.8");
        let oklch = Oklab::try_from(color.as_slice()).unwrap();
        assert_eq!(
            oklch,
            Oklab::Oklch(
                ColorValue::Percentage(Percentage::new(50.0)),
                ColorValue::Number(0.2),
                Hue(120.0),
                Alpha(0.8)
            )
        );
    }
}

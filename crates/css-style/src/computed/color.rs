use browser_preferences::theme::ThemeCategory;
use css_values::color::{
    Color, ColorValue, Fraction, Hue,
    base::{ColorBase, HexColor},
    function::ColorFunction,
    named::NamedColor,
    system::SystemColor,
};

use crate::{AbsoluteContext, RelativeContext, properties::CSSProperty};

/// RGBA color representation for rendering (values 0.0-1.0, sRGB gamma-encoded)
///
/// All conversion paths produce **sRGB gamma-encoded** values so that colours
/// specified via different CSS syntaxes (hex, named, `rgb()`, `hsl()`, `oklab()`,
/// `oklch()`, `lab()`, `lch()`, `hwb()`) are mutually consistent.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color4f<T = f64> {
    pub r: T,
    pub g: T,
    pub b: T,
    pub a: T,
}

impl Color4f {
    pub const BLACK: Self = Self::rgba(0.0, 0.0, 0.0, 1.0);
    pub const WHITE: Self = Self::rgba(1.0, 1.0, 1.0, 1.0);
    pub const TRANSPARENT: Self = Self::rgba(0.0, 0.0, 0.0, 0.0);

    /// Creates a new Color4f with the specified RGBA values (0.0-1.0, sRGB gamma-encoded).
    #[must_use]
    pub const fn rgba(r: f64, g: f64, b: f64, a: f64) -> Self {
        Self { r, g, b, a }
    }

    /// Creates a new Color4f from RGBA values specified as 8-bit integers (0-255), converting them to the 0.0-1.0 range.
    #[must_use]
    pub fn rgba_u8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            r: f64::from(r) / 255.0,
            g: f64::from(g) / 255.0,
            b: f64::from(b) / 255.0,
            a: f64::from(a) / 255.0,
        }
    }

    /// Compares two Color4f values for approximate equality within a specified tolerance.
    #[must_use]
    pub const fn compare_with_tolerance(&self, other: &Self, tolerance: f64) -> bool {
        (self.r - other.r).abs() <= tolerance
            && (self.g - other.g).abs() <= tolerance
            && (self.b - other.b).abs() <= tolerance
            && (self.a - other.a).abs() <= tolerance
    }

    /// Resolves the `color` property used by `currentColor` into a concrete color reference.
    ///
    /// Returns `None` when resolution loops back to `currentColor` (directly or through
    /// `light-dark(...)`) so callers can safely fall back to inherited parent color.
    fn resolve_current_color<'css>(
        text_color: &'css CSSProperty<Color>,
        absolute_ctx: &AbsoluteContext,
    ) -> Option<&'css Color> {
        let mut resolved = text_color.as_value()?;

        loop {
            match resolved {
                Color::Current => return None,
                Color::LightDark(light, dark) => {
                    resolved = match absolute_ctx.theme_category {
                        ThemeCategory::Light => light.as_ref(),
                        ThemeCategory::Dark => dark.as_ref(),
                    };
                }
                _ => return Some(resolved),
            }
        }
    }

    /// Converts a CSS Color to Color4f
    fn from_css_color(
        color: &Color,
        text_color: &CSSProperty<Color>,
        relative_ctx: &RelativeContext,
        absolute_ctx: &AbsoluteContext,
    ) -> Self {
        match color {
            Color::Base(ColorBase::Named(named)) => Self::from(*named),
            Color::Base(ColorBase::Hex(hex)) => Self::from(*hex),
            Color::Base(ColorBase::Function(func)) => Self::from(func.clone()),
            Color::Base(ColorBase::Transparent) => Self::TRANSPARENT,
            Color::Current => Self::resolve_current_color(text_color, absolute_ctx).map_or_else(
                || relative_ctx.parent.color,
                |resolved| Self::from_css_color(resolved, text_color, relative_ctx, absolute_ctx),
            ),
            Color::System(system) => Self::from(*system),
            Color::LightDark(light, dark) => {
                let branch = match absolute_ctx.theme_category {
                    ThemeCategory::Light => light.as_ref(),
                    ThemeCategory::Dark => dark.as_ref(),
                };
                Self::from_css_color(branch, text_color, relative_ctx, absolute_ctx)
            }
        }
    }

    /// Converts a CSS color property to Color4f, resolving 'currentColor' and inheriting from the parent if necessary.
    pub(crate) fn from_css_color_property(
        color: &CSSProperty<Color>,
        text_color: &CSSProperty<Color>,
        initial: &Color,
        parent: &Color,
        relative_ctx: &RelativeContext,
        absolute_ctx: &AbsoluteContext,
    ) -> Self {
        let initial = match initial {
            Color::Current => Self::resolve_current_color(text_color, absolute_ctx).unwrap_or(initial),
            _ => initial,
        };
        let resolved_color = color.resolve_with_context(parent, initial);

        Self::from_css_color(resolved_color, text_color, relative_ctx, absolute_ctx)
    }

    /// Parses a hex color string (e.g. "#RRGGBB") into an (r, g, b) tuple.
    fn hex_to_rgb_tuple(hex: &str) -> Option<(u8, u8, u8)> {
        let hex = hex.trim_start_matches('#');
        if hex.len() == 6
            && let Ok(parsed) = u32::from_str_radix(hex, 16)
        {
            let r = ((parsed >> 16) & 0xFF) as u8;
            let g = ((parsed >> 8) & 0xFF) as u8;
            let b = (parsed & 0xFF) as u8;
            return Some((r, g, b));
        }
        None
    }

    /// Converts a single linear-light component to sRGB gamma-encoded (0.0–1.0).
    #[inline]
    fn linear_component_to_srgb(c: f64) -> f64 {
        if c <= 0.003_130_8 {
            12.92 * c
        } else {
            1.055f64.mul_add(c.powf(1.0 / 2.4), -0.055)
        }
    }

    /// Resolves a CSS `Hue` value to radians.
    ///
    /// Per the CSS Color Level 4 spec, a bare `<number>` in a hue position is
    /// interpreted as **degrees**, not turns.
    #[inline]
    fn hue_to_radians(h: Hue) -> f64 {
        h.value().to_radians()
    }
}

impl From<Color4f<f32>> for [f32; 4] {
    fn from(c: Color4f<f32>) -> Self {
        [c.r, c.g, c.b, c.a]
    }
}

impl From<[f64; 4]> for Color4f {
    fn from([r, g, b, a]: [f64; 4]) -> Self {
        Self { r, g, b, a }
    }
}

impl From<(u8, u8, u8)> for Color4f {
    fn from((r, g, b): (u8, u8, u8)) -> Self {
        Self::rgba(f64::from(r) / 255.0, f64::from(g) / 255.0, f64::from(b) / 255.0, 1.0)
    }
}

impl From<SystemColor> for Color4f {
    fn from(system: SystemColor) -> Self {
        let Some(hex) = system.to_hex() else {
            return Self::BLACK;
        };

        if let Some(rgb) = Self::hex_to_rgb_tuple(hex) {
            return Self::from(rgb);
        }

        Self::BLACK
    }
}

impl From<NamedColor> for Color4f {
    fn from(named: NamedColor) -> Self {
        let Some(hex) = named.to_hex() else {
            return Self::BLACK;
        };

        if let Some(rgb) = Self::hex_to_rgb_tuple(hex) {
            return Self::from(rgb);
        }
        Self::BLACK
    }
}

impl From<HexColor> for Color4f {
    fn from(hex: HexColor) -> Self {
        Self {
            r: f64::from(hex.r) / 255.0,
            g: f64::from(hex.g) / 255.0,
            b: f64::from(hex.b) / 255.0,
            a: f64::from(hex.a) / 255.0,
        }
    }
}

impl From<ColorFunction> for Color4f {
    fn from(value: ColorFunction) -> Self {
        match value {
            ColorFunction::Rgb(red, green, blue, alpha) => Self::rgba(
                red.value(0.0..=255.0, Fraction::Unsigned) / 255.0,
                green.value(0.0..=255.0, Fraction::Unsigned) / 255.0,
                blue.value(0.0..=255.0, Fraction::Unsigned) / 255.0,
                alpha.value(),
            ),
            ColorFunction::Hsl(hue, saturation, lightness, alpha) => {
                let h_deg = ((hue.value() % 360.0) + 360.0) % 360.0;
                let s_frac = saturation.as_fraction();
                let l_frac = lightness.as_fraction();

                let c = (1.0 - 2.0f64.mul_add(l_frac, -1.0).abs()) * s_frac;
                let x = c * (1.0 - ((h_deg / 60.0) % 2.0 - 1.0).abs());
                let m = l_frac - c / 2.0;

                let (r1, g1, b1) = if h_deg < 60.0 {
                    (c, x, 0.0)
                } else if h_deg < 120.0 {
                    (x, c, 0.0)
                } else if h_deg < 180.0 {
                    (0.0, c, x)
                } else if h_deg < 240.0 {
                    (0.0, x, c)
                } else if h_deg < 300.0 {
                    (x, 0.0, c)
                } else {
                    (c, 0.0, x)
                };

                Self::rgba((r1 + m).clamp(0.0, 1.0), (g1 + m).clamp(0.0, 1.0), (b1 + m).clamp(0.0, 1.0), alpha.value())
            }
            ColorFunction::Hwb(hue, whiteness, blackness, alpha) => {
                let h_deg = ((hue.value() % 360.0) + 360.0) % 360.0;
                let mut w_frac = whiteness.as_fraction();
                let mut b_frac = blackness.as_fraction();

                let sum = w_frac + b_frac;
                if sum > 1.0 {
                    w_frac /= sum;
                    b_frac /= sum;
                }

                let c = 1.0_f64;
                let x = c * (1.0 - ((h_deg / 60.0) % 2.0 - 1.0).abs());

                let (r1, g1, b1) = if h_deg < 60.0 {
                    (c, x, 0.0)
                } else if h_deg < 120.0 {
                    (x, c, 0.0)
                } else if h_deg < 180.0 {
                    (0.0, c, x)
                } else if h_deg < 240.0 {
                    (0.0, x, c)
                } else if h_deg < 300.0 {
                    (x, 0.0, c)
                } else {
                    (c, 0.0, x)
                };

                let scale = 1.0 - w_frac - b_frac;
                Self::rgba(
                    r1.mul_add(scale, w_frac).clamp(0.0, 1.0),
                    g1.mul_add(scale, w_frac).clamp(0.0, 1.0),
                    b1.mul_add(scale, w_frac).clamp(0.0, 1.0),
                    alpha.value(),
                )
            }
            ColorFunction::Lab(lightness, green_red, blue_yellow, alpha) => {
                let l_val = lightness.value(0.0..=100.0, Fraction::Unsigned);
                let a_val = green_red.value(-125.0..=125.0, Fraction::Signed);
                let b_val = blue_yellow.value(-125.0..=125.0, Fraction::Signed);

                let fy = (l_val + 16.0) / 116.0;
                let fx = a_val / 500.0 + fy;
                let fz = fy - b_val / 200.0;

                let delta = 6.0_f64 / 29.0;
                let delta_sq = delta * delta;
                let delta_cu = delta_sq * delta;

                let x_ref = if fx.powi(3) > delta_cu {
                    fx.powi(3)
                } else {
                    116.0f64.mul_add(fx, -16.0) / 903.3
                };
                let y_ref = if l_val > (delta_cu * 903.3) {
                    fy.powi(3)
                } else {
                    l_val / 903.3
                };
                let z_ref = if fz.powi(3) > delta_cu {
                    fz.powi(3)
                } else {
                    116.0f64.mul_add(fz, -16.0) / 903.3
                };

                // D65 white point
                let x_final = x_ref * 0.950_47;
                let y_final = y_ref * 1.000_00;
                let z_final = z_ref * 1.088_83;

                let r = x_final * 3.2406 + y_final * -1.5372 + z_final * -0.4986;
                let g = x_final * -0.9689 + y_final * 1.8758 + z_final * 0.0415;
                let b = x_final * 0.0557 + y_final * -0.2040 + z_final * 1.0570;

                Self::rgba(
                    Self::linear_component_to_srgb(r).clamp(0.0, 1.0),
                    Self::linear_component_to_srgb(g).clamp(0.0, 1.0),
                    Self::linear_component_to_srgb(b).clamp(0.0, 1.0),
                    alpha.value(),
                )
            }
            ColorFunction::Lch(brightness, colorfulness, hue, alpha) => {
                let h_rad = Self::hue_to_radians(hue);
                let green_red = colorfulness.value(0.0..=150.0, Fraction::Unsigned) * h_rad.cos();
                let blue_yellow = colorfulness.value(0.0..=150.0, Fraction::Unsigned) * h_rad.sin();
                Self::from(ColorFunction::Lab(
                    brightness,
                    ColorValue::from(green_red),
                    ColorValue::from(blue_yellow),
                    alpha,
                ))
            }
            ColorFunction::Oklab(lightness, green_red, blue_yellow, alpha) => {
                let l_val = lightness.value(0.0..=1.0, Fraction::Unsigned);
                let a_val = green_red.value(-0.4..=0.4, Fraction::Signed);
                let b_val = blue_yellow.value(-0.4..=0.4, Fraction::Signed);

                let l_ = 0.215_803_76f64.mul_add(b_val, 0.396_337_78f64.mul_add(a_val, l_val));
                let m_ = 0.063_854_17f64.mul_add(-b_val, 0.105_561_346f64.mul_add(-a_val, l_val));
                let s_ = 1.291_485_5f64.mul_add(-b_val, 0.089_484_18f64.mul_add(-a_val, l_val));

                let l_lin = l_ * l_ * l_;
                let m_lin = m_ * m_ * m_;
                let s_lin = s_ * s_ * s_;

                let r_lin = 0.230_969_94f64.mul_add(s_lin, 4.076_741_7f64.mul_add(l_lin, -(3.307_711_6 * m_lin)));
                let g_lin = 0.341_319_38f64.mul_add(-s_lin, (-1.268_438f64).mul_add(l_lin, 2.609_757_4 * m_lin));
                let b_lin = 1.707_614_7f64.mul_add(s_lin, (-0.004_196_086_3_f64).mul_add(l_lin, -(0.703_419 * m_lin)));

                Self::rgba(
                    Self::linear_component_to_srgb(r_lin).clamp(0.0, 1.0),
                    Self::linear_component_to_srgb(g_lin).clamp(0.0, 1.0),
                    Self::linear_component_to_srgb(b_lin).clamp(0.0, 1.0),
                    alpha.value(),
                )
            }
            ColorFunction::Oklch(lightness, colorfulness, hue, alpha) => {
                let h_rad = Self::hue_to_radians(hue);
                let a = colorfulness.value(0.0..=0.4, Fraction::Unsigned) * h_rad.cos();
                let b = colorfulness.value(0.0..=0.4, Fraction::Unsigned) * h_rad.sin();
                Self::from(ColorFunction::Oklab(lightness, ColorValue::from(a), ColorValue::from(b), alpha))
            }
        }
    }
}

impl From<Color4f<f64>> for Color4f<f32> {
    fn from(c: Color4f<f64>) -> Self {
        Self {
            r: c.r as f32,
            g: c.g as f32,
            b: c.b as f32,
            a: c.a as f32,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{net::Ipv4Addr, sync::Arc};

    use url::Url;

    use super::*;
    use crate::ComputedStyle;

    fn relative_ctx_with_parent_color(color: Color4f) -> RelativeContext {
        let parent = ComputedStyle {
            color,
            ..Default::default()
        };
        RelativeContext {
            parent: Arc::new(parent),
            font_size: 16.0,
        }
    }

    #[test]
    fn current_color_self_reference_falls_back_to_parent_color() {
        let url = Box::leak(Box::new(Url::parse(&format!("http://{}", Ipv4Addr::LOCALHOST)).unwrap()));
        let parent_color = [0.2, 0.3, 0.4, 1.0].into();
        let relative_ctx = relative_ctx_with_parent_color(parent_color);
        let absolute_ctx = AbsoluteContext::default_url(url);

        let text_color = CSSProperty::Value(Color::Current);
        let color = Color4f::from_css_color_property(
            &CSSProperty::Value(Color::Current),
            &text_color,
            &Color::Current,
            &Color::Current,
            &relative_ctx,
            &absolute_ctx,
        );

        assert_eq!(color, parent_color);
    }

    #[test]
    fn light_dark_current_in_light_theme_falls_back_to_parent_color() {
        let url = Box::leak(Box::new(Url::parse(&format!("http://{}", Ipv4Addr::LOCALHOST)).unwrap()));
        let parent_color = [0.1, 0.2, 0.3, 1.0].into();
        let relative_ctx = relative_ctx_with_parent_color(parent_color);
        let absolute_ctx = AbsoluteContext {
            theme_category: ThemeCategory::Light,
            ..AbsoluteContext::default_url(url)
        };

        let text_color = CSSProperty::Value(Color::LightDark(
            Box::new(Color::Current),
            Box::new(Color::Base(ColorBase::Named(NamedColor::Red))),
        ));

        let color = Color4f::from_css_color_property(
            &CSSProperty::Value(Color::Current),
            &text_color,
            &Color::Current,
            &Color::Current,
            &relative_ctx,
            &absolute_ctx,
        );

        assert_eq!(color, parent_color);
    }

    #[test]
    fn light_dark_current_in_dark_theme_uses_dark_branch() {
        let url = Box::leak(Box::new(Url::parse(&format!("http://{}", Ipv4Addr::LOCALHOST)).unwrap()));
        let relative_ctx = relative_ctx_with_parent_color([0.1, 0.2, 0.3, 1.0].into());
        let absolute_ctx = AbsoluteContext {
            theme_category: ThemeCategory::Dark,
            ..AbsoluteContext::default_url(url)
        };

        let text_color = CSSProperty::Value(Color::LightDark(
            Box::new(Color::Current),
            Box::new(Color::Base(ColorBase::Named(NamedColor::Red))),
        ));

        let color = Color4f::from_css_color_property(
            &CSSProperty::Value(Color::Current),
            &text_color,
            &Color::Current,
            &Color::Current,
            &relative_ctx,
            &absolute_ctx,
        );

        assert_eq!(color, Color4f::from(NamedColor::Red));
    }
}

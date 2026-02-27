use preferences::ThemeCategory;

use crate::{
    AbsoluteContext, Color, RelativeContext,
    color::{
        ColorValue, Fraction, FunctionColor, Hue,
        cielab::Cielab,
        hex::HexColor,
        named::{NamedColor, SystemColor},
        oklab::Oklab,
        srgba::SRGBAColor,
    },
    properties::CSSProperty,
};

/// RGBA color representation for rendering (values 0.0-1.0, sRGB gamma-encoded)
///
/// All conversion paths produce **sRGB gamma-encoded** values so that colours
/// specified via different CSS syntaxes (hex, named, rgb(), hsl(), oklab(),
/// oklch(), lab(), lch(), hwb()) are mutually consistent.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color4f {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Default for Color4f {
    fn default() -> Self {
        Self {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }
    }
}

impl Color4f {
    /// Creates a new Color4f with the specified RGBA values (0.0-1.0, sRGB gamma-encoded).
    pub(crate) fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Converts a CSS Color to Color4f
    fn from_css_color(
        color: &Color,
        text_color: &CSSProperty<Color>,
        relative_ctx: &RelativeContext,
        absolute_ctx: &AbsoluteContext,
    ) -> Self {
        match color {
            Color::Named(named) => Self::from(*named),
            Color::Hex(hex) => Self::from(*hex),
            Color::Functional(func) => match func {
                FunctionColor::LightDark(light, dark) => match absolute_ctx.theme_category {
                    ThemeCategory::Light => Self::from_css_color(light, text_color, relative_ctx, absolute_ctx),
                    ThemeCategory::Dark => Self::from_css_color(dark, text_color, relative_ctx, absolute_ctx),
                },
                _ => Self::from(func.clone()),
            },
            Color::Current => {
                if let Some(resolved) = text_color.as_value() {
                    Self::from_css_color(resolved, text_color, relative_ctx, absolute_ctx)
                } else {
                    relative_ctx.parent.color
                }
            }
            Color::System(system) => Self::from(*system),
            Color::Transparent => Self::new(0.0, 0.0, 0.0, 0.0),
        }
    }

    /// Converts a CSS color property to Color4f, resolving 'currentColor' and inheriting from the parent if necessary.
    pub(crate) fn from_css_color_property(
        color: &CSSProperty<Color>,
        text_color: &CSSProperty<Color>,
        initial: &Color,
        parent: Option<Color>,
        relative_ctx: &RelativeContext,
        absolute_ctx: &AbsoluteContext,
    ) -> Self {
        let initial = match initial {
            Color::Current => text_color.as_value().unwrap_or(initial),
            _ => initial,
        };

        let resolved_color = color.resolve_with_context(parent.as_ref(), initial);

        Self::from_css_color(resolved_color, text_color, relative_ctx, absolute_ctx)
    }

    /// Parses a hex color string (e.g. "#RRGGBB") into an (r, g, b) tuple.
    fn to_rgb_tuple(hex: &str) -> Option<(u8, u8, u8)> {
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

    /// Returns color as [r, g, b, a] array (sRGB gamma-encoded) for GPU upload.
    ///
    /// This is the correct choice for most rendering pipelines, including
    /// `Rgb10a2Unorm` surfaces where the compositor still expects sRGB values.
    pub fn to_array(self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }

    /// Returns color as [r, g, b, a] array converted to **linear** light.
    ///
    /// Only use this for pipelines that genuinely operate in linear space
    /// (e.g. physically-based rendering or intermediate compute passes).
    /// Most compositor-facing surfaces (`Rgb10a2Unorm`, `Bgra8Unorm`, etc.)
    /// expect sRGB-encoded values — use [`to_array`](Self::to_array) instead.
    pub fn to_linear_array(self) -> [f32; 4] {
        [
            Self::srgb_component_to_linear(self.r),
            Self::srgb_component_to_linear(self.g),
            Self::srgb_component_to_linear(self.b),
            self.a,
        ]
    }

    /// Converts a single sRGB gamma-encoded component (0.0–1.0) to linear light.
    #[inline]
    fn srgb_component_to_linear(c: f32) -> f32 {
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    }

    /// Converts a single linear-light component to sRGB gamma-encoded (0.0–1.0).
    #[inline]
    fn linear_component_to_srgb(c: f32) -> f32 {
        if c <= 0.0031308 {
            12.92 * c
        } else {
            1.055 * c.powf(1.0 / 2.4) - 0.055
        }
    }

    /// Resolves a CSS `Hue` value to radians.
    ///
    /// Per the CSS Color Level 4 spec, a bare `<number>` in a hue position is
    /// interpreted as **degrees**, not turns.
    #[inline]
    fn hue_to_radians(h: &Hue) -> f32 {
        h.value().to_radians()
    }
}

impl From<SystemColor> for Color4f {
    fn from(system: SystemColor) -> Self {
        let hex = if let Some(hex) = system.to_hex() {
            hex
        } else {
            return Self::default();
        };

        if let Some(rgb) = Self::to_rgb_tuple(hex) {
            return Self::new(rgb.0 as f32 / 255.0, rgb.1 as f32 / 255.0, rgb.2 as f32 / 255.0, 1.0);
        }
        Self::default()
    }
}

impl From<NamedColor> for Color4f {
    fn from(named: NamedColor) -> Self {
        let hex = if let Some(hex) = named.to_hex() {
            hex
        } else {
            return Self::default();
        };

        if let Some(rgb) = Self::to_rgb_tuple(hex) {
            return Self::new(rgb.0 as f32 / 255.0, rgb.1 as f32 / 255.0, rgb.2 as f32 / 255.0, 1.0);
        }
        Self::default()
    }
}

impl From<HexColor> for Color4f {
    fn from(hex: HexColor) -> Self {
        Self {
            r: hex.r as f32 / 255.0,
            g: hex.g as f32 / 255.0,
            b: hex.b as f32 / 255.0,
            a: hex.a as f32 / 255.0,
        }
    }
}

impl From<Oklab> for Color4f {
    fn from(oklab: Oklab) -> Self {
        match oklab {
            Oklab::Oklab(l, a, b, alpha) => {
                let l_val = l.value(0.0..=1.0, Fraction::Unsigned);
                let a_val = a.value(-0.4..=0.4, Fraction::Signed);
                let b_val = b.value(-0.4..=0.4, Fraction::Signed);

                let l_ = l_val + 0.396_337_78 * a_val + 0.215_803_76 * b_val;
                let m_ = l_val - 0.105_561_346 * a_val - 0.063_854_17 * b_val;
                let s_ = l_val - 0.089_484_18 * a_val - 1.291_485_5 * b_val;

                let l_lin = l_ * l_ * l_;
                let m_lin = m_ * m_ * m_;
                let s_lin = s_ * s_ * s_;

                let r_lin = 4.076_741_7 * l_lin - 3.307_711_6 * m_lin + 0.230_969_94 * s_lin;
                let g_lin = -1.268_438 * l_lin + 2.609_757_4 * m_lin - 0.341_319_38 * s_lin;
                let b_lin = -0.0041960863 * l_lin - 0.703_419 * m_lin + 1.707_614_7 * s_lin;

                Self::new(
                    Self::linear_component_to_srgb(r_lin).clamp(0.0, 1.0),
                    Self::linear_component_to_srgb(g_lin).clamp(0.0, 1.0),
                    Self::linear_component_to_srgb(b_lin).clamp(0.0, 1.0),
                    alpha.value(),
                )
            }
            Oklab::Oklch(l, c, h, alpha) => {
                let h_rad = Self::hue_to_radians(&h);
                let a = c.value(0.0..=0.4, Fraction::Unsigned) * h_rad.cos();
                let b = c.value(0.0..=0.4, Fraction::Unsigned) * h_rad.sin();
                Self::from(Oklab::Oklab(l, ColorValue::from(a), ColorValue::from(b), alpha))
            }
        }
    }
}

impl From<Cielab> for Color4f {
    fn from(cielab: Cielab) -> Self {
        match cielab {
            Cielab::Lab(l, a, b, alpha) => {
                let l_val = l.value(0.0..=100.0, Fraction::Unsigned);
                let a_val = a.value(-125.0..=125.0, Fraction::Signed);
                let b_val = b.value(-125.0..=125.0, Fraction::Signed);

                let fy = (l_val + 16.0) / 116.0;
                let fx = a_val / 500.0 + fy;
                let fz = fy - b_val / 200.0;

                let delta = 6.0_f32 / 29.0;
                let delta_sq = delta * delta;
                let delta_cu = delta_sq * delta;

                let x_ref = if fx.powi(3) > delta_cu {
                    fx.powi(3)
                } else {
                    (116.0 * fx - 16.0) / 903.3
                };
                let y_ref = if l_val > (delta_cu * 903.3) {
                    fy.powi(3)
                } else {
                    l_val / 903.3
                };
                let z_ref = if fz.powi(3) > delta_cu {
                    fz.powi(3)
                } else {
                    (116.0 * fz - 16.0) / 903.3
                };

                // D65 white point
                let x_final = x_ref * 0.950_47;
                let y_final = y_ref * 1.000_00;
                let z_final = z_ref * 1.088_83;

                let r = x_final * 3.2406 + y_final * -1.5372 + z_final * -0.4986;
                let g = x_final * -0.9689 + y_final * 1.8758 + z_final * 0.0415;
                let b = x_final * 0.0557 + y_final * -0.2040 + z_final * 1.0570;

                Self::new(
                    Self::linear_component_to_srgb(r).clamp(0.0, 1.0),
                    Self::linear_component_to_srgb(g).clamp(0.0, 1.0),
                    Self::linear_component_to_srgb(b).clamp(0.0, 1.0),
                    alpha.value(),
                )
            }
            Cielab::Lch(l, c, h, alpha) => {
                let h_rad = Self::hue_to_radians(&h);
                let a = c.value(0.0..=125.0, Fraction::Unsigned) * h_rad.cos();
                let b = c.value(0.0..=125.0, Fraction::Unsigned) * h_rad.sin();
                Self::from(Cielab::Lab(l, ColorValue::from(a), ColorValue::from(b), alpha))
            }
        }
    }
}

impl From<FunctionColor> for Color4f {
    fn from(func: FunctionColor) -> Self {
        match func {
            FunctionColor::Srgba(srgba) => match srgba {
                SRGBAColor::Rgb(r, g, b, a) => Self::new(
                    r.value(0.0..=255.0, Fraction::Unsigned) / 255.0,
                    g.value(0.0..=255.0, Fraction::Unsigned) / 255.0,
                    b.value(0.0..=255.0, Fraction::Unsigned) / 255.0,
                    a.value(),
                ),
                SRGBAColor::Hsl(h, s, l, a) => {
                    let h_deg = h.value();

                    let s = s.as_fraction();
                    let l = l.as_fraction();

                    let h_deg = ((h_deg % 360.0) + 360.0) % 360.0;

                    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
                    let x = c * (1.0 - ((h_deg / 60.0) % 2.0 - 1.0).abs());
                    let m = l - c / 2.0;

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

                    Self::new((r1 + m).clamp(0.0, 1.0), (g1 + m).clamp(0.0, 1.0), (b1 + m).clamp(0.0, 1.0), a.value())
                }
                SRGBAColor::Hwb(h, w, b, a) => {
                    let h_deg = h.value();

                    let mut w_frac = w.as_fraction();
                    let mut b_frac = b.as_fraction();

                    let sum = w_frac + b_frac;
                    if sum > 1.0 {
                        w_frac /= sum;
                        b_frac /= sum;
                    }

                    let h_deg = ((h_deg % 360.0) + 360.0) % 360.0;
                    let c = 1.0_f32;
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
                    Self::new(
                        (r1 * scale + w_frac).clamp(0.0, 1.0),
                        (g1 * scale + w_frac).clamp(0.0, 1.0),
                        (b1 * scale + w_frac).clamp(0.0, 1.0),
                        a.value(),
                    )
                }
            },
            FunctionColor::Oklab(oklab) => Self::from(oklab),
            FunctionColor::Cielab(cielab) => Self::from(cielab),
            _ => {
                // For unsupported function colors, return transparent black as a fallback.
                Self::new(0.0, 0.0, 0.0, 0.0)
            }
        }
    }
}

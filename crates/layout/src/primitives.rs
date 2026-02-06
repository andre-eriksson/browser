use css_style::{
    Color, Property,
    color::{
        ColorValue, FunctionColor, Hue, cielab::Cielab, named::NamedColor, oklab::Oklab,
        srgba::SRGBAColor,
    },
};

/// Rectangle representation for layout dimensions and positions
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn contains_point(&self, px: f32, py: f32) -> bool {
        px >= self.x && px <= self.x + self.width && py >= self.y && py <= self.y + self.height
    }
}

/// RGBA color representation for rendering (values 0.0-1.0, sRGB gamma-encoded)
///
/// All conversion paths produce **sRGB gamma-encoded** values so that colours
/// specified via different CSS syntaxes (hex, named, rgb(), hsl(), oklab(),
/// oklch(), lab(), lch(), hwb()) are mutually consistent.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Color4f {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color4f {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn from_css_color(color: &Color) -> Self {
        match color {
            Color::Named(named) => Self::from_named_color(named),
            Color::Hex(hex) => Self {
                r: hex.r as f32 / 255.0,
                g: hex.g as f32 / 255.0,
                b: hex.b as f32 / 255.0,
                a: hex.a as f32 / 255.0,
            },
            Color::Functional(func) => Self::from_functional_color(func),
            Color::Current => Self::new(0.0, 0.0, 0.0, 1.0), // TODO: Handle currentColor properly
            Color::System(_) => Self::new(0.0, 0.0, 0.0, 1.0), // TODO: Handle system colors
            Color::Transparent => Self::new(0.0, 0.0, 0.0, 0.0),
        }
    }

    /// Converts a CSS color to Color4f
    pub fn from_css_color_property(color: &Property<Color>) -> Self {
        if let Ok(resolved_color) = Property::resolve(color) {
            Self::from_css_color(resolved_color)
        } else {
            Self::new(0.0, 0.0, 0.0, 1.0)
        }
    }

    /// Returns a copy with RGB channels converted from sRGB gamma to linear
    /// light.  Use this when uploading colours to a **linear** surface format
    /// such as `Rgb10a2Unorm` where the GPU will NOT apply an automatic sRGB
    /// transfer function.
    ///
    /// Alpha is left untouched (it is always linear).
    pub fn to_linear(&self) -> Self {
        Self {
            r: Self::srgb_component_to_linear(self.r),
            g: Self::srgb_component_to_linear(self.g),
            b: Self::srgb_component_to_linear(self.b),
            a: self.a,
        }
    }

    /// Returns a copy with RGB channels converted from linear light to sRGB
    /// gamma. This is the inverse of [`to_linear`](Self::to_linear).
    pub fn to_srgb(&self) -> Self {
        Self {
            r: Self::linear_component_to_srgb(self.r),
            g: Self::linear_component_to_srgb(self.g),
            b: Self::linear_component_to_srgb(self.b),
            a: self.a,
        }
    }

    fn from_named_color(named: &NamedColor) -> Self {
        if let Some(rgb) = named.to_rgb_tuple() {
            return Self::new(
                rgb.0 as f32 / 255.0,
                rgb.1 as f32 / 255.0,
                rgb.2 as f32 / 255.0,
                1.0,
            );
        }
        Self::new(0.0, 0.0, 0.0, 1.0)
    }

    fn from_oklab(oklab: Oklab) -> Self {
        match oklab {
            Oklab::Oklab(l, a, b, alpha) => {
                let l_val = l.as_number();
                let a_val = a.as_number();
                let b_val = b.as_number();

                // OKLab → approximate LMS (cube-root domain)
                let l_ = l_val + 0.396_337_78 * a_val + 0.215_803_76 * b_val;
                let m_ = l_val - 0.105_561_346 * a_val - 0.063_854_17 * b_val;
                let s_ = l_val - 0.089_484_18 * a_val - 1.291_485_5 * b_val;

                // Undo cube-root to get linear LMS
                let l_lin = l_ * l_ * l_;
                let m_lin = m_ * m_ * m_;
                let s_lin = s_ * s_ * s_;

                // Linear LMS → linear sRGB
                let r_lin = 4.076_741_7 * l_lin - 3.307_711_6 * m_lin + 0.230_969_94 * s_lin;
                let g_lin = -1.268_438 * l_lin + 2.609_757_4 * m_lin - 0.341_319_38 * s_lin;
                let b_lin = -0.0041960863 * l_lin - 0.703_419 * m_lin + 1.707_614_7 * s_lin;

                // Apply sRGB gamma encoding + clamp so the output is in the
                // same space as every other conversion path.
                Self::new(
                    Self::linear_component_to_srgb(r_lin).clamp(0.0, 1.0),
                    Self::linear_component_to_srgb(g_lin).clamp(0.0, 1.0),
                    Self::linear_component_to_srgb(b_lin).clamp(0.0, 1.0),
                    alpha.as_fraction(),
                )
            }
            Oklab::Oklch(l, c, h, alpha) => {
                let h_rad = Self::hue_to_radians(&h);
                let a = c.as_number() * h_rad.cos();
                let b = c.as_number() * h_rad.sin();
                Self::from_oklab(Oklab::Oklab(
                    l,
                    ColorValue::Number(a),
                    ColorValue::Number(b),
                    alpha,
                ))
            }
        }
    }

    fn from_cielab(cielab: Cielab) -> Self {
        match cielab {
            Cielab::Lab(l, a, b, alpha) => {
                let l_val = l.as_number();
                let a_val = a.as_number();
                let b_val = b.as_number();

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
                    alpha.as_fraction(),
                )
            }
            Cielab::Lch(l, c, h, alpha) => {
                let h_rad = Self::hue_to_radians(&h);
                let a = c.as_number() * h_rad.cos();
                let b = c.as_number() * h_rad.sin();
                Self::from_cielab(Cielab::Lab(
                    l,
                    ColorValue::Number(a),
                    ColorValue::Number(b),
                    alpha,
                ))
            }
        }
    }

    fn from_functional_color(func: &FunctionColor) -> Self {
        match func {
            FunctionColor::Srgba(srgba) => match srgba {
                SRGBAColor::Rgb(r, g, b, a) => Self::new(
                    r.as_number() / 255.0,
                    g.as_number() / 255.0,
                    b.as_number() / 255.0,
                    a.as_fraction(),
                ),
                SRGBAColor::Hsl(h, s, l, a) => {
                    let h_deg = match h {
                        Hue::Angle(deg) => deg.to_degrees(),
                        Hue::Number(num) => *num,
                    };

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

                    Self::new(
                        (r1 + m).clamp(0.0, 1.0),
                        (g1 + m).clamp(0.0, 1.0),
                        (b1 + m).clamp(0.0, 1.0),
                        a.as_fraction(),
                    )
                }
                SRGBAColor::Hwb(h, w, b, a) => {
                    let h_deg = match h {
                        Hue::Angle(deg) => deg.to_degrees(),
                        Hue::Number(num) => *num,
                    };

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
                        a.as_fraction(),
                    )
                }
            },
            FunctionColor::Oklab(oklab) => Self::from_oklab(*oklab),
            FunctionColor::Cielab(cielab) => Self::from_cielab(*cielab),
        }
    }

    /// Returns color as [r, g, b, a] array (sRGB gamma-encoded) for GPU upload.
    ///
    /// This is the correct choice for most rendering pipelines, including
    /// `Rgb10a2Unorm` surfaces where the compositor still expects sRGB values.
    pub fn to_array(&self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }

    /// Returns color as [r, g, b, a] array converted to **linear** light.
    ///
    /// Only use this for pipelines that genuinely operate in linear space
    /// (e.g. physically-based rendering or intermediate compute passes).
    /// Most compositor-facing surfaces (`Rgb10a2Unorm`, `Bgra8Unorm`, etc.)
    /// expect sRGB-encoded values — use [`to_array`](Self::to_array) instead.
    pub fn to_linear_array(&self) -> [f32; 4] {
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
        match h {
            Hue::Angle(angle) => angle.to_radians(),
            Hue::Number(deg) => deg.to_radians(), // bare number = degrees
        }
    }
}

/// Resolved edge values (margins, padding) in pixels
#[derive(Debug, Clone, Copy, Default)]
pub struct SideOffset {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl SideOffset {
    pub fn horizontal(&self) -> f32 {
        self.left + self.right
    }

    pub fn vertical(&self) -> f32 {
        self.top + self.bottom
    }

    pub fn zero() -> Self {
        Self {
            top: 0.0,
            right: 0.0,
            bottom: 0.0,
            left: 0.0,
        }
    }
}

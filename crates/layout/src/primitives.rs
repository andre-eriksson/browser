use css_style::{
    Color, Property,
    color::{
        ColorValue, FunctionColor, Hue, cielab::Cielab, named::NamedColor, oklab::Oklab,
        srgba::SRGBAColor,
    },
};

/// Rectangle representation for layout dimensions and positions
#[repr(C)] // Prevent struct reordering
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

/// RGBA color representation for rendering (values 0.0-1.0)
#[repr(C)] // Prevent struct reordering
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
            Self::new(1.0, 0.0, 0.0, 1.0) // Default to red for error indication
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
                let l = l.as_number();
                let a = a.as_number();
                let b = b.as_number();
                let l_ = l + 0.396_337_78 * a + 0.215_803_76 * b;
                let m_ = l - 0.105_561_346 * a - 0.063_854_17 * b;
                let s_ = l - 0.089_484_18 * a - 1.291_485_5 * b;

                let l_lin = l_ * l_ * l_;
                let m_lin = m_ * m_ * m_;
                let s_lin = s_ * s_ * s_;

                Color4f {
                    r: 4.076_741_7 * l_lin - 3.307_711_6 * m_lin + 0.230_969_94 * s_lin,
                    g: -1.268_438 * l_lin + 2.609_757_4 * m_lin - 0.341_319_38 * s_lin,
                    b: -0.0041960863 * l_lin - 0.703_419 * m_lin + 1.707_614_7 * s_lin,
                    a: alpha.as_fraction(),
                }
            }
            Oklab::Oklch(l, c, h, alpha) => {
                let h_rad = match h {
                    Hue::Angle(deg) => deg.to_degrees().to_radians(),
                    Hue::Number(num) => num * std::f32::consts::TAU,
                };
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
                let fy = (l.as_fraction() + 16.0) / 116.0;
                let fx = a.as_fraction() / 500.0 + fy;
                let fz = fy - b.as_fraction() / 200.0;

                let x_ref = if fx.powi(3) > 0.008856 {
                    fx.powi(3)
                } else {
                    (116.0 * fx - 16.0) / 903.3
                };
                let y_ref = if fy.powi(3) > 0.008856 {
                    fy.powi(3)
                } else {
                    (116.0 * fy - 16.0) / 903.3
                };
                let z_ref = if fz.powi(3) > 0.008856 {
                    fz.powi(3)
                } else {
                    (116.0 * fz - 16.0) / 903.3
                };

                // D65 white point
                let x_final = x_ref * 0.95047;
                let y_final = y_ref * 1.00000;
                let z_final = z_ref * 1.08883;

                // XYZ to sRGB (D65)
                let r = x_final * 3.2406 + y_final * -1.5372 + z_final * -0.4986;
                let g = x_final * -0.9689 + y_final * 1.8758 + z_final * 0.0415;
                let b = x_final * 0.0557 + y_final * -0.2040 + z_final * 1.0570;

                // sRGB gamma correction
                let r_corrected = if r > 0.0031308 {
                    1.055 * r.powf(1.0 / 2.4) - 0.055
                } else {
                    12.92 * r
                };
                let g_corrected = if g > 0.0031308 {
                    1.055 * g.powf(1.0 / 2.4) - 0.055
                } else {
                    12.92 * g
                };
                let b_corrected = if b > 0.0031308 {
                    1.055 * b.powf(1.0 / 2.4) - 0.055
                } else {
                    12.92 * b
                };

                Self::new(
                    r_corrected.clamp(0.0, 1.0),
                    g_corrected.clamp(0.0, 1.0),
                    b_corrected.clamp(0.0, 1.0),
                    alpha.as_fraction(),
                )
            }
            Cielab::Lch(l, c, h, alpha) => {
                let h_rad = match h {
                    Hue::Angle(deg) => deg.to_radians(),
                    Hue::Number(num) => num * std::f32::consts::TAU,
                };
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

                    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
                    let x = c * (1.0 - ((h_deg / 60.0) % 2.0 - 1.0).abs());
                    let m = l - c / 2.0;

                    let (r1, g1, b1) = if (0.0..60.0).contains(&h_deg) {
                        (c, x, 0.0)
                    } else if (60.0..120.0).contains(&h_deg) {
                        (x, c, 0.0)
                    } else if (120.0..180.0).contains(&h_deg) {
                        (0.0, c, x)
                    } else if (180.0..240.0).contains(&h_deg) {
                        (0.0, x, c)
                    } else if (240.0..300.0).contains(&h_deg) {
                        (x, 0.0, c)
                    } else {
                        (c, 0.0, x)
                    };

                    Self::new(r1 + m, g1 + m, b1 + m, a.as_fraction())
                }
                _ => Self::new(0.0, 0.0, 0.0, 1.0), // TODO: HWB
            },
            FunctionColor::Oklab(oklab) => Self::from_oklab(*oklab),
            FunctionColor::Cielab(cielab) => Self::from_cielab(*cielab),
        }
    }

    /// Returns color as [r, g, b, a] array for GPU upload
    pub fn to_array(&self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
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

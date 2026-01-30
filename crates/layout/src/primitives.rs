use css_style::types::color::{Color, FunctionColor, NamedColor, Oklab, SRGBAColor};

/// Rectangle representation for layout dimensions and positions
#[repr(C)] // Prevent struct reordering
#[derive(Debug, Default, Clone, Copy)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
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

    /// Converts a CSS color to Color4f
    pub fn from_css_color(color: &Color) -> Self {
        match color {
            Color::Named(named) => Self::from_named_color(named),
            Color::Hex([r, g, b]) => {
                Self::new(*r as f32 / 255.0, *g as f32 / 255.0, *b as f32 / 255.0, 1.0)
            }
            Color::Functional(func) => Self::from_functional_color(func),
            Color::CurrentColor => Self::new(0.0, 0.0, 0.0, 1.0), // Default to black
            Color::System(_) => Self::new(0.0, 0.0, 0.0, 1.0), // Default to black for system colors
        }
    }

    fn from_named_color(named: &NamedColor) -> Self {
        if matches!(named, NamedColor::Transparent) {
            return Self::new(0.0, 0.0, 0.0, 0.0);
        }

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

    pub fn from_oklab(oklab: Oklab) -> Self {
        match oklab {
            Oklab::Oklab(l, a, b) => {
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
                    a: 1.0,
                }
            }
            Oklab::Oklch(l, c, h) => {
                let h_rad = h.to_radians();
                let a = c * h_rad.cos();
                let b = c * h_rad.sin();
                Self::from_oklab(Oklab::Oklab(l, a, b))
            }
        }
    }

    fn from_functional_color(func: &FunctionColor) -> Self {
        match func {
            FunctionColor::SRGBA(srgba) => match srgba {
                SRGBAColor::RGB(r, g, b) => {
                    Self::new(*r as f32 / 255.0, *g as f32 / 255.0, *b as f32 / 255.0, 1.0)
                }
                SRGBAColor::RGBA(r, g, b, a) => {
                    Self::new(*r as f32 / 255.0, *g as f32 / 255.0, *b as f32 / 255.0, *a)
                }
                _ => Self::new(0.0, 0.0, 0.0, 1.0), // TODO: HSL/HSLA/HWB
            },
            FunctionColor::Oklab(oklab) => Self::from_oklab(*oklab),
            _ => Self::new(0.0, 0.0, 0.0, 1.0), // TODO: CIELAB
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

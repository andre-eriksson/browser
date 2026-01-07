use css_style::types::color::{Color, FunctionColor, NamedColor, SRGBAColor};

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
            _ => Self::new(0.0, 0.0, 0.0, 1.0), // TODO: CIELAB/Oklab
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

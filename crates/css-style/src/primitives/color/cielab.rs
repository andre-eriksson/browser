//! CIELAB color function with L, a, b components, e.g., lab(50, 20, -30) or lch(50, 20, 30)

use crate::color::{Alpha, ColorValue, Hue};

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

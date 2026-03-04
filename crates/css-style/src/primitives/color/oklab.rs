//! Oklab color function with L, a, b components, e.g., oklab(0.5, 0.1, -0.1) or oklch(0.5, 0.1, 30)

use crate::color::{Alpha, ColorValue, Hue};

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

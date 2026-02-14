//! Angle representation for CSS properties that accept angles, such as hue in HSL colors or rotation in transforms
//! It supports the following formats:
//! * "45deg" (degrees)
//! * "3.14rad" (radians)
//! * "100grad" (gradians)
//! * "0.5turn" (turns)
//! * "none" (treated as 0 degrees)
//! * A plain number (treated as degrees, e.g., "90" is treated as "90deg")
//!
//! The value is normalized to the appropriate range for each unit type:
//! * Degrees: "450deg" would be treated as "90deg".
//! * Radians: "7.28rad" would be treated as "0.28rad".
//! * Gradians: "450grad" would be treated as "50grad".
//! * Turns: "1.5turn" would be treated as "0.5turn".

/// Angle representation for CSS properties that accept angles, such as hue in HSL colors or rotation in transforms
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Angle {
    /// Degrees (e.g., "45deg")
    ///
    /// Note: The value is normalized to the [0, 360) range, so "450deg" would be treated as "90deg".
    Deg(f32),

    /// Radians (e.g., "3.14rad")
    ///
    /// Note: The value is normalized to the [0, 2π) range, so "7.28rad" would be treated as "0.28rad".
    Rad(f32),

    /// Gradians (e.g., "100grad")
    ///
    /// Note: The value is normalized to the [0, 400) range, so "450grad" would be treated as "50grad".
    Grad(f32),

    /// Turns (e.g., "0.5turn")
    ///
    /// Note: The value is normalized to the [0, 1) range, so "1.5turn" would be treated as "0.5turn".
    Turn(f32),
}

impl Angle {
    /// Convert the angle to degrees
    pub fn to_degrees(self) -> f32 {
        match self {
            Angle::Deg(v) => v,
            Angle::Rad(v) => v.to_degrees(),
            Angle::Grad(v) => v * 0.9,
            Angle::Turn(v) => v * 360.0,
        }
    }

    /// Convert the angle to radians
    pub fn to_radians(self) -> f32 {
        match self {
            Angle::Deg(v) => v.to_radians(),
            Angle::Rad(v) => v,
            Angle::Grad(v) => v * 0.9 * std::f32::consts::PI / 180.0,
            Angle::Turn(v) => v * 2.0 * std::f32::consts::PI,
        }
    }

    /// Convert f32 degrees to an Angle, normalizing it to the [0, 360) range
    pub fn from_degrees(deg: f32) -> Self {
        Angle::Deg((deg / 360.0).fract() * 360.0)
    }

    /// Convert f32 radians to an Angle, normalizing it to the [0, 2π) range
    pub fn from_radians(rad: f32) -> Self {
        Angle::Rad((rad / (2.0 * std::f32::consts::PI)).fract() * (2.0 * std::f32::consts::PI))
    }

    /// Convert f32 gradians to an Angle, normalizing it to the [0, 400) range
    pub fn from_gradians(grad: f32) -> Self {
        Angle::Grad((grad / 400.0).fract() * 400.0)
    }

    /// Convert f32 turns to an Angle, normalizing it to the [0, 1) range
    pub fn from_turns(turn: f32) -> Self {
        Angle::Turn((turn).fract())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_degrees() {
        let angle_deg = Angle::Deg(90.0);
        assert_eq!(angle_deg.to_degrees(), 90.0);

        let angle_rad = Angle::Rad(std::f32::consts::PI / 2.0);
        assert!((angle_rad.to_degrees() - 90.0).abs() < 1e-6);

        let angle_grad = Angle::Grad(100.0);
        assert_eq!(angle_grad.to_degrees(), 90.0);

        let angle_turn = Angle::Turn(0.25);
        assert_eq!(angle_turn.to_degrees(), 90.0);
    }
}

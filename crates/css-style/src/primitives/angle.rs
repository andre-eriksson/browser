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

use std::str::FromStr;

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
    fn strip_unit<'a>(s: &'a str, suffix: &str) -> Option<&'a str> {
        if s.len() >= suffix.len() && s[s.len() - suffix.len()..].eq_ignore_ascii_case(suffix) {
            Some(&s[..s.len() - suffix.len()])
        } else {
            None
        }
    }

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

impl FromStr for Angle {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        if s.eq_ignore_ascii_case("none") {
            Ok(Self::Deg(0.0))
        } else if let Some(num_str) = Self::strip_unit(s, "grad")
            && let Ok(num) = num_str.parse::<f32>()
        {
            Ok(Self::from_gradians(num))
        } else if let Some(num_str) = Self::strip_unit(s, "rad")
            && let Ok(num) = num_str.parse::<f32>()
        {
            Ok(Self::from_radians(num))
        } else if let Some(num_str) = Self::strip_unit(s, "deg")
            && let Ok(num) = num_str.parse::<f32>()
        {
            Ok(Self::from_degrees(num))
        } else if let Some(num_str) = Self::strip_unit(s, "turn")
            && let Ok(num) = num_str.parse::<f32>()
        {
            Ok(Self::from_turns(num))
        } else if let Ok(num) = s.trim().parse::<f32>() {
            Ok(Self::from_degrees(num))
        } else {
            Err(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_unit() {
        assert_eq!(Angle::strip_unit("45deg", "Deg"), Some("45"));
        assert_eq!(Angle::strip_unit("3.14RAD", "raD"), Some("3.14"));
        assert_eq!(Angle::strip_unit("100Grad", "gRAd"), Some("100"));
        assert_eq!(Angle::strip_unit("0.5TURN", "turn"), Some("0.5"));
        assert_eq!(Angle::strip_unit("invalid", "dEg"), None);
    }

    #[allow(clippy::approx_constant)]
    #[test]
    fn test_angle_parse() {
        assert_eq!("45deg".parse(), Ok(Angle::Deg(45.0)));
        assert_eq!("3.14rad".parse(), Ok(Angle::Rad(3.14)));
        assert_eq!("100grad".parse(), Ok(Angle::Grad(100.0)));
        assert_eq!("0.5turn".parse(), Ok(Angle::Turn(0.5)));
        assert!("invalid".parse::<Angle>().is_err());
    }

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

use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Angle {
    Deg(f32),
    Rad(f32),
    Grad(f32),
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

    pub fn to_degrees(self) -> f32 {
        match self {
            Angle::Deg(v) => v,
            Angle::Rad(v) => v.to_degrees(),
            Angle::Grad(v) => v * 0.9,
            Angle::Turn(v) => v * 360.0,
        }
    }

    pub fn to_radians(self) -> f32 {
        match self {
            Angle::Deg(v) => v.to_radians(),
            Angle::Rad(v) => v,
            Angle::Grad(v) => v * 0.9 * std::f32::consts::PI / 180.0,
            Angle::Turn(v) => v * 2.0 * std::f32::consts::PI,
        }
    }

    pub fn from_degrees(deg: f32) -> Self {
        Angle::Deg(deg)
    }

    pub fn from_radians(rad: f32) -> Self {
        Angle::Rad(rad)
    }

    pub fn from_gradians(grad: f32) -> Self {
        Angle::Grad(grad)
    }

    pub fn from_turns(turn: f32) -> Self {
        Angle::Turn(turn)
    }
}

impl FromStr for Angle {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if let Some(num_str) = Self::strip_unit(value, "grad")
            && let Ok(num) = num_str.parse::<f32>()
        {
            Ok(Self::Grad(num))
        } else if let Some(num_str) = Self::strip_unit(value, "rad")
            && let Ok(num) = num_str.parse::<f32>()
        {
            Ok(Self::Rad(num))
        } else if let Some(num_str) = Self::strip_unit(value, "deg")
            && let Ok(num) = num_str.parse::<f32>()
        {
            Ok(Self::Deg(num))
        } else if let Some(num_str) = Self::strip_unit(value, "turn")
            && let Ok(num) = num_str.parse::<f32>()
        {
            Ok(Self::Turn(num))
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

use crate::types::Parseable;

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

    pub fn to_degrees(&self) -> f32 {
        match self {
            Angle::Deg(v) => *v,
            Angle::Rad(v) => v.to_degrees(),
            Angle::Grad(v) => *v * 0.9,
            Angle::Turn(v) => *v * 360.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AnglePercentage {
    pub angle: Angle,
    pub percentage: f32,
}

impl Parseable for Angle {
    fn parse(value: &str) -> Option<Self> {
        if let Some(num_str) = Self::strip_unit(value, "grad")
            && let Ok(num) = num_str.parse::<f32>()
        {
            return Some(Self::Grad(num));
        } else if let Some(num_str) = Self::strip_unit(value, "rad")
            && let Ok(num) = num_str.parse::<f32>()
        {
            return Some(Self::Rad(num));
        } else if let Some(num_str) = Self::strip_unit(value, "deg")
            && let Ok(num) = num_str.parse::<f32>()
        {
            return Some(Self::Deg(num));
        } else if let Some(num_str) = Self::strip_unit(value, "turn")
            && let Ok(num) = num_str.parse::<f32>()
        {
            return Some(Self::Turn(num));
        }
        None
    }
}

impl Parseable for AnglePercentage {
    fn parse(value: &str) -> Option<Self> {
        let parts: Vec<&str> = value.split_whitespace().collect();
        if parts.len() != 2 {
            return None;
        }

        let angle = Angle::parse(parts[0])?;
        let percentage_str = parts[1].strip_suffix('%')?;
        let percentage = percentage_str.parse::<f32>().ok()?;

        Some(Self { angle, percentage })
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
        assert_eq!(Angle::parse("45deg"), Some(Angle::Deg(45.0)));
        assert_eq!(Angle::parse("3.14rad"), Some(Angle::Rad(3.14)));
        assert_eq!(Angle::parse("100grad"), Some(Angle::Grad(100.0)));
        assert_eq!(Angle::parse("0.5turn"), Some(Angle::Turn(0.5)));
        assert_eq!(Angle::parse("invalid"), None);
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

    #[test]
    fn test_angle_percentage_parse() {
        let angle_percentage = AnglePercentage::parse("45deg 50%");
        assert_eq!(
            angle_percentage,
            Some(AnglePercentage {
                angle: Angle::Deg(45.0),
                percentage: 50.0,
            })
        );
        let invalid = AnglePercentage::parse("45deg");
        assert_eq!(invalid, None);
    }
}

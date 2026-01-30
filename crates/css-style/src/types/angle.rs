#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Angle {
    Deg(f32),
    Rad(f32),
    Grad(f32),
    Turn(f32),
}

impl Angle {
    pub fn parse(value: &str) -> Option<Self> {
        if let Some(num_str) = value.strip_suffix("deg")
            && let Ok(num) = num_str.parse::<f32>()
        {
            return Some(Angle::Deg(num));
        } else if let Some(num_str) = value.strip_suffix("rad")
            && !value.ends_with("grad")
            && let Ok(num) = num_str.parse::<f32>()
        {
            return Some(Angle::Rad(num));
        } else if let Some(num_str) = value.strip_suffix("grad")
            && let Ok(num) = num_str.parse::<f32>()
        {
            return Some(Angle::Grad(num));
        } else if let Some(num_str) = value.strip_suffix("turn")
            && let Ok(num) = num_str.parse::<f32>()
        {
            return Some(Angle::Turn(num));
        }
        None
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

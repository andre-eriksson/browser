use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Percentage {
    value: f32,
}

impl Percentage {
    pub fn new(value: f32) -> Self {
        Self { value }
    }

    pub fn value(&self) -> f32 {
        self.value
    }

    pub fn to_px(self, parent_px: f32) -> f32 {
        parent_px * self.value / 100.0
    }
}

impl FromStr for Percentage {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(stripped) = s.strip_suffix('%')
            && let Ok(num) = stripped.trim().parse::<f32>()
        {
            Ok(Self { value: num })
        } else {
            Err(format!("Invalid percentage value: {}", s))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_percentage() {
        assert_eq!("50%".parse(), Ok(Percentage::new(50.0)));
        assert_eq!("100%".parse(), Ok(Percentage::new(100.0)));
        assert_eq!("75.5%".parse(), Ok(Percentage::new(75.5)));
        assert!("invalid".parse::<Percentage>().is_err());
        assert!("50".parse::<Percentage>().is_err());
    }
}

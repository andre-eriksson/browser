/// A resolver for CSS property values
pub struct Unit;

impl Unit {
    pub fn resolve_percentage(value: &str) -> Option<f32> {
        if let Some(stripped) = value.strip_suffix('%')
            && let Ok(num) = stripped.trim().parse::<f32>()
        {
            return Some(num);
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_percentage() {
        assert_eq!(Unit::resolve_percentage("50%"), Some(50.0));
        assert_eq!(Unit::resolve_percentage("100%"), Some(100.0));
        assert_eq!(Unit::resolve_percentage("75.5%"), Some(75.5));
        assert_eq!(Unit::resolve_percentage("invalid"), None);
        assert_eq!(Unit::resolve_percentage("50"), None);
    }
}

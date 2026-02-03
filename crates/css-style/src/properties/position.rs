use strum::EnumString;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, EnumString)]
#[strum(serialize_all = "kebab_case", ascii_case_insensitive, parse_err_ty = String, parse_err_fn = String::from)]
pub enum Position {
    Static,
    Relative,
    Absolute,
    Fixed,
    Sticky,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_position() {
        assert_eq!("static".parse(), Ok(Position::Static));
        assert_eq!("relative".parse(), Ok(Position::Relative));
        assert_eq!("absolute".parse(), Ok(Position::Absolute));
        assert_eq!("fixed".parse(), Ok(Position::Fixed));
        assert_eq!("sticky".parse(), Ok(Position::Sticky));
        assert!("unknown".parse::<Position>().is_err());
    }
}

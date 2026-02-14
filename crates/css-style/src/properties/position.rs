use css_cssom::{ComponentValue, CssTokenKind};
use strum::EnumString;

#[derive(Debug, Clone, Default, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, EnumString)]
#[strum(serialize_all = "kebab_case", ascii_case_insensitive, parse_err_ty = String, parse_err_fn = String::from)]
pub enum Position {
    #[default]
    Static,
    Relative,
    Absolute,
    Fixed,
    Sticky,
}

impl TryFrom<&[ComponentValue]> for Position {
    type Error = String;

    fn try_from(value: &[ComponentValue]) -> Result<Self, Self::Error> {
        for cv in value {
            match cv {
                ComponentValue::Token(token) => {
                    if let CssTokenKind::Ident(ident) = &token.kind
                        && let Ok(pos) = ident.parse()
                    {
                        return Ok(pos);
                    }
                }
                _ => continue,
            }
        }
        Err("No valid position value found".to_string())
    }
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

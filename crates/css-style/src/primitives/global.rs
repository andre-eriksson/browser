use strum::EnumString;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, EnumString)]
#[strum(serialize_all = "kebab_case", ascii_case_insensitive, parse_err_ty = String, parse_err_fn = String::from)]
pub enum Global {
    Inherit,
    Initial,
    Revert,
    RevertLayer,
    Unset,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_global() {
        assert_eq!(Global::try_from("inherit"), Ok(Global::Inherit));
        assert_eq!(Global::try_from("inHErit"), Ok(Global::Inherit));
        assert_eq!(Global::try_from("initial"), Ok(Global::Initial));
        assert_eq!(Global::try_from("revert"), Ok(Global::Revert));
        assert_eq!(Global::try_from("revert-layer"), Ok(Global::RevertLayer));
        assert_eq!(Global::try_from("unset"), Ok(Global::Unset));
        assert!(Global::try_from("unknown").is_err());
    }
}

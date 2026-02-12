//! Global CSS primitives, used by most properties.

use strum::EnumString;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, EnumString)]
#[strum(serialize_all = "kebab_case", ascii_case_insensitive, parse_err_ty = String, parse_err_fn = String::from)]
pub enum Global {
    /// The property takes the same specified value as the property for the element's parent, if there is one, or the initial value of the property if there is no parent.
    Inherit,

    /// The property resets to its initial value, which is defined in the CSS specification. This is the default behavior for properties that are not inherited by default.
    Initial,

    /// The property resets to the value established by the user-agent stylesheet (or by user styles, if any exist).
    /// It behaves as initial if the property is inherited by default, and inherit if not.
    Revert,

    /// The property resets to the value established by the user-agent stylesheet (or by user styles, if any exist). It behaves as either inherit or
    /// initial depending on whether the property is inherited by default.
    RevertLayer,

    /// The property resets to its inherited value if it inherits from its parent, and to its initial value if not. It behaves as either inherit or initial depending on
    /// whether the property is inherited by default.
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

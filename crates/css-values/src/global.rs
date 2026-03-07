use css_cssom::{ComponentValue, ComponentValueStream, CssTokenKind};
use strum::EnumString;

use crate::CSSParsable;

/// Represents the global values that can be used in CSS properties. These values have special meanings and affect how the property is computed and inherited.
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

impl CSSParsable for Global {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, String> {
        stream.skip_whitespace();

        if let Some(ComponentValue::Token(token)) = stream.peek()
            && let CssTokenKind::Ident(ident) = &token.kind
            && let Ok(global) = ident.parse()
        {
            stream.next_cv();
            return Ok(global);
        }

        Err("Expected a global value (inherit, initial, revert, revert-layer, unset)".to_string())
    }
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

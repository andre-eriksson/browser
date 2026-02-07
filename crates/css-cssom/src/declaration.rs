use css_parser::{ComponentValue, CssToken, CssTokenKind, Declaration, Property};
use serde::{Deserialize, Serialize};

use crate::string::component_value_to_string;

/// A CSS declaration (property: value)
///
/// <https://www.w3.org/TR/css-syntax-3/#declaration>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CSSDeclaration {
    /// The property name
    pub property: Property,

    /// The value as a string
    pub value: String,

    /// Whether this declaration has !important
    pub important: bool,

    /// The original component values (for reference)
    pub original_values: Vec<ComponentValue>,
}

impl CSSDeclaration {
    /// Create a new declaration
    pub fn new(property: Property, value: String, important: bool) -> Self {
        CSSDeclaration {
            property,
            value,
            important,
            original_values: Vec::new(),
        }
    }

    /// Create a declaration from component values
    pub fn from_values(property: Property, values: Vec<ComponentValue>) -> Self {
        let mut value_parts: Vec<String> = Vec::new();
        let mut important = false;

        let mut check_values = values.clone();

        while let Some(ComponentValue::Token(token)) = check_values.last() {
            if matches!(
                token,
                CssToken {
                    kind: CssTokenKind::Whitespace,
                    ..
                }
            ) {
                check_values.pop();
            } else {
                break;
            }
        }

        if check_values.len() >= 2 {
            let len = check_values.len();
            let last = &check_values[len - 1];
            let second_last = &check_values[len - 2];

            use css_parser::CssTokenKind;
            if let (
                ComponentValue::Token(CssToken {
                    kind: CssTokenKind::Delim('!'),
                    ..
                }),
                ComponentValue::Token(CssToken {
                    kind: CssTokenKind::Ident(ident),
                    ..
                }),
            ) = (second_last, last)
                && ident.eq_ignore_ascii_case("important")
            {
                important = true;
                check_values.pop();
                check_values.pop();

                // Remove whitespace before !important
                while matches!(
                    check_values.last(),
                    Some(ComponentValue::Token(CssToken {
                        kind: CssTokenKind::Whitespace,
                        ..
                    }))
                ) {
                    check_values.pop();
                }
            }
        }

        for cv in &check_values {
            value_parts.push(component_value_to_string(cv));
        }

        let value = value_parts.join("").trim().to_string();

        CSSDeclaration {
            property,
            value,
            important,
            original_values: values,
        }
    }

    pub fn from_parser_declaration(declaration: Declaration) -> Self {
        let mut value_parts: Vec<String> = Vec::new();

        for cv in &declaration.value {
            value_parts.push(component_value_to_string(cv));
        }

        let value = value_parts.join("").trim().to_string();

        CSSDeclaration {
            property: declaration.property,
            value,
            important: declaration.important,
            original_values: declaration.value,
        }
    }

    /// Get the property name
    pub fn property(&self) -> &Property {
        &self.property
    }

    /// Get the property value
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Check if this declaration is !important
    pub fn is_important(&self) -> bool {
        self.important
    }

    /// Set the important flag
    pub fn set_important(&mut self, important: bool) {
        self.important = important;
    }

    /// Serialize this declaration to CSS text
    pub fn to_css_string(&self) -> String {
        if self.important {
            format!("{:?}: {} !important", self.property, self.value)
        } else {
            format!("{:?}: {}", self.property, self.value)
        }
    }
}

use css_cssom::{ComponentValue, CssToken, CssTokenKind, Function, Property, SimpleBlock};

use crate::tree::PropertyRegistry;

/// Resolves CSS variables in a given value by replacing any `var()` functions with their corresponding values from the provided variables list.
/// This function recursively resolves nested `var()` functions and handles fallback values if a variable is not found.
///
/// Returns `None` if any `var()` reference cannot be resolved (variable not found and no fallback),
/// which signals that the entire property value is invalid at computed-value time per the CSS spec.
pub fn resolve_css_variables(
    variables: &[(Property, Vec<ComponentValue>)],
    property_registry: &PropertyRegistry,
    value: &[ComponentValue],
) -> Option<Vec<ComponentValue>> {
    let mut output: Vec<ComponentValue> = Vec::new();

    for (i, cv) in value.iter().enumerate() {
        match cv {
            ComponentValue::Function(func) if func.name.eq_ignore_ascii_case("var") => {
                let resolved = resolve_var_function(variables, property_registry, func)?;

                if !resolved.is_empty() {
                    let needs_leading_whitespace = !output.is_empty()
                        && !output.last().unwrap().is_whitespace()
                        && !resolved.first().unwrap().is_whitespace()
                        && i > 0
                        && value[i - 1].is_whitespace();

                    let needs_trailing_whitespace = !resolved.is_empty()
                        && !resolved.last().unwrap().is_whitespace()
                        && i + 1 < value.len()
                        && !&value[i + 1].is_whitespace();

                    if needs_leading_whitespace {
                        output.push(ComponentValue::Token(CssToken {
                            kind: CssTokenKind::Whitespace,
                            position: None,
                        }));
                    }

                    output.extend(resolved);

                    if needs_trailing_whitespace {
                        output.push(ComponentValue::Token(CssToken {
                            kind: CssTokenKind::Whitespace,
                            position: None,
                        }));
                    }
                }
            }
            ComponentValue::Function(func) => {
                let resolved_inner = resolve_css_variables(variables, property_registry, &func.value)?;
                output.push(ComponentValue::Function(Function {
                    name: func.name.clone(),
                    value: resolved_inner,
                }));
            }
            ComponentValue::SimpleBlock(block) => {
                let resolved_inner = resolve_css_variables(variables, property_registry, &block.value)?;
                output.push(ComponentValue::SimpleBlock(SimpleBlock {
                    associated_token: block.associated_token,
                    value: resolved_inner,
                }));
            }
            _ => {
                output.push(cv.clone());
            }
        }
    }

    Some(output)
}

/// Resolves a `var()` function by extracting the variable name and fallback values, then attempting to find the variable in the provided list of variables.
/// If the variable is found, its value is returned. If not, the fallback values are resolved and returned.
/// Returns `None` if the variable is not found and there are no fallback values.
fn resolve_var_function(
    variables: &[(Property, Vec<ComponentValue>)],
    property_registry: &PropertyRegistry,
    func: &Function,
) -> Option<Vec<ComponentValue>> {
    let mut var_name = String::new();
    let mut fallback_values = Vec::new();
    let mut found_comma = false;

    if func.value.is_empty() {
        return Some(Vec::new());
    }

    for cv in func.value.iter() {
        match cv {
            ComponentValue::Token(token) if matches!(token.kind, CssTokenKind::Comma) => {
                found_comma = true;
            }
            ComponentValue::Token(token) if token.kind == CssTokenKind::Whitespace => {
                if !found_comma {
                    continue;
                }
                fallback_values.push(cv.clone());
            }
            _ => {
                if !found_comma {
                    var_name.push_str(&cv.to_string());
                } else {
                    fallback_values.push(cv.clone());
                }
            }
        }
    }

    let var_name = var_name.trim();

    if let Some(resolved) = try_resolve_variable(variables, property_registry, var_name) {
        return Some(resolved);
    }

    if !fallback_values.is_empty() {
        return resolve_css_variables(variables, property_registry, &fallback_values);
    }

    None
}

/// Attempts to resolve a CSS variable by searching for its name in the provided list of variables. If the variable is found, its value is resolved and returned.
/// If the variable is not found or if its value is empty, `None` is returned.
///
/// For registered custom properties (@property), this function also:
/// - Validates the resolved value against the property's syntax
/// - Falls back to initial-value if the value is invalid
/// - Returns initial-value if no value is set and the property has one
fn try_resolve_variable(
    variables: &[(Property, Vec<ComponentValue>)],
    property_registry: &PropertyRegistry,
    var_name: &str,
) -> Option<Vec<ComponentValue>> {
    let descriptor = property_registry.descriptors.get(var_name);

    for (property, vals) in variables {
        if let Some(custom) = property.as_custom()
            && custom != var_name
        {
            continue;
        }

        if vals.is_empty() {
            if let Some(desc) = descriptor {
                return desc.initial_value.clone();
            }
            return None;
        }

        let resolved = resolve_css_variables(variables, property_registry, vals)?;

        if resolved.is_empty() {
            if let Some(desc) = descriptor {
                return desc.initial_value.clone();
            }
            return None;
        }

        if let Some(desc) = descriptor
            && !desc.syntax.validate(&resolved)
        {
            return desc.initial_value.clone();
        }

        return Some(resolved);
    }

    if let Some(desc) = descriptor {
        return desc.initial_value.clone();
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use css_cssom::{CssToken, CssTokenKind, NumericValue};
    use css_values::property::{PropertyDescriptor, PropertySyntax, SyntaxComponent};

    fn make_token(kind: CssTokenKind) -> ComponentValue {
        ComponentValue::Token(CssToken {
            kind,
            position: None,
        })
    }

    fn make_dimension(value: f64, unit: &str) -> ComponentValue {
        make_token(CssTokenKind::Dimension {
            value: NumericValue::Number(value),
            unit: unit.to_string(),
        })
    }

    fn make_ident(name: &str) -> ComponentValue {
        make_token(CssTokenKind::Ident(name.to_string()))
    }

    #[test]
    fn test_resolve_unregistered_variable() {
        let variables = vec![(Property::Custom("--color".to_string()), vec![make_ident("red")])];
        let registry = PropertyRegistry::default();

        let value = vec![ComponentValue::Function(Function {
            name: "var".to_string(),
            value: vec![make_ident("--color")],
        })];

        let result = resolve_css_variables(&variables, &registry, &value);
        assert!(result.is_some());
        let resolved = result.unwrap();
        assert_eq!(resolved.len(), 1);
    }

    #[test]
    fn test_registered_property_uses_initial_value_when_not_set() {
        let variables: Vec<(Property, Vec<ComponentValue>)> = vec![];
        let mut registry = PropertyRegistry::default();

        registry.descriptors.insert(
            "--my-length".to_string(),
            PropertyDescriptor {
                name: "--my-length".to_string(),
                syntax: PropertySyntax::Typed(vec![SyntaxComponent::Length]),
                inherits: false,
                initial_value: Some(vec![make_dimension(10.0, "px")]),
            },
        );

        let result = try_resolve_variable(&variables, &registry, "--my-length");
        assert!(result.is_some());
        let resolved = result.unwrap();
        assert_eq!(resolved.len(), 1);
    }

    #[test]
    fn test_registered_property_validates_value() {
        let variables = vec![(Property::Custom("--my-length".to_string()), vec![make_dimension(20.0, "px")])];
        let mut registry = PropertyRegistry::default();

        registry.descriptors.insert(
            "--my-length".to_string(),
            PropertyDescriptor {
                name: "--my-length".to_string(),
                syntax: PropertySyntax::Typed(vec![SyntaxComponent::Length]),
                inherits: false,
                initial_value: Some(vec![make_dimension(10.0, "px")]),
            },
        );

        let result = try_resolve_variable(&variables, &registry, "--my-length");
        assert!(result.is_some());
        let resolved = result.unwrap();
        assert_eq!(resolved.len(), 1);
    }

    #[test]
    fn test_registered_property_falls_back_on_invalid_value() {
        let variables = vec![(Property::Custom("--my-length".to_string()), vec![make_ident("red")])];
        let mut registry = PropertyRegistry::default();

        registry.descriptors.insert(
            "--my-length".to_string(),
            PropertyDescriptor {
                name: "--my-length".to_string(),
                syntax: PropertySyntax::Typed(vec![SyntaxComponent::Length]),
                inherits: false,
                initial_value: Some(vec![make_dimension(10.0, "px")]),
            },
        );

        let result = try_resolve_variable(&variables, &registry, "--my-length");
        assert!(result.is_some());
    }

    #[test]
    fn test_universal_syntax_accepts_any_value() {
        let variables = vec![(Property::Custom("--anything".to_string()), vec![make_ident("whatever")])];
        let mut registry = PropertyRegistry::default();

        registry.descriptors.insert(
            "--anything".to_string(),
            PropertyDescriptor {
                name: "--anything".to_string(),
                syntax: PropertySyntax::Universal,
                inherits: true,
                initial_value: None,
            },
        );

        let result = try_resolve_variable(&variables, &registry, "--anything");
        assert!(result.is_some());
    }
}

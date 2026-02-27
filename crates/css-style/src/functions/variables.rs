use css_cssom::{ComponentValue, CssToken, CssTokenKind, Function, Property, SimpleBlock};

/// Resolves CSS variables in a given value by replacing any `var()` functions with their corresponding values from the provided variables list.
/// This function recursively resolves nested `var()` functions and handles fallback values if a variable is not found.
pub(crate) fn resolve_css_variables(
    variables: &[(Property, Vec<ComponentValue>)],
    value: &[ComponentValue],
) -> Vec<ComponentValue> {
    let mut output: Vec<ComponentValue> = Vec::new();

    for (i, cv) in value.iter().enumerate() {
        match cv {
            ComponentValue::Function(func) if func.name.eq_ignore_ascii_case("var") => {
                let resolved = resolve_var_function(variables, func);

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
                let resolved_inner = resolve_css_variables(variables, &func.value);
                output.push(ComponentValue::Function(Function {
                    name: func.name.clone(),
                    value: resolved_inner,
                }));
            }
            ComponentValue::SimpleBlock(block) => {
                let resolved_inner = resolve_css_variables(variables, &block.value);
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

    output
}

/// Resolves a `var()` function by extracting the variable name and fallback values, then attempting to find the variable in the provided list of variables.
/// If the variable is found, its value is returned. If not, the fallback values are resolved and returned. If there are no fallback values,
/// the original `var()` function is returned as a component value.
fn resolve_var_function(variables: &[(Property, Vec<ComponentValue>)], func: &Function) -> Vec<ComponentValue> {
    let mut var_name = String::new();
    let mut fallback_values = Vec::new();
    let mut found_comma = false;

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
                    var_name.push_str(&cv.to_css_string());
                } else {
                    fallback_values.push(cv.clone());
                }
            }
        }
    }

    let var_name = var_name.trim();

    if let Some(resolved) = try_resolve_variable(variables, var_name) {
        return resolved;
    }

    if !fallback_values.is_empty() {
        return resolve_css_variables(variables, &fallback_values);
    }

    vec![ComponentValue::Function(func.clone())]
}

/// Attempts to resolve a CSS variable by searching for its name in the provided list of variables. If the variable is found, its value is resolved and returned.
/// If the variable is not found or if its value is empty, `None` is returned.
fn try_resolve_variable(variables: &[(Property, Vec<ComponentValue>)], var_name: &str) -> Option<Vec<ComponentValue>> {
    for (name, vals) in variables {
        if name.to_string() != var_name {
            continue;
        }

        if vals.is_empty() {
            return None;
        }

        let resolved = resolve_css_variables(variables, vals);

        if resolved.is_empty() {
            return None;
        }

        return Some(resolved);
    }

    None
}

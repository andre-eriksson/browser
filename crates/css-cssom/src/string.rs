use css_parser::{AssociatedToken, ComponentValue};

/// Convert a prelude (list of component values) to a selector text string
pub fn prelude_to_selector_text(prelude: &[ComponentValue]) -> String {
    let mut result = String::new();
    for cv in prelude {
        result.push_str(&component_value_to_string(cv));
    }
    result.trim().to_string()
}

/// Convert a prelude to a general string representation
pub fn prelude_to_string(prelude: &[ComponentValue]) -> String {
    prelude
        .iter()
        .map(|cv| cv.to_css_string())
        .collect::<String>()
        .trim()
        .to_string()
}

/// Convert a component value to its string representation
pub fn component_value_to_string(cv: &ComponentValue) -> String {
    match cv {
        ComponentValue::Token(token) => token.kind.to_string(),
        ComponentValue::Function(f) => {
            let mut s = format!("{}(", f.name);
            for v in &f.value {
                s.push_str(&component_value_to_string(v));
            }
            s.push(')');
            s
        }
        ComponentValue::SimpleBlock(b) => {
            let (open, close) = match b.associated_token {
                AssociatedToken::CurlyBracket => ('{', '}'),
                AssociatedToken::SquareBracket => ('[', ']'),
                AssociatedToken::Parenthesis => ('(', ')'),
            };
            let mut s = String::new();
            s.push(open);
            for v in &b.value {
                s.push_str(&component_value_to_string(v));
            }
            s.push(close);
            s
        }
    }
}

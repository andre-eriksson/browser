use css_cssom::{CssToken, CssTokenKind};

use crate::{matching::AttributeOperator, selector::AttributeSelector};

#[derive(Debug, Clone, PartialEq, Default)]
pub enum CaseSensitivity {
    /// 's' or 'S' flag (default)
    #[default]
    CaseSensitive,

    /// 'i' or 'I' flag
    CaseInsensitive,
}

/// Parse an attribute selector from a list of CSS tokens
///
/// # Arguments
/// * `tokens` - A vector of CSS tokens representing the attribute selector
///
/// # Returns
/// * `Option<AttributeSelector>` - An optional AttributeSelector if parsing was successful
pub(crate) fn parse_attribute_selector(tokens: Vec<CssToken>) -> Option<AttributeSelector> {
    let mut attribute_selector = AttributeSelector::default();
    let mut temp_buffer = String::new();

    for token in tokens {
        if attribute_selector.attribute.is_none() {
            if let CssTokenKind::Ident(_) = token.kind {
                attribute_selector.attribute = Some(token);
                continue;
            } else {
                return None;
            }
        }

        if attribute_selector.operator.is_none() {
            if !temp_buffer.is_empty() {
                match temp_buffer.as_str() {
                    "~" => attribute_selector.operator = Some(AttributeOperator::Includes),
                    "|" => attribute_selector.operator = Some(AttributeOperator::DashMatch),
                    "^" => attribute_selector.operator = Some(AttributeOperator::PrefixMatch),
                    "$" => attribute_selector.operator = Some(AttributeOperator::SuffixMatch),
                    "*" => attribute_selector.operator = Some(AttributeOperator::SubstringMatch),
                    _ => return None,
                }
                temp_buffer.clear();
                continue;
            }

            match &token.kind {
                CssTokenKind::Delim('=') => {
                    attribute_selector.operator = Some(AttributeOperator::Equals);
                    continue;
                }
                CssTokenKind::Delim('~') => {
                    temp_buffer.push('~');
                    continue;
                }
                CssTokenKind::Delim('|') => {
                    temp_buffer.push('|');
                    continue;
                }
                CssTokenKind::Delim('^') => {
                    temp_buffer.push('^');
                    continue;
                }
                CssTokenKind::Delim('$') => {
                    temp_buffer.push('$');
                    continue;
                }
                CssTokenKind::Delim('*') => {
                    temp_buffer.push('*');
                    continue;
                }
                _ => {}
            }
        }

        if attribute_selector.value.is_none() {
            match &token.kind {
                CssTokenKind::Ident(_) | CssTokenKind::String(_) => {
                    attribute_selector.value = Some(token);
                    continue;
                }
                _ => {}
            }
        }

        if attribute_selector.case.is_none()
            && let CssTokenKind::Ident(ident) = &token.kind
        {
            match ident.as_str() {
                "i" | "I" => attribute_selector.case = Some(CaseSensitivity::CaseInsensitive),
                _ => attribute_selector.case = Some(CaseSensitivity::default()),
            }
        }
    }

    Some(attribute_selector)
}

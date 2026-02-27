use css_cssom::{ComponentValue, CssToken, CssTokenKind};

use crate::{matching::AttributeOperator, selector::AttributeSelector};

#[derive(Debug, Clone, PartialEq, Default)]
pub enum CaseSensitivity {
    /// 's' or 'S' flag (default)
    #[default]
    CaseSensitive,

    /// 'i' or 'I' flag
    CaseInsensitive,
}

pub(crate) fn parse_attribute_selectors_components(components: &[ComponentValue]) -> Option<AttributeSelector> {
    let mut tokens: Vec<CssToken> = Vec::with_capacity(6);

    for cv in components {
        if !cv.is_token() {
            continue;
        }

        tokens.push(cv.as_token().unwrap().clone());
    }

    parse_attribute_selector(tokens.as_slice())
}

/// Parse an attribute selector from a list of CSS tokens
///
/// # Arguments
/// * `tokens` - A vector of CSS tokens representing the attribute selector
///
/// # Returns
/// * `Option<AttributeSelector>` - An optional AttributeSelector if parsing was successful
pub(crate) fn parse_attribute_selector(tokens: &[CssToken]) -> Option<AttributeSelector> {
    let mut attribute_selector = AttributeSelector::default();
    let mut temp_buffer_ch = '0';

    for token in tokens {
        if attribute_selector.attribute.is_none()
            && let CssTokenKind::Ident(_) = token.kind
        {
            attribute_selector.attribute = Some(token.clone());

            continue;
        }

        if attribute_selector.operator.is_none() {
            if !temp_buffer_ch.eq(&'0') {
                match temp_buffer_ch {
                    '~' => attribute_selector.operator = Some(AttributeOperator::Includes),
                    '|' => attribute_selector.operator = Some(AttributeOperator::DashMatch),
                    '^' => attribute_selector.operator = Some(AttributeOperator::PrefixMatch),
                    '$' => attribute_selector.operator = Some(AttributeOperator::SuffixMatch),
                    '*' => attribute_selector.operator = Some(AttributeOperator::SubstringMatch),
                    _ => return None,
                }
                temp_buffer_ch = '0';
                continue;
            }

            match &token.kind {
                CssTokenKind::Delim('=') => {
                    attribute_selector.operator = Some(AttributeOperator::Equals);
                }
                CssTokenKind::Delim('~') => temp_buffer_ch = '~',
                CssTokenKind::Delim('|') => temp_buffer_ch = '|',
                CssTokenKind::Delim('^') => temp_buffer_ch = '^',
                CssTokenKind::Delim('$') => temp_buffer_ch = '$',
                CssTokenKind::Delim('*') => temp_buffer_ch = '*',
                _ => {}
            }

            continue;
        }

        if attribute_selector.value.is_none() {
            match &token.kind {
                CssTokenKind::Ident(_) | CssTokenKind::String(_) => {
                    attribute_selector.value = Some(token.clone());

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

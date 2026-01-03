use css_tokenizer::{CssToken, CssTokenKind};

use crate::{
    ComponentValue, CssParser,
    consumers::{block::consume_simple_block, function::consume_function},
};

/// Consume a component value
///
/// <https://www.w3.org/TR/css-syntax-3/#consume-a-component-value>
pub(crate) fn consume_component_value(css_parser: &mut CssParser) -> ComponentValue {
    let next_token = css_parser.peek();

    let token_kind = match next_token {
        Some(token) => &token.kind,
        None => &CssTokenKind::Eof,
    };

    match token_kind {
        CssTokenKind::OpenCurly | CssTokenKind::OpenSquare | CssTokenKind::OpenParen => {
            ComponentValue::SimpleBlock(consume_simple_block(css_parser))
        }
        CssTokenKind::Function(_) => ComponentValue::Function(consume_function(css_parser)),
        _ => {
            let consumed_token = css_parser.consume();

            match consumed_token {
                Some(token) => ComponentValue::Token(CssToken {
                    kind: token.kind,
                    position: token.position,
                }),
                None => ComponentValue::Token(css_parser.consume().unwrap_or(CssToken {
                    kind: CssTokenKind::Eof,
                    position: None,
                })),
            }
        }
    }
}

use css_tokenizer::{CssToken, CssTokenKind};

use crate::{
    ComponentValue, CssParser,
    consumers::{block::consume_simple_block, function::consume_function},
};

/// Consume a component value
///
/// <https://www.w3.org/TR/css-syntax-3/#consume-a-component-value>
pub fn consume_component_value(css_parser: &mut CssParser) -> ComponentValue {
    let next_token = css_parser.peek();

    let token_kind = next_token.map_or(&CssTokenKind::Eof, |token| &token.kind);

    match token_kind {
        CssTokenKind::OpenCurly | CssTokenKind::OpenSquare | CssTokenKind::OpenParen => {
            ComponentValue::SimpleBlock(consume_simple_block(css_parser))
        }
        CssTokenKind::Function(_) => ComponentValue::Function(consume_function(css_parser)),
        CssTokenKind::Eof => ComponentValue::Token(CssToken {
            kind: CssTokenKind::Eof,
            position: None,
        }),
        _ => {
            let consumed_token = css_parser.consume();

            consumed_token.map_or_else(
                || {
                    ComponentValue::Token(css_parser.consume().unwrap_or(CssToken {
                        kind: CssTokenKind::Eof,
                        position: None,
                    }))
                },
                ComponentValue::Token,
            )
        }
    }
}

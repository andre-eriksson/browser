use css_tokenizer::CssToken;

use crate::{
    ComponentValue, CssParser,
    consumers::{block::consume_simple_block, function::consume_function},
};

/// Consume a component value
///
/// <https://www.w3.org/TR/css-syntax-3/#consume-a-component-value>
pub(crate) fn consume_component_value(css_parser: &mut CssParser) -> ComponentValue {
    match css_parser.peek() {
        Some(CssToken::OpenCurly) | Some(CssToken::OpenSquare) | Some(CssToken::OpenParen) => {
            ComponentValue::SimpleBlock(consume_simple_block(css_parser))
        }
        Some(CssToken::Function(_)) => ComponentValue::Function(consume_function(css_parser)),
        _ => ComponentValue::Token(css_parser.consume().unwrap_or(CssToken::Eof)),
    }
}

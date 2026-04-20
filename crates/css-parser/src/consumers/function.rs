use crate::errors::CssParsingError;
use css_tokenizer::CssTokenKind;

use crate::{CssParser, Function, consumers::component::consume_component_value};

/// Consume a function
///
/// <https://www.w3.org/TR/css-syntax-3/#consume-a-function>
pub fn consume_function(css_parser: &mut CssParser) -> Function {
    let (name, pos) = match css_parser.consume() {
        Some(token) => {
            let pos = token.position.unwrap_or_default();

            match token.kind {
                CssTokenKind::Function(name) => (name, pos),
                _ => unreachable!("consume_function should only be called when the current token is a function token"),
            }
        }
        _ => unreachable!("consume_function should only be called when there is a current token"),
    };

    let mut function = Function::new(name);

    loop {
        if let Some(token) = css_parser.peek() {
            match &token.kind {
                CssTokenKind::Eof => {
                    css_parser.record_error(CssParsingError::EofInFunction(pos));
                    break;
                }
                CssTokenKind::CloseParen => {
                    css_parser.consume();
                    break;
                }
                CssTokenKind::CloseCurly | CssTokenKind::CloseSquare => {
                    css_parser.record_error(CssParsingError::IncompleteFunction(pos));
                    break;
                }
                _ => {
                    function.value.push(consume_component_value(css_parser));
                }
            }
        } else {
            css_parser.record_error(CssParsingError::IncompleteFunction(pos));
            break;
        }
    }

    function
}

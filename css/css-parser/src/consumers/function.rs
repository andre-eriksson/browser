use css_tokenizer::{CssTokenKind, SourcePosition};
use errors::parsing::CssParsingError;

use crate::{CssParser, Function, consumers::component::consume_component_value};

/// Consume a function
///
/// <https://www.w3.org/TR/css-syntax-3/#consume-a-function>
pub(crate) fn consume_function(css_parser: &mut CssParser) -> Function {
    let (name, pos) = match css_parser.consume() {
        Some(token) => {
            let pos = token.position.unwrap_or_default();

            match token.kind {
                CssTokenKind::Function(name) => (name, pos),
                _ => (String::new(), pos), // NOTE: Should not happen
            }
        }
        _ => (String::new(), SourcePosition::default()), // NOTE: Should not happen
    };

    let mut function = Function::new(name);

    loop {
        match css_parser.peek() {
            Some(token) => match &token.kind {
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
            },
            None => {
                css_parser.record_error(CssParsingError::IncompleteFunction(pos));
                break;
            }
        }
    }

    function
}

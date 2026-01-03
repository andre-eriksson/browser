use css_tokenizer::CssTokenKind;

use crate::{CssParser, Function, consumers::component::consume_component_value};

/// Consume a function
///
/// <https://www.w3.org/TR/css-syntax-3/#consume-a-function>
pub(crate) fn consume_function(css_parser: &mut CssParser) -> Function {
    let name = match css_parser.consume() {
        Some(token) => match token.kind {
            CssTokenKind::Function(name) => name,
            _ => String::new(), // NOTE: Should not happen
        },
        _ => String::new(), // NOTE: Should not happen
    };

    let mut function = Function::new(name);

    #[allow(clippy::while_let_loop)]
    loop {
        match css_parser.peek() {
            Some(token) => match &token.kind {
                CssTokenKind::Eof => {
                    break;
                }
                CssTokenKind::CloseParen => {
                    css_parser.consume();
                    break;
                }
                _ => {
                    function.value.push(consume_component_value(css_parser));
                }
            },
            None => {
                // Parse error, but return the function
                // TODO: Collect an error
                break;
            }
        }
    }

    function
}

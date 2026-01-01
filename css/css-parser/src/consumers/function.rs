use css_tokenizer::CssToken;

use crate::{CssParser, Function, consumers::component::consume_component_value};

/// Consume a function
///
/// <https://www.w3.org/TR/css-syntax-3/#consume-a-function>
pub(crate) fn consume_function(css_parser: &mut CssParser) -> Function {
    let name = match css_parser.consume() {
        Some(CssToken::Function(name)) => name,
        _ => String::new(), // NOTE: Should not happen
    };

    let mut function = Function::new(name);

    loop {
        match css_parser.peek() {
            None | Some(CssToken::Eof) => {
                break;
            }
            Some(CssToken::CloseParen) => {
                css_parser.consume();
                break;
            }
            _ => {
                function.value.push(consume_component_value(css_parser));
            }
        }
    }

    function
}

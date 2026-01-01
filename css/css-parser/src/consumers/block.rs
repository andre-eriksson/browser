use css_tokenizer::CssToken;

use crate::{
    AssociatedToken, CssParser, SimpleBlock, consumers::component::consume_component_value,
};

/// Consume a simple block
///
/// <https://www.w3.org/TR/css-syntax-3/#consume-a-simple-block>
pub(crate) fn consume_simple_block(css_parser: &mut CssParser) -> SimpleBlock {
    let current = css_parser.consume();

    let (associated_token, ending_token) = match current {
        Some(CssToken::OpenCurly) => (AssociatedToken::CurlyBracket, CssToken::CloseCurly),
        Some(CssToken::OpenSquare) => (AssociatedToken::SquareBracket, CssToken::CloseSquare),
        Some(CssToken::OpenParen) => (AssociatedToken::Parenthesis, CssToken::CloseParen),
        _ => (AssociatedToken::CurlyBracket, CssToken::CloseCurly), // Should not happen
    };

    let mut block = SimpleBlock::new(associated_token);

    loop {
        match css_parser.peek() {
            None | Some(CssToken::Eof) => {
                // Parse error, but return the block
                break;
            }
            Some(token)
                if std::mem::discriminant(token) == std::mem::discriminant(&ending_token) =>
            {
                css_parser.consume();
                break;
            }
            _ => {
                block.value.push(consume_component_value(css_parser));
            }
        }
    }

    block
}

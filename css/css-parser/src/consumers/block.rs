use css_tokenizer::CssTokenKind;

use crate::{
    AssociatedToken, CssParser, SimpleBlock, consumers::component::consume_component_value,
};

/// Consume a simple block
///
/// <https://www.w3.org/TR/css-syntax-3/#consume-a-simple-block>
pub(crate) fn consume_simple_block(css_parser: &mut CssParser) -> SimpleBlock {
    let Some(current) = css_parser.consume() else {
        // Should not happen
        return SimpleBlock::new(AssociatedToken::CurlyBracket);
    };

    let (associated_token, ending_token) = match current.kind {
        CssTokenKind::OpenCurly => (AssociatedToken::CurlyBracket, CssTokenKind::CloseCurly),
        CssTokenKind::OpenSquare => (AssociatedToken::SquareBracket, CssTokenKind::CloseSquare),
        CssTokenKind::OpenParen => (AssociatedToken::Parenthesis, CssTokenKind::CloseParen),
        _ => (AssociatedToken::CurlyBracket, CssTokenKind::CloseCurly), // Should not happen
    };

    let mut block = SimpleBlock::new(associated_token);

    #[allow(clippy::while_let_loop)]
    loop {
        match css_parser.peek() {
            Some(token) => {
                if token.kind == CssTokenKind::Eof {
                    break;
                }

                if std::mem::discriminant(&token.kind) == std::mem::discriminant(&ending_token) {
                    css_parser.consume();
                    break;
                }

                let component_value = consume_component_value(css_parser);
                block.value.push(component_value);
            }

            None => {
                // Parse error, but return the block
                // TODO: Collect an error
                break;
            }
        }
    }

    block
}

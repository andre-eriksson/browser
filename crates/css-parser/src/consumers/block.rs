use crate::errors::CssParsingError;
use css_tokenizer::CssTokenKind;

use crate::{AssociatedToken, CssParser, SimpleBlock, consumers::component::consume_component_value};

/// Consume a simple block
///
/// <https://www.w3.org/TR/css-syntax-3/#consume-a-simple-block>
pub fn consume_simple_block(css_parser: &mut CssParser) -> SimpleBlock {
    let Some(current) = css_parser.consume() else {
        unreachable!("consume_simple_block should only be called when there is a current token");
    };

    let (associated_token, ending_token) = match current.kind {
        CssTokenKind::OpenCurly => (AssociatedToken::CurlyBracket, CssTokenKind::CloseCurly),
        CssTokenKind::OpenSquare => (AssociatedToken::SquareBracket, CssTokenKind::CloseSquare),
        CssTokenKind::OpenParen => (AssociatedToken::Parenthesis, CssTokenKind::CloseParen),
        _ => unreachable!("consume_simple_block should only be called when the current token is an opening token"),
    };

    let mut block = SimpleBlock::new(associated_token);

    loop {
        if let Some(token) = css_parser.peek() {
            if token.kind == CssTokenKind::Eof {
                let pos = current.position.unwrap_or_default();
                css_parser.record_error(CssParsingError::EofInSimpleBlock(pos));
                break;
            }

            if std::mem::discriminant(&token.kind) == std::mem::discriminant(&ending_token) {
                css_parser.consume();
                break;
            }

            let component_value = consume_component_value(css_parser);
            block.value.push(component_value);
        } else {
            let pos = current.position.unwrap_or_default();
            css_parser.record_error(CssParsingError::IncompleteSimpleBlock(pos));
            break;
        }
    }

    block
}

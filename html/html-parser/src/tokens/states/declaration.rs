use std::collections::HashMap;

use crate::tokens::{
    state::{ParserState, Token, TokenKind},
    tokenizer::HtmlTokenizer,
};

/// Handles the start of a declaration, after `<!`
pub fn handle_start_declaration_state(tokenizer: &mut HtmlTokenizer, ch: char) {
    match ch {
        '-' => {
            tokenizer.state = ParserState::CommentStart;
        }
        'd' | 'D' => {
            tokenizer.current_token = Some(Token {
                kind: TokenKind::DoctypeDeclaration,
                attributes: HashMap::new(),
                data: ch.to_string(),
            });
            tokenizer.state = ParserState::DoctypeDeclaration;
        }
        ch if ch.is_whitespace() => {}
        _ => {
            tokenizer.state = ParserState::BogusComment;
        }
    }
}

/// Handles the XML declaration state, after `<?`
pub fn handle_xml_declaration_state(tokenizer: &mut HtmlTokenizer, ch: char) {
    match ch {
        '?' => {
            if let Some(token) = tokenizer.current_token.as_mut() {
                token.data.push(ch);
            } else {
                tokenizer.current_token = Some(Token {
                    kind: TokenKind::XmlDeclaration,
                    attributes: HashMap::new(),
                    data: ch.to_string(),
                });
            }
        }
        '>' => {
            if let Some(token) = &tokenizer.current_token
                && token.data.ends_with('?')
            {
                if let Some(token) = tokenizer.current_token.take() {
                    tokenizer.emit_token(token);
                }
                tokenizer.state = ParserState::Data;
                return;
            }
            if let Some(token) = tokenizer.current_token.as_mut() {
                token.data.push(ch);
            } else {
                tokenizer.current_token = Some(Token {
                    kind: TokenKind::XmlDeclaration,
                    attributes: HashMap::new(),
                    data: ch.to_string(),
                });
            }
        }
        _ => {
            if let Some(token) = tokenizer.current_token.as_mut() {
                token.data.push(ch);
            } else {
                tokenizer.current_token = Some(Token {
                    kind: TokenKind::XmlDeclaration,
                    attributes: HashMap::new(),
                    data: ch.to_string(),
                });
            }
        }
    }
}

/// Handles the doctype declaration state, after the `<!D` or `<!d`
pub fn handle_doctype_declaration_state(tokenizer: &mut HtmlTokenizer, ch: char) {
    match ch {
        '>' => {
            if let Some(token) = tokenizer.current_token.take() {
                tokenizer.emit_token(token);
            }
            tokenizer.state = ParserState::Data;
        }
        _ => {
            if let Some(token) = tokenizer.current_token.as_mut() {
                token.data.push(ch);
            } else {
                tokenizer.current_token = Some(Token {
                    kind: TokenKind::DoctypeDeclaration,
                    attributes: HashMap::new(),
                    data: ch.to_string(),
                });
            }
        }
    }
}

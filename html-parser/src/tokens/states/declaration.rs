use std::collections::HashMap;

use crate::tokens::{
    state::{ParserState, Token, TokenKind},
    tokenizer::HtmlTokenizer,
};

pub fn handle_start_declaration_state(tokenizer: &mut HtmlTokenizer, ch: char) {
    match ch {
        '-' => {
            // Handle comments
            tokenizer.state = ParserState::CommentStart;
        }
        'd' | 'D' => {
            // Handle DOCTYPE declarations
            tokenizer.current_token = Some(Token {
                kind: TokenKind::DoctypeDeclaration,
                attributes: HashMap::new(),
                data: ch.to_string(),
            });
            tokenizer.state = ParserState::DoctypeDeclaration;
        }
        ch if ch.is_whitespace() => {
            // Ignore whitespace
        }
        _ => {
            tokenizer.state = ParserState::BogusComment;
        }
    }
}

pub fn handle_xml_declaration_state(tokenizer: &mut HtmlTokenizer, ch: char) {
    match ch {
        '?' => {
            // Don't end immediately on '?', add it to the data and continue
            // The actual end will be detected by checking for "?>" sequence
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
            // Check if this is the end of the "?>" sequence
            if let Some(token) = &tokenizer.current_token {
                if token.data.ends_with('?') {
                    // This is the end of the XML declaration
                    if let Some(token) = tokenizer.current_token.take() {
                        tokenizer.emit_token(token);
                    }
                    tokenizer.state = ParserState::Data; // Return to Data state
                    return;
                }
            }
            // If not the end sequence, treat as regular character
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

pub fn handle_doctype_declaration_state(tokenizer: &mut HtmlTokenizer, ch: char) {
    match ch {
        '>' => {
            // Handle the end of the DOCTYPE declaration
            if let Some(token) = tokenizer.current_token.take() {
                tokenizer.emit_token(token);
            }
            tokenizer.state = ParserState::Data; // Return to Data state
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

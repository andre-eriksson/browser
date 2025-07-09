use crate::tokens::{
    state::{ParserState, TokenKind},
    tokenizer::HtmlTokenizer,
};

pub fn handle_before_attribute_name_state(tokenizer: &mut HtmlTokenizer, ch: char) {
    match ch {
        '>' => {
            if let Some(token) = tokenizer.current_token.take() {
                tokenizer.emit_token(token);
            }
            tokenizer.state = ParserState::Data;
        }
        '/' => {
            tokenizer.state = ParserState::SelfClosingTagStart;
        }
        ch if ch.is_whitespace() => {
        }
        ch if ch.is_alphabetic() => {
            tokenizer.current_attribute_name.clear();
            tokenizer.current_attribute_name.push(ch);
            tokenizer.state = ParserState::AttributeName;
        }
        _ => {
            tokenizer.state = ParserState::Data;
        }
    }
}

pub fn handle_attribute_name_state(tokenizer: &mut HtmlTokenizer, ch: char) {
    match ch {
        '=' => {
            tokenizer.state = ParserState::BeforeAttributeValue;
        }
        '>' => {
            if let Some(mut token) = tokenizer.current_token.take() {
                token.attributes.insert(
                    tokenizer.current_attribute_name.clone(),
                    tokenizer.current_attribute_value.clone(),
                );

                tokenizer.emit_token(token);
            }
            tokenizer.state = ParserState::Data;
        }
        '/' => {
            tokenizer.state = ParserState::SelfClosingTagStart;
        }
        ch if ch.is_whitespace() => {
            tokenizer.state = ParserState::AfterAttributeName;
        }
        _ => {
            tokenizer.current_attribute_name.push(ch);
        }
    }
}

pub fn handle_after_attribute_name_state(tokenizer: &mut HtmlTokenizer, ch: char) {
    match ch {
        '>' => {
            if let Some(mut token) = tokenizer.current_token.take() {
                token.attributes.insert(
                    tokenizer.current_attribute_name.clone(),
                    tokenizer.current_attribute_value.clone(),
                );

                tokenizer.emit_token(token);
            }
            tokenizer.state = ParserState::Data;
        }
        '/' => {
            tokenizer.state = ParserState::SelfClosingTagStart;
        }
        '=' => {
            tokenizer.state = ParserState::BeforeAttributeValue;
        }
        ch if ch.is_whitespace() => {
        }
        ch if ch.is_alphabetic() => {
            if let Some(token) = tokenizer.current_token.as_mut() {
                token.attributes.insert(
                    tokenizer.current_attribute_name.clone(),
                    tokenizer.current_attribute_value.clone(),
                );
            }

            tokenizer.current_attribute_name.clear();
            tokenizer.current_attribute_name.push(ch);
            tokenizer.state = ParserState::AttributeName;
        }
        _ => {
            tokenizer.state = ParserState::Data;
        }
    }
}

pub fn handle_before_attribute_value_state(tokenizer: &mut HtmlTokenizer, ch: char) {
    match ch {
        '"' => {
            tokenizer.state = ParserState::AttributeValueDoubleQuoted;
        }
        '\'' => {
            tokenizer.state = ParserState::AttributeValueSingleQuoted;
        }
        ch if ch.is_whitespace() => {
        }
        _ => {
            tokenizer.current_attribute_value.clear();
            tokenizer.current_attribute_value.push(ch);
            tokenizer.state = ParserState::AttributeValueUnquoted;
        }
    }
}

pub fn handle_attribute_value_double_quoted_state(tokenizer: &mut HtmlTokenizer, ch: char) {
    match ch {
        '"' => {
            tokenizer.state = ParserState::AfterAttributeValueQuoted;
        }
        _ => {
            tokenizer.current_attribute_value.push(ch);
        }
    }
}

pub fn handle_attribute_value_single_quoted_state(tokenizer: &mut HtmlTokenizer, ch: char) {
    match ch {
        '\'' => {
            tokenizer.state = ParserState::AfterAttributeValueQuoted;
        }
        _ => {
            tokenizer.current_attribute_value.push(ch);
        }
    }
}

pub fn handle_attribute_value_unquoted_state(tokenizer: &mut HtmlTokenizer, ch: char) {
    match ch {
        '>' => {
            if let Some(mut token) = tokenizer.current_token.take() {
                token.attributes.insert(
                    tokenizer.current_attribute_name.clone(),
                    tokenizer.current_attribute_value.clone(),
                );

                tokenizer.current_attribute_name.clear();
                tokenizer.current_attribute_value.clear();

                tokenizer.emit_token(token);
            }
            tokenizer.state = ParserState::Data;
        }
        ch if ch.is_ascii_whitespace() => {
            if let Some(token) = tokenizer.current_token.as_mut() {
                token.attributes.insert(
                    tokenizer.current_attribute_name.clone(),
                    tokenizer.current_attribute_value.clone(),
                );

                tokenizer.current_attribute_name.clear();
                tokenizer.current_attribute_value.clear();
            }
            tokenizer.state = ParserState::BeforeAttributeName;
        }
        _ => {
            tokenizer.current_attribute_value.push(ch);
        }
    }
}

pub fn handle_after_attribute_value_quoted_state(tokenizer: &mut HtmlTokenizer, ch: char) {
    match ch {
        '>' => {
            if let Some(mut token) = tokenizer.current_token.take() {
                token.attributes.insert(
                    tokenizer.current_attribute_name.clone(),
                    tokenizer.current_attribute_value.clone(),
                );

                tokenizer.current_attribute_name.clear();
                tokenizer.current_attribute_value.clear();

                if token.data == "script" {
                    tokenizer.state = ParserState::ScriptData;
                } else {
                    if token.data == "pre" {
                        if token.kind == TokenKind::StartTag {
                            tokenizer.context.inside_preformatted = true;
                        } else if token.kind == TokenKind::EndTag {
                            tokenizer.context.inside_preformatted = false;
                        }
                    }
                    tokenizer.state = ParserState::Data;
                }

                tokenizer.emit_token(token);
            }
        }
        '/' => {
            tokenizer.state = ParserState::SelfClosingTagStart;
        }
        _ => {
            if let Some(token) = tokenizer.current_token.as_mut() {
                token.attributes.insert(
                    tokenizer.current_attribute_name.clone(),
                    tokenizer.current_attribute_value.clone(),
                );

                tokenizer.current_attribute_name.clear();
                tokenizer.current_attribute_value.clear();
            }

            tokenizer.state = ParserState::BeforeAttributeName;
        }
    }
}

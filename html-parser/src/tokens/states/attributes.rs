use crate::tokens::{state::ParserState, tokenizer::HtmlTokenizer};

pub fn handle_before_attribute_name_state(tokenizer: &mut HtmlTokenizer, ch: char) {
    match ch {
        '>' => {
            // Emit the start tag token
            if let Some(token) = tokenizer.current_token.take() {
                tokenizer.emit_token(token);
            }
            tokenizer.state = ParserState::Data; // Return to Data state
        }
        '/' => {
            tokenizer.state = ParserState::SelfClosingTagStart; // Transition to SelfClosingTagStart state
        }
        ch if ch.is_whitespace() => {
            // Ignore whitespace
        }
        ch if ch.is_alphabetic() => {
            // Start a new attribute name
            tokenizer.current_attribute_name.clear();
            tokenizer.current_attribute_name.push(ch);
            tokenizer.state = ParserState::AttributeName;
        }
        _ => {
            // Handle invalid characters before attribute name
            tokenizer.state = ParserState::Data; // Return to Data state
        }
    }
}

pub fn handle_attribute_name_state(tokenizer: &mut HtmlTokenizer, ch: char) {
    match ch {
        '=' => {
            tokenizer.state = ParserState::BeforeAttributeValue;
        }
        '>' => {
            // Emit the start tag token
            if let Some(mut token) = tokenizer.current_token.take() {
                token.attributes.insert(
                    tokenizer.current_attribute_name.clone(),
                    tokenizer.current_attribute_value.clone(),
                );

                tokenizer.emit_token(token);
            }
            tokenizer.state = ParserState::Data; // Return to Data state
        }
        '/' => {
            tokenizer.state = ParserState::SelfClosingTagStart; // Transition to SelfClosingTagStart state
        }
        ch if ch.is_whitespace() => {
            tokenizer.state = ParserState::AfterAttributeName;
        }
        _ => {
            // Continue accumulating the attribute name
            tokenizer.current_attribute_name.push(ch);
        }
    }
}

pub fn handle_after_attribute_name_state(tokenizer: &mut HtmlTokenizer, ch: char) {
    match ch {
        '>' => {
            // Emit the start tag token
            if let Some(mut token) = tokenizer.current_token.take() {
                token.attributes.insert(
                    tokenizer.current_attribute_name.clone(),
                    tokenizer.current_attribute_value.clone(),
                );

                tokenizer.emit_token(token);
            }
            tokenizer.state = ParserState::Data; // Return to Data state
        }
        '/' => {
            tokenizer.state = ParserState::SelfClosingTagStart; // Transition to SelfClosingTagStart state
        }
        '=' => {
            tokenizer.state = ParserState::BeforeAttributeValue; // Transition to BeforeAttributeValue state
        }
        ch if ch.is_whitespace() => {
            // Ignore whitespace
        }
        ch if ch.is_alphabetic() => {
            // Start a new attribute name
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
            // Handle invalid characters after attribute name
            tokenizer.state = ParserState::Data; // Return to Data state
        }
    }
}

pub fn handle_before_attribute_value_state(tokenizer: &mut HtmlTokenizer, ch: char) {
    match ch {
        '"' => {
            tokenizer.state = ParserState::AttributeValueDoubleQuoted; // Transition to AttributeValueDoubleQuoted state
        }
        '\'' => {
            tokenizer.state = ParserState::AttributeValueSingleQuoted; // Transition to AttributeValueSingleQuoted state
        }
        ch if ch.is_whitespace() => {
            // Ignore whitespace
        }
        _ => {
            // Start an unquoted attribute value
            tokenizer.current_attribute_value.clear();
            tokenizer.current_attribute_value.push(ch);
            tokenizer.state = ParserState::AttributeValueUnquoted;
        }
    }
}

pub fn handle_attribute_value_double_quoted_state(tokenizer: &mut HtmlTokenizer, ch: char) {
    match ch {
        '"' => {
            // End of double-quoted attribute value
            tokenizer.state = ParserState::AfterAttributeValueQuoted; // Transition to AfterAttributeValueQuoted state
        }
        _ => {
            // Continue accumulating the attribute value
            tokenizer.current_attribute_value.push(ch);
        }
    }
}

pub fn handle_attribute_value_single_quoted_state(tokenizer: &mut HtmlTokenizer, ch: char) {
    match ch {
        '\'' => {
            tokenizer.state = ParserState::AfterAttributeValueQuoted; // Transition to AfterAttributeValueQuoted state
        }
        _ => {
            // Continue accumulating the attribute value
            tokenizer.current_attribute_value.push(ch);
        }
    }
}

pub fn handle_attribute_value_unquoted_state(tokenizer: &mut HtmlTokenizer, ch: char) {
    match ch {
        '>' => {
            // End of unquoted attribute value
            if let Some(mut token) = tokenizer.current_token.take() {
                token.attributes.insert(
                    tokenizer.current_attribute_name.clone(),
                    tokenizer.current_attribute_value.clone(),
                );

                tokenizer.current_attribute_name.clear();
                tokenizer.current_attribute_value.clear();

                tokenizer.emit_token(token);
            }
            tokenizer.state = ParserState::Data; // Return to Data state
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
            // Continue accumulating the attribute value
            tokenizer.current_attribute_value.push(ch);
        }
    }
}

pub fn handle_after_attribute_value_quoted_state(tokenizer: &mut HtmlTokenizer, ch: char) {
    match ch {
        '>' => {
            // End of tag, emit the token
            if let Some(mut token) = tokenizer.current_token.take() {
                token.attributes.insert(
                    tokenizer.current_attribute_name.clone(),
                    tokenizer.current_attribute_value.clone(),
                );

                tokenizer.current_attribute_name.clear();
                tokenizer.current_attribute_value.clear();

                if token.data == "script" {
                    // If the tag is a script tag, switch to ScriptData state
                    tokenizer.state = ParserState::ScriptData;
                } else {
                    tokenizer.state = ParserState::Data; // Return to Data state
                }

                tokenizer.emit_token(token);
            }
        }
        '/' => {
            tokenizer.state = ParserState::SelfClosingTagStart; // Transition to SelfClosingTagStart state
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

            tokenizer.state = ParserState::BeforeAttributeName; // Return to Data state
        }
    }
}

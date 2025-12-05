use html_syntax::token::TokenKind;

use crate::tokens::{state::ParserState, tokenizer::HtmlTokenizer};

/// Handles the "Before Attribute Name" state in the HTML tokenizer.
///
/// # Arguments
/// * `tokenizer` - A mutable reference to the HTML tokenizer.
/// * `ch` - The current character being processed.
///
/// # Behavior
/// - If the character is '>', it emits the current token and transitions to the `ParserState::Data` state.
/// - If the character is '/', it transitions to the `ParserState::SelfClosingTagStart` state.
/// - If the character is whitespace, it remains in the `ParserState::BeforeAttributeName` state.
/// - If the character is alphabetic, it initializes a new attribute name and transitions to the `ParserState::AttributeName` state.
/// - For any other character, it transitions to the `ParserState::Data` state.
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
        ch if ch.is_whitespace() => {}
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

/// Handles the "Attribute Name" state in the HTML tokenizer.
///
/// # Arguments
/// * `tokenizer` - A mutable reference to the HTML tokenizer.
/// * `ch` - The current character being processed.
///
/// # Behavior
/// - If the character is '=', it transitions to the `ParserState::BeforeAttributeValue` state.
/// - If the character is '>', it finalizes the current attribute and emits the token, then transitions to the `ParserState::Data` state.
/// - If the character is '/', it transitions to the `ParserState::SelfClosingTagStart` state.
/// - If the character is whitespace, it transitions to the `ParserState::AfterAttributeName` state.
/// - For any other character, it appends the character to the current attribute name.
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

/// Handles the "After Attribute Name" state in the HTML tokenizer.
///
/// # Arguments
/// * `tokenizer` - A mutable reference to the HTML tokenizer.
/// * `ch` - The current character being processed.
///
/// # Behavior
/// - If the character is '>', it finalizes the current attribute and emits the token, then transitions to the `ParserState::Data` state.
/// - If the character is '/', it transitions to the `ParserState::SelfClosingTagStart` state.
/// - If the character is '=', it transitions to the `ParserState::BeforeAttributeValue` state.
/// - If the character is whitespace, it remains in the `ParserState::AfterAttributeName` state.
/// - If the character is alphabetic, it finalizes the current attribute, initializes a new attribute name, and transitions to the `ParserState::AttributeName` state.
/// - For any other character, it transitions to the `ParserState::Data` state.
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
        ch if ch.is_whitespace() => {}
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

/// Handles the "Before Attribute Value" state in the HTML tokenizer.
///
/// # Arguments
/// * `tokenizer` - A mutable reference to the HTML tokenizer.
/// * `ch` - The current character being processed.
///
/// # Behavior
/// - If the character is a double quote (`"`), it transitions to the `ParserState::AttributeValueDoubleQuoted` state.
/// - If the character is a single quote (`'`), it transitions to the `ParserState::AttributeValueSingleQuoted` state.
/// - If the character is whitespace, it remains in the `ParserState::BeforeAttributeValue` state.
/// - For any other character, it initializes the current attribute value with the character and transitions to the `ParserState::AttributeValueUnquoted` state.
pub fn handle_before_attribute_value_state(tokenizer: &mut HtmlTokenizer, ch: char) {
    match ch {
        '"' => {
            tokenizer.state = ParserState::AttributeValueDoubleQuoted;
        }
        '\'' => {
            tokenizer.state = ParserState::AttributeValueSingleQuoted;
        }
        ch if ch.is_whitespace() => {}
        _ => {
            tokenizer.current_attribute_value.clear();
            tokenizer.current_attribute_value.push(ch);
            tokenizer.state = ParserState::AttributeValueUnquoted;
        }
    }
}

/// Handles the "Attribute Value Double Quoted" state in the HTML tokenizer.
///
/// # Arguments
/// * `tokenizer` - A mutable reference to the HTML tokenizer.
/// * `ch` - The current character being processed.
///
/// # Behavior
/// - If the character is a double quote (`"`), it transitions to the `ParserState::AfterAttributeValueQuoted` state.
/// - For any other character, it appends the character to the current attribute value.
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

/// Handles the "Attribute Value Single Quoted" state in the HTML tokenizer.
///
/// # Arguments
/// * `tokenizer` - A mutable reference to the HTML tokenizer.
/// * `ch` - The current character being processed.
///
/// # Behavior
/// - If the character is a single quote (`'`), it transitions to the `ParserState::AfterAttributeValueQuoted` state.
/// - For any other character, it appends the character to the current attribute value.
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

/// Handles the "Attribute Value Unquoted" state in the HTML tokenizer.
///
/// # Arguments
/// * `tokenizer` - A mutable reference to the HTML tokenizer.
/// * `ch` - The current character being processed.
///
/// # Behavior
/// - If the character is '>', it finalizes the current attribute, emits the token, and transitions to the `ParserState::Data` state.
/// - If the character is whitespace, it finalizes the current attribute and transitions to the `ParserState::BeforeAttributeName` state.
/// - For any other character, it appends the character to the current attribute value.
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

/// Handles the "After Attribute Value Quoted" state in the HTML tokenizer.
///
/// # Arguments
/// * `tokenizer` - A mutable reference to the HTML tokenizer.
/// * `ch` - The current character being processed.
///
/// # Behavior
/// - If the character is '>', it finalizes the current attribute, emits the token, and transitions to the `ParserState::Data` state.
/// - If the character is '/', it transitions to the `ParserState::SelfClosingTagStart` state.
/// - For any other character, it finalizes the current attribute and transitions to the `ParserState::BeforeAttributeName` state.
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

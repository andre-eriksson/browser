use crate::{
    Token,
    state::TokenState,
    states::tag::handle_closing_tag,
    tokenizer::{HtmlTokenizer, TokenizerState},
};

/// Handles the "Before Attribute Name" state in the HTML tokenizer.
///
/// # Arguments
/// * `tokenizer` - A mutable reference to the HTML tokenizer.
/// * `ch` - The current character being processed.
/// * `tokens` - A mutable reference to the vector of tokens to which new tokens will be emitted.
///
/// # Behavior
/// - If the character is '>', it emits the current token and transitions to the `ParserState::Data` state.
/// - If the character is '/', it transitions to the `ParserState::SelfClosingTagStart` state.
/// - If the character is whitespace, it remains in the `ParserState::BeforeAttributeName` state.
/// - If the character is alphabetic, it initializes a new attribute name and transitions to the `ParserState::AttributeName` state.
/// - For any other character, it transitions to the `ParserState::Data` state.
pub fn handle_before_attribute_name_state(
    state: &mut TokenizerState,
    ch: char,
    tokens: &mut Vec<Token>,
) {
    match ch {
        '>' => {
            if let Some(token) = state.current_token.take() {
                HtmlTokenizer::emit_token(tokens, token);
            }
            state.state = TokenState::Data;
        }
        '/' => {
            state.state = TokenState::SelfClosingTagStart;
        }
        ch if ch.is_whitespace() => {}
        ch if ch.is_alphabetic() => {
            state.current_attribute_name.clear();
            state.current_attribute_name.push(ch);
            state.state = TokenState::AttributeName;
        }
        _ => {
            state.state = TokenState::Data;
        }
    }
}

/// Handles the "Attribute Name" state in the HTML tokenizer.
///
/// # Arguments
/// * `tokenizer` - A mutable reference to the HTML tokenizer.
/// * `ch` - The current character being processed.
/// * `tokens` - A mutable reference to the vector of tokens to which new tokens will be emitted.
///
/// # Behavior
/// - If the character is '=', it transitions to the `ParserState::BeforeAttributeValue` state.
/// - If the character is '>', it finalizes the current attribute and emits the token, then transitions to the `ParserState::Data` state.
/// - If the character is '/', it transitions to the `ParserState::SelfClosingTagStart` state.
/// - If the character is whitespace, it transitions to the `ParserState::AfterAttributeName` state.
/// - For any other character, it appends the character to the current attribute name.
pub fn handle_attribute_name_state(state: &mut TokenizerState, ch: char, tokens: &mut Vec<Token>) {
    match ch {
        '=' => {
            state.state = TokenState::BeforeAttributeValue;
        }
        '>' => handle_closing_tag(state, tokens),
        '/' => {
            state.state = TokenState::SelfClosingTagStart;
        }
        ch if ch.is_whitespace() => {
            state.state = TokenState::AfterAttributeName;
        }
        _ => {
            state.current_attribute_name.push(ch);
        }
    }
}

/// Handles the "After Attribute Name" state in the HTML tokenizer.
///
/// # Arguments
/// * `tokenizer` - A mutable reference to the HTML tokenizer.
/// * `ch` - The current character being processed.
/// * `tokens` - A mutable reference to the vector of tokens to which new tokens will be emitted.
///
/// # Behavior
/// - If the character is '>', it finalizes the current attribute and emits the token, then transitions to the `ParserState::Data` state.
/// - If the character is '/', it transitions to the `ParserState::SelfClosingTagStart` state.
/// - If the character is '=', it transitions to the `ParserState::BeforeAttributeValue` state.
/// - If the character is whitespace, it remains in the `ParserState::AfterAttributeName` state.
/// - If the character is alphabetic, it finalizes the current attribute, initializes a new attribute name, and transitions to the `ParserState::AttributeName` state.
/// - For any other character, it transitions to the `ParserState::Data` state.
pub fn handle_after_attribute_name_state(
    state: &mut TokenizerState,
    ch: char,
    tokens: &mut Vec<Token>,
) {
    match ch {
        '>' => handle_closing_tag(state, tokens),
        '/' => {
            state.state = TokenState::SelfClosingTagStart;
        }
        '=' => {
            state.state = TokenState::BeforeAttributeValue;
        }
        ch if ch.is_whitespace() => {}
        ch if ch.is_alphabetic() => {
            if let Some(token) = state.current_token.as_mut() {
                token.attributes.insert(
                    state.current_attribute_name.clone(),
                    state.current_attribute_value.clone(),
                );
            }

            state.current_attribute_name.clear();
            state.current_attribute_name.push(ch);
            state.state = TokenState::AttributeName;
        }
        _ => {
            state.state = TokenState::Data;
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
pub fn handle_before_attribute_value_state(state: &mut TokenizerState, ch: char) {
    match ch {
        '"' => {
            state.state = TokenState::AttributeValueDoubleQuoted;
        }
        '\'' => {
            state.state = TokenState::AttributeValueSingleQuoted;
        }
        ch if ch.is_whitespace() => {}
        _ => {
            state.current_attribute_value.clear();
            state.current_attribute_value.push(ch);
            state.state = TokenState::AttributeValueUnquoted;
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
pub fn handle_attribute_value_double_quoted_state(state: &mut TokenizerState, ch: char) {
    match ch {
        '"' => {
            state.state = TokenState::AfterAttributeValueQuoted;
        }
        _ => {
            state.current_attribute_value.push(ch);
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
pub fn handle_attribute_value_single_quoted_state(state: &mut TokenizerState, ch: char) {
    match ch {
        '\'' => {
            state.state = TokenState::AfterAttributeValueQuoted;
        }
        _ => {
            state.current_attribute_value.push(ch);
        }
    }
}

/// Handles the "Attribute Value Unquoted" state in the HTML tokenizer.
///
/// # Arguments
/// * `tokenizer` - A mutable reference to the HTML tokenizer.
/// * `ch` - The current character being processed.
/// * `tokens` - A mutable reference to the vector of tokens to which new tokens will be emitted.
///
/// # Behavior
/// - If the character is '>', it finalizes the current attribute, emits the token, and transitions to the `ParserState::Data` state.
/// - If the character is whitespace, it finalizes the current attribute and transitions to the `ParserState::BeforeAttributeName` state.
/// - For any other character, it appends the character to the current attribute value.
pub fn handle_attribute_value_unquoted_state(
    state: &mut TokenizerState,
    ch: char,
    tokens: &mut Vec<Token>,
) {
    match ch {
        '>' => handle_closing_tag(state, tokens),
        ch if ch.is_ascii_whitespace() => {
            if let Some(token) = state.current_token.as_mut() {
                token.attributes.insert(
                    state.current_attribute_name.clone(),
                    state.current_attribute_value.clone(),
                );

                state.current_attribute_name.clear();
                state.current_attribute_value.clear();
            }
            state.state = TokenState::BeforeAttributeName;
        }
        _ => {
            state.current_attribute_value.push(ch);
        }
    }
}

/// Handles the "After Attribute Value Quoted" state in the HTML tokenizer.
///
/// # Arguments
/// * `tokenizer` - A mutable reference to the HTML tokenizer.
/// * `ch` - The current character being processed.
/// * `tokens` - A mutable reference to the vector of tokens to which new tokens will be emitted.
///
/// # Behavior
/// - If the character is '>', it finalizes the current attribute, emits the token, and transitions to the `ParserState::Data` state.
/// - If the character is '/', it transitions to the `ParserState::SelfClosingTagStart` state.
/// - For any other character, it finalizes the current attribute and transitions to the `ParserState::BeforeAttributeName` state.
pub fn handle_after_attribute_value_quoted_state(
    state: &mut TokenizerState,
    ch: char,
    tokens: &mut Vec<Token>,
) {
    match ch {
        '>' => handle_closing_tag(state, tokens),
        '/' => {
            state.state = TokenState::SelfClosingTagStart;
        }
        _ => {
            if let Some(token) = state.current_token.as_mut() {
                token.attributes.insert(
                    state.current_attribute_name.clone(),
                    state.current_attribute_value.clone(),
                );

                state.current_attribute_name.clear();
                state.current_attribute_value.clear();
            }

            state.state = TokenState::BeforeAttributeName;
        }
    }
}

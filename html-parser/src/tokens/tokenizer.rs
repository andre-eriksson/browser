use std::collections::{HashMap, VecDeque};

use crate::tokens::{
    state::{ParserState, Token, TokenKind},
    states::{
        attributes::{
            handle_after_attribute_name_state, handle_after_attribute_value_quoted_state,
            handle_attribute_name_state, handle_attribute_value_double_quoted_state,
            handle_attribute_value_single_quoted_state, handle_attribute_value_unquoted_state,
            handle_before_attribute_name_state, handle_before_attribute_value_state,
        },
        comment::{
            handle_bogus_comment_state, handle_comment_end_state, handle_comment_start_state,
            handle_comment_state,
        },
        data::{handle_data_state, preserve_significant_whitespace},
        declaration::{
            handle_doctype_declaration_state, handle_start_declaration_state,
            handle_xml_declaration_state,
        },
        script::{handle_script_data_end_tag_open_state, handle_script_data_state},
        tag::{
            handle_end_tag_open_state, handle_self_closing_tag_start_state, handle_tag_name_state,
            handle_tag_open_state,
        },
    },
};

/// A tokenizer for HTML content that processes chunks of HTML and emits tokens.
/// This tokenizer handles various HTML states, including text, tags, attributes, comments, and declarations.
///
/// # Fields
/// * `state` - The current state of the parser.
/// * `current_token` - The token currently being constructed.
/// * `temporary_buffer` - A buffer for accumulating text data between tokens.
/// * `current_attribute_name` - The name of the current attribute being processed.
/// * `current_attribute_value` - The value of the current attribute being processed.
/// * `tokens` - A queue of tokens that have been parsed and are ready to be emitted.
pub struct HtmlTokenizer {
    pub state: ParserState,
    pub current_token: Option<Token>,
    pub temporary_buffer: String,
    pub current_attribute_name: String,
    pub current_attribute_value: String,
    pub tokens: VecDeque<Token>,
}

impl HtmlTokenizer {
    /// Creates a new instance of `HtmlTokenizer`.
    pub fn new() -> Self {
        Self {
            state: ParserState::Data,
            current_token: None,
            temporary_buffer: String::new(),
            current_attribute_name: String::new(),
            current_attribute_value: String::new(),
            tokens: VecDeque::new(),
        }
    }

    /// Tokenize a chunk of HTML content
    ///
    /// # Arguments
    /// * `chunk` - A slice of bytes representing a chunk of HTML content
    ///
    /// # Returns
    /// A vector of `Token` objects representing the parsed HTML content.
    pub fn tokenize(&mut self, chunk: &[u8]) -> Vec<Token> {
        let text = String::from_utf8_lossy(chunk);

        for current_char in text.chars() {
            match self.state {
                ParserState::Data => handle_data_state(self, current_char),
                ParserState::TagOpen => handle_tag_open_state(self, current_char),
                ParserState::EndTagOpen => handle_end_tag_open_state(self, current_char),
                ParserState::SelfClosingTagStart => {
                    handle_self_closing_tag_start_state(self, current_char)
                }
                ParserState::TagName => handle_tag_name_state(self, current_char),
                ParserState::BeforeAttributeName => {
                    handle_before_attribute_name_state(self, current_char)
                }
                ParserState::AttributeName => handle_attribute_name_state(self, current_char),
                ParserState::AfterAttributeName => {
                    handle_after_attribute_name_state(self, current_char)
                }
                ParserState::BeforeAttributeValue => {
                    handle_before_attribute_value_state(self, current_char)
                }
                ParserState::AttributeValueDoubleQuoted => {
                    handle_attribute_value_double_quoted_state(self, current_char)
                }
                ParserState::AttributeValueSingleQuoted => {
                    handle_attribute_value_single_quoted_state(self, current_char)
                }
                ParserState::AttributeValueUnquoted => {
                    handle_attribute_value_unquoted_state(self, current_char)
                }
                ParserState::AfterAttributeValueQuoted => {
                    handle_after_attribute_value_quoted_state(self, current_char);
                }
                ParserState::StartDeclaration => handle_start_declaration_state(self, current_char),
                ParserState::BogusComment => handle_bogus_comment_state(self, current_char),
                ParserState::CommentStart => handle_comment_start_state(self, current_char),
                ParserState::Comment => handle_comment_state(self, current_char),
                ParserState::CommentEnd => handle_comment_end_state(self, current_char),
                ParserState::XmlDeclaration => handle_xml_declaration_state(self, current_char),
                ParserState::DoctypeDeclaration => {
                    handle_doctype_declaration_state(self, current_char)
                }
                ParserState::ScriptData => handle_script_data_state(self, current_char),
                ParserState::ScriptDataEndTagOpen => {
                    handle_script_data_end_tag_open_state(self, current_char)
                }
            }
        }

        self.tokens.drain(..).collect()
    }

    /// A helper function to emit a token into the queue.
    pub fn emit_token(&mut self, token: Token) {
        self.tokens.push_back(token);
    }

}


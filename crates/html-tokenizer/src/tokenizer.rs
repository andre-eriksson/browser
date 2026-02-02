use std::collections::HashMap;

use crate::{
    Token, TokenKind,
    state::TokenState,
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
        data::handle_data_state,
        declaration::{
            handle_doctype_declaration_state, handle_start_declaration_state,
            handle_xml_declaration_state,
        },
        tag::{
            handle_end_tag_open_state, handle_self_closing_tag_start_state, handle_tag_name_state,
            handle_tag_open_state,
        },
    },
};

#[derive(Debug, Default)]
pub struct TokenizerState {
    /// The current state of the HTML parser.
    pub state: TokenState,

    /// The current token being constructed by the tokenizer.
    pub current_token: Option<Token>,

    /// A temporary buffer used for accumulating characters during tokenization.
    pub temporary_buffer: String,

    /// The name of the current attribute being processed.
    pub current_attribute_name: String,

    /// The value of the current attribute being processed.
    pub current_attribute_value: String,
}

/// A tokenizer for HTML content that processes chunks of HTML and emits tokens.
/// This tokenizer handles various HTML states, including text, tags, attributes, comments, and declarations.
pub struct HtmlTokenizer;

impl HtmlTokenizer {
    /// Processes a single character based on the current parser state and updates the tokenizer state accordingly.
    ///
    /// # Arguments
    /// * `state` - A mutable reference to the current tokenizer state.
    /// * `ch` - The character to be processed.
    /// * `tokens` - A mutable reference to the vector of tokens to which new tokens will be emitted.
    pub fn process_char(state: &mut TokenizerState, ch: char, tokens: &mut Vec<Token>) {
        match state.state {
            TokenState::Data => handle_data_state(state, ch, tokens),
            TokenState::TagOpen => handle_tag_open_state(state, ch),
            TokenState::EndTagOpen => handle_end_tag_open_state(state, ch, tokens),
            TokenState::SelfClosingTagStart => {
                handle_self_closing_tag_start_state(state, ch, tokens)
            }
            TokenState::TagName => handle_tag_name_state(state, ch, tokens),
            TokenState::BeforeAttributeName => {
                handle_before_attribute_name_state(state, ch, tokens)
            }
            TokenState::AttributeName => handle_attribute_name_state(state, ch, tokens),
            TokenState::AfterAttributeName => handle_after_attribute_name_state(state, ch, tokens),
            TokenState::BeforeAttributeValue => handle_before_attribute_value_state(state, ch),
            TokenState::AttributeValueDoubleQuoted => {
                handle_attribute_value_double_quoted_state(state, ch)
            }
            TokenState::AttributeValueSingleQuoted => {
                handle_attribute_value_single_quoted_state(state, ch)
            }
            TokenState::AttributeValueUnquoted => {
                handle_attribute_value_unquoted_state(state, ch, tokens)
            }
            TokenState::AfterAttributeValueQuoted => {
                handle_after_attribute_value_quoted_state(state, ch, tokens);
            }
            TokenState::StartDeclaration => handle_start_declaration_state(state, ch),
            TokenState::BogusComment => handle_bogus_comment_state(state, ch),
            TokenState::CommentStart => handle_comment_start_state(state, ch),
            TokenState::Comment => handle_comment_state(state, ch),
            TokenState::CommentEnd => handle_comment_end_state(state, ch, tokens),
            TokenState::XmlDeclaration => handle_xml_declaration_state(state, ch, tokens),
            TokenState::DoctypeDeclaration => handle_doctype_declaration_state(state, ch, tokens),
            TokenState::ScriptData => {
                HtmlTokenizer::emit_token(
                    tokens,
                    Token {
                        kind: TokenKind::EndTag,
                        attributes: HashMap::new(),
                        data: "script".to_string(),
                    },
                );

                state.temporary_buffer.clear();
                state.state = TokenState::Data;
                HtmlTokenizer::process_char(state, ch, tokens);
            }
            TokenState::StyleData => {
                HtmlTokenizer::emit_token(
                    tokens,
                    Token {
                        kind: TokenKind::EndTag,
                        attributes: HashMap::new(),
                        data: "style".to_string(),
                    },
                );

                state.temporary_buffer.clear();
                state.state = TokenState::Data;
                HtmlTokenizer::process_char(state, ch, tokens);
            }
            TokenState::SvgData => {
                HtmlTokenizer::emit_token(
                    tokens,
                    Token {
                        kind: TokenKind::EndTag,
                        attributes: HashMap::new(),
                        data: "svg".to_string(),
                    },
                );

                state.temporary_buffer.clear();
                state.state = TokenState::Data;
                HtmlTokenizer::process_char(state, ch, tokens);
            }
        }
    }

    /// An **inline** helper function to emit a token by pushing it onto the tokens vector.
    ///
    /// # Arguments
    /// * `tokens` - A mutable reference to the vector of tokens.
    /// * `token` - The token to be emitted.
    #[inline]
    pub fn emit_token(tokens: &mut Vec<Token>, token: Token) {
        tokens.push(token);
    }
}

use std::collections::{HashMap, VecDeque};

use crate::tokens::state::{ParserState, Token, TokenKind};

pub struct HtmlTokenizer {
    state: ParserState,
    current_token: Option<Token>,
    temporary_buffer: String,
    current_attribute_name: String,
    current_attribute_value: String,
    tokens: VecDeque<Token>,
}

impl HtmlTokenizer {
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
                ParserState::Data => self.handle_data_state(current_char),
                ParserState::TagOpen => self.handle_tag_open_state(current_char),
                ParserState::EndTagOpen => self.handle_end_tag_open_state(current_char),
                ParserState::SelfClosingTagStart => {
                    self.handle_self_closing_tag_start_state(current_char)
                }
                ParserState::TagName => self.handle_tag_name_state(current_char),
                ParserState::BeforeAttributeName => {
                    self.handle_before_attribute_name_state(current_char)
                }
                ParserState::AttributeName => self.handle_attribute_name_state(current_char),
                ParserState::AfterAttributeName => {
                    self.handle_after_attribute_name_state(current_char)
                }
                ParserState::BeforeAttributeValue => {
                    self.handle_before_attribute_value_state(current_char)
                }
                ParserState::AttributeValueDoubleQuoted => {
                    self.handle_attribute_value_double_quoted_state(current_char)
                }
                ParserState::AttributeValueSingleQuoted => {
                    self.handle_attribute_value_single_quoted_state(current_char)
                }
                ParserState::AttributeValueUnquoted => {
                    self.handle_attribute_value_unquoted_state(current_char)
                }
                ParserState::AfterAttributeValueQuoted => {
                    self.handle_after_attribute_value_quoted_state(current_char);
                }
                ParserState::StartDeclaration => self.handle_start_declaration_state(current_char),
                ParserState::BogusComment => self.handle_bogus_comment_state(current_char),
                ParserState::CommentStart => self.handle_comment_start_state(current_char),
                ParserState::Comment => self.handle_comment_state(current_char),
                ParserState::CommentEnd => self.handle_comment_end_state(current_char),
                ParserState::XmlDeclaration => self.handle_xml_declaration_state(current_char),
                ParserState::DoctypeDeclaration => {
                    self.handle_doctype_declaration_state(current_char)
                }
                ParserState::ScriptData => self.handle_script_data_state(current_char),
                ParserState::ScriptDataEndTagOpen => {
                    self.handle_script_data_end_tag_open_state(current_char)
                }
            }
        }

        self.tokens.drain(..).collect()
    }

    /// Preserves significant whitespace in the text.
    fn preserve_significant_whitespace(&self, text: &str) -> String {
        // Check if we're inside a <pre> or <code> tag by looking at current token context
        let preserve_exact = self.is_in_preformatted_context();

        if preserve_exact {
            // For preformatted content, preserve all whitespace exactly including \r\n
            return text.to_string();
        }

        // If the text is only whitespace, preserve it as a single space
        // This is important for whitespace between tags like "</span> world"
        if text.trim().is_empty() && !text.is_empty() {
            return " ".to_string();
        }

        // For text with actual content, do minimal normalization
        // Replace sequences of whitespace within the text with single spaces
        // but preserve leading/trailing whitespace as single spaces if present
        let has_leading_ws = text.starts_with(char::is_whitespace);
        let has_trailing_ws = text.ends_with(char::is_whitespace);
        let has_leading_newline = text.starts_with('\n') || text.starts_with('\r');
        let has_trailing_newline = text.ends_with('\n') || text.ends_with('\r');

        let normalized_middle = text.trim().split_whitespace().collect::<Vec<_>>().join(" ");

        let mut result = String::new();
        if has_leading_ws && !normalized_middle.is_empty() && !has_leading_newline {
            result.push(' ');
        }
        result.push_str(&normalized_middle);
        if has_trailing_ws && !normalized_middle.is_empty() && !has_trailing_newline {
            result.push(' ');
        }

        result
    }

    /// Checks if the current context is within a preformatted tag, like <pre>, <code>, or <textarea>.
    /// This is used to determine if whitespace should be preserved exactly.
    fn is_in_preformatted_context(&self) -> bool {
        if self.tokens.back().is_some() {
            if let Some(token) = self.tokens.back() {
                return matches!(token.kind, TokenKind::StartTag | TokenKind::EndTag)
                    && (token.data == "pre" || token.data == "code" || token.data == "textarea");
            }
        }

        false
    }

    fn handle_data_state(&mut self, ch: char) {
        match ch {
            '<' => {
                if self.temporary_buffer.len() > 0 && !self.temporary_buffer.trim().is_empty() {
                    // Emit the data token if there's accumulated data
                    self.tokens.push_back(Token {
                        kind: TokenKind::Text,
                        attributes: HashMap::new(),
                        data: self.preserve_significant_whitespace(&self.temporary_buffer),
                    });
                    self.temporary_buffer.clear();
                }
                self.state = ParserState::TagOpen;
            }
            _ => {
                self.temporary_buffer.push(ch);
            }
        }
    }

    fn handle_tag_open_state(&mut self, ch: char) {
        match ch {
            '!' => {
                self.state = ParserState::StartDeclaration;
            }
            '?' => {
                self.current_token = Some(Token {
                    kind: TokenKind::XmlDeclaration,
                    attributes: HashMap::new(),
                    data: ch.to_string(),
                });
                self.state = ParserState::XmlDeclaration;
            }
            '/' => {
                self.current_token = Some(Token {
                    kind: TokenKind::EndTag,
                    attributes: HashMap::new(),
                    data: String::new(),
                });
                self.state = ParserState::EndTagOpen; // Transition to EndTagOpen state
            }
            ch if ch.is_alphabetic() => {
                self.current_token = Some(Token {
                    kind: TokenKind::StartTag,
                    attributes: HashMap::new(),
                    data: ch.to_string(),
                });
                self.state = ParserState::TagName; // Transition to TagName state
            }
            _ => {
                // Handle invalid tag opening
                self.state = ParserState::Data; // Return to Data state
            }
        }
    }

    fn handle_end_tag_open_state(&mut self, ch: char) {
        match ch {
            '>' => {
                // Handle end tag closing

                if let Some(token) = self.current_token.take() {
                    self.tokens.push_back(token);
                }

                self.state = ParserState::Data; // Return to Data state
            }
            ch if ch.is_alphabetic() => {
                // Start accumulating the end tag name
                if let Some(token) = self.current_token.as_mut() {
                    token.data.push(ch);
                } else {
                    self.current_token = Some(Token {
                        kind: TokenKind::EndTag,
                        attributes: HashMap::new(),
                        data: ch.to_string(),
                    });
                }
                self.state = ParserState::TagName; // Transition to TagName state
            }
            _ => {
                // Handle invalid end tag opening
                self.state = ParserState::Data; // Return to Data state
            }
        }
    }

    fn handle_self_closing_tag_start_state(&mut self, ch: char) {
        match ch {
            '>' => {
                // Emit the self-closing tag token
                if let Some(mut token) = self.current_token.take() {
                    if !self.current_attribute_name.is_empty() {
                        token.attributes.insert(
                            self.current_attribute_name.clone(),
                            self.current_attribute_value.clone(),
                        );

                        self.current_attribute_name.clear();
                        self.current_attribute_value.clear();
                    }

                    self.tokens.push_back(token);
                }
                self.state = ParserState::Data; // Return to Data state
            }
            _ => {
                panic!(
                    "Unexpected character in SelfClosingTagStart state: '{}', previous_token_data: '{:?}' '{}' current_token_data: '{:?}' '{}', buffer: '{}'",
                    ch,
                    self.tokens
                        .back()
                        .map_or(TokenKind::Comment, |t| t.kind.clone()),
                    self.tokens
                        .back()
                        .map_or("None".to_string(), |t| t.data.clone()),
                    self.current_token
                        .as_ref()
                        .map_or(TokenKind::Comment, |t| t.kind.clone()),
                    self.current_token
                        .as_ref()
                        .map_or("None".to_string(), |t| t.data.clone()),
                    self.temporary_buffer
                );
            }
        }
    }

    fn handle_tag_name_state(&mut self, ch: char) {
        match ch {
            ch if ch.is_whitespace() => {
                self.state = ParserState::BeforeAttributeName;
            }
            '>' => {
                // Emit the start tag token

                if let Some(token) = self.current_token.take() {
                    if token.data == "script" {
                        // If the tag is a script tag, switch to ScriptData state
                        self.state = ParserState::ScriptData;
                    } else {
                        self.state = ParserState::Data; // Return to Data state
                    }

                    self.tokens.push_back(token);
                }
            }
            '/' => {
                self.state = ParserState::SelfClosingTagStart; // Transition to SelfClosingTagStart state
            }
            _ => {
                // Continue accumulating the tag name
                if let Some(token) = self.current_token.as_mut() {
                    token.data.push(ch);
                } else {
                    self.current_token = Some(Token {
                        kind: TokenKind::StartTag,
                        attributes: HashMap::new(),
                        data: ch.to_string(),
                    });
                }
            }
        }
    }

    fn handle_before_attribute_name_state(&mut self, ch: char) {
        match ch {
            '>' => {
                // Emit the start tag token
                if let Some(token) = self.current_token.take() {
                    self.tokens.push_back(token);
                }
                self.state = ParserState::Data; // Return to Data state
            }
            '/' => {
                self.state = ParserState::SelfClosingTagStart; // Transition to SelfClosingTagStart state
            }
            ch if ch.is_whitespace() => {
                // Ignore whitespace
            }
            ch if ch.is_alphabetic() => {
                // Start a new attribute name
                self.current_attribute_name.clear();
                self.current_attribute_name.push(ch);
                self.state = ParserState::AttributeName;
            }
            _ => {
                // Handle invalid characters before attribute name
                self.state = ParserState::Data; // Return to Data state
            }
        }
    }

    fn handle_attribute_name_state(&mut self, ch: char) {
        match ch {
            '=' => {
                self.state = ParserState::BeforeAttributeValue;
            }
            '>' => {
                // Emit the start tag token
                if let Some(mut token) = self.current_token.take() {
                    token.attributes.insert(
                        self.current_attribute_name.clone(),
                        self.current_attribute_value.clone(),
                    );

                    self.tokens.push_back(token);
                }
                self.state = ParserState::Data; // Return to Data state
            }
            '/' => {
                self.state = ParserState::SelfClosingTagStart; // Transition to SelfClosingTagStart state
            }
            ch if ch.is_whitespace() => {
                self.state = ParserState::AfterAttributeName;
            }
            _ => {
                // Continue accumulating the attribute name
                self.current_attribute_name.push(ch);
            }
        }
    }

    fn handle_after_attribute_name_state(&mut self, ch: char) {
        match ch {
            '>' => {
                // Emit the start tag token
                if let Some(mut token) = self.current_token.take() {
                    token.attributes.insert(
                        self.current_attribute_name.clone(),
                        self.current_attribute_value.clone(),
                    );

                    self.tokens.push_back(token);
                }
                self.state = ParserState::Data; // Return to Data state
            }
            '/' => {
                self.state = ParserState::SelfClosingTagStart; // Transition to SelfClosingTagStart state
            }
            '=' => {
                self.state = ParserState::BeforeAttributeValue; // Transition to BeforeAttributeValue state
            }
            ch if ch.is_whitespace() => {
                // Ignore whitespace
            }
            ch if ch.is_alphabetic() => {
                // Start a new attribute name
                if let Some(token) = self.current_token.as_mut() {
                    token.attributes.insert(
                        self.current_attribute_name.clone(),
                        self.current_attribute_value.clone(),
                    );
                }

                self.current_attribute_name.clear();
                self.current_attribute_name.push(ch);
                self.state = ParserState::AttributeName;
            }
            _ => {
                // Handle invalid characters after attribute name
                self.state = ParserState::Data; // Return to Data state
            }
        }
    }

    fn handle_before_attribute_value_state(&mut self, ch: char) {
        match ch {
            '"' => {
                self.state = ParserState::AttributeValueDoubleQuoted; // Transition to AttributeValueDoubleQuoted state
            }
            '\'' => {
                self.state = ParserState::AttributeValueSingleQuoted; // Transition to AttributeValueSingleQuoted state
            }
            ch if ch.is_whitespace() => {
                // Ignore whitespace
            }
            _ => {
                // Start an unquoted attribute value
                self.current_attribute_value.clear();
                self.current_attribute_value.push(ch);
                self.state = ParserState::AttributeValueUnquoted;
            }
        }
    }

    fn handle_attribute_value_double_quoted_state(&mut self, ch: char) {
        match ch {
            '"' => {
                // End of double-quoted attribute value
                self.state = ParserState::AfterAttributeValueQuoted; // Transition to AfterAttributeValueQuoted state
            }
            _ => {
                // Continue accumulating the attribute value
                self.current_attribute_value.push(ch);
            }
        }
    }

    fn handle_attribute_value_single_quoted_state(&mut self, ch: char) {
        match ch {
            '\'' => {
                self.state = ParserState::AfterAttributeValueQuoted; // Transition to AfterAttributeValueQuoted state
            }
            _ => {
                // Continue accumulating the attribute value
                self.current_attribute_value.push(ch);
            }
        }
    }

    fn handle_attribute_value_unquoted_state(&mut self, ch: char) {
        match ch {
            '>' => {
                // End of unquoted attribute value
                if let Some(mut token) = self.current_token.take() {
                    token.attributes.insert(
                        self.current_attribute_name.clone(),
                        self.current_attribute_value.clone(),
                    );

                    self.current_attribute_name.clear();
                    self.current_attribute_value.clear();

                    self.tokens.push_back(token);
                }
                self.state = ParserState::Data; // Return to Data state
            }
            ch if ch.is_ascii_whitespace() => {
                if let Some(token) = self.current_token.as_mut() {
                    token.attributes.insert(
                        self.current_attribute_name.clone(),
                        self.current_attribute_value.clone(),
                    );

                    self.current_attribute_name.clear();
                    self.current_attribute_value.clear();
                }
                self.state = ParserState::BeforeAttributeName;
            }
            _ => {
                // Continue accumulating the attribute value
                self.current_attribute_value.push(ch);
            }
        }
    }

    fn handle_after_attribute_value_quoted_state(&mut self, ch: char) {
        match ch {
            '>' => {
                // End of tag, emit the token
                if let Some(mut token) = self.current_token.take() {
                    token.attributes.insert(
                        self.current_attribute_name.clone(),
                        self.current_attribute_value.clone(),
                    );

                    self.current_attribute_name.clear();
                    self.current_attribute_value.clear();

                    if token.data == "script" {
                        // If the tag is a script tag, switch to ScriptData state
                        self.state = ParserState::ScriptData;
                    } else {
                        self.state = ParserState::Data; // Return to Data state
                    }

                    self.tokens.push_back(token);
                }
            }
            '/' => {
                self.state = ParserState::SelfClosingTagStart; // Transition to SelfClosingTagStart state
            }
            _ => {
                if let Some(token) = self.current_token.as_mut() {
                    token.attributes.insert(
                        self.current_attribute_name.clone(),
                        self.current_attribute_value.clone(),
                    );

                    self.current_attribute_name.clear();
                    self.current_attribute_value.clear();
                }

                self.state = ParserState::BeforeAttributeName; // Return to Data state
            }
        }
    }

    fn handle_start_declaration_state(&mut self, ch: char) {
        match ch {
            '-' => {
                // Handle comments
                self.state = ParserState::CommentStart;
            }
            'd' | 'D' => {
                // Handle DOCTYPE declarations
                self.current_token = Some(Token {
                    kind: TokenKind::DoctypeDeclaration,
                    attributes: HashMap::new(),
                    data: ch.to_string(),
                });
                self.state = ParserState::DoctypeDeclaration;
            }
            ch if ch.is_whitespace() => {
                // Ignore whitespace
            }
            _ => {
                self.state = ParserState::BogusComment;
            }
        }
    }

    fn handle_bogus_comment_state(&mut self, ch: char) {
        match ch {
            '>' => {
                // End of bogus comment
                self.state = ParserState::Data; // Return to Data state
            }
            _ => {
                // Ignore characters in bogus comment
            }
        }
    }

    fn handle_comment_start_state(&mut self, ch: char) {
        match ch {
            '-' => {
                self.current_token = Some(Token {
                    kind: TokenKind::Comment,
                    attributes: HashMap::new(),
                    data: String::new(),
                });

                self.state = ParserState::Comment; // Transition to Comment state
            }
            _ => {
                self.state = ParserState::BogusComment; // Transition to BogusComment state
            }
        }
    }

    fn handle_comment_state(&mut self, ch: char) {
        match ch {
            '-' => {
                self.state = ParserState::CommentEnd; // Transition to CommentEnd state
            }
            _ => {
                if let Some(token) = self.current_token.as_mut() {
                    token.data.push(ch);
                } else {
                    self.current_token = Some(Token {
                        kind: TokenKind::Comment,
                        attributes: HashMap::new(),
                        data: ch.to_string(),
                    });
                }
            }
        }
    }

    fn handle_comment_end_state(&mut self, ch: char) {
        match ch {
            '>' => {
                // End of comment
                if let Some(token) = self.current_token.take() {
                    self.tokens.push_back(token);
                }
                self.state = ParserState::Data; // Return to Data state
            }
            '-' => {
                // Ignore consecutive dashes in comments
            }
            _ => {
                // Handle invalid characters after comment end
                if let Some(token) = self.current_token.as_mut() {
                    token.data.push('-'); // Add the dash back to the comment data
                    token.data.push(ch);
                } else {
                    self.current_token = Some(Token {
                        kind: TokenKind::Comment,
                        attributes: HashMap::new(),
                        data: format!("-{}", ch),
                    });
                }
                self.state = ParserState::Comment; // Return to Comment state
            }
        }
    }

    fn handle_xml_declaration_state(&mut self, ch: char) {
        match ch {
            '?' => {
                // Handle the end of the XML declaration
                if let Some(token) = self.current_token.take() {
                    self.tokens.push_back(token);
                }
                self.state = ParserState::Data; // Return to Data state
            }
            _ => {
                if let Some(token) = self.current_token.as_mut() {
                    token.data.push(ch);
                } else {
                    self.current_token = Some(Token {
                        kind: TokenKind::XmlDeclaration,
                        attributes: HashMap::new(),
                        data: ch.to_string(),
                    });
                }
            }
        }
    }

    fn handle_doctype_declaration_state(&mut self, ch: char) {
        match ch {
            '>' => {
                // Handle the end of the DOCTYPE declaration
                if let Some(token) = self.current_token.take() {
                    self.tokens.push_back(token);
                }
                self.state = ParserState::Data; // Return to Data state
            }
            _ => {
                if let Some(token) = self.current_token.as_mut() {
                    token.data.push(ch);
                } else {
                    self.current_token = Some(Token {
                        kind: TokenKind::DoctypeDeclaration,
                        attributes: HashMap::new(),
                        data: ch.to_string(),
                    });
                }
            }
        }
    }

    fn handle_script_data_state(&mut self, ch: char) {
        match ch {
            '<' => {
                // Handle the start of a script end tag
                self.temporary_buffer.clear();
                self.temporary_buffer.push(ch);
                self.state = ParserState::ScriptDataEndTagOpen;
            }
            _ => {
                if let Some(token) = self.current_token.as_mut() {
                    token.data.push(ch);
                } else {
                    self.current_token = Some(Token {
                        kind: TokenKind::Text,
                        attributes: HashMap::new(),
                        data: ch.to_string(),
                    });
                }
            }
        }
    }

    fn handle_script_data_end_tag_open_state(&mut self, ch: char) {
        let expected = "</script>";
        match ch {
            '>' => {
                self.temporary_buffer.push('>'); // Complete the end tag
                if self.temporary_buffer == expected {
                    if let Some(token) = self.current_token.take() {
                        // Emit the script start tag token
                        self.tokens.push_back(token);
                    }

                    // Emit a script end tag token
                    self.tokens.push_back(Token {
                        kind: TokenKind::EndTag,
                        attributes: HashMap::new(),
                        data: "script".to_string(),
                    });

                    self.temporary_buffer.clear();
                    self.state = ParserState::Data; // Return to Data state
                } else {
                    if let Some(token) = self.current_token.as_mut() {
                        self.temporary_buffer.push(ch);
                        token.data.push_str(&self.temporary_buffer);
                    }
                    self.temporary_buffer.clear();
                    self.state = ParserState::ScriptData; // Return to ScriptData state
                }
            }
            ch if ch.is_whitespace() => {
                // Ignore whitespace in script end tag open state
            }
            _ => {
                if self.temporary_buffer.len() == expected.len() {
                    if self.temporary_buffer != expected {
                        // If we have accumulated enough characters, emit the current token
                        if let Some(token) = self.current_token.as_mut() {
                            token.data.push_str(&self.temporary_buffer);
                            self.state = ParserState::ScriptData; // Return to ScriptData state
                            self.temporary_buffer.clear();
                        }
                    }
                } else {
                    // Continue accumulating characters for the script end tag
                    self.temporary_buffer.push(ch);
                }
            }
        }
    }
}

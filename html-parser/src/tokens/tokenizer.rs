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
    /// Preserve significant whitespace in text content:
    /// - Keep whitespace that appears between tags (significant for layout)
    /// - Collapse only excessive internal whitespace within text runs
    /// - Preserve single spaces that separate words
    fn preserve_significant_whitespace(&self, text: &str) -> String {
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

        let normalized_middle = text.trim().split_whitespace().collect::<Vec<_>>().join(" ");

        let mut result = String::new();
        if has_leading_ws && !normalized_middle.is_empty() {
            result.push(' ');
        }
        result.push_str(&normalized_middle);
        if has_trailing_ws && !normalized_middle.is_empty() {
            result.push(' ');
        }

        result
    }

    pub fn tokenize(&mut self, chunk: &[u8]) -> Vec<Token> {
        for &current_byte in chunk {
            match self.state {
                ParserState::Data => self.handle_data_state(current_byte),
                ParserState::TagOpen => self.handle_tag_open_state(current_byte),
                ParserState::EndTagOpen => self.handle_end_tag_open_state(current_byte),
                ParserState::SelfClosingTagStart => {
                    self.handle_self_closing_tag_start_state(current_byte)
                }
                ParserState::TagName => self.handle_tag_name_state(current_byte),
                ParserState::BeforeAttributeName => {
                    self.handle_before_attribute_name_state(current_byte)
                }
                ParserState::AttributeName => self.handle_attribute_name_state(current_byte),
                ParserState::AfterAttributeName => {
                    self.handle_after_attribute_name_state(current_byte)
                }
                ParserState::BeforeAttributeValue => {
                    self.handle_before_attribute_value_state(current_byte)
                }
                ParserState::AttributeValueDoubleQuoted => {
                    self.handle_attribute_value_double_quoted_state(current_byte)
                }
                ParserState::AttributeValueSingleQuoted => {
                    self.handle_attribute_value_single_quoted_state(current_byte)
                }
                ParserState::AttributeValueUnquoted => {
                    self.handle_attribute_value_unquoted_state(current_byte)
                }
                ParserState::AfterAttributeValueQuoted => {
                    self.handle_after_attribute_value_quoted_state(current_byte);
                }
                ParserState::StartDeclaration => self.handle_start_declaration_state(current_byte),
                ParserState::BogusComment => self.handle_bogus_comment_state(current_byte),
                ParserState::CommentStart => self.handle_comment_start_state(current_byte),
                ParserState::Comment => self.handle_comment_state(current_byte),
                ParserState::CommentEnd => self.handle_comment_end_state(current_byte),
                ParserState::XmlDeclaration => self.handle_xml_declaration_state(current_byte),
                ParserState::DoctypeDeclaration => {
                    self.handle_doctype_declaration_state(current_byte)
                }
                ParserState::ScriptData => self.handle_script_data_state(current_byte),
                ParserState::ScriptDataEndTagOpen => {
                    self.handle_script_data_end_tag_open_state(current_byte)
                }
            }
        }

        self.tokens.drain(..).collect()
    }
    fn handle_data_state(&mut self, byte: u8) {
        match byte {
            b'<' => {
                if self.temporary_buffer.len() > 0 {
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
                self.temporary_buffer.push(byte as char);
            }
        }
    }

    fn handle_tag_open_state(&mut self, byte: u8) {
        match byte {
            b'!' => {
                self.state = ParserState::StartDeclaration;
            }
            b'?' => {
                self.current_token = Some(Token {
                    kind: TokenKind::XmlDeclaration,
                    attributes: HashMap::new(),
                    data: (byte as char).to_string(),
                });
                self.state = ParserState::XmlDeclaration;
            }
            b'/' => {
                self.current_token = Some(Token {
                    kind: TokenKind::EndTag,
                    attributes: HashMap::new(),
                    data: String::new(),
                });
                self.state = ParserState::EndTagOpen; // Transition to EndTagOpen state
            }
            byte if byte.is_ascii_alphabetic() => {
                self.current_token = Some(Token {
                    kind: TokenKind::StartTag,
                    attributes: HashMap::new(),
                    data: (byte as char).to_string(),
                });
                self.state = ParserState::TagName; // Transition to TagName state
            }
            _ => {
                // Handle invalid tag opening
                self.state = ParserState::Data; // Return to Data state
            }
        }
    }

    fn handle_end_tag_open_state(&mut self, byte: u8) {
        match byte {
            b'>' => {
                // Handle end tag closing

                if let Some(token) = self.current_token.take() {
                    self.tokens.push_back(token);
                }

                self.state = ParserState::Data; // Return to Data state
            }
            byte if byte.is_ascii_alphabetic() => {
                // Start accumulating the end tag name
                if let Some(token) = self.current_token.as_mut() {
                    token.data.push(byte as char);
                } else {
                    self.current_token = Some(Token {
                        kind: TokenKind::EndTag,
                        attributes: HashMap::new(),
                        data: (byte as char).to_string(),
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

    fn handle_self_closing_tag_start_state(&mut self, byte: u8) {
        match byte {
            b'>' => {
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
                panic!("Unexpected byte in SelfClosingTagStart state: {}", byte);
            }
        }
    }

    fn handle_tag_name_state(&mut self, byte: u8) {
        match byte {
            byte if byte.is_ascii_whitespace() => {
                self.state = ParserState::BeforeAttributeName;
            }
            b'>' => {
                // Emit the start tag token
                if let Some(token) = self.current_token.take() {
                    self.tokens.push_back(token);
                }
                self.state = ParserState::Data; // Return to Data state
            }
            b'/' => {
                self.state = ParserState::SelfClosingTagStart; // Transition to SelfClosingTagStart state
            }
            _ => {
                // Continue accumulating the tag name
                if let Some(token) = self.current_token.as_mut() {
                    token.data.push(byte as char);
                } else {
                    self.current_token = Some(Token {
                        kind: TokenKind::StartTag,
                        attributes: HashMap::new(),
                        data: (byte as char).to_string(),
                    });
                }
            }
        }
    }

    fn handle_before_attribute_name_state(&mut self, byte: u8) {
        match byte {
            b'>' => {
                // Emit the start tag token
                if let Some(token) = self.current_token.take() {
                    self.tokens.push_back(token);
                }
                self.state = ParserState::Data; // Return to Data state
            }
            b'/' => {
                self.state = ParserState::SelfClosingTagStart; // Transition to SelfClosingTagStart state
            }
            byte if byte.is_ascii_whitespace() => {
                // Ignore whitespace
            }
            byte if byte.is_ascii_alphabetic() => {
                // Start a new attribute name
                self.current_attribute_name.clear();
                self.current_attribute_name.push(byte as char);
                self.state = ParserState::AttributeName;
            }
            _ => {
                // Handle invalid characters before attribute name
                self.state = ParserState::Data; // Return to Data state
            }
        }
    }

    fn handle_attribute_name_state(&mut self, byte: u8) {
        match byte {
            b'=' => {
                self.state = ParserState::BeforeAttributeValue;
            }
            b'>' => {
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
            b'/' => {
                self.state = ParserState::SelfClosingTagStart; // Transition to SelfClosingTagStart state
            }
            byte if byte.is_ascii_whitespace() => {
                self.state = ParserState::AfterAttributeName;
            }
            _ => {
                // Continue accumulating the attribute name
                self.current_attribute_name.push(byte as char);
            }
        }
    }

    fn handle_after_attribute_name_state(&mut self, byte: u8) {
        match byte {
            b'>' => {
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
            b'=' => {
                self.state = ParserState::BeforeAttributeValue; // Transition to BeforeAttributeValue state
            }
            byte if byte.is_ascii_whitespace() => {
                // Ignore whitespace
            }
            byte if byte.is_ascii_alphabetic() => {
                // Start a new attribute name
                if let Some(token) = self.current_token.as_mut() {
                    token.attributes.insert(
                        self.current_attribute_name.clone(),
                        self.current_attribute_value.clone(),
                    );
                }

                self.current_attribute_name.clear();
                self.current_attribute_name.push(byte as char);
                self.state = ParserState::AttributeName;
            }
            _ => {
                // Handle invalid characters after attribute name
                self.state = ParserState::Data; // Return to Data state
            }
        }
    }

    fn handle_before_attribute_value_state(&mut self, byte: u8) {
        match byte {
            b'"' => {
                self.state = ParserState::AttributeValueDoubleQuoted; // Transition to AttributeValueDoubleQuoted state
            }
            b'\'' => {
                self.state = ParserState::AttributeValueSingleQuoted; // Transition to AttributeValueSingleQuoted state
            }
            byte if byte.is_ascii_whitespace() => {
                // Ignore whitespace
            }
            _ => {
                // Start an unquoted attribute value
                self.current_attribute_value.clear();
                self.current_attribute_value.push(byte as char);
                self.state = ParserState::AttributeValueUnquoted;
            }
        }
    }

    fn handle_attribute_value_double_quoted_state(&mut self, byte: u8) {
        match byte {
            b'"' => {
                // End of double-quoted attribute value
                self.state = ParserState::AfterAttributeValueQuoted; // Transition to AfterAttributeValueQuoted state
            }
            _ => {
                // Continue accumulating the attribute value
                self.current_attribute_value.push(byte as char);
            }
        }
    }

    fn handle_attribute_value_single_quoted_state(&mut self, byte: u8) {
        match byte {
            b'\'' => {
                self.state = ParserState::AfterAttributeValueQuoted; // Transition to AfterAttributeValueQuoted state
            }
            _ => {
                // Continue accumulating the attribute value
                self.current_attribute_value.push(byte as char);
            }
        }
    }

    fn handle_attribute_value_unquoted_state(&mut self, byte: u8) {
        match byte {
            b'>' => {
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
            byte if byte.is_ascii_whitespace() => {
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
                self.current_attribute_value.push(byte as char);
            }
        }
    }

    fn handle_after_attribute_value_quoted_state(&mut self, byte: u8) {
        // This state is reached after a quoted attribute value
        // We can reset the current attribute name and value
        match byte {
            b'>' => {
                // End of tag, emit the token
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
            b'/' => {
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

    fn handle_start_declaration_state(&mut self, byte: u8) {
        // Handle the start of a declaration (like <!DOCTYPE or <?xml)
        match byte {
            b'-' => {
                // Handle comments
                self.state = ParserState::CommentStart;
            }
            b'd' | b'D' => {
                // Handle DOCTYPE declarations
                self.current_token = Some(Token {
                    kind: TokenKind::DoctypeDeclaration,
                    attributes: HashMap::new(),
                    data: (byte as char).to_string(),
                });
                self.state = ParserState::DoctypeDeclaration;
            }
            byte if byte.is_ascii_whitespace() => {
                // Ignore whitespace
            }
            _ => {
                self.state = ParserState::BogusComment;
            }
        }
    }

    fn handle_bogus_comment_state(&mut self, byte: u8) {
        // Handle bogus comments
        match byte {
            b'>' => {
                // End of bogus comment
                self.state = ParserState::Data; // Return to Data state
            }
            _ => {
                // Ignore characters in bogus comment
            }
        }
    }

    fn handle_comment_start_state(&mut self, byte: u8) {
        // Handle the start of a comment
        match byte {
            b'-' => {
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

    fn handle_comment_state(&mut self, byte: u8) {
        // Handle characters inside a comment
        match byte {
            b'-' => {
                self.state = ParserState::CommentEnd; // Transition to CommentEnd state
            }
            _ => {
                if let Some(token) = self.current_token.as_mut() {
                    token.data.push(byte as char);
                } else {
                    self.current_token = Some(Token {
                        kind: TokenKind::Comment,
                        attributes: HashMap::new(),
                        data: (byte as char).to_string(),
                    });
                }
            }
        }
    }

    fn handle_comment_end_state(&mut self, byte: u8) {
        match byte {
            b'>' => {
                // End of comment
                if let Some(token) = self.current_token.take() {
                    self.tokens.push_back(token);
                }
                self.state = ParserState::Data; // Return to Data state
            }
            b'-' => {
                // Ignore consecutive dashes in comments
            }
            _ => {
                // Handle invalid characters after comment end
                if let Some(token) = self.current_token.as_mut() {
                    token.data.push('-'); // Add the dash back to the comment data
                    token.data.push(byte as char);
                } else {
                    self.current_token = Some(Token {
                        kind: TokenKind::Comment,
                        attributes: HashMap::new(),
                        data: format!("-{}", byte as char),
                    });
                }
                self.state = ParserState::Comment; // Return to Comment state
            }
        }
    }

    fn handle_xml_declaration_state(&mut self, byte: u8) {
        match byte {
            b'?' => {
                // Handle the end of the XML declaration
                if let Some(token) = self.current_token.take() {
                    self.tokens.push_back(token);
                }
                self.state = ParserState::Data; // Return to Data state
            }
            _ => {
                if let Some(token) = self.current_token.as_mut() {
                    token.data.push(byte as char);
                } else {
                    self.current_token = Some(Token {
                        kind: TokenKind::XmlDeclaration,
                        attributes: HashMap::new(),
                        data: (byte as char).to_string(),
                    });
                }
            }
        }
    }

    fn handle_doctype_declaration_state(&mut self, byte: u8) {
        match byte {
            b'>' => {
                // Handle the end of the DOCTYPE declaration
                if let Some(token) = self.current_token.take() {
                    self.tokens.push_back(token);
                }
                self.state = ParserState::Data; // Return to Data state
            }
            _ => {
                if let Some(token) = self.current_token.as_mut() {
                    token.data.push(byte as char);
                } else {
                    self.current_token = Some(Token {
                        kind: TokenKind::DoctypeDeclaration,
                        attributes: HashMap::new(),
                        data: (byte as char).to_string(),
                    });
                }
            }
        }
    }

    fn handle_script_data_state(&mut self, byte: u8) {
        // Handle characters inside a script tag
        match byte {
            b'<' => {
                // Handle the start of a script end tag
                self.state = ParserState::ScriptDataEndTagOpen;
            }
            _ => {
                if let Some(token) = self.current_token.as_mut() {
                    if !self.temporary_buffer.is_empty() {
                        token.data.push_str(&self.temporary_buffer);
                        self.temporary_buffer.clear();
                    }

                    token.data.push(byte as char);
                } else {
                    self.current_token = Some(Token {
                        kind: TokenKind::Text,
                        attributes: HashMap::new(),
                        data: (byte as char).to_string(),
                    });
                }
            }
        }
    }

    fn handle_script_data_end_tag_open_state(&mut self, byte: u8) {
        match byte {
            b'>' => {
                if !self.temporary_buffer.eq_ignore_ascii_case("script") {
                    self.state = ParserState::ScriptData; // Return to ScriptData state
                }
            }
            _ => {
                if self.temporary_buffer.eq_ignore_ascii_case("script") {
                    // If the tag is a script end tag, emit the token
                    if let Some(token) = self.current_token.take() {
                        self.tokens.push_back(token);
                    }
                    self.state = ParserState::Data; // Return to Data state
                } else {
                    // Otherwise, continue accumulating the tag name
                    self.temporary_buffer.push(byte as char);
                }
            }
        }
    }
}

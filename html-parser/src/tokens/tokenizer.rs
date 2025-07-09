use std::collections::VecDeque;

use crate::tokens::{
    state::{ParserState, Token},
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
        script::{handle_script_data_end_tag_open_state, handle_script_data_state},
        tag::{
            handle_end_tag_open_state, handle_self_closing_tag_start_state, handle_tag_name_state,
            handle_tag_open_state,
        },
    },
};

/// Context for the tokenizer that keeps track of the current parsing state between chunks.
///
/// # Fields
/// * `inside_preformatted` - A boolean indicating whether the tokenizer is currently inside a preformatted text block (e.g., `<pre>` tag).
#[derive(Debug, Default)]
pub struct TokenizerContext {
    pub inside_preformatted: bool,
}

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
    pub context: TokenizerContext,
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
            context: TokenizerContext::default(),
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

        //for token in self.tokens.clone() {
        //    println!("Token emitted: {:?}", token);
        //}

        self.tokens.drain(..).collect()
    }

    /// A helper function to emit a token into the queue.
    pub fn emit_token(&mut self, token: Token) {
        self.tokens.push_back(token);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::state::{Token, TokenKind};

    fn assert_token_eq(actual: &Token, expected_kind: TokenKind, expected_data: &str) {
        assert_eq!(actual.kind, expected_kind);
        assert_eq!(actual.data, expected_data);
    }

    fn assert_token_with_attrs(
        actual: &Token,
        expected_kind: TokenKind,
        expected_data: &str,
        expected_attrs: Vec<(&str, &str)>,
    ) {
        assert_eq!(actual.kind, expected_kind);
        assert_eq!(actual.data, expected_data);
        assert_eq!(actual.attributes.len(), expected_attrs.len());
        for (key, value) in expected_attrs {
            assert_eq!(actual.attributes.get(key), Some(&value.to_string()));
        }
    }

    #[test]
    fn test_simple_text() {
        let mut tokenizer = HtmlTokenizer::new();
        let tokens = tokenizer.tokenize(b"Hello, World!");

        assert_eq!(tokens.len(), 0);
    }

    #[test]
    fn test_simple_start_tag() {
        let mut tokenizer = HtmlTokenizer::new();
        let tokens = tokenizer.tokenize(b"<div>");

        assert_eq!(tokens.len(), 1);
        assert_token_eq(&tokens[0], TokenKind::StartTag, "div");
    }

    #[test]
    fn test_simple_end_tag() {
        let mut tokenizer = HtmlTokenizer::new();
        let tokens = tokenizer.tokenize(b"</div>");

        assert_eq!(tokens.len(), 1);
        assert_token_eq(&tokens[0], TokenKind::EndTag, "div");
    }

    #[test]
    fn test_self_closing_tag() {
        let mut tokenizer = HtmlTokenizer::new();
        let tokens = tokenizer.tokenize(b"<br/>");

        assert_eq!(tokens.len(), 1);
        assert_token_eq(&tokens[0], TokenKind::StartTag, "br");
    }

    #[test]
    fn test_tag_with_single_attribute() {
        let mut tokenizer = HtmlTokenizer::new();
        let tokens = tokenizer.tokenize(b"<div class=\"container\">");

        assert_eq!(tokens.len(), 1);
        assert_token_with_attrs(
            &tokens[0],
            TokenKind::StartTag,
            "div",
            vec![("class", "container")],
        );
    }

    #[test]
    fn test_tag_with_multiple_attributes() {
        let mut tokenizer = HtmlTokenizer::new();
        let tokens = tokenizer.tokenize(b"<input type=\"text\" name=\"username\" id=\"user\">");

        assert_eq!(tokens.len(), 1);
        assert_token_with_attrs(
            &tokens[0],
            TokenKind::StartTag,
            "input",
            vec![("type", "text"), ("name", "username"), ("id", "user")],
        );
    }

    #[test]
    fn test_tag_with_single_quoted_attribute() {
        let mut tokenizer = HtmlTokenizer::new();
        let tokens = tokenizer.tokenize(b"<div class='container'>");

        assert_eq!(tokens.len(), 1);
        assert_token_with_attrs(
            &tokens[0],
            TokenKind::StartTag,
            "div",
            vec![("class", "container")],
        );
    }

    #[test]
    fn test_tag_with_unquoted_attribute() {
        let mut tokenizer = HtmlTokenizer::new();
        let tokens = tokenizer.tokenize(b"<div class=container>");

        assert_eq!(tokens.len(), 1);
        assert_token_with_attrs(
            &tokens[0],
            TokenKind::StartTag,
            "div",
            vec![("class", "container")],
        );
    }

    #[test]
    fn test_tag_with_boolean_attribute() {
        let mut tokenizer = HtmlTokenizer::new();
        let tokens = tokenizer.tokenize(b"<input disabled>");

        assert_eq!(tokens.len(), 1);
        assert_token_with_attrs(
            &tokens[0],
            TokenKind::StartTag,
            "input",
            vec![("disabled", "")],
        );
    }

    #[test]
    fn test_complete_element() {
        let mut tokenizer = HtmlTokenizer::new();
        let tokens = tokenizer.tokenize(b"<div>Hello</div>");

        assert_eq!(tokens.len(), 3);
        assert_token_eq(&tokens[0], TokenKind::StartTag, "div");
        assert_token_eq(&tokens[1], TokenKind::Text, "Hello");
        assert_token_eq(&tokens[2], TokenKind::EndTag, "div");
    }

    #[test]
    fn test_nested_elements() {
        let mut tokenizer = HtmlTokenizer::new();
        let tokens = tokenizer.tokenize(b"<div><span>Hello</span></div>");

        assert_eq!(tokens.len(), 5);
        assert_token_eq(&tokens[0], TokenKind::StartTag, "div");
        assert_token_eq(&tokens[1], TokenKind::StartTag, "span");
        assert_token_eq(&tokens[2], TokenKind::Text, "Hello");
        assert_token_eq(&tokens[3], TokenKind::EndTag, "span");
        assert_token_eq(&tokens[4], TokenKind::EndTag, "div");
    }

    #[test]
    fn test_comment() {
        let mut tokenizer = HtmlTokenizer::new();
        let tokens = tokenizer.tokenize(b"<!-- This is a comment -->");

        assert_eq!(tokens.len(), 1);
        assert_token_eq(&tokens[0], TokenKind::Comment, " This is a comment ");
    }

    #[test]
    fn test_comment_with_content() {
        let mut tokenizer = HtmlTokenizer::new();
        let tokens = tokenizer.tokenize(b"<div><!-- comment --></div>");

        assert_eq!(tokens.len(), 3);
        assert_token_eq(&tokens[0], TokenKind::StartTag, "div");
        assert_token_eq(&tokens[1], TokenKind::Comment, " comment ");
        assert_token_eq(&tokens[2], TokenKind::EndTag, "div");
    }

    #[test]
    fn test_doctype() {
        let mut tokenizer = HtmlTokenizer::new();
        let tokens = tokenizer.tokenize(b"<!DOCTYPE html>");

        assert_eq!(tokens.len(), 1);
        assert_token_eq(&tokens[0], TokenKind::DoctypeDeclaration, "DOCTYPE html");
    }

    #[test]
    fn test_xml_declaration() {
        let mut tokenizer = HtmlTokenizer::new();
        let tokens = tokenizer.tokenize(b"<?xml version=\"1.0\"?>");

        assert_eq!(tokens.len(), 1);
        assert_token_eq(
            &tokens[0],
            TokenKind::XmlDeclaration,
            "?xml version=\"1.0\"?",
        );
    }

    #[test]
    fn test_script_tag() {
        let mut tokenizer = HtmlTokenizer::new();
        let tokens = tokenizer.tokenize(b"<script>console.log('hello');</script>");

        assert_eq!(tokens.len(), 3);
        assert_token_eq(&tokens[0], TokenKind::StartTag, "script");
        assert_token_eq(&tokens[1], TokenKind::Text, "console.log('hello');");
        assert_token_eq(&tokens[2], TokenKind::EndTag, "script");
    }

    #[test]
    fn test_script_tag_with_attributes() {
        let mut tokenizer = HtmlTokenizer::new();
        let tokens =
            tokenizer.tokenize(b"<script type=\"text/javascript\">alert('test');</script>");

        assert_eq!(tokens.len(), 3);
        assert_token_with_attrs(
            &tokens[0],
            TokenKind::StartTag,
            "script",
            vec![("type", "text/javascript")],
        );
        assert_token_eq(&tokens[1], TokenKind::Text, "alert('test');");
        assert_token_eq(&tokens[2], TokenKind::EndTag, "script");
    }

    #[test]
    fn test_whitespace_handling() {
        let mut tokenizer = HtmlTokenizer::new();
        let tokens = tokenizer.tokenize(b"<div> Hello </div>");

        assert_eq!(tokens.len(), 3);
        assert_token_eq(&tokens[0], TokenKind::StartTag, "div");
        assert_token_eq(&tokens[1], TokenKind::Text, " Hello ");
        assert_token_eq(&tokens[2], TokenKind::EndTag, "div");
    }

    #[test]
    fn test_multiple_whitespace_normalization() {
        let mut tokenizer = HtmlTokenizer::new();
        let tokens = tokenizer.tokenize(b"<div>  Hello   World  </div>");

        assert_eq!(tokens.len(), 3);
        assert_token_eq(&tokens[0], TokenKind::StartTag, "div");
        assert_token_eq(&tokens[1], TokenKind::Text, " Hello World ");
        assert_token_eq(&tokens[2], TokenKind::EndTag, "div");
    }

    #[test]
    fn test_whitespace_only_text() {
        let mut tokenizer = HtmlTokenizer::new();
        let tokens = tokenizer.tokenize(b"<div>   </div>");

        assert_eq!(tokens.len(), 2);
        assert_token_eq(&tokens[0], TokenKind::StartTag, "div");
        assert_token_eq(&tokens[1], TokenKind::EndTag, "div");
    }

    #[test]
    fn test_newlines_and_whitespace() {
        let mut tokenizer = HtmlTokenizer::new();
        let tokens = tokenizer.tokenize(b"<div>\n  Hello\n  World\n</div>");

        assert_eq!(tokens.len(), 3);
        assert_token_eq(&tokens[0], TokenKind::StartTag, "div");
        assert_token_eq(&tokens[1], TokenKind::Text, "Hello World");
        assert_token_eq(&tokens[2], TokenKind::EndTag, "div");
    }

    #[test]
    fn test_attribute_with_spaces() {
        let mut tokenizer = HtmlTokenizer::new();
        let tokens = tokenizer.tokenize(b"<div class = \"container\" >");

        assert_eq!(tokens.len(), 1);
        assert_token_with_attrs(
            &tokens[0],
            TokenKind::StartTag,
            "div",
            vec![("class", "container")],
        );
    }

    #[test]
    fn test_self_closing_with_attributes() {
        let mut tokenizer = HtmlTokenizer::new();
        let tokens = tokenizer.tokenize(b"<img src=\"image.jpg\" alt=\"description\"/>");

        assert_eq!(tokens.len(), 1);
        assert_token_with_attrs(
            &tokens[0],
            TokenKind::StartTag,
            "img",
            vec![("src", "image.jpg"), ("alt", "description")],
        );
    }

    #[test]
    fn test_empty_attributes() {
        let mut tokenizer = HtmlTokenizer::new();
        let tokens = tokenizer.tokenize(b"<input type=\"\" value=\"\">");

        assert_eq!(tokens.len(), 1);
        assert_token_with_attrs(
            &tokens[0],
            TokenKind::StartTag,
            "input",
            vec![("type", ""), ("value", "")],
        );
    }

    #[test]
    fn test_malformed_comment() {
        let mut tokenizer = HtmlTokenizer::new();
        let tokens = tokenizer.tokenize(b"<!--- This is malformed --->");

        assert_eq!(tokens.len(), 1);
        assert_token_eq(&tokens[0], TokenKind::Comment, "- This is malformed ");
    }

    #[test]
    fn test_case_sensitivity() {
        let mut tokenizer = HtmlTokenizer::new();
        let tokens = tokenizer.tokenize(b"<DIV CLASS=\"Container\">Text</DIV>");

        assert_eq!(tokens.len(), 3);
        assert_token_with_attrs(
            &tokens[0],
            TokenKind::StartTag,
            "DIV",
            vec![("CLASS", "Container")],
        );
        assert_token_eq(&tokens[1], TokenKind::Text, "Text");
        assert_token_eq(&tokens[2], TokenKind::EndTag, "DIV");
    }

    #[test]
    fn test_mixed_quote_types() {
        let mut tokenizer = HtmlTokenizer::new();
        let tokens = tokenizer.tokenize(b"<input type='text' name=\"username\">");

        assert_eq!(tokens.len(), 1);
        assert_token_with_attrs(
            &tokens[0],
            TokenKind::StartTag,
            "input",
            vec![("type", "text"), ("name", "username")],
        );
    }

    #[test]
    fn test_complex_html_document() {
        let mut tokenizer = HtmlTokenizer::new();
        let html = b"<!DOCTYPE html><html><head><title>Test</title></head><body><h1>Hello</h1><p>World</p></body></html>";
        let tokens = tokenizer.tokenize(html);

        assert_eq!(tokens.len(), 16);
        assert_token_eq(&tokens[0], TokenKind::DoctypeDeclaration, "DOCTYPE html");
        assert_token_eq(&tokens[1], TokenKind::StartTag, "html");
        assert_token_eq(&tokens[2], TokenKind::StartTag, "head");
        assert_token_eq(&tokens[3], TokenKind::StartTag, "title");
        assert_token_eq(&tokens[4], TokenKind::Text, "Test");
        assert_token_eq(&tokens[5], TokenKind::EndTag, "title");
        assert_token_eq(&tokens[6], TokenKind::EndTag, "head");
        assert_token_eq(&tokens[7], TokenKind::StartTag, "body");
        assert_token_eq(&tokens[8], TokenKind::StartTag, "h1");
        assert_token_eq(&tokens[9], TokenKind::Text, "Hello");
        assert_token_eq(&tokens[10], TokenKind::EndTag, "h1");
        assert_token_eq(&tokens[11], TokenKind::StartTag, "p");
        assert_token_eq(&tokens[12], TokenKind::Text, "World");
        assert_token_eq(&tokens[13], TokenKind::EndTag, "p");
        assert_token_eq(&tokens[14], TokenKind::EndTag, "body");
        assert_token_eq(&tokens[15], TokenKind::EndTag, "html");
    }

    #[test]
    fn test_chunked_parsing() {
        let mut tokenizer = HtmlTokenizer::new();

        let mut all_tokens = Vec::new();
        all_tokens.extend(tokenizer.tokenize(b"<div"));
        all_tokens.extend(tokenizer.tokenize(b" class=\"test\""));
        all_tokens.extend(tokenizer.tokenize(b">Hello"));
        all_tokens.extend(tokenizer.tokenize(b"</div>"));

        assert_eq!(all_tokens.len(), 3);
        assert_token_with_attrs(
            &all_tokens[0],
            TokenKind::StartTag,
            "div",
            vec![("class", "test")],
        );
        assert_token_eq(&all_tokens[1], TokenKind::Text, "Hello");
        assert_token_eq(&all_tokens[2], TokenKind::EndTag, "div");
    }

    #[test]
    fn test_special_characters_in_text() {
        let mut tokenizer = HtmlTokenizer::new();
        let tokens = tokenizer.tokenize(b"<div>&amp; &lt; &gt;</div>");

        assert_eq!(tokens.len(), 3);
        assert_token_eq(&tokens[0], TokenKind::StartTag, "div");
        assert_token_eq(&tokens[1], TokenKind::Text, "&amp; &lt; &gt;");
        assert_token_eq(&tokens[2], TokenKind::EndTag, "div");
    }

    #[test]
    fn test_special_characters_in_attributes() {
        let mut tokenizer = HtmlTokenizer::new();
        let tokens = tokenizer.tokenize(b"<div title=\"&quot;quoted&quot;\">");

        assert_eq!(tokens.len(), 1);
        assert_token_with_attrs(
            &tokens[0],
            TokenKind::StartTag,
            "div",
            vec![("title", "&quot;quoted&quot;")],
        );
    }

    #[test]
    fn test_numeric_attributes() {
        let mut tokenizer = HtmlTokenizer::new();
        let tokens = tokenizer.tokenize(b"<input maxlength=10 tabindex=1>");

        assert_eq!(tokens.len(), 1);
        assert_token_with_attrs(
            &tokens[0],
            TokenKind::StartTag,
            "input",
            vec![("maxlength", "10"), ("tabindex", "1")],
        );
    }

    #[test]
    fn test_hyphenated_attributes() {
        let mut tokenizer = HtmlTokenizer::new();
        let tokens = tokenizer.tokenize(b"<div data-test=\"value\" aria-label=\"description\">");

        assert_eq!(tokens.len(), 1);
        assert_token_with_attrs(
            &tokens[0],
            TokenKind::StartTag,
            "div",
            vec![("data-test", "value"), ("aria-label", "description")],
        );
    }

    #[test]
    fn test_empty_tag() {
        let mut tokenizer = HtmlTokenizer::new();
        let tokens = tokenizer.tokenize(b"<>");

        let _len = tokens.len();
    }

    #[test]
    fn test_tag_with_equals_in_unquoted_value() {
        let mut tokenizer = HtmlTokenizer::new();
        let tokens = tokenizer.tokenize(b"<input value=test=123>");

        assert_eq!(tokens.len(), 1);
        assert_token_with_attrs(
            &tokens[0],
            TokenKind::StartTag,
            "input",
            vec![("value", "test=123")],
        );
    }

    #[test]
    fn test_deeply_nested_structure() {
        let mut tokenizer = HtmlTokenizer::new();
        let tokens = tokenizer.tokenize(b"<div><span><em><strong>Text</strong></em></span></div>");

        assert_eq!(tokens.len(), 9);
        assert_token_eq(&tokens[0], TokenKind::StartTag, "div");
        assert_token_eq(&tokens[1], TokenKind::StartTag, "span");
        assert_token_eq(&tokens[2], TokenKind::StartTag, "em");
        assert_token_eq(&tokens[3], TokenKind::StartTag, "strong");
        assert_token_eq(&tokens[4], TokenKind::Text, "Text");
        assert_token_eq(&tokens[5], TokenKind::EndTag, "strong");
        assert_token_eq(&tokens[6], TokenKind::EndTag, "em");
        assert_token_eq(&tokens[7], TokenKind::EndTag, "span");
        assert_token_eq(&tokens[8], TokenKind::EndTag, "div");
    }

    #[test]
    fn test_script_with_html_content() {
        let mut tokenizer = HtmlTokenizer::new();
        let tokens = tokenizer.tokenize(b"<script>console.log('hello');</script>");

        assert_eq!(tokens.len(), 3);
        assert_token_eq(&tokens[0], TokenKind::StartTag, "script");
        assert_token_eq(&tokens[1], TokenKind::Text, "console.log('hello');");
        assert_token_eq(&tokens[2], TokenKind::EndTag, "script");
    }

    #[test]
    fn test_comment_with_nested_tags() {
        let mut tokenizer = HtmlTokenizer::new();
        let tokens = tokenizer.tokenize(b"<!-- <div>This is commented out</div> -->");

        assert_eq!(tokens.len(), 1);
        assert_token_eq(
            &tokens[0],
            TokenKind::Comment,
            " <div>This is commented out</div> ",
        );
    }

    #[test]
    fn test_multiple_comments() {
        let mut tokenizer = HtmlTokenizer::new();
        let tokens = tokenizer.tokenize(b"<!-- First comment --><!-- Second comment -->");

        assert_eq!(tokens.len(), 2);
        assert_token_eq(&tokens[0], TokenKind::Comment, " First comment ");
        assert_token_eq(&tokens[1], TokenKind::Comment, " Second comment ");
    }

    #[test]
    fn test_text_between_tags() {
        let mut tokenizer = HtmlTokenizer::new();
        let tokens = tokenizer.tokenize(b"<p>Hello</p> world <span>!</span>");

        assert_eq!(tokens.len(), 7);
        assert_token_eq(&tokens[0], TokenKind::StartTag, "p");
        assert_token_eq(&tokens[1], TokenKind::Text, "Hello");
        assert_token_eq(&tokens[2], TokenKind::EndTag, "p");
        assert_token_eq(&tokens[3], TokenKind::Text, " world ");
        assert_token_eq(&tokens[4], TokenKind::StartTag, "span");
        assert_token_eq(&tokens[5], TokenKind::Text, "!");
        assert_token_eq(&tokens[6], TokenKind::EndTag, "span");
    }

    #[test]
    fn test_attributes_with_special_characters() {
        let mut tokenizer = HtmlTokenizer::new();
        let tokens = tokenizer.tokenize(b"<div data-test=\"value with spaces & symbols\">");

        assert_eq!(tokens.len(), 1);
        assert_token_with_attrs(
            &tokens[0],
            TokenKind::StartTag,
            "div",
            vec![("data-test", "value with spaces & symbols")],
        );
    }

    #[test]
    fn test_empty_comment() {
        let mut tokenizer = HtmlTokenizer::new();
        let tokens = tokenizer.tokenize(b"<!---->");

        assert_eq!(tokens.len(), 1);
        assert_token_eq(&tokens[0], TokenKind::Comment, "");
    }

    #[test]
    fn test_tag_with_slash_in_name() {
        let mut tokenizer = HtmlTokenizer::new();
        let tokens = tokenizer.tokenize(b"<test>");
        assert_eq!(tokens.len(), 1);
        assert_token_eq(&tokens[0], TokenKind::StartTag, "test");
    }

    #[test]
    fn test_realistic_html_form() {
        let mut tokenizer = HtmlTokenizer::new();
        let html = br#"<form action="/submit" method="post">
    <label for="name">Name:</label>
    <input type="text" id="name" name="name" required>
    <button type="submit">Submit</button>
</form>"#;
        let tokens = tokenizer.tokenize(html);

        assert_eq!(tokens.len(), 9);

        assert_token_with_attrs(
            &tokens[0],
            TokenKind::StartTag,
            "form",
            vec![("action", "/submit"), ("method", "post")],
        );

        let input_token = tokens
            .iter()
            .find(|t| t.kind == TokenKind::StartTag && t.data == "input")
            .unwrap();
        assert_eq!(
            input_token.attributes.get("type"),
            Some(&"text".to_string())
        );
        assert_eq!(
            input_token.attributes.get("required"),
            Some(&"".to_string())
        );
    }

    #[test]
    fn test_preformatted_multiline_text() {
        let mut tokenizer = HtmlTokenizer::new();
        let tokens = tokenizer
            .tokenize(b"<pre>  <span>Some Text</span>\r\n<span>A new line!</span>  </pre>");

        assert_eq!(tokens.len(), 11);
        assert_token_eq(&tokens[0], TokenKind::StartTag, "pre");
        assert_token_eq(&tokens[1], TokenKind::Text, "  ");
        assert_token_eq(&tokens[2], TokenKind::StartTag, "span");
        assert_token_eq(&tokens[3], TokenKind::Text, "Some Text");
        assert_token_eq(&tokens[4], TokenKind::EndTag, "span");
        assert_token_eq(&tokens[5], TokenKind::Text, "\r\n");
        assert_token_eq(&tokens[6], TokenKind::StartTag, "span");
        assert_token_eq(&tokens[7], TokenKind::Text, "A new line!");
        assert_token_eq(&tokens[8], TokenKind::EndTag, "span");
        assert_token_eq(&tokens[9], TokenKind::Text, "  ");
        assert_token_eq(&tokens[10], TokenKind::EndTag, "pre");
    }

    #[test]
    fn test_preformatted_text_code() {
        let mut tokenizer = HtmlTokenizer::new();
        let tokens = tokenizer
            .tokenize(b"<pre>  <code>\r\n// Sample code block\nfunction test() {\r\n    return \"Hello, World!\";\n}\r\n</code>  </pre>");

        assert_eq!(tokens.len(), 11);
        assert_token_eq(&tokens[0], TokenKind::StartTag, "pre");
        assert_token_eq(&tokens[1], TokenKind::Text, "  ");
        assert_token_eq(&tokens[2], TokenKind::StartTag, "code");
        assert_token_eq(&tokens[3], TokenKind::Text, "\r\n");
        assert_token_eq(&tokens[4], TokenKind::Text, "// Sample code block\n");
        assert_token_eq(&tokens[5], TokenKind::Text, "function test() {\r\n");
        assert_token_eq(
            &tokens[6],
            TokenKind::Text,
            "    return \"Hello, World!\";\n",
        );
        assert_token_eq(&tokens[7], TokenKind::Text, "}\r\n");
        assert_token_eq(&tokens[8], TokenKind::EndTag, "code");
        assert_token_eq(&tokens[9], TokenKind::Text, "  ");
        assert_token_eq(&tokens[10], TokenKind::EndTag, "pre");
    }
}

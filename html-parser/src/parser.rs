use logos::Logos;
use regex::Regex;
use shared_types::dom;
use shared_types::dom::{DomNode, Element};
use std::collections::HashMap;

use crate::decode::Decoder;
use crate::token::Token;

pub struct Parser<'input> {
    lexer: logos::Lexer<'input, Token>,
    current_token: Option<Token>,
    element_stack: Vec<Element>,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        let mut parser = Parser {
            lexer: Token::lexer(input),
            current_token: None,
            element_stack: Vec::new(),
        };

        parser.next_token(); // Initialize the first token
        parser
    }

    fn next_token(&mut self) -> Option<Token> {
        self.current_token = self.lexer.next().and_then(|token| token.ok());
        self.current_token.clone()
    }

    pub fn parse_document(mut self) -> Result<DomNode, String> {
        let mut document_children: Vec<DomNode> = Vec::new();

        fn is_void_element(tag_name: &str) -> bool {
            matches!(
                tag_name.to_lowercase().as_str(),
                "img"
                    | "br"
                    | "hr"
                    | "meta"
                    | "input"
                    | "area"
                    | "base"
                    | "col"
                    | "embed"
                    | "link"
                    | "param"
                    | "source"
                    | "track"
                    | "wbr"
            )
        }

        fn should_auto_close(current_tag: &str, new_tag: &str) -> bool {
            let current_lower = current_tag.to_lowercase();
            let new_lower = new_tag.to_lowercase();

            match current_lower.as_str() {
                "p" => {
                    // Automatically close <p> when encountering block-level elements
                    matches!(
                        new_lower.as_str(),
                        "div"
                            | "p"
                            | "h1"
                            | "h2"
                            | "h3"
                            | "h4"
                            | "h5"
                            | "h6"
                            | "ul"
                            | "ol"
                            | "li"
                            | "dl"
                            | "dt"
                            | "dd"
                            | "blockquote"
                            | "pre"
                            | "form"
                            | "table"
                            | "section"
                            | "article"
                            | "aside"
                            | "header"
                            | "footer"
                            | "nav"
                            | "main"
                            | "figure"
                            | "hr"
                    )
                }
                "li" => {
                    // Automatically close <li> when encountering another <li>
                    new_lower == "li"
                }
                "dd" | "dt" => {
                    // Automatically close <dd> or <dt> when encountering another <dd> or <dt>
                    matches!(new_lower.as_str(), "dd" | "dt")
                }
                "option" => {
                    // Automatically close <option> when encountering another <option> or <optgroup>
                    matches!(new_lower.as_str(), "option" | "optgroup")
                }
                "tr" => &new_lower == "tr",
                "td" | "th" => {
                    // Automatically close <td> or <th> when encountering another <td> or <th>
                    matches!(new_lower.as_str(), "td" | "th" | "tr")
                }
                _ => false,
            }
        }

        fn auto_close_elements(
            element_stack: &mut Vec<Element>,
            document_children: &mut Vec<DomNode>,
            new_tag: &str,
        ) {
            let mut elements_to_close: Vec<usize> = Vec::new();

            for (i, element) in element_stack.iter().enumerate().rev() {
                if should_auto_close(&element.tag_name, new_tag) {
                    elements_to_close.push(i);
                } else {
                    // Stop closing elements if we reach one that doesn't need to be closed
                    break;
                }
            }

            for &index in elements_to_close.iter().rev() {
                if let Some(element) = element_stack.get(index).cloned() {
                    element_stack.remove(index);

                    if element_stack.is_empty() {
                        document_children.push(DomNode::Element(element));
                    } else {
                        if let Some(parent_element) = element_stack.last_mut() {
                            parent_element.children.push(DomNode::Element(element));
                        } else {
                            // This case should ideally not be reached if the stack logic is correct,
                            // but it's a fallback for robustness.
                            document_children.push(DomNode::Element(element));
                        }
                    }
                }
            }
        }

        while let Some(token) = &self.current_token {
            match token {
                Token::Doctype => {
                    let doctype_declaration = self.extract_doctype_declaration(&self.lexer.slice());

                    let doctype_node = DomNode::Doctype(doctype_declaration);
                    document_children.push(doctype_node);
                    self.next_token();
                }
                Token::XmlDeclaration => {
                    let xml_declaration = self.extract_xml_declaration(&self.lexer.slice());
                    let xml_node = DomNode::XmlDeclaration(xml_declaration);
                    document_children.push(xml_node);
                    self.next_token();
                }

                Token::StartTag | Token::StartTagWithAttributes => {
                    let (tag_name, attributes) = if matches!(token, Token::StartTagWithAttributes) {
                        self.extract_tag_name_and_attributes(&self.lexer.slice())
                    } else {
                        (self.extract_tag_name(&self.lexer.slice()), HashMap::new())
                    };

                    auto_close_elements(&mut self.element_stack, &mut document_children, tag_name);

                    let element = Element {
                        tag_name: tag_name.to_string(),
                        attributes,
                        children: Vec::new(),
                    };

                    if is_void_element(&element.tag_name) {
                        if let Some(last_element) = self.element_stack.last_mut() {
                            last_element.children.push(DomNode::Element(element));
                        } else {
                            document_children.push(DomNode::Element(element));
                        }
                    } else {
                        self.element_stack.push(element);
                    }

                    self.next_token();
                }

                Token::EndTag => {
                    let tag_name = self.extract_tag_name(&self.lexer.slice());

                    if is_void_element(tag_name) {
                        eprintln!(
                            "Ignoring closing tag </{}> for void element. Void elements should not have closing tags.",
                            tag_name
                        );
                        self.next_token();
                        continue;
                    }

                    let mut completed_element: Option<Element> = None;

                    while let Some(element) = self.element_stack.pop() {
                        if element.tag_name == tag_name {
                            completed_element = Some(element);
                            break;
                        }

                        eprintln!(
                            "Mismatched end tag: expected </{}>, found closing </{}>. Implicitly closing <{}>.",
                            tag_name,
                            self.element_stack.last().map_or("N/A", |e| &e.tag_name),
                            element.tag_name
                        );
                        if let Some(last_element) = self.element_stack.last_mut() {
                            last_element.children.push(DomNode::Element(element));
                        } else {
                            document_children.push(DomNode::Element(element));
                        }
                    }

                    if let Some(element) = completed_element {
                        if self.element_stack.is_empty() {
                            document_children.push(DomNode::Element(element));
                        } else {
                            if let Some(parent_element) = self.element_stack.last_mut() {
                                parent_element.children.push(DomNode::Element(element));
                            } else {
                                // This case should ideally not be reached if the stack logic is correct,
                                // but it's a fallback for robustness.
                                document_children.push(DomNode::Element(element));
                            }
                        }
                    } else {
                        eprintln!(
                            "Unmatched end tag: </{}>. No corresponding start tag found.",
                            tag_name
                        );
                    }

                    self.next_token();
                }

                Token::Text => {
                    let text_content = self.lexer.slice().to_string();

                    // Handle decoding HTML entities
                    let decoder = Decoder::new(&text_content);
                    let text_content = decoder.decode().unwrap_or_else(|err| {
                        eprintln!("Error decoding text: {}", err);
                        text_content
                    });

                    if let Some(last_element) = self.element_stack.last_mut() {
                        last_element.children.push(DomNode::Text(text_content));
                    } else {
                        document_children.push(DomNode::Text(text_content));
                    }
                    self.next_token();
                }

                Token::Comment => {
                    let comment_content = self
                        .lexer
                        .slice()
                        .trim_start_matches("<!--")
                        .trim_end_matches("-->")
                        .trim()
                        .to_string();
                    let comment_node = DomNode::Comment(comment_content);
                    if let Some(last_element) = self.element_stack.last_mut() {
                        last_element.children.push(comment_node);
                    } else {
                        document_children.push(comment_node);
                    }
                    self.next_token();
                }

                _ => {
                    let tag_name = self.extract_tag_name(&self.lexer.slice());
                    println!("Unexpected tag: {:?}", tag_name);
                    self.next_token();
                }
            }
        }

        while let Some(element) = self.element_stack.pop() {
            let dom_node = DomNode::Element(element);
            document_children.push(dom_node);
        }

        Ok(DomNode::Document(document_children))
    }

    fn extract_doctype_declaration<'b>(&self, tag_slice: &'b str) -> dom::DoctypeDeclaration {
        let doctype_regex = Regex::new(
            r#"<!DOCTYPE\s+([a-zA-Z0-9]+)(?:\s+PUBLIC\s+\"([^\"]*)\"\s+\"([^\"]*)\")?>"#,
        )
        .unwrap();
        if let Some(caps) = doctype_regex.captures(tag_slice) {
            let name = caps.get(1).map_or("", |m| m.as_str()).to_string();
            let public_id = caps.get(2).map(|m| m.as_str().to_string());
            let system_id = caps.get(3).map(|m| m.as_str().to_string());

            dom::DoctypeDeclaration {
                name,
                public_id,
                system_id,
            }
        } else {
            dom::DoctypeDeclaration {
                name: String::new(),
                public_id: None,
                system_id: None,
            }
        }
    }

    fn extract_xml_declaration<'b>(&self, tag_slice: &'b str) -> dom::XmlDeclaration {
        let xml_regex = Regex::new(r#"^<\?xml\s+version=\"([^\"]+)\"(?:\s+encoding=\"([^\"]+)\")?(?:\s+standalone=\"(yes|no)\")?\s*\?>"#).unwrap();
        if let Some(caps) = xml_regex.captures(tag_slice) {
            let version = caps.get(1).map_or("", |m| m.as_str()).to_string();
            let encoding = caps.get(2).map(|m| m.as_str().to_string());
            let standalone = caps.get(3).map(|m| m.as_str() == "yes");

            dom::XmlDeclaration {
                version,
                encoding,
                standalone,
            }
        } else {
            dom::XmlDeclaration {
                version: String::new(),
                encoding: None,
                standalone: None,
            }
        }
    }

    fn extract_tag_name_and_attributes<'b>(
        &self,
        tag_slice: &'b str,
    ) -> (&'b str, HashMap<String, String>) {
        let tag_name = self.extract_tag_name(tag_slice);
        let attributes = self.extract_attributes(tag_slice);
        (tag_name, attributes)
    }

    fn extract_attributes<'b>(&self, tag_slice: &'b str) -> HashMap<String, String> {
        let mut attributes = HashMap::new();
        // Regex for attributes with values: name="value"
        let attr_regex = Regex::new(r#"([a-zA-Z][a-zA-Z0-9-]*)\s*=\s*\"([^\"]*)\""#).unwrap();
        // Regex for boolean attributes without values: hidden, disabled, etc.
        let bool_attr_regex = Regex::new(r#"\s+([a-zA-Z][a-zA-Z0-9-]*)(\s|>)"#).unwrap();

        // Process attributes with values
        for cap in attr_regex.captures_iter(tag_slice) {
            if let (Some(name), Some(value)) = (cap.get(1), cap.get(2)) {
                attributes.insert(name.as_str().to_string(), value.as_str().to_string());
            }
        }

        // Process boolean attributes without values
        for cap in bool_attr_regex.captures_iter(tag_slice) {
            if let Some(name) = cap.get(1) {
                let attr_name = name.as_str().to_string();
                // Skip if already processed by first regex
                if !attributes.contains_key(&attr_name) {
                    attributes.insert(attr_name, "".to_string());
                }
            }
        }

        attributes
    }

    fn extract_tag_name<'b>(&self, tag_slice: &'b str) -> &'b str {
        let tag_name_end = tag_slice
            .find(|c: char| c.is_whitespace() || c == '>')
            .unwrap_or(tag_slice.len());
        let tag_name_start = if tag_slice.starts_with("</") {
            2 // Skip "</"
        } else {
            1 // Skip "<"
        };

        &tag_slice[tag_name_start..tag_name_end]
    }
}

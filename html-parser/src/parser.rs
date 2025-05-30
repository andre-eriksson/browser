use logos::Logos;
use shared_types::dom::{DomNode, Element};
use std::collections::HashMap;

use crate::decode::Decoder;
use crate::extractors::attributes::extract_attributes;
use crate::extractors::declaration::{extract_doctype_declaration, extract_xml_declaration};
use crate::extractors::tags::extract_tag_name;
use crate::patterns::init_regexes;
use crate::rules::auto_close::auto_close_elements;
use crate::rules::void_elements::is_void_element;
use crate::token::Token;

pub struct Parser<'input> {
    lexer: logos::Lexer<'input, Token>,
    current_token: Option<Token>,
    element_stack: Vec<Element>,
    max_depth: usize,
    current_depth: usize,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str, max_depth: Option<usize>) -> Self {
        let mut parser = Parser {
            lexer: Token::lexer(input),
            current_token: None,
            element_stack: Vec::new(),
            max_depth: max_depth.unwrap_or(1000), // Default max depth if not specified
            current_depth: 0,
        };

        parser.next_token(); // Initialize the first token
        parser
    }

    pub fn parse_document(&mut self) -> Result<DomNode, String> {
        let mut document_children: Vec<DomNode> = Vec::new();
        init_regexes();

        while let Some(token) = self.current_token.clone() {
            match token {
                Token::Doctype => {
                    let doctype_declaration = extract_doctype_declaration(&self.lexer.slice());

                    let doctype_node = DomNode::Doctype(doctype_declaration);
                    document_children.push(doctype_node);
                    self.next_token();
                }
                Token::XmlDeclaration => {
                    let xml_declaration = extract_xml_declaration(&self.lexer.slice());
                    let xml_node = DomNode::XmlDeclaration(xml_declaration);
                    document_children.push(xml_node);
                    self.next_token();
                }

                Token::StartTag | Token::StartTagWithAttributes => {
                    self.handle_start_tag(&token, &mut document_children)
                        .map_err(|e| format!("Error handling start tag: {}", e))?;

                    self.next_token();
                }

                Token::EndTag => {
                    let tag_name = extract_tag_name(&self.lexer.slice());

                    if is_void_element(tag_name) {
                        //eprintln!(
                        //    "Ignoring closing tag </{}> for void element. Void elements should not have closing tags.",
                        //    tag_name
                        //);
                        self.next_token();
                        continue;
                    }

                    self.handle_end_tag(tag_name, &mut document_children);

                    self.next_token();
                }

                Token::Text => {
                    let text_content = self.lexer.slice().to_string();

                    self.handle_text(&text_content, &mut document_children)
                        .map_err(|e| format!("Error handling text: {}", e))?;

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
                    let tag_name = extract_tag_name(&self.lexer.slice());
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

    fn next_token(&mut self) -> Option<Token> {
        self.current_token = self.lexer.next().and_then(|token| token.ok());
        self.current_token.clone()
    }

    fn push_element(&mut self, element: Element) -> Result<(), String> {
        if self.current_depth >= self.max_depth {
            return Err(format!(
                "Maximum depth of {} exceeded while parsing HTML.",
                self.max_depth
            ));
        }
        self.element_stack.push(element);
        self.current_depth += 1;
        Ok(())
    }

    fn handle_start_tag(
        &mut self,
        token: &Token,
        document_children: &mut Vec<DomNode>,
    ) -> Result<(), String> {
        let (tag_name, attributes) = if matches!(token, Token::StartTagWithAttributes) {
            let attributes = extract_attributes(self.lexer.slice());
            (extract_tag_name(&self.lexer.slice()), attributes)
        } else {
            (extract_tag_name(&self.lexer.slice()), HashMap::new())
        };

        auto_close_elements(&mut self.element_stack, document_children, tag_name);

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
            self.push_element(element)?;
        }

        Ok(())
    }

    fn handle_end_tag(&mut self, tag_name: &str, document_children: &mut Vec<DomNode>) {
        let mut completed_element: Option<Element> = None;

        while let Some(element) = self.element_stack.pop() {
            if element.tag_name == tag_name {
                completed_element = Some(element);
                break;
            }

            //eprintln!(
            //    "Mismatched end tag: expected </{}>, found closing </{}>. Implicitly closing <{}>.",
            //    tag_name,
            //    self.element_stack.last().map_or("N/A", |e| &e.tag_name),
            //    element.tag_name
            //);
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
    }

    fn handle_text(
        &mut self,
        text_content: &str,
        document_children: &mut Vec<DomNode>,
    ) -> Result<(), String> {
        // Handle decoding HTML entities
        let decoder = Decoder::new(&text_content);
        let text_content = decoder.decode().unwrap_or_else(|err| {
            eprintln!("Error decoding text: {}", err);
            text_content.to_string()
        });

        if let Some(last_element) = self.element_stack.last_mut() {
            last_element.children.push(DomNode::Text(text_content));
        } else {
            document_children.push(DomNode::Text(text_content));
        }

        Ok(())
    }
}

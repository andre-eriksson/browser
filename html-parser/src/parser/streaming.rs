use logos::Logos;
use std::{cell::RefCell, io::BufRead, rc::Rc};

use shared_types::dom::{DomNode, Element, SharedDomNode};

use crate::{
    decode::Decoder,
    extractors::{
        attributes::extract_attributes,
        declaration::{extract_doctype_declaration, extract_xml_declaration},
        tags::extract_tag_name,
    },
    rules::{auto_close::should_auto_close, void_elements::is_void_element},
    token::Token,
};

pub struct StreamingParser<R: BufRead> {
    reader: R,
    buffer: String,
    buffer_size: usize,
    dom_tree: Vec<SharedDomNode>,
    open_elements: Vec<SharedDomNode>,
}

impl<R: BufRead> StreamingParser<R> {
    pub fn new(reader: R, buffer_size: Option<usize>) -> Self {
        let buffer_size = buffer_size.unwrap_or(1024 * 8);
        Self {
            reader,
            buffer: String::with_capacity(buffer_size),
            buffer_size: buffer_size,
            dom_tree: Vec::new(),
            open_elements: Vec::new(),
        }
    }

    pub fn parse(&mut self) -> Result<DomNode, String> {
        let mut buf = vec![0u8; self.buffer_size];

        while let Ok(bytes_read) = self.reader.read(&mut buf) {
            if bytes_read == 0 {
                break; // EOF
            }

            // Convert bytes to string and append to buffer
            let chunk = str::from_utf8(&buf[..bytes_read])
                .map_err(|e| format!("Invalid UTF-8 sequence: {}", e))?;

            // Prepend the buffer to the chunk
            let full_chunk = format!("{}{}", self.buffer, chunk);
            self.buffer.clear();

            self.process_chunk(&full_chunk)?;
        }

        if !self.open_elements.is_empty() {
            return Err(format!(
                "Unclosed elements: {}",
                self.open_elements
                    .iter()
                    .map(|node| {
                        if let DomNode::Element(ref element) = *node.borrow() {
                            element.tag_name.clone()
                        } else {
                            "Unknown".to_string()
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }

        Ok(DomNode::Document(self.dom_tree.clone()))
    }

    fn process_chunk(&mut self, chunk: &str) -> Result<(), String> {
        let mut lexer = Token::lexer(chunk);

        while let Some(token) = lexer.next() {
            match token {
                Ok(Token::Doctype) => {
                    let doctype_declaration = extract_doctype_declaration(lexer.slice());

                    self.dom_tree
                        .push(Rc::new(RefCell::new(DomNode::Doctype(doctype_declaration))));
                }

                Ok(Token::XmlDeclaration) => {
                    let xml_declaration = extract_xml_declaration(lexer.slice());

                    self.dom_tree
                        .push(Rc::new(RefCell::new(DomNode::XmlDeclaration(
                            xml_declaration,
                        ))));
                }

                Ok(Token::StartTag) | Ok(Token::StartTagWithAttributes) => {
                    let inside_script = if let Some(last_open) = self.open_elements.last() {
                        if let DomNode::Element(ref last_element) = *last_open.borrow() {
                            last_element.tag_name.eq_ignore_ascii_case("script")
                        } else {
                            false
                        }
                    } else {
                        false
                    };

                    if inside_script {
                        continue;
                    }

                    let tag_name = extract_tag_name(lexer.slice());
                    let attributes = extract_attributes(lexer.slice());

                    let element = Element {
                        tag_name: tag_name.to_string(),
                        attributes,
                        children: Vec::new(),
                    };

                    let new_node = Rc::new(RefCell::new(DomNode::Element(element)));

                    let should_close = if let Some(last_open) = self.open_elements.last() {
                        if let DomNode::Element(ref last_element) = *last_open.borrow() {
                            should_auto_close(&last_element.tag_name, &tag_name)
                        } else {
                            false
                        }
                    } else {
                        false
                    };

                    if should_close {
                        self.open_elements.pop();
                    }

                    if let Some(parent) = self.open_elements.last() {
                        if let DomNode::Element(ref mut element) = *parent.borrow_mut() {
                            element.children.push(Rc::clone(&new_node));
                        }
                    } else {
                        self.dom_tree.push(Rc::clone(&new_node));
                    }

                    if !is_void_element(&tag_name) {
                        self.open_elements.push(Rc::clone(&new_node));
                    }
                }

                Ok(Token::EndTag) => {
                    let tag_name = extract_tag_name(lexer.slice());
                    let inside_script = if let Some(last_open) = self.open_elements.last() {
                        if let DomNode::Element(ref last_element) = *last_open.borrow() {
                            last_element.tag_name.eq_ignore_ascii_case("script")
                        } else {
                            false
                        }
                    } else {
                        false
                    };

                    if inside_script && !tag_name.eq_ignore_ascii_case("script") {
                        continue;
                    }

                    if is_void_element(&tag_name) {
                        continue;
                    }

                    if let Some(last_open) = self.open_elements.pop() {
                        if let DomNode::Element(ref element) = *last_open.borrow() {
                            if element.tag_name == tag_name {
                                continue;
                            }

                            if !is_void_element(&tag_name) {
                                self.open_elements.pop();
                            }
                        } else {
                            return Err("Expected an element to close".to_string());
                        }
                    } else {
                        return Err(format!("Unexpected end tag: </{}>", tag_name));
                    }
                }

                Ok(Token::Text) => {
                    let mut text_content = lexer.slice().to_string();
                    if !text_content.trim().is_empty() {
                        if text_content.contains('&') {
                            let decoder = Decoder::new(text_content.as_str());
                            text_content = decoder
                                .decode()
                                .map_err(|e| format!("Decoding error: {}", e))?;
                        }

                        let text_node =
                            Rc::new(RefCell::new(DomNode::Text(text_content.trim().to_string())));
                        if let Some(parent) = self.open_elements.last() {
                            if let DomNode::Element(ref mut element) = *parent.borrow_mut() {
                                element.children.push(Rc::clone(&text_node));
                            }
                        } else {
                            self.dom_tree.push(Rc::clone(&text_node));
                        }
                    }
                }

                Ok(Token::Unknown) => {
                    self.buffer.push_str(lexer.slice());
                }

                Ok(Token::Comment) => {
                    // No need to handle comments currently
                }

                Err(_) => {
                    let slice = lexer.slice();

                    self.buffer.push_str(slice);
                    continue;
                }
            }
        }

        Ok(())
    }
}

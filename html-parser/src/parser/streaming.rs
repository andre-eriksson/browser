use logos::Logos;
use std::{cell::RefCell, collections::HashMap, io::BufRead, rc::Rc};

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

use super::options::{ParseMetadata, ParserOptions};

pub struct ParseResult {
    pub dom_tree: Vec<SharedDomNode>,
    pub metadata: Option<ParseMetadata>,
}

pub struct StreamingParser<R: BufRead> {
    reader: R,
    buffer: String,
    buffer_size: usize,
    dom_tree: Vec<SharedDomNode>,
    open_elements: Vec<SharedDomNode>,
    options: Option<ParserOptions>,
    byte_buffer: Vec<u8>, // Buffer for incomplete UTF-8 sequences
}

impl<R: BufRead> StreamingParser<R> {
    pub fn new_with_options(reader: R, buffer_size: Option<usize>, options: ParserOptions) -> Self {
        let buffer_size = buffer_size.unwrap_or(1024 * 8);
        let dom_tree = Vec::new();
        let open_elements = Vec::new();

        Self {
            reader,
            buffer: String::with_capacity(buffer_size),
            buffer_size,
            dom_tree,
            open_elements,
            options: Some(options),
            byte_buffer: Vec::new(),
        }
    }
    pub fn new(reader: R, buffer_size: Option<usize>) -> Self {
        let buffer_size = buffer_size.unwrap_or(1024 * 8);
        Self {
            reader,
            buffer: String::with_capacity(buffer_size),
            buffer_size: buffer_size,
            dom_tree: Vec::new(),
            open_elements: Vec::new(),
            options: Some(ParserOptions::default()),
            byte_buffer: Vec::new(),
        }
    }

    pub fn parse(&mut self) -> Result<ParseResult, String> {
        let mut buf = vec![0u8; self.buffer_size];
        let mut id_map: Option<HashMap<String, SharedDomNode>> = self
            .options
            .as_ref()
            .filter(|opts| opts.collect_ids)
            .map(|_| HashMap::new());
        let mut class_map: Option<HashMap<String, Vec<SharedDomNode>>> = self
            .options
            .as_ref()
            .filter(|opts| opts.collect_classes)
            .map(|_| HashMap::new());

        while let Ok(bytes_read) = self.reader.read(&mut buf) {
            if bytes_read == 0 {
                break; // EOF
            }

            // Combine any leftover bytes from previous chunk with new data
            let mut combined_bytes = self.byte_buffer.clone();
            combined_bytes.extend_from_slice(&buf[..bytes_read]);

            // Try to convert to UTF-8, handling incomplete sequences
            let (chunk, remaining_bytes) = match self.try_decode_utf8(&combined_bytes) {
                Ok((text, remaining)) => (text, remaining),
                Err(e) => return Err(e),
            };

            // Store any incomplete bytes for the next iteration
            self.byte_buffer = remaining_bytes;

            if !chunk.is_empty() {
                // Prepend the string buffer to the chunk
                let full_chunk = format!("{}{}", self.buffer, chunk);
                self.buffer.clear();

                self.process_chunk(&full_chunk, &mut id_map, &mut class_map)?;
            }
        }

        // Handle any remaining bytes at EOF
        if !self.byte_buffer.is_empty() {
            let remaining_text = String::from_utf8_lossy(&self.byte_buffer);
            if !remaining_text.is_empty() {
                let full_chunk = format!("{}{}", self.buffer, remaining_text);
                self.buffer.clear();
                self.process_chunk(&full_chunk, &mut id_map, &mut class_map)?;
            }
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

        Ok(ParseResult {
            dom_tree: vec![Rc::new(RefCell::new(DomNode::Document(
                self.dom_tree.clone(),
            )))],
            metadata: if let Some(options) = &self.options {
                Some(ParseMetadata {
                    id_map,
                    class_map: if options.collect_classes {
                        class_map
                    } else {
                        None
                    },
                })
            } else {
                None
            },
        })
    }

    fn process_chunk(
        &mut self,
        chunk: &str,
        id_map: &mut Option<HashMap<String, SharedDomNode>>,
        class_map: &mut Option<HashMap<String, Vec<SharedDomNode>>>,
    ) -> Result<(), String> {
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

                    let slice = lexer.slice();
                    let tag_name = extract_tag_name(slice);
                    let mut attributes = extract_attributes(slice);
                    attributes.remove(tag_name);

                    let id: String = attributes
                        .get("id")
                        .cloned()
                        .unwrap_or_else(|| String::new());
                    let class_names = attributes
                        .get("class")
                        .cloned()
                        .map(|s| s.split_whitespace().map(String::from).collect::<Vec<_>>())
                        .unwrap_or_else(Vec::new);

                    let element = Element {
                        tag_name: tag_name.to_string(),
                        attributes,
                        children: Vec::new(),
                    };

                    let new_node = Rc::new(RefCell::new(DomNode::Element(element)));

                    if let Some(id_map) = id_map {
                        if !id.is_empty() {
                            id_map.insert(id, Rc::clone(&new_node));
                        }
                    }

                    if let Some(class_map) = class_map {
                        for class in class_names {
                            class_map
                                .entry(class)
                                .or_insert_with(Vec::new)
                                .push(Rc::clone(&new_node));
                        }
                    }

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

    fn try_decode_utf8(&self, bytes: &[u8]) -> Result<(String, Vec<u8>), String> {
        match str::from_utf8(bytes) {
            Ok(text) => Ok((text.to_string(), Vec::new())),
            Err(error) => {
                let valid_up_to = error.valid_up_to();

                if valid_up_to == 0 && bytes.len() < 4 {
                    // Might be an incomplete sequence at the start, keep all bytes
                    return Ok((String::new(), bytes.to_vec()));
                }

                // We have some valid UTF-8, decode up to the error point
                let valid_text = str::from_utf8(&bytes[..valid_up_to])
                    .map_err(|e| format!("Unexpected UTF-8 error: {}", e))?;

                // Check if we have an incomplete sequence at the end
                let remaining_bytes = &bytes[valid_up_to..];

                if remaining_bytes.len() < 4 && self.could_be_incomplete_utf8(remaining_bytes) {
                    // Keep the incomplete bytes for next chunk
                    Ok((valid_text.to_string(), remaining_bytes.to_vec()))
                } else {
                    // Invalid UTF-8 sequence, use replacement character
                    let mut result = valid_text.to_string();
                    result.push('�'); // U+FFFD replacement character

                    // Skip the invalid byte(s) and continue with remaining
                    let skip_bytes = error.error_len().unwrap_or(1);
                    let remaining = if valid_up_to + skip_bytes < bytes.len() {
                        bytes[valid_up_to + skip_bytes..].to_vec()
                    } else {
                        Vec::new()
                    };

                    Ok((result, remaining))
                }
            }
        }
    }

    fn could_be_incomplete_utf8(&self, bytes: &[u8]) -> bool {
        if bytes.is_empty() {
            return false;
        }

        let first_byte = bytes[0];

        // Check if this could be the start of a multi-byte sequence
        if first_byte & 0x80 == 0 {
            false
        } else if first_byte & 0xE0 == 0xC0 {
            bytes.len() < 2
        } else if first_byte & 0xF0 == 0xE0 {
            bytes.len() < 3
        } else if first_byte & 0xF8 == 0xF0 {
            bytes.len() < 4
        } else {
            false
        }
    }
}

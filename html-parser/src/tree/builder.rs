use std::{cell::RefCell, rc::Rc};

use crate::{
    tokens::token::Token,
    tree::{
        decode::Decoder,
        extractors::{
            attributes::extract_attributes,
            declaration::{extract_doctype_declaration, extract_xml_declaration},
            tags::extract_tag_name,
        },
        rules::{auto_close::should_auto_close, void_elements::is_void_element},
    },
};
use api::{
    collector::{Collector, TagInfo},
    dom::{DomNode, Element, SharedDomNode},
};

/// Represents a malformed partial tag that could not be parsed correctly.
///
/// # Fields
/// * `tag_name` - The name of the tag that was malformed.
/// * `buffer` - The content of the tag that was not parsed correctly, which may contain incomplete or invalid HTML.
pub struct MalformedPartial {
    buffer: String,
}

/// A builder for constructing a DOM tree from HTML tokens.
///
/// # Type Parameters
/// `C` - The type of the collector used to gather metadata during parsing, which must implement the `Collector` trait.
///
/// # Fields
/// * `collector` - An instance of the collector used to gather metadata during parsing.
/// * `dom_tree` - A vector of shared DOM nodes representing the parsed document structure.
/// * `open_elements` - A stack of currently open elements, used to manage the hierarchy of the DOM tree.
pub struct DomTreeBuilder<C: Collector> {
    collector: C,
    pub dom_tree: Vec<SharedDomNode>,
    open_elements: Vec<SharedDomNode>,
    pub pending_malformed_tag: Option<MalformedPartial>,
}

impl<C: Collector + Default> DomTreeBuilder<C> {
    /// Creates a new `DomTreeBuilder` instance with a default collector.
    ///
    /// # Returns
    /// A new instance of `DomTreeBuilder` initialized with an empty DOM tree and no open elements.
    pub fn new() -> Self {
        DomTreeBuilder {
            collector: C::default(),
            dom_tree: Vec::new(),
            open_elements: Vec::new(),
            pending_malformed_tag: None,
        }
    }

    fn close_matching_element(&mut self, tag_name: &str) {
        // Handle the case where the last open element is not the one we are closing, for example:
        // </div> when the last open element is <span> or other standalone closing tags.
        if let Some(last_open) = self.open_elements.last() {
            if let DomNode::Element(ref element) = *last_open.borrow() {
                if element.tag_name != tag_name {
                    return;
                }
            } else {
                eprintln!(
                    "Warning: Expected an element node for end tag </{}>, found {:?}",
                    tag_name,
                    last_open.borrow()
                );
            }
        }

        // If the last open element matches the tag name, pop it from the stack.
        if let Some(last_open) = self.open_elements.pop() {
            if let DomNode::Element(ref element) = *last_open.borrow() {
                if element.tag_name == tag_name {
                    return;
                }
            } else {
                eprintln!(
                    "Warning: Expected an element node for end tag </{}>, found {:?}",
                    tag_name,
                    last_open.borrow()
                );
            }
        } else {
            eprintln!(
                "Warning: Unmatched end tag </{}> found, no open elements to match",
                tag_name
            );
        }
    }

    fn is_inside_script_tag(&self) -> bool {
        if let Some(last_open) = self.open_elements.last() {
            if let DomNode::Element(ref last_element) = *last_open.borrow() {
                return last_element.tag_name.eq_ignore_ascii_case("script");
            }
        }
        false
    }

    fn collect_tag_info(
        &mut self,
        tag_name: &str,
        attributes: &std::collections::HashMap<String, String>,
        dom_node: &SharedDomNode,
    ) {
        self.collector.collect(&TagInfo {
            tag_name,
            attributes,
            dom_node,
        });
    }

    fn should_auto_close(&self, tag_name: &str) -> bool {
        if let Some(last_open) = self.open_elements.last() {
            if let DomNode::Element(ref last_element) = *last_open.borrow() {
                return should_auto_close(&last_element.tag_name, tag_name);
            }
        }
        false
    }

    fn insert_new_node(&mut self, new_node: SharedDomNode) {
        if let Some(parent) = self.open_elements.last() {
            if let DomNode::Element(ref mut element) = *parent.borrow_mut() {
                element.children.push(Rc::clone(&new_node));
            }
        } else {
            self.dom_tree.push(Rc::clone(&new_node));
        }
    }

    fn handle_doctype_tag(&mut self, content: &str) {
        let doctype_declaration = extract_doctype_declaration(content);
        self.dom_tree
            .push(Rc::new(RefCell::new(DomNode::Doctype(doctype_declaration))));
    }

    fn handle_xml_declaration(&mut self, content: &str) {
        let xml_declaration = extract_xml_declaration(content);
        self.dom_tree
            .push(Rc::new(RefCell::new(DomNode::XmlDeclaration(
                xml_declaration,
            ))));
    }

    fn handle_start_tag(&mut self, content: &str) {
        if self.is_inside_script_tag() {
            return; // Ignore start tags inside script tags
        }

        let tag_name = extract_tag_name(content);
        let mut attributes = extract_attributes(content);
        attributes.remove(tag_name);
        let cloned_attributes = attributes.clone();

        let element = Element {
            tag_name: tag_name.to_string(),
            attributes,
            children: Vec::new(),
        };

        let new_node = Rc::new(RefCell::new(DomNode::Element(element)));

        self.collect_tag_info(&tag_name, &cloned_attributes, &new_node);

        if self.should_auto_close(&tag_name) {
            self.open_elements.pop();
        }

        self.insert_new_node(Rc::clone(&new_node));

        if !is_void_element(&tag_name) {
            self.open_elements.push(Rc::clone(&new_node));
        }
    }

    fn handle_end_tag(&mut self, content: &str) {
        let tag_name = extract_tag_name(content);

        // If we are inside a script tag, ignore end tags that are not for the script tag
        if self.is_inside_script_tag() && !tag_name.eq_ignore_ascii_case("script") {
            return;
        }

        if is_void_element(&tag_name) {
            return; // Void elements do not have end tags
        }

        self.close_matching_element(&tag_name);
    }

    fn handle_text_content(&mut self, content: &str) {
        let original_text_content = content.to_string();
        let trimmed_original = original_text_content.trim();

        if !trimmed_original.is_empty() {
            let mut processed_text_content = trimmed_original.to_string();

            if original_text_content.contains('&') {
                let decoder = Decoder::new(&original_text_content);

                match decoder.decode() {
                    Ok(decoded_text) => {
                        let final_decoded_trimmed = decoded_text.trim();
                        if !final_decoded_trimmed.is_empty() {
                            processed_text_content = final_decoded_trimmed.to_string();
                        } else {
                            // If decoding + trimming results in empty, skip adding node
                            return;
                        }
                    }
                    Err(e) => {
                        eprintln!("Error decoding text content: {}", e);
                        processed_text_content = trimmed_original.to_string();
                    }
                }
            }

            if processed_text_content.is_empty() {
                return; // Skip empty text nodes
            }

            let text_node = Rc::new(RefCell::new(DomNode::Text(processed_text_content)));
            if let Some(parent) = self.open_elements.last() {
                if let DomNode::Element(ref mut element) = *parent.borrow_mut() {
                    element.children.push(Rc::clone(&text_node));
                }
            } else {
                self.dom_tree.push(Rc::clone(&text_node));
            }
        }
    }

    /// Handles a malformed tag by buffering its content until it can be processed.
    /// Will assign `pending_malformed_tag` if it is not already set. Call `build_malformed` to process in the next iteration.
    ///
    /// # Arguments
    /// * `content` - A string slice containing the content of the malformed tag.
    fn handle_malformed_tag(&mut self, content: &str) {
        // If there is a pending malformed tag, append the content to its buffer
        if let Some(ref mut malformed) = self.pending_malformed_tag {
            malformed.buffer.push_str(content);
            return; // Do not process further, just buffer the content
        }

        // If no pending malformed tag, create a new one and buffer the content
        self.pending_malformed_tag = Some(MalformedPartial {
            buffer: content.to_string(),
        });
    }

    fn handle_comment(&mut self, _content: &str) {
        // Currently, comments are not processed, but this method can be extended in the future
    }

    /// Builds the DOM tree from a vector of HTML tokens.
    ///
    /// # Arguments
    /// * `html_tokens` - A vector of tuples containing a `Token` and its associated content as a string slice.
    ///
    /// # Returns
    /// A string of remaining content that was not processed as tokens, if any.
    pub fn build(&mut self, html_tokens: Vec<(Token, &str)>) {
        if let Some(ref mut malformed) = self.pending_malformed_tag {
            panic!(
                "Malformed content found, Use `build_malformed` instead of `build` to process it\nBuffer: {}",
                malformed.buffer
            );
        }

        for html_token in html_tokens {
            let token = html_token.0;
            let content = html_token.1;

            match token {
                Token::Doctype => self.handle_doctype_tag(content),
                Token::XmlDeclaration => self.handle_xml_declaration(content),
                Token::StartTag | Token::StartTagWithAttributes => self.handle_start_tag(content),
                Token::EndTag => self.handle_end_tag(content),
                Token::Text => self.handle_text_content(content),
                Token::MalformedTag => self.handle_malformed_tag(content),
                Token::Comment => self.handle_comment(content),
            }
        }
    }

    /// Builds a malformed tag from a chunk of content, handling cases where the tag is incomplete or malformed.
    /// Will be pased on to the next chunk if it is not complete.
    ///
    /// # Arguments
    /// * `chunk` - A string slice containing the chunk of content to be processed.
    ///
    /// # Returns
    /// An `Option<String>` containing the malformed content if it could not be processed, or `None` if the tag was successfully handled.
    pub fn build_malformed(&mut self, chunk: &str) -> Option<String> {
        if self.pending_malformed_tag.is_none() {
            panic!("No pending malformed tag to build from. Call `build` first.");
        }

        let parts = chunk.split_once('>'); // Split the chunk at the first '>'
        if let Some((before, _)) = parts {
            // If there are multiple parts, the first part is the malformed tag
            let mut malformed_content = before.to_string();

            // Join malformed content with '>' to ensure it is treated as a single tag and add the buffer from pending malformed tag if it exists
            if let Some(ref mut malformed) = self.pending_malformed_tag {
                malformed_content = format!("{}{}", malformed.buffer, malformed_content);
                malformed.buffer.clear();
            }

            if !malformed_content.is_empty() {
                self.handle_start_tag(malformed_content.as_str());

                self.pending_malformed_tag = None;
            }

            None
        } else {
            // If there's no '>', treat the whole chunk as malformed
            let malformed_content = chunk.to_string();

            if let Some(ref mut malformed) = self.pending_malformed_tag {
                // If there is a pending malformed tag, append the remaining content to its buffer
                malformed.buffer.push_str(&malformed_content);
            } else {
                // If there is no pending malformed tag, create a new one
                self.pending_malformed_tag = Some(MalformedPartial {
                    buffer: malformed_content.clone(),
                });
            }

            Some(malformed_content)
        }
    }
}

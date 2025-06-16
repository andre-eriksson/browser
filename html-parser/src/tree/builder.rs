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
        }
    }

    /// Builds the DOM tree from a vector of HTML tokens.
    ///
    /// # Arguments
    /// * `html_tokens` - A vector of tuples containing a `Token` and its associated content as a string slice.
    ///
    /// # Returns
    /// A string of remaining content that was not processed as tokens, if any.
    pub fn build(&mut self, html_tokens: Vec<(Token, &str)>) -> String {
        let mut buffered_content = String::new();
        for html_token in html_tokens {
            let token = html_token.0;
            let content = html_token.1;

            match token {
                Token::Doctype => {
                    let doctype_declaration = extract_doctype_declaration(content);

                    self.dom_tree
                        .push(Rc::new(RefCell::new(DomNode::Doctype(doctype_declaration))));
                }

                Token::XmlDeclaration => {
                    let xml_declaration = extract_xml_declaration(content);

                    self.dom_tree
                        .push(Rc::new(RefCell::new(DomNode::XmlDeclaration(
                            xml_declaration,
                        ))));
                }

                Token::StartTag | Token::StartTagWithAttributes => {
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

                    self.collector.collect(&TagInfo {
                        tag_name: &tag_name,
                        attributes: &cloned_attributes,
                        dom_node: &new_node,
                    });

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

                Token::EndTag => {
                    let tag_name = extract_tag_name(content);
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

                Token::Text => {
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
                                        continue;
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Error decoding text content: {}", e);
                                    processed_text_content = trimmed_original.to_string();
                                }
                            }
                        }

                        if processed_text_content.is_empty() {
                            continue; // Skip empty text nodes
                        }

                        let text_node =
                            Rc::new(RefCell::new(DomNode::Text(processed_text_content)));
                        if let Some(parent) = self.open_elements.last() {
                            if let DomNode::Element(ref mut element) = *parent.borrow_mut() {
                                element.children.push(Rc::clone(&text_node));
                            }
                        } else {
                            self.dom_tree.push(Rc::clone(&text_node));
                        }
                    }
                }

                Token::Unknown => {
                    // Will handle unknown tokens i.e. malformed HTML, incomplete tags, etc, and add them to the next parse iteration.
                    buffered_content.push_str(content);
                }

                Token::Comment => {
                    // No need to handle comments currently
                }
            }
        }

        buffered_content
    }
}

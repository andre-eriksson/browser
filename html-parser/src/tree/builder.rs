use crate::{
    tokens::state::{Token, TokenKind},
    tree::{
        decode::Decoder,
        rules::{auto_close::should_auto_close, void_elements::is_void_element},
    },
};
use api::{
    collector::{Collector, TagInfo},
    dom::{DomNode, Element, RefDomNode},
};
use std::{cell::RefCell, rc::Rc};

/// A builder for constructing a DOM tree from HTML tokens.
///
/// # Type Parameters
/// * `C` - The type of the collector used to gather metadata during parsing, which must implement the `Collector` trait.
///
/// # Fields
/// * `collector` - An instance of the collector used to gather metadata during parsing.
/// * `dom_tree` - A vector of shared DOM nodes representing the parsed document structure.
/// * `open_elements` - A stack of currently open elements, used to manage the hierarchy of the DOM tree.
pub struct DomTreeBuilder<C: Collector> {
    pub current_id: u32,
    pub collector: C,
    dom_tree: Vec<RefDomNode>,
    open_elements: Vec<RefDomNode>,
}

impl<C: Collector + Default> DomTreeBuilder<C> {
    /// Creates a new `DomTreeBuilder` instance with a default collector.
    ///
    /// # Arguments
    /// * `collector` - An optional instance of the collector used to gather metadata during parsing. If `None`, a default collector will be used.
    ///
    /// # Returns
    /// A new instance of `DomTreeBuilder` initialized with an empty DOM tree and no open elements.
    pub fn new(collector: Option<C>) -> Self {
        DomTreeBuilder {
            current_id: 0,
            collector: collector.unwrap_or_default(),
            dom_tree: Vec::new(),
            open_elements: Vec::with_capacity(16),
        }
    }

    /// A getter for the DOM tree, returning a reference to the root document node.
    /// This method allows access to the entire DOM structure built by the parser.
    pub fn get_dom_tree(&self) -> RefDomNode {
        Rc::new(RefCell::new(DomNode::Document(self.dom_tree.clone())))
    }

    /// Builds the DOM tree from a vector of HTML tokens.
    ///
    /// # Arguments
    /// * `html_tokens` - A vector of `Token` representing the parsed HTML content.
    ///
    /// # Returns
    /// A string of remaining content that was not processed as tokens, if any.
    pub fn build_from_tokens(&mut self, tokens: Vec<Token>) {
        for token in tokens {
            match token.kind {
                TokenKind::StartTag => {
                    self.handle_start_tag(&token);
                }
                TokenKind::EndTag => {
                    self.handle_end_tag(&token);
                }
                TokenKind::Comment => {
                    self.handle_comment(&token);
                }
                TokenKind::Text => {
                    self.handle_text_content(&token);
                }
                TokenKind::DoctypeDeclaration => {
                    self.handle_doctype_tag(&token);
                }
                TokenKind::XmlDeclaration => {
                    self.handle_xml_declaration(&token);
                }
            }
        }
    }

    /// Inserts a new node into the DOM tree, either as a child of the last open element or as a root node.
    ///
    /// # Arguments
    /// * `node` - A shared reference to the `DomNode` to be inserted into the DOM tree.
    fn insert_new_node(&mut self, node: RefDomNode) {
        if let Some(last) = self.open_elements.last() {
            if let DomNode::Element(parent) = &mut *last.borrow_mut() {
                parent.children.push(node);
            }
        } else {
            self.dom_tree.push(node);
        }
    }

    /// Handles auto-closing of elements based on the new tag name.
    ///
    /// # Arguments
    /// * `new_tag_name` - The name of the new tag being processed, which may trigger auto-closing of previous tags.
    fn handle_auto_close(&mut self, new_tag_name: &str) {
        let should_pop = if let Some(last) = self.open_elements.last() {
            if let DomNode::Element(ref parent) = *last.borrow() {
                should_auto_close(&parent.tag_name, new_tag_name)
            } else {
                false
            }
        } else {
            false
        };

        if should_pop {
            self.open_elements.pop();
        }
    }

    /// Handles the start tag token, creating a new element and adding it to the DOM tree.
    ///
    /// # Arguments
    /// * `token` - A reference to the `Token` representing the start tag to be processed.
    fn handle_start_tag(&mut self, token: &Token) {
        let tag_name = &token.data.to_lowercase();
        let attributes = &token.attributes;
        self.current_id += 1;

        let element = Element {
            id: self.current_id,
            tag_name: tag_name.to_string(),
            attributes: attributes.clone(),
            children: Vec::new(),
        };

        self.collector.collect(&TagInfo {
            tag_name: tag_name,
            attributes: &token.attributes,
            dom_node: &Rc::new(RefCell::new(DomNode::Element(element.clone()))),
        });

        let new_node = Rc::new(RefCell::new(DomNode::Element(element)));

        self.handle_auto_close(tag_name);

        if is_void_element(tag_name) {
            self.insert_new_node(new_node.clone());
            return;
        }

        self.insert_new_node(new_node.clone());
        self.open_elements.push(new_node);
    }

    /// Handles the end tag token, closing the most recent open element if it matches the tag name.
    ///
    /// # Arguments
    /// * `token` - A reference to the `Token` representing the end tag to be processed.
    fn handle_end_tag(&mut self, token: &Token) {
        let tag_name = &token.data.to_lowercase();

        let should_close = if let Some(last) = self.open_elements.last() {
            if let DomNode::Element(ref parent) = *last.borrow() {
                parent.tag_name == *tag_name
            } else {
                false
            }
        } else {
            false
        };

        if should_close {
            self.open_elements.pop();
        }
    }

    /// Handles comment tokens, currently theres no need to process them, but this method is provided.
    ///
    /// # Arguments
    /// * `token` - A reference to the `Token` representing the comment to be processed.
    fn handle_comment(&mut self, _token: &Token) {
        // NOTE: Handle comments if necessary
    }

    /// Handles text content tokens, normalizing whitespace and decoding HTML entities.
    ///
    /// # Arguments
    /// * `token` - A reference to the `Token` containing the text content to be processed.
    fn handle_text_content(&mut self, token: &Token) {
        let mut text_content = token.data.clone();

        if text_content.contains('&') {
            let decoder = Decoder::new(&text_content);
            let result = decoder.decode();

            match result {
                Ok(decoded) => text_content = decoded,
                Err(_) => {}
            }
        }

        if let Some(last) = self.open_elements.last() {
            if let DomNode::Element(parent) = &mut *last.borrow_mut() {
                let text_node = Rc::new(RefCell::new(DomNode::Text(text_content)));

                self.collector.collect(&TagInfo {
                    tag_name: &parent.tag_name,
                    attributes: &parent.attributes,
                    dom_node: &text_node.clone(),
                });

                parent.children.push(text_node);
            }
        }

        // Skip adding text content if there are no open elements
    }

    /// Handles doctype declaration tokens, currently there's no need to process them, but this method is provided.
    ///
    /// # Arguments
    /// * `token` - A reference to the `Token` representing the doctype declaration to be processed.
    fn handle_doctype_tag(&mut self, _token: &Token) {
        // NOTE: Handle doctype declarations if necessary
    }

    /// Handles XML declaration tokens, currently there's no need to process them, but this method is provided.
    ///
    /// # Arguments
    /// * `token` - A reference to the `Token` representing the XML declaration to be processed.
    fn handle_xml_declaration(&mut self, _token: &Token) {
        // NOTE: Handle XML declarations if necessary
    }
}

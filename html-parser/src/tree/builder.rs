use api::html::{HtmlTag, is_void_element, should_auto_close, tag_from_str};

use crate::{
    collector::{Collector, TagInfo},
    dom::{DocumentNode, DocumentRoot, DomIndex, Element, NodeContext, SingleThreaded},
    tokens::state::{Token, TokenKind},
    tree::decode::Decoder,
};
use std::{cell::RefCell, rc::Rc};

/// Represents the result of building a DOM tree.
pub struct BuildResult<M> {
    /// A vector of shared DOM nodes representing the parsed document structure.
    pub dom_tree: DocumentRoot<SingleThreaded>,

    /// The metadata collected during parsing, which is of type `M`.
    pub metadata: M,
}

/// A builder for constructing a DOM tree from HTML tokens.
///
/// # Type Parameters
/// * `C` - The type of the collector used to gather metadata during parsing, which must implement the `Collector` trait.
pub struct DomTreeBuilder<C: Collector> {
    /// The current ID to assign to new elements.
    pub current_id: u16,

    /// An instance of the collector used to gather metadata during parsing.
    pub collector: C,

    /// A vector of shared DOM nodes representing the parsed document structure.
    dom_tree: Vec<Rc<RefCell<DocumentNode<SingleThreaded>>>>,

    /// An index for tracking the position of elements in the DOM tree.
    index: DomIndex<SingleThreaded>,

    /// A stack of currently open elements, used to manage the hierarchy of the DOM tree.
    open_elements: Vec<Rc<RefCell<DocumentNode<SingleThreaded>>>>,
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
            index: DomIndex::default(),
            open_elements: Vec::with_capacity(16),
        }
    }

    /// Finalizes the DOM tree building process and consumes the builder, returning the result.
    ///
    /// # Returns
    /// A `BuildResult` containing the constructed DOM tree and collected metadata.
    pub fn finalize(self) -> BuildResult<C::Output> {
        BuildResult {
            dom_tree: DocumentRoot::new(self.dom_tree, self.index),
            metadata: self.collector.into_result(),
        }
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
                TokenKind::Text => {
                    self.handle_text_content(&token);
                }
                _ => {}
            }
        }
    }

    /// Inserts a node reference into the DOM tree, either as a child of the last open element or as a root node.
    ///
    /// # Arguments
    /// * `node_ref` - A shared reference to the `DomNode` to be inserted into the DOM tree.
    fn insert_node(&mut self, node_ref: Rc<RefCell<DocumentNode<SingleThreaded>>>) {
        if let Some(last) = self.open_elements.last() {
            if let DocumentNode::Element(parent) = &mut *last.borrow_mut() {
                parent.children.push(node_ref);
            }
        } else {
            self.dom_tree.push(node_ref);
        }
    }

    /// Handles auto-closing of elements based on the new tag name.
    ///
    /// # Arguments
    /// * `new_tag_name` - The name of the new tag being processed, which may trigger auto-closing of previous tags.
    fn handle_auto_close(&mut self, new_tag: &HtmlTag) {
        let should_pop = if let Some(last) = self.open_elements.last() {
            if let DocumentNode::Element(parent) = &*last.borrow() {
                should_auto_close(&parent.tag, new_tag)
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
        let tag = tag_from_str(&token.data.to_lowercase());
        let attributes = &token.attributes;
        self.current_id += 1;

        let element = Element {
            id: self.current_id,
            tag: tag.clone(),
            attributes: attributes.clone(),
            children: Vec::new(),
        };

        self.collector.collect(&TagInfo {
            tag: &tag,
            attributes: &token.attributes,
            dom_node: &DocumentNode::Element(element.clone()),
        });

        let new_node = DocumentNode::Element(element);
        let new_node_ref = SingleThreaded::new_node(&new_node);

        self.handle_auto_close(&tag);

        if is_void_element(&tag) {
            self.insert_node(new_node_ref);
            return;
        }

        self.insert_node(new_node_ref.clone());

        self.index.flat.push(new_node_ref.clone());
        self.index.id.insert(self.current_id, new_node_ref.clone());
        self.index
            .tag
            .entry(tag)
            .or_default()
            .push(new_node_ref.clone());

        self.open_elements.push(new_node_ref);
    }

    /// Handles the end tag token, closing the most recent open element if it matches the tag name.
    ///
    /// # Arguments
    /// * `token` - A reference to the `Token` representing the end tag to be processed.
    fn handle_end_tag(&mut self, token: &Token) {
        let tag_name = token.data.to_lowercase();

        let should_close = if let Some(last) = self.open_elements.last() {
            if let DocumentNode::Element(parent) = &*last.borrow() {
                parent.tag == tag_from_str(&tag_name)
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

    /// Handles text content tokens, normalizing whitespace and decoding HTML entities.
    ///
    /// # Arguments
    /// * `token` - A reference to the `Token` containing the text content to be processed.
    fn handle_text_content(&mut self, token: &Token) {
        let mut text_content = token.data.clone();

        if text_content.contains('&') {
            let decoder = Decoder::new(&text_content);
            let result = decoder.decode();

            if let Ok(decoded) = result {
                text_content = decoded;
            }
        }

        if let Some(last) = self.open_elements.last() {
            if let DocumentNode::Element(parent) = &mut *last.borrow_mut() {
                let text_node = DocumentNode::<SingleThreaded>::Text(text_content);

                self.collector.collect(&TagInfo {
                    tag: &parent.tag,
                    attributes: &parent.attributes,
                    dom_node: &text_node.clone(),
                });

                parent.children.push(SingleThreaded::new_node(&text_node));
            }
        }
    }
}

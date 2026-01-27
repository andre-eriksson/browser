use html_tokenizer::{Token, TokenKind};

use crate::{
    collector::{Collector, TagInfo},
    decode::Decoder,
    dom::{DocumentRoot, Element, NodeData, NodeId},
    tag::Tag,
};

/// Represents the result of building a DOM tree.
pub struct BuildResult<M> {
    /// A vector of shared DOM nodes representing the parsed document structure.
    pub dom_tree: DocumentRoot,

    /// The metadata collected during parsing, which is of type `M`.
    pub metadata: M,
}

/// A builder for constructing a DOM tree from HTML tokens.
///
/// # Type Parameters
/// * `C` - The type of the collector used to gather metadata during parsing, which must implement the `Collector` trait.
pub struct DomTreeBuilder<C: Collector> {
    /// The collector instance used to gather metadata during parsing.
    pub collector: C,

    /// The root of the DOM tree being constructed.
    dom_tree: DocumentRoot,

    /// A stack of currently open element node IDs.
    open_elements: Vec<NodeId>,
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
            collector: collector.unwrap_or_default(),
            dom_tree: DocumentRoot::new(),
            open_elements: Vec::with_capacity(16),
        }
    }

    /// Finalizes the DOM tree building process and consumes the builder, returning the result.
    ///
    /// # Returns
    /// A `BuildResult` containing the constructed DOM tree and collected metadata.
    pub fn finalize(self) -> BuildResult<C> {
        BuildResult {
            dom_tree: self.dom_tree,
            metadata: self.collector.into_result(),
        }
    }

    /// Builds the DOM tree from a vector of HTML tokens.
    ///
    /// # Arguments
    /// * `tokens` - A vector of `Token` instances representing the HTML content to be processed.
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
    /// * `data` - The `NodeData` representing the node to be inserted.
    ///
    /// # Returns
    /// The `NodeId` of the newly inserted node.
    fn insert_node(&mut self, data: NodeData) -> NodeId {
        let parent = self.open_elements.last().copied();
        self.dom_tree.push_node(data, parent)
    }

    /// Handles auto-closing of elements based on the new tag name.
    ///
    /// # Arguments
    /// * `new_tag` - A reference to the `HtmlTag` representing the new tag being processed.
    fn handle_auto_close(&mut self, new_tag: &Tag) {
        let should_pop = if let Some(last_id) = self.open_elements.last() {
            if let Some(node) = self.dom_tree.get_node(last_id) {
                if let NodeData::Element(elem) = &node.data {
                    elem.tag.should_auto_close(new_tag)
                } else {
                    false
                }
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
        let tag = Tag::from(token.data.to_lowercase().as_str());
        let attributes = &token.attributes;

        let element = Element {
            tag: tag.clone(),
            attributes: attributes.clone(),
        };

        let node_data = NodeData::Element(element);

        self.handle_auto_close(&tag);

        let new_id = self.insert_node(node_data);

        self.collector.collect(&TagInfo {
            tag: &tag,
            attributes: &token.attributes,
            node_id: new_id,
            data: None,
        });

        if !tag.is_void_element() {
            self.open_elements.push(new_id);
        }
    }

    /// Handles the end tag token, closing the most recent open element if it matches the tag name.
    ///
    /// # Arguments
    /// * `token` - A reference to the `Token` representing the end tag to be processed.
    fn handle_end_tag(&mut self, token: &Token) {
        let target_tag = Tag::from(token.data.to_lowercase().as_str());

        let should_close = if let Some(last) = self.open_elements.last() {
            if let Some(node) = self.dom_tree.get_node(last) {
                if let NodeData::Element(elem) = &node.data {
                    elem.tag == target_tag
                } else {
                    false
                }
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

        if let Some(last_id) = self.open_elements.last()
            && let Some(parent_node) = self.dom_tree.get_node(last_id)
            && let NodeData::Element(parent_elem) = &parent_node.data
        {
            let tag = &parent_elem.tag.clone();
            let attributes = &parent_elem.attributes.clone();
            let text_data = NodeData::Text(text_content);
            let new_id = self.insert_node(text_data.clone());

            self.collector.collect(&TagInfo {
                tag,
                attributes,
                node_id: new_id,
                data: text_data.as_text(),
            });
        }
    }
}

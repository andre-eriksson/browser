use std::collections::HashMap;

use crate::tag::HtmlTag;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub usize);

#[derive(Debug, Clone)]
pub struct Element {
    pub id: u16,
    pub attributes: HashMap<String, String>,
    pub tag: HtmlTag,
}

impl PartialEq for Element {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Default for Element {
    fn default() -> Self {
        Element {
            id: 0,
            attributes: HashMap::new(),
            tag: HtmlTag::Unknown("".to_string()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum NodeData {
    Element(Element),
    Text(String),
}

impl NodeData {
    pub fn as_element(&self) -> Option<&Element> {
        match self {
            NodeData::Element(elem) => Some(elem),
            _ => None,
        }
    }

    pub fn as_text(&self) -> Option<&String> {
        match self {
            NodeData::Text(text) => Some(text),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DomNode {
    pub id: NodeId,
    pub parent: Option<NodeId>,
    pub children: Vec<NodeId>,
    pub data: NodeData,
}

#[derive(Debug, Clone, Default)]
pub struct DomIndex {
    pub id: HashMap<u16, NodeId>,
    pub tag: HashMap<HtmlTag, Vec<NodeId>>,
}

#[derive(Debug, Clone, Default)]
pub struct DocumentRoot {
    pub nodes: Vec<DomNode>,
    pub root_nodes: Vec<NodeId>,
    pub index: DomIndex,
}

impl DocumentRoot {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_node(&self, node_id: &NodeId) -> Option<&DomNode> {
        self.nodes.get(node_id.0)
    }

    pub fn get_node_mut(&mut self, node_id: &NodeId) -> Option<&mut DomNode> {
        self.nodes.get_mut(node_id.0)
    }

    pub fn push_node(&mut self, data: NodeData, parent: Option<NodeId>) -> NodeId {
        let node_id = NodeId(self.nodes.len());
        let new_node = DomNode {
            id: node_id,
            parent,
            children: Vec::new(),
            data: data.clone(),
        };

        self.nodes.push(new_node);

        if let Some(parent_id) = parent {
            if let Some(parent_node) = self.get_node_mut(&parent_id) {
                parent_node.children.push(node_id);
            }
        } else {
            self.root_nodes.push(node_id);
        }

        if let NodeData::Element(elem) = data {
            self.index.id.insert(elem.id, node_id);
            self.index
                .tag
                .entry(elem.tag.clone())
                .or_default()
                .push(node_id);
        }

        node_id
    }
}

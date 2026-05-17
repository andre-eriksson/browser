use html_dom::NodeId;

#[derive(Debug, Clone)]
pub struct BoxNode {
    pub node_id: Option<NodeId>,
    pub children: Vec<BoxNode>,
}

impl BoxNode {
    pub fn new(node_id: &NodeId, children: Vec<BoxNode>) -> Self {
        Self {
            node_id: Some(*node_id),
            children,
        }
    }

    pub fn new_anonymous_node(buffer: Vec<BoxNode>) -> Self {
        Self {
            node_id: None,
            children: buffer,
        }
    }
}

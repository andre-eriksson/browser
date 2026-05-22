use html_dom::NodeId;

use crate::LayoutNode;

/// The root of the layout tree containing all layout nodes
#[derive(Debug, Clone, Default)]
pub struct LayoutTree {
    /// The root layout nodes
    pub root_nodes: Vec<LayoutNode>,

    /// The total content height of the layout tree
    pub content_height: f64,

    /// The total content width of the layout tree
    pub content_width: f64,
}

impl LayoutTree {
    /// Resolves the layout node at the given (x, y) coordinates
    #[must_use]
    pub fn resolve(&self, x: f64, y: f64) -> Vec<&LayoutNode> {
        let mut collected = Vec::new();
        for node in &self.root_nodes {
            Self::resolve_in_node(&mut collected, node, x, y);
        }
        collected
    }

    fn resolve_in_node<'nodes>(collected: &mut Vec<&'nodes LayoutNode>, node: &'nodes LayoutNode, x: f64, y: f64) {
        if node.dimensions.contains_point(x, y) {
            for child in &node.children {
                Self::resolve_in_node(collected, child, x, y);
            }
            collected.push(node);
        }
    }

    /// Finds the path to the layout node corresponding to the given `NodeId`, if it exists.
    #[must_use]
    pub fn find_path(&self, node_id: NodeId) -> Option<Vec<usize>> {
        for (idx, root) in self.root_nodes.iter().enumerate() {
            if let Some(mut path) = Self::find_path_in_node(root, node_id) {
                path.insert(0, idx);
                return Some(path);
            }
        }

        None
    }

    fn find_path_in_node(node: &LayoutNode, node_id: NodeId) -> Option<Vec<usize>> {
        if node.node_id == Some(node_id) {
            return Some(vec![]);
        }

        for (idx, child) in node.children.iter().enumerate() {
            if let Some(mut path) = Self::find_path_in_node(child, node_id) {
                path.insert(0, idx);
                return Some(path);
            }
        }

        None
    }

    /// Retrieves a reference to the layout node at the specified path, if it exists.
    #[must_use]
    pub fn node_at(&self, path: &[usize]) -> Option<&LayoutNode> {
        if path.is_empty() {
            return None;
        }

        let mut current = self.root_nodes.get(path[0]);
        for &idx in &path[1..] {
            current = match current {
                None => return current,
                Some(node) => node.children.get(idx),
            };
        }
        current
    }

    /// Retrieves a mutable reference to the layout node at the specified path, if it exists.
    pub fn node_at_mut(&mut self, path: &[usize]) -> Option<&mut LayoutNode> {
        if path.is_empty() {
            return None;
        }

        let mut current = self.root_nodes.get_mut(path[0]);
        for &idx in &path[1..] {
            current = match current {
                None => return current,
                Some(node) => node.children.get_mut(idx),
            };
        }
        current
    }
}

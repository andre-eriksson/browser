use css_display::LayoutNodeId;

use crate::LayoutNode;

/// The root of the layout tree containing all layout nodes
#[derive(Debug, Clone, Default)]
pub struct LayoutTree {
    /// The root layout nodes
    pub root_nodes: Vec<LayoutNodeId>,

    pub nodes: Vec<Option<LayoutNode>>,

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
            self.resolve_in_node(&mut collected, node, x, y);
        }

        collected
            .into_iter()
            .flat_map(|id| &self.nodes[id.index()])
            .collect()
    }

    fn resolve_in_node<'nodes>(
        &'nodes self,
        collected: &mut Vec<&'nodes LayoutNodeId>,
        node_id: &'nodes LayoutNodeId,
        x: f64,
        y: f64,
    ) {
        let Some(node) = &self.nodes[node_id.index()] else {
            return;
        };

        if node.dimensions.contains_point(x, y) {
            for child in &node.children {
                self.resolve_in_node(collected, child, x, y);
            }
            collected.push(&node.layout_id);
        }
    }

    /// Finds the path to the layout node corresponding to the given `NodeId`, if it exists.
    #[must_use]
    pub fn find_path(&self, id: LayoutNodeId) -> Option<Vec<usize>> {
        for (idx, root) in self.root_nodes.iter().enumerate() {
            if let Some(mut path) = self.find_path_in_node(root, id) {
                path.insert(0, idx);
                return Some(path);
            }
        }

        None
    }

    fn find_path_in_node(&self, curr: &LayoutNodeId, seek: LayoutNodeId) -> Option<Vec<usize>> {
        if *curr == seek {
            return Some(vec![]);
        }

        let Some(node) = &self.nodes[curr.index()] else {
            return None;
        };

        let children = &node.children;

        for (idx, child) in children.iter().enumerate() {
            if let Some(mut path) = self.find_path_in_node(child, seek) {
                path.insert(0, idx);
                return Some(path);
            }
        }

        None
    }

    /// Retrieves a reference to the layout node at the specified path, if it exists.
    #[must_use]
    pub fn node_at(&self, path: &[usize]) -> Option<&LayoutNodeId> {
        if path.is_empty() {
            return None;
        }

        let mut current = self.root_nodes.get(path[0])?;
        for &idx in &path[1..] {
            let node = self.nodes[current.index()].as_ref()?;
            current = node.children.get(idx)?;
        }

        Some(current)
    }
}

use std::{collections::HashMap, sync::Arc};

use html_dom::NodeId;

#[derive(Debug, Clone)]
pub struct LayoutImage {
    /// Intrinsic width in CSS pixels.
    pub width: u32,
    /// Intrinsic height in CSS pixels.
    pub height: u32,
    /// Raw RGBA pixel data (4 bytes per pixel)
    pub rgba: Vec<u8>,
}

#[derive(Debug, Clone, Default)]
pub struct ImageContext {
    known: HashMap<NodeId, Arc<LayoutImage>>,
}

impl ImageContext {
    /// Creates an empty `ImageContext` with no known images.
    #[must_use]
    pub fn new() -> Self {
        Self {
            known: HashMap::new(),
        }
    }

    pub fn insert(&mut self, node_id: NodeId, image: Arc<LayoutImage>) {
        self.known.insert(node_id, image);
    }

    pub fn get(&self, node_id: &NodeId) -> Option<Arc<LayoutImage>> {
        self.known.get(node_id).map(Arc::clone)
    }

    pub fn clear(&mut self) {
        self.known.clear();
    }
}

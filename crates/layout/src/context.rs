use std::collections::HashMap;

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
    known: HashMap<NodeId, LayoutImage>,
}

impl ImageContext {
    /// Creates an empty `ImageContext` with no known images.
    #[must_use]
    pub fn new() -> Self {
        Self {
            known: HashMap::new(),
        }
    }

    pub fn insert(&mut self, node_id: NodeId, image: LayoutImage) {
        self.known.insert(node_id, image);
    }

    pub fn update_dimension(&mut self, node_id: NodeId, width: u32, height: u32) {
        if let Some(image) = self.known.get_mut(&node_id) {
            image.width = width;
            image.height = height;
        }
    }

    pub fn get(&self, node_id: &NodeId) -> Option<&LayoutImage> {
        self.known.get(node_id)
    }

    pub fn clear(&mut self) {
        self.known.clear();
    }
}

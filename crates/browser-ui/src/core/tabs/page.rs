use std::sync::{Arc, Mutex};

use browser_core::{Document, PageMetadata};
use layout::ImageContext;

#[derive(Debug, Clone)]
pub struct Page {
    pub document: Document,
    pub metadata: PageMetadata,

    image_ctx: Arc<Mutex<ImageContext>>,
}

impl Page {
    pub fn new(document: Document, metadata: PageMetadata, image_ctx: ImageContext) -> Self {
        Self {
            document,
            metadata,
            image_ctx: Arc::new(Mutex::new(image_ctx)),
        }
    }

    pub fn image_context(&self) -> Arc<Mutex<ImageContext>> {
        Arc::clone(&self.image_ctx)
    }
}

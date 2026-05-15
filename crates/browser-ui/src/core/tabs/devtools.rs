use std::sync::{Arc, Mutex};

use browser_core::Document;
use css_cssom::CSSStyleSheet;
use html_dom::DocumentRoot;
use iced::Size;
use layout::{ImageContext, LayoutTree};

use crate::{core::ScrollOffset, windows::devtools::window::DevtoolsWindow};

#[derive(Debug, Clone)]
pub struct DevtoolsPage {
    document: Document,
    layout_tree: LayoutTree,
    pub scroll_offset: ScrollOffset,
}

impl DevtoolsPage {
    pub fn new(page: Document, layout_tree: LayoutTree) -> Self {
        Self {
            document: page,
            layout_tree,
            scroll_offset: ScrollOffset::default(),
        }
    }

    pub const fn dom(&self) -> &DocumentRoot {
        self.document.dom()
    }

    pub fn stylesheets(&self) -> &[CSSStyleSheet] {
        self.document.stylesheets()
    }

    pub const fn layout_tree(&self) -> &LayoutTree {
        &self.layout_tree
    }

    pub fn update_layout_tree(&mut self, new_layout_tree: LayoutTree) {
        self.layout_tree = new_layout_tree;
    }
}

#[derive(Debug, Clone)]
pub struct DevtoolsContext {
    pub viewport: Size,
    pub page: Option<DevtoolsPage>,

    image_ctx: Arc<Mutex<ImageContext>>,
}

impl DevtoolsContext {
    pub fn image_context(&self) -> Arc<Mutex<ImageContext>> {
        Arc::clone(&self.image_ctx)
    }
}

impl Default for DevtoolsContext {
    fn default() -> Self {
        Self {
            viewport: DevtoolsWindow::DEFAULT_VIEWPORT_SIZE,
            page: None,
            image_ctx: Arc::new(Mutex::new(ImageContext::new())),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Devtools {
    pub window_id: iced::window::Id,
    pub context: DevtoolsContext,
}

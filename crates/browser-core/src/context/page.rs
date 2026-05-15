use std::collections::HashMap;

use css_cssom::CSSStyleSheet;
use html_dom::{DocumentRoot, NodeId};
use io::DocumentPolicy;
use url::Url;

/// Represents the favicon of a web page, including its size, content type, and binary data.
#[derive(Debug, Clone, Default)]
pub struct Favicon {
    pub size: Option<(u32, u32)>,
    pub content_type: Option<String>,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct PageMetadata {
    pub url: Url,
    pub title: String,
    pub favicon: Option<Favicon>,
    pub policies: DocumentPolicy,
}

/// Represents a web page loaded in a tab.
#[derive(Debug, Clone)]
pub struct Document {
    dom: DocumentRoot,
    images: HashMap<String, Vec<NodeId>>,
    stylesheets: Vec<CSSStyleSheet>,
}

impl Document {
    #[must_use]
    pub fn new(dom: DocumentRoot, stylesheets: Vec<CSSStyleSheet>) -> Self {
        Self {
            dom,
            images: HashMap::new(),
            stylesheets,
        }
    }

    /// Creates a new blank page with default settings.
    #[must_use]
    pub fn blank() -> Self {
        Self {
            dom: DocumentRoot::new(),
            images: HashMap::new(),
            stylesheets: Vec::new(),
        }
    }

    /// Loads the page with the given title, document URL, document root, stylesheets, and policies.
    #[must_use]
    pub fn load(
        mut self,
        dom: DocumentRoot,
        images: HashMap<String, Vec<NodeId>>,
        stylesheets: Vec<CSSStyleSheet>,
    ) -> Self {
        self.dom = dom;
        self.images = images;
        self.stylesheets = stylesheets;
        self
    }

    #[must_use]
    pub const fn dom(&self) -> &DocumentRoot {
        &self.dom
    }

    #[must_use]
    pub const fn stylesheets(&self) -> &Vec<CSSStyleSheet> {
        &self.stylesheets
    }

    #[must_use]
    pub const fn images(&self) -> &HashMap<String, Vec<NodeId>> {
        &self.images
    }
}

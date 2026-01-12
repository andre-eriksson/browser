use std::sync::Arc;

use css_cssom::CSSStyleSheet;
use html_dom::DocumentRoot;
use url::Url;

use crate::service::network::policy::DocumentPolicy;

/// Represents a web page loaded in a tab.
#[derive(Debug, Clone)]
pub struct Page {
    pub document_url: Option<Url>,
    title: String,
    document: DocumentRoot,
    stylesheets: Vec<CSSStyleSheet>,
    policies: Arc<DocumentPolicy>,
}

impl Page {
    /// Creates a new blank page with default settings.
    pub fn blank() -> Self {
        Self {
            title: "New Tab".to_string(),
            document: DocumentRoot::new(),
            stylesheets: Vec::new(),
            document_url: None,
            policies: Arc::new(DocumentPolicy::default()),
        }
    }

    /// Loads the page with the given title, document URL, document root, stylesheets, and policies.
    pub fn load(
        mut self,
        title: String,
        document_url: Option<Url>,
        document: DocumentRoot,
        stylesheets: Vec<CSSStyleSheet>,
        policies: Arc<DocumentPolicy>,
    ) -> Self {
        self.title = title;
        self.document_url = document_url;
        self.document = document;
        self.stylesheets = stylesheets;
        self.policies = policies;
        self
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn policies(&self) -> &Arc<DocumentPolicy> {
        &self.policies
    }

    pub fn document(&self) -> &DocumentRoot {
        &self.document
    }

    pub fn stylesheets(&self) -> &Vec<CSSStyleSheet> {
        &self.stylesheets
    }
}

use browser_core::tab::TabId;
use css_cssom::CSSStyleSheet;
use html_syntax::dom::DocumentRoot;
use layout::LayoutTree;
use url::Url;

/// Represents the scroll position of a tab's content.
#[derive(Debug, Clone, Copy, Default)]
pub struct ScrollOffset {
    pub x: f32,
    pub y: f32,
}

/// Represents a tab in the UI.
#[derive(Debug, Clone)]
pub struct UiTab {
    /// The unique identifier for the tab.
    pub id: TabId,

    /// The title of the tab, if available.
    pub title: Option<String>,

    /// The current URL loaded in the tab.
    pub current_url: Option<Url>,

    /// The layout tree of the tab's content.
    pub layout_tree: LayoutTree,

    /// The current scroll offset of the tab's content.
    pub scroll_offset: ScrollOffset,

    /// The document root of the tab's content.
    pub document: DocumentRoot,

    /// The stylesheets associated with the tab.
    pub stylesheets: Vec<CSSStyleSheet>,
}

impl UiTab {
    pub fn new(id: TabId) -> Self {
        Self {
            id,
            title: None,
            current_url: None,
            layout_tree: LayoutTree::default(),
            scroll_offset: ScrollOffset::default(),
            document: DocumentRoot::new(),
            stylesheets: Vec::new(),
        }
    }
}

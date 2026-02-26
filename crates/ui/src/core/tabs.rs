use std::collections::{HashMap, HashSet};

use css_cssom::CSSStyleSheet;
use html_dom::DocumentRoot;
use kernel::TabId;
use layout::{ImageContext, LayoutTree};
use url::Url;

/// Represents the scroll position of a tab's content.
#[derive(Debug, Clone, Copy, Default)]
pub struct ScrollOffset {
    pub x: f32,
    pub y: f32,
}

/// Known metadata for a decoded image, persisted across relayouts.
#[derive(Debug, Clone)]
pub struct KnownImageMeta {
    pub width: f32,
    pub height: f32,
    pub vary_key: String,
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

    /// Known intrinsic image metadata from previously decoded images, keyed by
    /// source URL.  Persisted across relayouts (e.g. window resize) so that
    /// images that have already been fetched keep their real dimensions and
    /// vary keys.
    pub known_images: HashMap<String, KnownImageMeta>,

    /// Set of image source URLs that are still being fetched / decoded.
    /// A relayout is only triggered once this set becomes empty, so that all
    /// pending images are batched into a single layout pass instead of
    /// relaying out after every individual image.
    pub pending_image_urls: HashSet<String>,

    /// Monotonically increasing generation counter, incremented on every
    /// navigation.  Background relayout results carry the generation they were
    /// started with; if it no longer matches the tab's current generation the
    /// result is stale and gets discarded.
    pub layout_generation: u64,
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
            known_images: HashMap::new(),
            pending_image_urls: HashSet::new(),
            layout_generation: 0,
        }
    }

    /// Prepare the tab for a brand-new navigation.  Clears stale image
    /// metadata and pending state, and increments the layout generation so
    /// that any in-flight background relayout from the previous page is
    /// automatically discarded.
    pub fn prepare_for_navigation(&mut self) {
        self.known_images.clear();
        self.pending_image_urls.clear();
        self.layout_generation += 1;
    }

    /// Record (or update) the intrinsic dimensions for an image source URL,
    /// preserving any previously stored vary key.
    pub fn set_image_dimensions(&mut self, src: String, width: f32, height: f32) {
        self.known_images
            .entry(src)
            .and_modify(|m| {
                m.width = width;
                m.height = height;
            })
            .or_insert(KnownImageMeta {
                width,
                height,
                vary_key: String::new(),
            });
    }

    /// Record (or update) the vary key for an image source URL, preserving any
    /// previously stored dimensions.
    pub fn set_image_vary_key(&mut self, src: &str, vary_key: String) {
        if let Some(meta) = self.known_images.get_mut(src) {
            meta.vary_key = vary_key;
        }
    }

    /// Mark an image URL as no longer pending (because it finished loading or
    /// failed).  Returns `true` when the pending set has become empty, meaning
    /// all images have been resolved and a batched relayout should be
    /// triggered.
    pub fn resolve_pending_image(&mut self, url: &str) -> bool {
        self.pending_image_urls.remove(url);
        self.pending_image_urls.is_empty()
    }

    /// Build an [`ImageContext`] from the tab's currently known image
    /// metadata.  This is passed into
    /// [`LayoutEngine::compute_layout`] so that decoded images are
    /// laid out at their real intrinsic size (with the correct vary key for
    /// disk-cache lookups) instead of a placeholder.
    pub fn image_context(&self) -> ImageContext {
        let mut ctx = ImageContext::new();
        for (src, meta) in &self.known_images {
            ctx.insert_with_vary(src.clone(), meta.width, meta.height, meta.vary_key.clone());
        }
        ctx
    }
}

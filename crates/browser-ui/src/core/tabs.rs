use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    sync::MutexGuard,
};

use browser_config::ThemeCategory;
use browser_core::{History, Page, PageMetadata};
use css_cssom::CSSStyleSheet;
use css_style::{AbsoluteContext, StyleTree};
use css_values::color::Color;
use html_dom::DocumentRoot;
use iced::{Size, window::Id};
use layout::{ImageContext, LayoutEngine, LayoutTree, Rect, TextContext};

use crate::views::devtools::window::DevtoolsContext;

pub mod manager;

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

#[derive(Debug, Clone)]
pub struct UiDevtools {
    pub page: Page,
    pub layout_tree: LayoutTree,
    pub scroll_offset: ScrollOffset,
}

impl UiDevtools {
    pub fn new(page: Page, layout_tree: LayoutTree) -> Self {
        Self {
            page,
            layout_tree,
            scroll_offset: ScrollOffset::default(),
        }
    }

    pub const fn document(&self) -> &DocumentRoot {
        self.page.document()
    }

    pub fn stylesheets(&self) -> &[CSSStyleSheet] {
        self.page.stylesheets()
    }

    pub const fn layout_tree(&self) -> &LayoutTree {
        &self.layout_tree
    }
}

#[derive(Debug, Clone)]
pub struct Devtools {
    pub window_id: Id,
    pub context: DevtoolsContext,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TabId(usize);

impl TabId {
    pub const fn new(id: usize) -> Self {
        Self(id)
    }
}

impl Display for TabId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct PageContext {
    pub page: Page,
    pub metadata: PageMetadata,
}

/// Represents a tab in the UI.
#[derive(Debug, Clone)]
pub struct UiTab {
    pub id: TabId,

    pub page_ctx: Option<PageContext>,
    pub devtools: Option<Devtools>,

    pub style_tree: Option<StyleTree>,
    pub layout_tree: Option<LayoutTree>,
    pub layout_generation: u64,

    pub scroll_offset: ScrollOffset,

    pub known_images: HashMap<String, KnownImageMeta>,
    pub pending_image_urls: HashSet<String>,

    pub history: History,
}

impl UiTab {
    pub fn new(id: TabId) -> Self {
        Self {
            id,
            page_ctx: None,
            devtools: None,
            style_tree: None,
            layout_tree: None,
            layout_generation: 0,
            scroll_offset: ScrollOffset::default(),
            known_images: HashMap::new(),
            pending_image_urls: HashSet::new(),
            history: History::new(),
        }
    }

    pub fn resize_current_page(
        &mut self,
        viewport: Size,
        text_context: &mut MutexGuard<'_, TextContext>,
        theme_category: ThemeCategory,
    ) {
        let Some(page) = self.page_ctx.as_ref().map(|ctx| &ctx.page) else {
            return;
        };

        let Some(metadata) = self.page_ctx.as_ref().map(|ctx| &ctx.metadata) else {
            return;
        };

        let absolute_ctx = AbsoluteContext {
            root_font_size: 16.0,
            viewport_width: viewport.width,
            viewport_height: viewport.height,
            theme_category,
            document_url: &metadata.url,
            root_line_height_multiplier: 1.2,
            root_color: Color::BLACK,
        };

        let style_tree = StyleTree::build(&absolute_ctx, page.document(), page.stylesheets());
        let layout_tree = LayoutEngine::compute_layout(
            &style_tree,
            Rect::new(0.0, 0.0, viewport.width, viewport.height),
            text_context,
            Some(&self.image_context()),
        );

        self.style_tree = Some(style_tree);
        self.layout_tree = Some(layout_tree);
    }

    pub fn resolve_page(
        &mut self,
        viewport: Size,
        text_context: &mut MutexGuard<'_, TextContext>,
        page: Page,
        metadata: PageMetadata,
        theme_category: ThemeCategory,
        scroll_offset: Option<ScrollOffset>,
    ) {
        let absolute_ctx = AbsoluteContext {
            root_font_size: 16.0,
            viewport_width: viewport.width,
            viewport_height: viewport.height,
            theme_category,
            document_url: &metadata.url,
            root_line_height_multiplier: 1.2,
            root_color: Color::BLACK,
        };

        let style_tree = StyleTree::build(&absolute_ctx, page.document(), page.stylesheets());
        let layout_tree = LayoutEngine::compute_layout(
            &style_tree,
            Rect::new(0.0, 0.0, viewport.width, viewport.height),
            text_context,
            Some(&self.image_context()),
        );

        self.style_tree = Some(style_tree);
        self.layout_tree = Some(layout_tree);
        self.page_ctx = Some(PageContext { page, metadata });
        self.scroll_offset = scroll_offset.unwrap_or_default();
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

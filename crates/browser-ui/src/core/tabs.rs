use std::{
    fmt::Display,
    sync::{Arc, Mutex, MutexGuard},
};

use browser_config::BrowserConfig;
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

    image_ctx: Arc<Mutex<ImageContext>>,

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
            image_ctx: Arc::new(Mutex::new(ImageContext::new())),
            history: History::new(),
        }
    }

    pub fn resize_current_page(
        &mut self,
        viewport: Size,
        text_context: &mut MutexGuard<'_, TextContext>,
        config: &BrowserConfig,
    ) {
        let Some(page) = self.page_ctx.as_ref().map(|ctx| &ctx.page) else {
            return;
        };

        let Some(metadata) = self.page_ctx.as_ref().map(|ctx| &ctx.metadata) else {
            return;
        };

        let absolute_ctx = AbsoluteContext {
            root_font_size: 16.0,
            viewport_width: f64::from(viewport.width),
            viewport_height: f64::from(viewport.height),
            theme_category: config.preferences().theme().category,
            document_url: &metadata.url,
            root_line_height_multiplier: 1.2,
            root_color: Color::BLACK,
        };

        let style_tree = StyleTree::build(config, &absolute_ctx, page.document(), page.stylesheets());
        let layout_tree = {
            let image_ctx = self.image_ctx.lock().unwrap();
            LayoutEngine::compute_layout(
                page.document(),
                &style_tree,
                Rect::new(0.0, 0.0, f64::from(viewport.width), f64::from(viewport.height)),
                text_context,
                &image_ctx,
            )
        };

        self.style_tree = Some(style_tree);
        self.layout_tree = Some(layout_tree);
    }

    pub fn resolve_page(
        &mut self,
        viewport: Size,
        text_context: &mut MutexGuard<'_, TextContext>,
        page: Page,
        metadata: PageMetadata,
        config: &BrowserConfig,
        scroll_offset: Option<ScrollOffset>,
    ) {
        let absolute_ctx = AbsoluteContext {
            root_font_size: 16.0,
            viewport_width: f64::from(viewport.width),
            viewport_height: f64::from(viewport.height),
            theme_category: config.preferences().theme().category,
            document_url: &metadata.url,
            root_line_height_multiplier: 1.2,
            root_color: Color::BLACK,
        };

        let style_tree = StyleTree::build(config, &absolute_ctx, page.document(), page.stylesheets());
        let layout_tree = {
            let image_ctx = self.image_ctx.lock().unwrap();
            LayoutEngine::compute_layout(
                page.document(),
                &style_tree,
                Rect::new(0.0, 0.0, f64::from(viewport.width), f64::from(viewport.height)),
                text_context,
                &image_ctx,
            )
        };

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
        {
            let mut image_ctx = self.image_ctx.lock().unwrap();
            image_ctx.clear();
        }
        self.layout_generation += 1;
    }

    /// Build an [`ImageContext`] from the tab's currently known image
    /// metadata.  This is passed into
    /// [`LayoutEngine::compute_layout`] so that decoded images are
    /// laid out at their real intrinsic size (with the correct vary key for
    /// disk-cache lookups) instead of a placeholder.
    pub fn image_context(&self) -> Arc<Mutex<ImageContext>> {
        Arc::clone(&self.image_ctx)
    }
}

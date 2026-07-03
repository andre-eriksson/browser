use std::{fmt::Display, ops::Deref, sync::MutexGuard};

use browser_core::{Document, History, PageMetadata};
use browser_preferences::BrowserPreferences;
use css_display::BoxTree;
use css_style::{AbsoluteContext, StyleTree};
use css_values::color::Color;
use iced::Size;
use layout::{ImageContext, LayoutInput, LayoutTree, Rect, TextContext};

use crate::core::{Devtools, Page, ScrollOffset};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TabId(usize);

impl TabId {
    pub const fn new(id: usize) -> Self {
        Self(id)
    }
}

impl Deref for TabId {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for TabId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Represents a tab in the UI.
#[derive(Debug, Clone)]
pub struct Tab {
    pub id: TabId,

    pub page: Option<Page>,
    pub devtools: Option<Devtools>,

    pub style_tree: Option<StyleTree>,
    pub layout_tree: Option<LayoutTree>,
    pub layout_generation: u64,

    pub scroll_offset: ScrollOffset,

    pub history: History,
}

impl Tab {
    pub fn new(id: TabId) -> Self {
        Self {
            id,
            page: None,
            devtools: None,
            style_tree: None,
            layout_tree: None,
            layout_generation: 0,
            scroll_offset: ScrollOffset::default(),
            history: History::new(),
        }
    }

    pub fn resize_current_page(
        &mut self,
        viewport: Size,
        text_context: &mut MutexGuard<'_, TextContext>,
        preferences: &BrowserPreferences,
    ) {
        let Some(page_ctx) = self.page.as_ref() else {
            return;
        };

        let page = &page_ctx.document;

        let Some(metadata) = self.page.as_ref().map(|ctx| &ctx.metadata) else {
            return;
        };

        let absolute_ctx = AbsoluteContext {
            root_font_size: 16.0,
            viewport_width: f64::from(viewport.width),
            viewport_height: f64::from(viewport.height),
            theme_category: preferences.theme().category,
            document_url: &metadata.url,
            root_line_height_multiplier: 1.2,
            root_color: Color::BLACK,
        };

        let style_tree = StyleTree::build(Some(preferences), &absolute_ctx, page.dom(), page.stylesheets());
        let box_tree = BoxTree::new(page.dom(), &style_tree);
        let layout_tree = {
            let image_ctx = page_ctx.image_context();
            let image_ctx = image_ctx.lock().unwrap();
            LayoutTree::compute_layout(
                &mut LayoutInput {
                    dom: page.dom(),
                    box_tree: &box_tree,
                    text: text_context,
                    image: &image_ctx,
                },
                Rect::new(0.0, 0.0, f64::from(viewport.width), f64::from(viewport.height)),
            )
        };

        self.style_tree = Some(style_tree);
        self.layout_tree = Some(layout_tree);
    }

    pub fn resolve_page(
        &mut self,
        viewport: Size,
        text_context: &mut MutexGuard<'_, TextContext>,
        document: Document,
        metadata: PageMetadata,
        preferences: &BrowserPreferences,
        scroll_offset: Option<ScrollOffset>,
    ) {
        let absolute_ctx = AbsoluteContext {
            root_font_size: 16.0,
            viewport_width: f64::from(viewport.width),
            viewport_height: f64::from(viewport.height) - 87.0 - 60.0,
            theme_category: preferences.theme().category,
            document_url: &metadata.url,
            root_line_height_multiplier: 1.2,
            root_color: Color::BLACK,
        };

        let style_tree = StyleTree::build(Some(preferences), &absolute_ctx, document.dom(), document.stylesheets());
        let box_tree = BoxTree::new(document.dom(), &style_tree);
        let image_ctx = ImageContext::new();
        let layout_tree = LayoutTree::compute_layout(
            &mut LayoutInput {
                dom: document.dom(),
                box_tree: &box_tree,
                text: text_context,
                image: &image_ctx,
            },
            Rect::new(0.0, 0.0, f64::from(viewport.width), f64::from(viewport.height) - 87.0 - 60.0),
        );

        self.style_tree = Some(style_tree);
        self.layout_tree = Some(layout_tree);
        self.page = Some(Page::new(document, metadata, image_ctx));
        self.scroll_offset = scroll_offset.unwrap_or_default();
    }

    /// Prepare the tab for a brand-new navigation.  Clears stale image
    /// metadata and pending state, and increments the layout generation so
    /// that any in-flight background relayout from the previous page is
    /// automatically discarded.
    pub fn prepare_for_navigation(&mut self) {
        if let Some(page_ctx) = &self.page {
            let image_ctx = page_ctx.image_context();
            let mut image_ctx = image_ctx.lock().unwrap();
            image_ctx.clear();
        }

        self.layout_generation += 1;
    }
}

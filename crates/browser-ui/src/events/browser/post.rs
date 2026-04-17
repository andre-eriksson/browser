use css_style::{AbsoluteContext, StyleTree};
use css_values::color::Color;
use iced::{Task, window::Id};
use io::{CacheEntry, CacheRead};
use layout::{LayoutEngine, Rect};
use tracing::{debug, error};

use crate::{
    core::{Application, TabId},
    errors::BrowserError,
    events::{Event, browser::BrowserEvent},
};

/// Handles the completion of image loading, updating the tab's state and triggering a targeted
/// async relayout of only the image node and its ancestors.
///
/// Image loads are **batched**: a relayout is only triggered once every pending image for the
/// page has resolved (loaded or failed). This means a page with N images pays the layout cost
/// once rather than N times. The `pending_image_urls` set on [`UiTab`] tracks what is still
/// outstanding; `resolve_pending_image` removes the URL and returns `true` when the set is empty.
///
/// Once all images are ready the relayout runs off the UI thread via `spawn_blocking` so that
/// scrolling and other interactions remain responsive while the work is in progress.
pub fn on_image_loaded(
    application: &mut Application,
    window_id: Id,
    tab_id: TabId,
    url: &str,
    vary_key: &str,
) -> Task<Event> {
    let ctx = if let Some(ctx) = application.browser_windows.get_mut(&window_id) {
        ctx
    } else {
        error!("Browser context not found for window ID: {}", window_id);
        return Task::none();
    };

    let url = url.to_string();

    if let Some(ref cache) = application.image_cache
        && let Ok(CacheEntry::Loaded(decoded)) = cache.get_with_vary(&url, vary_key)
        && let CacheRead::Hit(decoded) = (*decoded).clone()
    {
        let intrinsic_w = decoded.width as f32;
        let intrinsic_h = decoded.height as f32;
        if let Some(tab) = ctx.tab_manager.get_tab_mut(tab_id) {
            tab.set_image_dimensions(url.to_string(), intrinsic_w, intrinsic_h);
            tab.set_image_vary_key(&url, vary_key.to_string());
        }
    }

    let Some(tab) = ctx.tab_manager.get_tab_mut(tab_id) else {
        return Task::none();
    };

    if !tab.resolve_pending_image(&url) {
        return Task::none();
    }

    let image_node_ids: Vec<_> = tab
        .known_images
        .keys()
        .flat_map(|src| {
            tab.layout_tree
                .as_ref()
                .map_or_else(Vec::new, |lt| lt.find_image_nodes_by_src(&src.to_string()))
        })
        .collect();

    if image_node_ids.is_empty() {
        return Task::none();
    }

    let viewport = ctx.viewport;

    let theme_category = application.config.preferences().theme().category;
    let Some(page_ctx) = tab.page_ctx.clone() else {
        return Task::none();
    };
    let image_ctx = tab.image_context();
    let layout_tree = tab.layout_tree.clone();
    let text_ctx = ctx.text_context.clone();
    let generation = tab.layout_generation;

    Task::perform(
        async move {
            tokio::task::spawn_blocking(move || {
                let ctx = AbsoluteContext {
                    root_font_size: 16.0,
                    viewport_width: viewport.width,
                    viewport_height: viewport.height,
                    theme_category,
                    document_url: &page_ctx.metadata.url,
                    root_color: Color::BLACK,
                    root_line_height_multiplier: 1.2,
                };
                let dom_tree = page_ctx.page.document();
                let style_tree = StyleTree::build(&ctx, dom_tree, page_ctx.page.stylesheets());
                let mut tc = text_ctx.lock().unwrap();
                let mut layout_tree = layout_tree?;

                for node_id in image_node_ids {
                    LayoutEngine::relayout_node(
                        node_id,
                        Rect::new(0.0, 0.0, viewport.width, viewport.height),
                        &mut layout_tree,
                        &style_tree,
                        dom_tree,
                        &mut tc,
                        &image_ctx,
                    );
                }

                Some(layout_tree)
            })
            .await
            .unwrap()
        },
        move |layout_tree| {
            layout_tree.map_or_else(
                || Event::Browser(BrowserEvent::Error(BrowserError::ImageLoad(url))),
                |layout_tree| {
                    Event::Browser(BrowserEvent::RelayoutComplete(window_id, tab_id, generation, layout_tree))
                },
            )
        },
    )
}

/// Handles the completion of a relayout operation, updating the tab's layout tree if the
/// generation matches.
pub fn on_relayout_complete(
    application: &mut Application,
    window_id: Id,
    tab_id: TabId,
    generation: u64,
    layout_tree: layout::LayoutTree,
) -> Task<Event> {
    let ctx = if let Some(ctx) = application.browser_windows.get_mut(&window_id) {
        ctx
    } else {
        error!("Browser context not found for window ID: {}", window_id);
        return Task::none();
    };

    if let Some(tab) = ctx.tab_manager.get_tab_mut(tab_id) {
        if tab.layout_generation == generation {
            tab.layout_tree = Some(layout_tree);
        } else {
            debug!("Discarding stale relayout for tab {} (gen {} vs {})", tab_id, generation, tab.layout_generation);
        }
    }

    Task::none()
}

use std::sync::Arc;

use html_dom::NodeId;
use iced::{Task, window::Id};
use layout::{LayoutEngine, LayoutImage, Rect};
use tracing::{debug, error};

use crate::{
    core::{Application, TabId},
    errors::BrowserError,
    events::{Event, browser::BrowserEvent},
};

/// Handles the completion of an image decoding operation by updating the image context and triggering a relayout
/// for the affected nodes.
pub fn on_image_decoded(
    application: &mut Application,
    window_id: Id,
    tab_id: TabId,
    node_ids: Vec<NodeId>,
    url: String,
    image_data: LayoutImage,
) -> Task<Event> {
    let Some(ctx) = application.browser_windows.get_mut(&window_id) else {
        error!("Browser context not found for window ID: {}", window_id);
        return Task::none();
    };

    let Some(tab) = ctx.tab_manager.get_tab_mut(tab_id) else {
        return Task::none();
    };

    let Some(page_ctx) = &tab.page_ctx else {
        return Task::none();
    };

    {
        let image_ctx = page_ctx.image_context();
        let mut image_ctx = image_ctx.lock().unwrap();
        let data = Arc::new(image_data);
        for node_id in &node_ids {
            image_ctx.insert(*node_id, Arc::clone(&data));
        }
    }

    let viewport = ctx.viewport;

    let Some(page_ctx) = tab.page_ctx.clone() else {
        return Task::none();
    };

    let image_ctx = page_ctx.image_context();
    let style_tree = tab.style_tree.clone();
    let layout_tree = tab.layout_tree.clone();
    let text_ctx = ctx.text_context.clone();
    let generation = tab.layout_generation;

    Task::perform(
        async move {
            tokio::task::spawn_blocking(move || {
                let dom_tree = page_ctx.page.document();
                let style_tree = style_tree?;
                let mut text_ctx = text_ctx.lock().unwrap();
                let image_ctx = image_ctx.lock().unwrap();
                let mut layout_tree = layout_tree?;

                for node_id in node_ids {
                    LayoutEngine::relayout_node(
                        node_id,
                        Rect::new(0.0, 0.0, f64::from(viewport.width), f64::from(viewport.height)),
                        &mut layout_tree,
                        &style_tree,
                        dom_tree,
                        &mut text_ctx,
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
    let Some(ctx) = application.browser_windows.get_mut(&window_id) else {
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

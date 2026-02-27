use css_style::{AbsoluteContext, StyleTree};
use iced::Task;
use io::{CacheEntry, CacheRead};
use kernel::TabId;
use layout::{LayoutEngine, Rect};
use tracing::debug;

use crate::{
    core::Application,
    events::{Event, UiEvent},
};

/// Handles the completion of image loading, updating the tab's state and triggering a relayout if necessary.
pub(crate) fn on_image_loaded(
    application: &mut Application,
    tab_id: TabId,
    url: &String,
    vary_key: &str,
) -> Task<Event> {
    if let Some(ref cache) = application.image_cache
        && let Ok(CacheEntry::Loaded(decoded)) = cache.get_with_vary(url, vary_key)
        && let CacheRead::Hit(decoded) = (*decoded).clone()
    {
        let intrinsic_w = decoded.width as f32;
        let intrinsic_h = decoded.height as f32;
        if let Some(tab) = application.tabs.iter_mut().find(|t| t.id == tab_id) {
            tab.set_image_dimensions(url.clone(), intrinsic_w, intrinsic_h);
            tab.set_image_vary_key(url, vary_key.to_string());
        }
    }

    if let Some(tab) = application.tabs.iter_mut().find(|t| t.id == tab_id) {
        let all_done = tab.resolve_pending_image(url);

        if all_done {
            let (vw, vh) = application
                .viewports
                .get(&application.id)
                .copied()
                .unwrap_or((800.0, 600.0));

            let text_ctx = application.text_context.clone();
            let document = tab.document.clone();
            let stylesheets = tab.stylesheets.clone();
            let image_ctx = tab.image_context();
            let generation = tab.layout_generation;

            return Task::perform(
                async move {
                    tokio::task::spawn_blocking(move || {
                        let ctx = AbsoluteContext {
                            root_font_size: 16.0,
                            viewport_width: vw,
                            viewport_height: vh,
                            ..Default::default()
                        };
                        let style_tree = StyleTree::build(&ctx, &document, &stylesheets);
                        let mut tc = text_ctx.lock().unwrap();
                        LayoutEngine::compute_layout(
                            &style_tree,
                            Rect::new(0.0, 0.0, vw, vh),
                            &mut tc,
                            Some(&image_ctx),
                        )
                    })
                    .await
                    .unwrap()
                },
                move |layout_tree| {
                    Event::Ui(UiEvent::RelayoutComplete(tab_id, generation, layout_tree))
                },
            );
        }
    }

    Task::none()
}

/// Handles the completion of a relayout operation, updating the tab's layout tree if the generation matches.
pub(crate) fn on_relayout_complete(
    application: &mut Application,
    tab_id: TabId,
    generation: u64,
    layout_tree: layout::LayoutTree,
) -> Task<Event> {
    if let Some(tab) = application.tabs.iter_mut().find(|t| t.id == tab_id) {
        if tab.layout_generation == generation {
            tab.layout_tree = layout_tree;
        } else {
            debug!(
                "Discarding stale relayout for tab {} (gen {} vs {})",
                tab_id, generation, tab.layout_generation
            );
        }
    }

    Task::none()
}

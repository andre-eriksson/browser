use std::sync::Arc;

use browser_core::{EngineCommand, Commandable, HistoryState, Page, TabId, errors::NavigationError};
use css_style::{AbsoluteContext, StyleTree};
use css_values::color::Color;
use http::HeaderMap;
use iced::Task;
use io::{CacheEntry, CacheRead};
use layout::{LayoutEngine, Rect};
use renderer::image::ImageCache;
use tracing::{debug, error};

use crate::{
    core::Application,
    events::{Event, browser::BrowserEvent},
    util::image::decode_image_bytes,
};

/// Handles successful navigation by updating the tab's document, stylesheets, layout tree, and initiating image
/// fetches for any images found on the page.
pub(crate) fn on_navigation_success(
    application: &mut Application,
    tab_id: TabId,
    page: Arc<Page>,
    history_state: HistoryState,
) -> Task<Event> {
    let current_tab = application.tabs.iter_mut().find(|tab| tab.id == tab_id);

    if let Some(tab) = current_tab {
        tab.prepare_for_navigation();
        tab.history_state = history_state;

        if application.current_url
            != page
                .document_url
                .as_ref()
                .map(|url| url.to_string())
                .unwrap_or_default()
        {
            application.current_url = page
                .document_url
                .as_ref()
                .map(|url| url.to_string())
                .unwrap_or_default();
        }

        let ctx = AbsoluteContext {
            root_font_size: 16.0,
            viewport_width: application
                .viewports
                .get(&application.id)
                .map(|(w, _)| *w)
                .unwrap_or(800.0),
            viewport_height: application
                .viewports
                .get(&application.id)
                .map(|(_, h)| *h)
                .unwrap_or(600.0),
            theme_category: application.config.preferences().active_theme().category,
            document_url: page.document_url.as_ref(),
            root_line_height_multiplier: 1.2,
            root_color: Color::BLACK,
        };

        let style_tree = StyleTree::build(&ctx, page.document(), page.stylesheets());

        let image_ctx = tab.image_context();

        let mut tc = application.text_context.lock().unwrap();
        let layout_tree = LayoutEngine::compute_layout(
            &style_tree,
            application
                .viewports
                .get(&application.id)
                .map(|(w, h)| Rect::new(0.0, 0.0, *w, *h))
                .unwrap_or(Rect::new(0.0, 0.0, 800.0, 600.0)),
            &mut tc,
            Some(&image_ctx),
        );
        drop(tc);

        let image_srcs = page.images().clone();
        tab.page = page;
        tab.layout_tree = layout_tree;

        let image_cache = ImageCache::new();
        application.image_cache = Some(image_cache.clone());

        let mut cache_hit = false;
        let mut fetch_srcs: Vec<String> = Vec::new();

        for src in image_srcs {
            if src.is_empty() {
                continue;
            }

            if let Ok(CacheEntry::Loaded(decoded)) = image_cache.get_with_vary(&src, "")
                && let CacheRead::Hit(ref data) = *decoded
            {
                debug!("Image cache hit (disk): {} ({}×{})", src, data.width, data.height);
                tab.set_image_dimensions(src.clone(), data.width as f32, data.height as f32);

                tab.set_image_vary_key(&src, String::new());
                cache_hit = true;
                continue;
            }

            if image_cache.mark_pending(src.to_string()) {
                fetch_srcs.push(src);
            }
        }

        if cache_hit {
            let image_ctx = tab.image_context();
            let mut tc = application.text_context.lock().unwrap();
            let layout_tree = LayoutEngine::compute_layout(
                &style_tree,
                application
                    .viewports
                    .get(&application.id)
                    .map(|(w, h)| Rect::new(0.0, 0.0, *w, *h))
                    .unwrap_or(Rect::new(0.0, 0.0, 800.0, 600.0)),
                &mut tc,
                Some(&image_ctx),
            );
            drop(tc);
            tab.layout_tree = layout_tree;
        }

        tab.pending_image_urls = fetch_srcs.iter().cloned().collect();

        let active_tab = application.active_tab;
        let tasks: Vec<Task<Event>> = fetch_srcs
            .into_iter()
            .map(|src| {
                let browser = application.browser.clone();
                let cache = image_cache.clone();
                let src_for_err = src.clone();
                Task::perform(
                    async move {
                        let mut lock = browser.lock().await;
                        lock.execute(EngineCommand::FetchImage {
                            tab_id: active_tab,
                            url: src,
                        })
                        .await
                    },
                    move |result| match result {
                        Ok(event) => Event::EngineResponse(event),
                        Err(err) => {
                            error!("{}", err);
                            cache.mark_failed(src_for_err.clone());
                            Event::Browser(BrowserEvent::ImageLoaded(active_tab, src_for_err, String::new()))
                        }
                    },
                )
            })
            .collect();

        if !tasks.is_empty() {
            return Task::batch(tasks);
        }
    }

    Task::none()
}

pub(crate) fn on_navigation_error(_application: &mut Application, error: NavigationError) -> Task<Event> {
    error!("Navigation error: {}", error);
    Task::none()
}

/// Handles successful image loads by decoding the image bytes, storing it in the cache, and updating the
/// corresponding image elements in the tab's layout tree. If the image fails to decode, it marks the cache
/// entry as failed and triggers a UI update to reflect the failed image load
/// (e.g., showing a broken image icon).
pub(crate) fn on_image_loaded(
    application: &mut Application,
    tab_id: TabId,
    url: String,
    bytes: Vec<u8>,
    headers: HeaderMap,
) -> Task<Event> {
    if let Some(ref cache) = application.image_cache {
        let cache = cache.clone();
        let vary_key = ImageCache::resolve_vary(&headers).unwrap_or_default();
        return Task::perform(
            async move {
                match decode_image_bytes(url.clone(), bytes.as_slice()) {
                    Ok(decoded) => Ok((url, decoded)),
                    Err(err) => Err((url, err)),
                }
            },
            move |result| match result {
                Ok((url, decoded)) => {
                    let _ = cache.store(url.clone(), decoded, &headers);
                    Event::Browser(BrowserEvent::ImageLoaded(tab_id, url, vary_key))
                }
                Err((url, err)) => {
                    debug!("Image decode error: {}", err);
                    cache.mark_failed(url.clone());
                    Event::Browser(BrowserEvent::ImageLoaded(tab_id, url, vary_key))
                }
            },
        );
    }

    Task::none()
}

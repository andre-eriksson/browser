use std::sync::Arc;

use css_style::{AbsoluteContext, StyleTree};
use http::HeaderMap;
use iced::Task;
use io::{CacheEntry, CacheRead};
use kernel::{
    BrowserCommand, BrowserEvent, Commandable, Page, TabId,
    errors::{BrowserError, NavigationError},
};
use layout::{LayoutEngine, Rect};
use regex::Regex;
use renderer::image::ImageCache;
use tracing::{debug, error};
use url::Url;

use crate::{
    core::Application,
    events::{Event, UiEvent},
    util::image::decode_image_bytes,
};

/// Handles navigation to a new URL, including resolving relative URLs and applying heuristics for missing schemes.
pub(crate) fn navigate_to_url(application: &mut Application, new_url: String) -> Task<Event> {
    let browser = application.browser.clone();
    let active_tab = application.active_tab;
    let current_url = application.current_url.clone();
    let relative = Url::parse(&current_url)
        .ok()
        .and_then(|base| base.join(&new_url).ok())
        .map(|url| url.to_string());

    let url = if let Some(rel_url) = relative {
        if rel_url.contains("://") || rel_url.starts_with("about:") {
            rel_url
        } else {
            format!("http://{}", rel_url)
        }
    } else {
        let local_regex =
            Regex::new(r"^(localhost|127\.0\.0\.1|192\.168\.|10\.|172\.(1[6-9]|2[0-9]|3[0-1])\.)")
                .unwrap();

        if new_url.starts_with("http://") || new_url.starts_with("https://") {
            new_url
        } else if local_regex.is_match(new_url.as_str()) {
            format!("http://{}", new_url)
        } else {
            format!("https://{}", new_url)
        }
    };

    application.current_url = url.clone();

    Task::perform(
        async move {
            let mut lock = browser.lock().await;
            lock.execute(BrowserCommand::Navigate {
                tab_id: active_tab,
                url,
            })
            .await
        },
        |result| match result {
            Ok(event) => Event::Browser(event),
            Err(err) => match err {
                BrowserError::NavigationError(nav_err) => {
                    Event::Browser(BrowserEvent::NavigateError(nav_err))
                }
                _ => Event::Browser(BrowserEvent::Error(err)),
            },
        },
    )
}

/// Handles successful navigation by updating the tab's document, stylesheets, layout tree, and initiating image
/// fetches for any images found on the page.
pub(crate) fn on_navigation_success(
    application: &mut Application,
    tab_id: TabId,
    page: Arc<Page>,
) -> Task<Event> {
    let current_tab = application.tabs.iter_mut().find(|tab| tab.id == tab_id);

    if let Some(tab) = current_tab {
        tab.prepare_for_navigation();

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
            ..Default::default()
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
                debug!(
                    "Image cache hit (disk): {} ({}×{})",
                    src, data.width, data.height
                );
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
                        lock.execute(BrowserCommand::FetchImage {
                            tab_id: active_tab,
                            url: src,
                        })
                        .await
                    },
                    move |result| match result {
                        Ok(event) => Event::Browser(event),
                        Err(err) => {
                            error!("Image fetch error: {}", err);
                            cache.mark_failed(src_for_err.clone());
                            Event::Ui(UiEvent::ImageLoaded(active_tab, src_for_err, String::new()))
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

/// Handles navigation errors by logging the error and optionally displaying an error page or message to the user.
pub(crate) fn on_navigation_error(
    _application: &mut Application,
    error: NavigationError,
) -> Task<Event> {
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
                    Event::Ui(UiEvent::ImageLoaded(tab_id, url, vary_key))
                }
                Err((url, err)) => {
                    debug!("Image decode error: {}", err);
                    cache.mark_failed(url.clone());
                    Event::Ui(UiEvent::ImageLoaded(tab_id, url, vary_key))
                }
            },
        );
    }

    Task::none()
}

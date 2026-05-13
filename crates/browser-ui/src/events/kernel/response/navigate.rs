use browser_core::{
    Commandable, EngineCommand, EngineResponse, NavigationType, Page, PageMetadata, errors::NavigationError,
};

use html_dom::NodeId;
use iced::{Task, window::Id};
use io::{DocumentPolicy, ReferrerPolicy};
use tracing::error;

use crate::{
    core::{Application, TabId},
    errors::BrowserError,
    events::{Event, browser::BrowserEvent},
    util::image::decode_image_bytes,
};

/// Handles successful navigation by updating the tab's document, stylesheets, layout tree, and initiating image
/// fetches for any images found on the page.
pub fn on_navigation_success(
    application: &mut Application,
    window_id: Id,
    tab_id: TabId,
    page: Page,
    metadata: PageMetadata,
    _navigation_type: NavigationType,
) -> Task<Event> {
    if let Some(ctx) = application.browser_windows.get_mut(&window_id)
        && let Some(tab) = ctx.tab_manager.get_tab_mut(tab_id)
    {
        tab.prepare_for_navigation();

        // TODO: Store in permanent history.
        //if matches!(navigation_type, NavigationType::Normal) {
        //
        //}

        let viewport = ctx.viewport;
        ctx.current_url = metadata.url.to_string();

        let mut text_context = ctx.text_context.lock().unwrap();
        tab.resolve_page(viewport, &mut text_context, page, metadata, application.config, None);
        drop(text_context);

        let page_ctx = tab.page_ctx.as_ref().unwrap();

        let tasks: Vec<Task<Event>> = page_ctx
            .page
            .images()
            .iter()
            .map(|(id, src)| {
                let node_id = *id;
                let browser = application.browser.clone();
                let src = src.clone();
                let request_url = tab.page_ctx.as_ref().unwrap().metadata.url.clone();

                Task::perform(
                    async move {
                        browser
                            .execute(EngineCommand::FetchImage {
                                node_id,
                                request_url,
                                request_policies: DocumentPolicy {
                                    referrer: ReferrerPolicy::SameOrigin,
                                },
                                image_url: src,
                            })
                            .await
                    },
                    move |result| match result {
                        Ok(event) => Event::EngineResponse(window_id, tab_id, Box::new(event)),
                        Err(err) => Event::EngineResponse(window_id, tab_id, Box::new(EngineResponse::Error(err))),
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

pub fn on_navigation_error(_application: &mut Application, error: &NavigationError) -> Task<Event> {
    error!(%error, "Navigation failed");
    Task::none()
}

/// Handles successful image loads by decoding the image bytes, storing it in the cache, and updating the
/// corresponding image elements in the tab's layout tree. If the image fails to decode, it marks the cache
/// entry as failed and triggers a UI update to reflect the failed image load
/// (e.g., showing a broken image icon).
pub fn on_image_loaded(
    _application: &Application,
    window_id: Id,
    tab_id: TabId,
    node_id: NodeId,
    url: String,
    bytes: Vec<u8>,
) -> Task<Event> {
    Task::perform(
        async move {
            match decode_image_bytes(&url, bytes.as_slice()) {
                Ok(decoded) => Ok((url, decoded)),
                Err(err) => Err(err),
            }
        },
        move |result| match result {
            Ok((url, decoded)) => Event::Browser(BrowserEvent::ImageDecoded {
                window_id,
                tab_id,
                node_id,
                url,
                image_data: decoded,
            }),
            Err(err) => Event::Browser(BrowserEvent::Error(BrowserError::ImageLoad(err))),
        },
    )
}

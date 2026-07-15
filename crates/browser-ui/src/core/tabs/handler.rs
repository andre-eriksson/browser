use std::sync::Arc;

use browser_core::{
    Commandable, Document, EngineCommand, EngineResponse, NavigationType, PageMetadata, errors::CoreError,
};
use css_display::BoxTree;
use iced::Task;
use image::ImageFormat;
use layout::{LayoutImage, LayoutInput, LayoutTree, NodeId, Rect};
use regex::Regex;
use tracing::{debug, error};
use url::Url;

use crate::{
    core::{Application, Tab, TabId},
    errors::{BrowserError, TabError},
    events::{BrowserEvent, Event},
    util::image::decode_image_bytes,
    windows::browser::window::BrowserContext,
};

impl Tab {
    /// Handles the creation of a new tab when a `NewTab` event is received from the UI.
    pub fn create_new_tab(application: &mut Application, window_id: iced::window::Id) -> Task<Event> {
        match application.browser_windows.get_mut(&window_id) {
            Some(window) => {
                let new_tab_id = window.tab_manager.next_tab_id();
                window.tab_manager.add_tab(Tab::new(TabId::new(new_tab_id)));
            }
            None => {
                error!("No browser context found for window ID: {:?}", window_id);
            }
        }

        Task::none()
    }

    /// Handles the closure of a tab when a `CloseTab` event is received from the UI.
    pub fn close_tab(application: &mut Application, window_id: iced::window::Id, tab_id: TabId) -> Task<Event> {
        match application.browser_windows.get_mut(&window_id) {
            Some(window) => {
                if window.tab_manager.tabs().len() == 1 {
                    debug!(
                        "Attempted to close the last remaining tab ID: {:?} in window ID: {:?}. Closing the last tab is not allowed.",
                        tab_id, window_id
                    );
                    return Task::none();
                }

                if window.tab_manager.close_tab(tab_id).is_err() {
                    debug!("Attempted to close non-existent tab ID: {:?} in window ID: {:?}", tab_id, window_id);
                }

                if window.tab_manager.active_tab_id() == tab_id && window.tab_manager.change_to_any_tab().is_err() {
                    debug!(
                        "No tabs left to switch to after closing tab ID: {:?} in window ID: {:?}",
                        tab_id, window_id
                    );
                }
            }
            None => {
                error!("No browser context found for window ID: {:?}", window_id);
            }
        }

        Task::none()
    }

    /// Handles the switching of the active tab when a `ChangeActiveTab` event is received from the UI.
    pub fn change_active_tab(application: &mut Application, window_id: iced::window::Id, tab_id: TabId) -> Task<Event> {
        match application.browser_windows.get_mut(&window_id) {
            Some(window) => {
                if window.tab_manager.change_active_tab(tab_id).is_err() {
                    debug!("Attempted to change to non-existent tab ID: {:?} in window ID: {:?}", tab_id, window_id);
                }

                if let Some(url) = window
                    .tab_manager
                    .active_tab()
                    .and_then(|tab| tab.page.as_ref().map(|ctx| &ctx.metadata.url))
                {
                    window.current_url = url.to_string();
                } else {
                    window.current_url = BrowserContext::DEFAULT_URL.to_string();
                }
            }
            None => {
                error!("No browser context found for window ID: {:?}", window_id);
            }
        }

        Task::none()
    }

    /// Handles navigation to a new URL, including resolving relative URLs and applying heuristics for missing schemes.
    pub fn navigate_to_url(application: &mut Application, window_id: iced::window::Id, new_url: String) -> Task<Event> {
        let ctx = application
            .browser_windows
            .get_mut(&window_id)
            .expect("No browser context found for window ID");
        let browser = application.browser.clone();
        let current_url = ctx.current_url.clone();
        let relative = Url::parse(&current_url)
            .ok()
            .and_then(|base| base.join(&new_url).ok())
            .map(|url| url.to_string());

        if let Some(tab) = ctx.tab_manager.active_tab_mut()
            && let Some(page_ctx) = std::mem::take(&mut tab.page)
        {
            tab.history.add_back(page_ctx.document, page_ctx.metadata);
        }

        let url = relative.map_or_else(
            || {
                let local_regex =
                    Regex::new(r"^(localhost|127\.0\.0\.1|192\.168\.|10\.|172\.(1[6-9]|2[0-9]|3[0-1])\.)").unwrap();

                if new_url.starts_with("http://") || new_url.starts_with("https://") {
                    new_url
                } else if local_regex.is_match(new_url.as_str()) {
                    format!("http://{new_url}")
                } else {
                    format!("https://{new_url}")
                }
            },
            |rel_url| {
                if rel_url.contains("://") || rel_url.starts_with("about:") {
                    rel_url
                } else {
                    format!("http://{rel_url}")
                }
            },
        );

        let tab_id = ctx
            .tab_manager
            .active_tab()
            .expect("There should always be an active tab in the browser")
            .id;
        ctx.current_url.clone_from(&url);

        Task::perform(
            async move {
                browser
                    .execute(EngineCommand::Navigate {
                        url,
                        navigation_type: NavigationType::Normal,
                    })
                    .await
            },
            move |result| match result {
                Ok(event) => Event::EngineResponse(window_id, tab_id, Box::new(event)),
                Err(err) => match err {
                    CoreError::Navigation(nav_err) => {
                        Event::EngineResponse(window_id, tab_id, Box::new(EngineResponse::NavigateError(nav_err)))
                    }
                    _ => Event::EngineResponse(window_id, tab_id, Box::new(EngineResponse::Error(err))),
                },
            },
        )
    }

    /// Handles navigation back in the tab's history by sending a `NavigateBack` command to the browser and processing the result,
    /// including handling any navigation errors that may occur (e.g., no history to navigate back to).
    pub fn navigate_back(application: &mut Application, window_id: iced::window::Id) -> Task<Event> {
        let ctx = application
            .browser_windows
            .get_mut(&window_id)
            .expect("No browser context found for window ID");

        let tab = ctx
            .tab_manager
            .active_tab_mut()
            .expect("There should always be an active tab in the browser");

        if !tab.history.can_go_back() {
            return Task::done(Event::Browser(BrowserEvent::Error(BrowserError::Tab(TabError::NoBackHistory))));
        }

        if let Some(page_ctx) = std::mem::take(&mut tab.page) {
            match tab.history.go_back(page_ctx.document, page_ctx.metadata) {
                (Some(page), metadata) => Task::done(Event::EngineResponse(
                    window_id,
                    tab.id,
                    Box::new(EngineResponse::NavigateSuccess(page, metadata, NavigationType::Back)),
                )),
                (None, metadata) => {
                    let tab_id = tab.id;
                    let url = metadata.url.to_string();
                    let browser = Arc::clone(&application.browser);

                    Task::perform(
                        async move {
                            browser
                                .execute(EngineCommand::Navigate {
                                    url,
                                    navigation_type: NavigationType::Back,
                                })
                                .await
                        },
                        move |result| match result {
                            Ok(event) => Event::EngineResponse(window_id, tab_id, Box::new(event)),
                            Err(err) => match err {
                                CoreError::Navigation(nav_err) => Event::EngineResponse(
                                    window_id,
                                    tab_id,
                                    Box::new(EngineResponse::NavigateError(nav_err)),
                                ),
                                _ => Event::EngineResponse(window_id, tab_id, Box::new(EngineResponse::Error(err))),
                            },
                        },
                    )
                }
            }
        } else {
            Task::none()
        }
    }

    /// Handles navigation forward in the tab's history by sending a `NavigateForward` command to the browser and processing the result,
    /// including handling any navigation errors that may occur (e.g., no history to navigate forward to).
    pub fn navigate_forward(application: &mut Application, window_id: iced::window::Id) -> Task<Event> {
        let ctx = application
            .browser_windows
            .get_mut(&window_id)
            .expect("No browser context found for window ID");

        let tab = ctx
            .tab_manager
            .active_tab_mut()
            .expect("There should always be an active tab in the browser");

        if !tab.history.can_go_forward() {
            return Task::done(Event::Browser(BrowserEvent::Error(BrowserError::Tab(TabError::NoForwardHistory))));
        }

        if let Some(page_ctx) = std::mem::take(&mut tab.page) {
            match tab.history.go_forward(page_ctx.document, page_ctx.metadata) {
                (Some(page), metadata) => Task::done(Event::EngineResponse(
                    window_id,
                    tab.id,
                    Box::new(EngineResponse::NavigateSuccess(page, metadata, NavigationType::Forward)),
                )),
                (None, metadata) => {
                    let tab_id = tab.id;
                    let url = metadata.url.to_string();
                    let browser = Arc::clone(&application.browser);

                    Task::perform(
                        async move {
                            browser
                                .execute(EngineCommand::Navigate {
                                    url,
                                    navigation_type: NavigationType::Forward,
                                })
                                .await
                        },
                        move |result| match result {
                            Ok(event) => Event::EngineResponse(window_id, tab_id, Box::new(event)),
                            Err(err) => match err {
                                CoreError::Navigation(nav_err) => Event::EngineResponse(
                                    window_id,
                                    tab_id,
                                    Box::new(EngineResponse::NavigateError(nav_err)),
                                ),
                                _ => Event::EngineResponse(window_id, tab_id, Box::new(EngineResponse::Error(err))),
                            },
                        },
                    )
                }
            }
        } else {
            Task::none()
        }
    }

    /// Handles refreshing the current page by re-navigating to the current URL. It retrieves the current URL from the active tab's page
    /// information and sends a `Navigate` command to the browser with that URL. If the current URL is empty
    /// (e.g., if the tab has no page loaded), it simply returns without performing any action.
    pub fn refresh_page(application: &Application, window_id: iced::window::Id) -> Task<Event> {
        let tab = application
            .browser_windows
            .get(&window_id)
            .expect("No browser context found for window ID")
            .tab_manager
            .active_tab()
            .expect("There should always be an active tab in the browser");

        let Some(page_ctx) = &tab.page else {
            return Task::none();
        };

        let tab_id = tab.id;
        let url = page_ctx.metadata.url.to_string();
        let browser = Arc::clone(&application.browser);

        Task::perform(
            async move {
                browser
                    .execute(EngineCommand::Navigate {
                        url,
                        navigation_type: NavigationType::Reload,
                    })
                    .await
            },
            move |result| match result {
                Ok(event) => Event::EngineResponse(window_id, tab_id, Box::new(event)),
                Err(err) => match err {
                    CoreError::Navigation(nav_err) => {
                        Event::EngineResponse(window_id, tab_id, Box::new(EngineResponse::NavigateError(nav_err)))
                    }
                    _ => Event::EngineResponse(window_id, tab_id, Box::new(EngineResponse::Error(err))),
                },
            },
        )
    }

    /// Handles successful navigation by updating the tab's document, stylesheets, layout tree, and initiating image
    /// fetches for any images found on the page.
    pub fn on_navigation_success(
        application: &mut Application,
        window_id: iced::window::Id,
        tab_id: TabId,
        page: Document,
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
            tab.resolve_page(viewport, &mut text_context, page, metadata, &application.preferences, None);
            drop(text_context);

            let page_ctx = tab.page.as_ref().unwrap();

            let tasks: Vec<Task<Event>> = page_ctx
                .document
                .images()
                .iter()
                .map(|(src, ids)| {
                    let node_ids = ids.clone();
                    let browser = application.browser.clone();
                    let src = src.clone();
                    let request_url = page_ctx.metadata.url.clone();

                    Task::perform(
                        async move {
                            browser
                                .execute(EngineCommand::FetchImage {
                                    node_ids,
                                    request_url,
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

    /// Handles the completion of a relayout operation, updating the tab's layout tree if the
    /// generation matches.
    pub fn on_relayout(
        application: &mut Application,
        window_id: iced::window::Id,
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
                debug!(
                    "Discarding stale relayout for tab {} (gen {} vs {})",
                    tab_id, generation, tab.layout_generation
                );
            }
        }

        Task::none()
    }

    /// Handles the completion of an image decoding operation by updating the image context and triggering a relayout
    /// for the affected nodes.
    pub fn on_image_decoded(
        application: &mut Application,
        window_id: iced::window::Id,
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

        let Some(page_ctx) = &tab.page else {
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

        let Some(page_ctx) = tab.page.clone() else {
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
                    let dom_tree = page_ctx.document.dom();
                    let style_tree = style_tree?;
                    let mut text_ctx = text_ctx.lock().unwrap();
                    let image_ctx = image_ctx.lock().unwrap();
                    let mut layout_tree = layout_tree?;
                    let box_tree = BoxTree::new(dom_tree, &style_tree);

                    for node_id in node_ids {
                        LayoutTree::relayout_node(
                            &node_id,
                            Rect::new(0.0, 0.0, f64::from(viewport.width), f64::from(viewport.height)),
                            &mut layout_tree,
                            &mut LayoutInput {
                                dom: dom_tree,
                                box_tree: &box_tree,
                                text: &mut text_ctx,
                                image: &image_ctx,
                            },
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

    /// Handles successful image loads by decoding the image bytes, storing it in the cache, and updating the
    /// corresponding image elements in the tab's layout tree. If the image fails to decode, it marks the cache
    /// entry as failed and triggers a UI update to reflect the failed image load
    /// (e.g., showing a broken image icon).
    pub fn on_image_loaded(
        _application: &Application,
        window_id: iced::window::Id,
        tab_id: TabId,
        node_ids: Vec<NodeId>,
        content_type: String,
        url: String,
        bytes: Vec<u8>,
    ) -> Task<Event> {
        Task::perform(
            async move {
                let format = if let Some(from_mime) = ImageFormat::from_mime_type(content_type) {
                    Some(from_mime)
                } else {
                    ImageFormat::from_extension(url.rsplit('.').next().unwrap_or_default())
                };

                match decode_image_bytes(&url, bytes.as_slice(), format) {
                    Ok(decoded) => Ok((url, decoded)),
                    Err(err) => Err(err),
                }
            },
            move |result| match result {
                Ok((url, decoded)) => Event::Browser(BrowserEvent::ImageDecoded {
                    window_id,
                    tab_id,
                    node_ids,
                    url,
                    image_data: decoded,
                }),
                Err(err) => Event::Browser(BrowserEvent::Error(BrowserError::ImageLoad(err))),
            },
        )
    }
}

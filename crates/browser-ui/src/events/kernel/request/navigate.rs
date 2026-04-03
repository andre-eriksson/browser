use std::sync::Arc;

use browser_core::{Commandable, EngineCommand, EngineResponse, errors::KernelError};
use iced::{Task, window::Id};
use regex::Regex;
use url::Url;

use crate::{core::Application, events::Event};

/// Handles navigation to a new URL, including resolving relative URLs and applying heuristics for missing schemes.
pub(crate) fn navigate_to_url(application: &mut Application, window_id: Id, new_url: String) -> Task<Event> {
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

    let url = if let Some(rel_url) = relative {
        if rel_url.contains("://") || rel_url.starts_with("about:") {
            rel_url
        } else {
            format!("http://{}", rel_url)
        }
    } else {
        let local_regex =
            Regex::new(r"^(localhost|127\.0\.0\.1|192\.168\.|10\.|172\.(1[6-9]|2[0-9]|3[0-1])\.)").unwrap();

        if new_url.starts_with("http://") || new_url.starts_with("https://") {
            new_url
        } else if local_regex.is_match(new_url.as_str()) {
            format!("http://{}", new_url)
        } else {
            format!("https://{}", new_url)
        }
    };

    ctx.current_url = url.clone();

    Task::perform(
        async move {
            let mut lock = browser.lock().await;
            let active_tab = lock.tab_manager().active_tab_id();
            lock.execute(EngineCommand::Navigate {
                tab_id: active_tab,
                url,
            })
            .await
        },
        move |result| match result {
            Ok(event) => Event::EngineResponse(window_id, event),
            Err(err) => match err {
                KernelError::NavigationError(nav_err) => {
                    Event::EngineResponse(window_id, EngineResponse::NavigateError(nav_err))
                }
                _ => Event::EngineResponse(window_id, EngineResponse::Error(err)),
            },
        },
    )
}

/// Handles navigation back in the tab's history by sending a `NavigateBack` command to the browser and processing the result,
/// including handling any navigation errors that may occur (e.g., no history to navigate back to).
pub(crate) fn navigate_back(application: &mut Application, window_id: Id) -> Task<Event> {
    let browser = application.browser.clone();

    Task::perform(
        async move {
            let mut lock = browser.lock().await;
            let active_tab = lock.tab_manager().active_tab_id();
            lock.execute(EngineCommand::NavigateBack { tab_id: active_tab })
                .await
        },
        move |result| match result {
            Ok(event) => Event::EngineResponse(window_id, event),
            Err(err) => match err {
                KernelError::NavigationError(nav_err) => {
                    Event::EngineResponse(window_id, EngineResponse::NavigateError(nav_err))
                }
                _ => Event::EngineResponse(window_id, EngineResponse::Error(err)),
            },
        },
    )
}

/// Handles navigation forward in the tab's history by sending a `NavigateForward` command to the browser and processing the result,
/// including handling any navigation errors that may occur (e.g., no history to navigate forward to).
pub(crate) fn navigate_forward(application: &mut Application, window_id: Id) -> Task<Event> {
    let browser = application.browser.clone();

    Task::perform(
        async move {
            let mut lock = browser.lock().await;
            let active_tab = lock.tab_manager().active_tab_id();
            lock.execute(EngineCommand::NavigateForward { tab_id: active_tab })
                .await
        },
        move |result| match result {
            Ok(event) => Event::EngineResponse(window_id, event),
            Err(err) => match err {
                KernelError::NavigationError(nav_err) => {
                    Event::EngineResponse(window_id, EngineResponse::NavigateError(nav_err))
                }
                _ => Event::EngineResponse(window_id, EngineResponse::Error(err)),
            },
        },
    )
}

/// Handles refreshing the current page by re-navigating to the current URL. It retrieves the current URL from the active tab's page
/// information and sends a `Navigate` command to the browser with that URL. If the current URL is empty
/// (e.g., if the tab has no page loaded), it simply returns without performing any action.
pub(crate) fn refresh_page(application: &mut Application, window_id: Id) -> Task<Event> {
    let browser = Arc::clone(&application.browser);

    Task::perform(
        async move {
            let mut lock = browser.lock().await;

            lock.execute(EngineCommand::Refresh).await
        },
        move |result| match result {
            Ok(event) => Event::EngineResponse(window_id, event),
            Err(err) => match err {
                KernelError::NavigationError(nav_err) => {
                    Event::EngineResponse(window_id, EngineResponse::NavigateError(nav_err))
                }
                _ => Event::EngineResponse(window_id, EngineResponse::Error(err)),
            },
        },
    )
}

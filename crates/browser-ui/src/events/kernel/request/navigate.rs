use browser_core::{Commandable, EngineCommand, EngineResponse, errors::KernelError};
use iced::Task;
use regex::Regex;
use url::Url;

use crate::{core::Application, events::Event};

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
            Regex::new(r"^(localhost|127\.0\.0\.1|192\.168\.|10\.|172\.(1[6-9]|2[0-9]|3[0-1])\.)").unwrap();

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
            lock.execute(EngineCommand::Navigate {
                tab_id: active_tab,
                url,
            })
            .await
        },
        |result| match result {
            Ok(event) => Event::EngineResponse(event),
            Err(err) => match err {
                KernelError::NavigationError(nav_err) => Event::EngineResponse(EngineResponse::NavigateError(nav_err)),
                _ => Event::EngineResponse(EngineResponse::Error(err)),
            },
        },
    )
}

/// Handles navigation back in the tab's history by sending a `NavigateBack` command to the browser and processing the result,
/// including handling any navigation errors that may occur (e.g., no history to navigate back to).
pub(crate) fn navigate_back(application: &mut Application) -> Task<Event> {
    let browser = application.browser.clone();
    let active_tab = application.active_tab;

    Task::perform(
        async move {
            let mut lock = browser.lock().await;
            lock.execute(EngineCommand::NavigateBack { tab_id: active_tab })
                .await
        },
        |result| match result {
            Ok(event) => Event::EngineResponse(event),
            Err(err) => match err {
                KernelError::NavigationError(nav_err) => Event::EngineResponse(EngineResponse::NavigateError(nav_err)),
                _ => Event::EngineResponse(EngineResponse::Error(err)),
            },
        },
    )
}

/// Handles navigation forward in the tab's history by sending a `NavigateForward` command to the browser and processing the result,
/// including handling any navigation errors that may occur (e.g., no history to navigate forward to).
pub(crate) fn navigate_forward(application: &mut Application) -> Task<Event> {
    let browser = application.browser.clone();
    let active_tab = application.active_tab;

    Task::perform(
        async move {
            let mut lock = browser.lock().await;
            lock.execute(EngineCommand::NavigateForward { tab_id: active_tab })
                .await
        },
        |result| match result {
            Ok(event) => Event::EngineResponse(event),
            Err(err) => match err {
                KernelError::NavigationError(nav_err) => Event::EngineResponse(EngineResponse::NavigateError(nav_err)),
                _ => Event::EngineResponse(EngineResponse::Error(err)),
            },
        },
    )
}

/// Handles refreshing the current page by re-navigating to the current URL. It retrieves the current URL from the active tab's page
/// information and sends a `Navigate` command to the browser with that URL. If the current URL is empty
/// (e.g., if the tab has no page loaded), it simply returns without performing any action.
pub(crate) fn refresh_page(application: &mut Application) -> Task<Event> {
    let browser = application.browser.clone();
    let active_tab = application.active_tab;
    let url = application
        .tabs
        .iter()
        .find(|tab| tab.id == active_tab)
        .and_then(|tab| tab.page.document_url.as_ref().map(|url| url.to_string()))
        .unwrap_or_default();

    if url.is_empty() {
        return Task::none();
    }

    Task::perform(
        async move {
            let mut lock = browser.lock().await;
            lock.execute(EngineCommand::Navigate {
                tab_id: active_tab,
                url,
            })
            .await
        },
        |result| match result {
            Ok(event) => Event::EngineResponse(event),
            Err(err) => match err {
                KernelError::NavigationError(nav_err) => Event::EngineResponse(EngineResponse::NavigateError(nav_err)),
                _ => Event::EngineResponse(EngineResponse::Error(err)),
            },
        },
    )
}

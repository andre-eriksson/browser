use browser_core::{Commandable, EngineCommand, EngineResponse, NavigationType, errors::CoreError};
use iced::{Task, window::Id};
use regex::Regex;
use url::Url;

use crate::{core::Application, events::Event};

/// Handles navigation to a new URL, including resolving relative URLs and applying heuristics for missing schemes.
pub fn navigate_to_url(application: &mut Application, window_id: Id, new_url: String) -> Task<Event> {
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
        && let Some(page_ctx) = std::mem::take(&mut tab.page_ctx)
    {
        tab.history.add_back(page_ctx.page, page_ctx.metadata);
    }

    let url = relative.map_or_else(
        || {
            let local_regex =
                Regex::new(r"^(localhost|127\.0\.0\.1|192\.168\.|10\.|172\.(1[6-9]|2[0-9]|3[0-1])\.)").unwrap();

            if new_url.starts_with("http://") || new_url.starts_with("https://") {
                new_url
            } else if local_regex.is_match(new_url.as_str()) {
                format!("http://{}", new_url)
            } else {
                format!("https://{}", new_url)
            }
        },
        |rel_url| {
            if rel_url.contains("://") || rel_url.starts_with("about:") {
                rel_url
            } else {
                format!("http://{}", rel_url)
            }
        },
    );

    let tab_id = ctx
        .tab_manager
        .active_tab()
        .expect("There should always be an active tab in the browser")
        .id;
    ctx.current_url = url.clone();

    Task::perform(
        async move {
            let mut lock = browser.lock().await;
            lock.execute(EngineCommand::Navigate {
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

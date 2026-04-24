use std::mem::take;

use browser_config::BrowserConfig;
use browser_core::{Commandable, EngineCommand, EngineResponse, NavigationType};
use tracing::info;

use crate::HeadlessEngine;

pub async fn cmd_navigate(
    engine: &mut HeadlessEngine,
    config: &BrowserConfig,
    url: &str,
    navigation_type: NavigationType,
) -> Result<(), String> {
    if let Some(page) = std::mem::take(&mut engine.page)
        && let Some(metadata) = std::mem::take(&mut engine.metadata)
    {
        engine.history.add_back(page, metadata);
    }

    let result = engine
        .browser
        .execute(EngineCommand::Navigate {
            url: url.to_string(),
            navigation_type,
        })
        .await;

    match result {
        Ok(res) => match res {
            EngineResponse::NavigateSuccess(page, metadata, _navigation_type) => {
                // TODO: Store in permanent history.
                //if matches!(navigation_type, NavigationType::Normal) {
                //
                //}

                engine.page = Some(page);
                engine.metadata = Some(metadata);
                engine.recompute_layout(config);
                info!("Navigated to: {}", url);
                Ok(())
            }
            EngineResponse::NavigateError(err) => Err(format!("Navigation error: {err}")),
            _ => Err("Unexpected response from navigation command".to_string()),
        },
        Err(e) => Err(format!("Navigation error: {e}")),
    }
}

pub async fn cmd_back(engine: &mut HeadlessEngine, config: &BrowserConfig) -> Result<(), String> {
    let Some(page) = take(&mut engine.page) else {
        return Err("No current page to navigate back from".to_string());
    };

    let Some(metadata) = take(&mut engine.metadata) else {
        return Err("No current page metadata to navigate back from".to_string());
    };

    match engine.history.go_back(page, metadata) {
        (Some(page), metadata) => {
            engine.page = Some(page);
            engine.metadata = Some(metadata);
            engine.recompute_layout(config);
            info!("Navigated back");
            Ok(())
        }
        (None, metadata) => cmd_navigate(engine, config, metadata.url.as_str(), NavigationType::Back).await,
    }
}

pub async fn cmd_forward(engine: &mut HeadlessEngine, config: &BrowserConfig) -> Result<(), String> {
    let Some(page) = std::mem::take(&mut engine.page) else {
        return Err("No current page to navigate forward from".to_string());
    };

    let Some(metadata) = std::mem::take(&mut engine.metadata) else {
        return Err("No current page metadata to navigate forward from".to_string());
    };

    match engine.history.go_forward(page, metadata) {
        (Some(page), metadata) => {
            engine.page = Some(page);
            engine.metadata = Some(metadata);
            engine.recompute_layout(config);
            info!("Navigated forward");
            Ok(())
        }
        (None, metadata) => cmd_navigate(engine, config, metadata.url.as_str(), NavigationType::Forward).await,
    }
}

pub async fn cmd_reload(engine: &mut HeadlessEngine, config: &BrowserConfig) -> Result<(), String> {
    let url = engine
        .metadata
        .as_ref()
        .map(|m| m.url.clone())
        .ok_or_else(|| "No page to reload".to_string())?
        .clone();

    cmd_navigate(engine, config, url.as_str(), NavigationType::Reload).await
}

use browser_core::{Commandable, EngineCommand};
use tracing::info;

use crate::HeadlessEngine;

pub(crate) async fn cmd_navigate(engine: &mut HeadlessEngine, url: &str) -> Result<(), String> {
    let tab_id = engine.active_tab_id()?;

    let result = engine
        .browser
        .execute(EngineCommand::Navigate {
            tab_id,
            url: url.to_string(),
        })
        .await;

    match result {
        Ok(_) => {
            engine.recompute_layout();
            info!("Navigated to: {}", url);
            Ok(())
        }
        Err(e) => Err(format!("Navigation error: {}", e)),
    }
}

pub(crate) async fn cmd_back(engine: &mut HeadlessEngine) -> Result<(), String> {
    let tab_id = engine.active_tab_id()?;

    match engine
        .browser
        .execute(EngineCommand::NavigateBack { tab_id })
        .await
    {
        Ok(_) => {
            engine.recompute_layout();
            info!("Navigated back");
            Ok(())
        }
        Err(e) => Err(format!("Cannot go back: {}", e)),
    }
}

pub(crate) async fn cmd_forward(engine: &mut HeadlessEngine) -> Result<(), String> {
    let tab_id = engine.active_tab_id()?;

    match engine
        .browser
        .execute(EngineCommand::NavigateForward { tab_id })
        .await
    {
        Ok(_) => {
            engine.recompute_layout();
            info!("Navigated forward");
            Ok(())
        }
        Err(e) => Err(format!("Cannot go forward: {}", e)),
    }
}

pub(crate) async fn cmd_reload(engine: &mut HeadlessEngine) -> Result<(), String> {
    let url = {
        let tab = engine.browser.tab_manager().active_tab();
        match tab.and_then(|t| t.page().document_url().map(|u| u.to_string())) {
            Some(url) => url,
            None => return Err("No URL to reload".to_string()),
        }
    };

    cmd_navigate(engine, &url).await
}

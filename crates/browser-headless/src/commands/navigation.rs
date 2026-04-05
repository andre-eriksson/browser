use std::sync::Arc;

use browser_core::{Commandable, EngineCommand, EngineResponse};
use tracing::info;

use crate::HeadlessEngine;

pub(crate) async fn cmd_navigate(engine: &mut HeadlessEngine, url: &str) -> Result<(), String> {
    let result = engine
        .browser
        .execute(EngineCommand::Navigate {
            url: url.to_string(),
        })
        .await;

    match result {
        Ok(res) => match res {
            EngineResponse::NavigateSuccess(page) => {
                engine.history.push(Arc::clone(&engine.page));
                engine.page = page;
                engine.recompute_layout();
                info!("Navigated to: {}", url);
                Ok(())
            }
            EngineResponse::NavigateError(err) => Err(format!("Navigation error: {}", err)),
            _ => Err("Unexpected response from navigation command".to_string()),
        },
        Err(e) => Err(format!("Navigation error: {}", e)),
    }
}

pub(crate) async fn cmd_back(engine: &mut HeadlessEngine) -> Result<(), String> {
    match engine.history.go_back(Arc::clone(&engine.page)) {
        Some(page) => {
            engine.page = page;
            engine.recompute_layout();
            info!("Navigated back");
            Ok(())
        }
        None => Err("Cannot go back".to_string()),
    }
}

pub(crate) async fn cmd_forward(engine: &mut HeadlessEngine) -> Result<(), String> {
    match engine.history.go_forward(Arc::clone(&engine.page)) {
        Some(page) => {
            engine.page = page;
            engine.recompute_layout();
            info!("Navigated forward");
            Ok(())
        }
        None => Err("Cannot go forward".to_string()),
    }
}

pub(crate) async fn cmd_reload(engine: &mut HeadlessEngine) -> Result<(), String> {
    let url = engine
        .page
        .document_url()
        .ok_or_else(|| "No page to reload".to_string())?
        .clone();

    cmd_navigate(engine, url.as_str()).await
}

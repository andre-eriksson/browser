use browser_core::{Commandable, EngineCommand, EngineResponse, TabId};
use tracing::info;

use crate::HeadlessEngine;

pub(crate) fn cmd_tabs(engine: &mut HeadlessEngine) -> Result<(), String> {
    let manager = engine.browser.tab_manager();
    let active = manager.active_tab();

    if let Some(tab) = active {
        let url = tab
            .page()
            .document_url()
            .map(|u| u.as_str())
            .unwrap_or("about:blank");
        let title = tab.page().title();
        println!("* Tab {} (active): {} - {}", tab.id.0, title, url);
    } else {
        println!("No tabs open");
    }

    Ok(())
}

pub(crate) async fn cmd_switch_tab(engine: &mut HeadlessEngine, id: usize) -> Result<(), String> {
    match engine
        .browser
        .execute(EngineCommand::ChangeActiveTab { tab_id: TabId(id) })
        .await
    {
        Ok(_) => {
            engine.recompute_layout();
            info!("Switched to tab {}", id);
            Ok(())
        }
        Err(e) => Err(format!("Cannot switch tab: {}", e)),
    }
}

pub(crate) async fn cmd_new_tab(engine: &mut HeadlessEngine) -> Result<(), String> {
    match engine.browser.execute(EngineCommand::AddTab).await {
        Ok(response) => {
            if let EngineResponse::TabAdded(tab_id) = response {
                info!("Created new tab: {}", tab_id.0);
            }
            Ok(())
        }
        Err(e) => Err(format!("Cannot create tab: {}", e)),
    }
}

pub(crate) async fn cmd_close_tab(engine: &mut HeadlessEngine, id: Option<usize>) -> Result<(), String> {
    let tab_id = match id {
        Some(id) => TabId(id),
        None => engine.active_tab_id()?,
    };

    match engine
        .browser
        .execute(EngineCommand::CloseTab { tab_id })
        .await
    {
        Ok(_) => {
            engine.layout_tree = None;
            engine.style_tree = None;
            info!("Closed tab {}", tab_id.0);
            Ok(())
        }
        Err(e) => Err(format!("Cannot close tab: {}", e)),
    }
}

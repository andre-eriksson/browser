use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use cookies::cookie_store::CookieJar;
use network::clients::reqwest::ReqwestClient;

use crate::{
    BrowserCommand, BrowserEvent, Commandable, Emitter, TabId,
    commands::{
        navigate::navigate,
        tab::{add_tab, change_active_tab, close_tab},
    },
    navigation::{NavigationContext, ScriptExecutor, StyleProcessor},
    service::network::{header::DefaultHeaders, service::NetworkService},
    tab::{Tab, TabManager, TabMetadata},
};

pub struct HeadlessBrowser {
    tab_manager: TabManager,
    _emitter: Box<dyn Emitter<BrowserEvent> + Send + Sync>,
    network: NetworkService,
}

impl HeadlessBrowser {
    pub fn new(emitter: Box<dyn Emitter<BrowserEvent> + Send + Sync>) -> Self {
        let http_client = Box::new(ReqwestClient::new());
        // TODO: Load cookies from persistent storage
        let cookie_jar = Arc::new(Mutex::new(CookieJar::new()));
        let headers = Arc::new(DefaultHeaders::create_headless_browser_headers());

        let tab_manager = TabManager::new(Tab::new(TabId(0)));

        HeadlessBrowser {
            tab_manager,
            _emitter: emitter,
            network: NetworkService::new(http_client, cookie_jar, headers),
        }
    }

    pub fn print_body(&self) {
        if let Some(active_tab) = self.tab_manager.active_tab() {
            println!("{}", active_tab.document());
        } else {
            println!("No active tab.");
        }
    }
}

impl StyleProcessor for HeadlessBrowser {
    fn process_css(&self, _css: &str, _stylesheets: &mut Vec<css_cssom::CSSStyleSheet>) {
        // Nothing
    }
}

impl ScriptExecutor for HeadlessBrowser {
    fn execute_script(&self, _script: &str) {
        // TODO: Implement script execution in headless browser since it can modify the DOM.
    }
}

impl NavigationContext for HeadlessBrowser {
    fn network_service(&mut self) -> &mut NetworkService {
        &mut self.network
    }

    fn script_executor(&self) -> &dyn ScriptExecutor {
        self
    }

    fn style_processor(&self) -> &dyn StyleProcessor {
        self
    }

    fn tab_manager(&mut self) -> &mut TabManager {
        &mut self.tab_manager
    }
}

#[async_trait]
impl Commandable for HeadlessBrowser {
    async fn execute(&mut self, command: BrowserCommand) -> Result<BrowserEvent, String> {
        match command {
            BrowserCommand::Navigate { tab_id, url } => {
                let result = navigate(self, tab_id, &url, &mut Vec::new()).await?;

                let tab = self
                    .tab_manager
                    .get_tab_mut(tab_id)
                    .ok_or_else(|| format!("Tab with id {:?} not found in TabManager", tab_id))?;

                tab.set_document(result.dom_tree.clone());

                let tab_metadata = TabMetadata {
                    id: tab.id,
                    title: result.metadata.title.unwrap_or(url.to_string()),
                    document: result.dom_tree,
                    stylesheets: tab.stylesheets().clone(),
                };

                Ok(BrowserEvent::NavigateSuccess(tab_metadata))
            }
            BrowserCommand::AddTab => Ok(add_tab(&mut self.tab_manager)),
            BrowserCommand::CloseTab { tab_id } => close_tab(&mut self.tab_manager, tab_id),
            BrowserCommand::ChangeActiveTab { tab_id } => {
                change_active_tab(&mut self.tab_manager, tab_id)
            }
        }
    }
}

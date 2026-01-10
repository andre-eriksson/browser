use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use cookies::cookie_store::CookieJar;
use http::HeaderMap;
use network::http::client::HttpClient;

use crate::{
    BrowserCommand, BrowserEvent, Commandable, Emitter, TabId,
    commands::{
        navigate::navigate_to,
        tab::{add_tab, change_active_tab, close_tab},
    },
    navigation::{NavigationContext, ScriptExecutor},
    tab::{Tab, TabManager},
};

pub struct HeadlessBrowser {
    tab_manager: TabManager,
    emitter: Box<dyn Emitter<BrowserEvent> + Send + Sync>,
    http_client: Box<dyn HttpClient>,
    _cookie_jar: Arc<Mutex<CookieJar>>,
    _headers: Arc<HeaderMap>,
}

impl HeadlessBrowser {
    pub fn new(
        emitter: Box<dyn Emitter<BrowserEvent> + Send + Sync>,
        http_client: Box<dyn HttpClient>,
        cookie_jar: Arc<Mutex<CookieJar>>,
        headers: Arc<HeaderMap>,
    ) -> Self {
        let tab_manager = TabManager::new(Tab::new(TabId(0)));

        HeadlessBrowser {
            tab_manager,
            emitter,
            http_client,
            _cookie_jar: cookie_jar,
            _headers: headers,
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

impl ScriptExecutor for HeadlessBrowser {
    fn execute_script(&mut self, _script: &str) {
        // TODO: Implement script execution in headless browser since it can modify the DOM.
    }
}

impl NavigationContext for HeadlessBrowser {
    fn http_client(&self) -> &dyn HttpClient {
        &*self.http_client
    }

    fn tab_manager(&mut self) -> &mut TabManager {
        &mut self.tab_manager
    }

    fn default_stylesheet(&self) -> Option<&css_cssom::CSSStyleSheet> {
        None
    }

    fn emit_event(&self, event: BrowserEvent) {
        self.emitter.emit(event);
    }

    fn process_css(&mut self, _css: &str, _stylesheets: &mut Vec<css_cssom::CSSStyleSheet>) {
        // No-op for headless browser
    }

    fn execute_script(&mut self, script: &str) {
        ScriptExecutor::execute_script(self, script);
    }
}

#[async_trait]
impl Commandable for HeadlessBrowser {
    async fn execute(&mut self, command: BrowserCommand) -> Result<BrowserEvent, String> {
        match command {
            BrowserCommand::Navigate { tab_id, url } => navigate_to(self, tab_id, url).await,
            BrowserCommand::AddTab => Ok(add_tab(&mut self.tab_manager)),
            BrowserCommand::CloseTab { tab_id } => close_tab(&mut self.tab_manager, tab_id),
            BrowserCommand::ChangeActiveTab { tab_id } => change_active_tab(self, tab_id),
        }
    }
}

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use cookies::CookieJar;
use errors::browser::{BrowserError, TabError};
use network::clients::reqwest::ReqwestClient;

use crate::{
    BrowserCommand, BrowserEvent, Commandable, Emitter,
    commands::{
        navigate::navigate,
        tab::{add_tab, change_active_tab, close_tab},
    },
    navigation::{NavigationContext, ScriptExecutor, StyleProcessor},
    service::network::{
        header::{DefaultHeaders, HeaderType},
        service::NetworkService,
    },
    tab::{
        manager::TabManager,
        tabs::{Tab, TabId},
    },
};

pub struct HeadlessBrowser {
    tab_manager: TabManager,
    _emitter: Box<dyn Emitter<BrowserEvent> + Send + Sync>,
    network: NetworkService,
}

impl HeadlessBrowser {
    pub fn new(emitter: Box<dyn Emitter<BrowserEvent> + Send + Sync>) -> Self {
        let http_client = Box::new(ReqwestClient::new());
        let cookie_jar = Arc::new(Mutex::new(CookieJar::load()));
        let headers = Arc::new(DefaultHeaders::create_browser_headers(
            HeaderType::HeadlessBrowser,
        ));

        let tab_manager = TabManager::new(Tab::new(TabId(0)));

        HeadlessBrowser {
            tab_manager,
            _emitter: emitter,
            network: NetworkService::new(http_client, cookie_jar, headers),
        }
    }

    pub fn print_body(&self) {
        if let Some(active_tab) = self.tab_manager.active_tab() {
            println!("{}", active_tab.page().document());
        } else {
            println!("No active tab.");
        }
    }

    pub fn print_cookies(&self) {
        let cookie_jar = self.network.cookies().lock().unwrap().clone();
        for cookie in cookie_jar.into_iter() {
            println!("{}", cookie);
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
    async fn execute(&mut self, command: BrowserCommand) -> Result<BrowserEvent, BrowserError> {
        match command {
            BrowserCommand::Navigate { tab_id, url } => {
                let stylesheets = Vec::new();

                let page = navigate(self, tab_id, &url, stylesheets).await?;

                let tab = self
                    .tab_manager
                    .get_tab_mut(tab_id)
                    .ok_or_else(|| BrowserError::TabError(TabError::TabNotFound(tab_id.0)))?;

                tab.set_page(page);
                let page = tab.page().clone();

                Ok(BrowserEvent::NavigateSuccess(tab_id, page))
            }
            BrowserCommand::AddTab => Ok(add_tab(&mut self.tab_manager)),
            BrowserCommand::CloseTab { tab_id } => close_tab(&mut self.tab_manager, tab_id),
            BrowserCommand::ChangeActiveTab { tab_id } => {
                change_active_tab(&mut self.tab_manager, tab_id)
            }
        }
    }
}

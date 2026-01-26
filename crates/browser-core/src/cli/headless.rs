use std::sync::{Arc, RwLock};

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
    pub fn new(
        custom_headers: &Vec<String>,
        emitter: Box<dyn Emitter<BrowserEvent> + Send + Sync>,
    ) -> Self {
        let http_client = Box::new(ReqwestClient::new());
        let cookie_jar = RwLock::new(CookieJar::load());

        let mut headers = DefaultHeaders::create_browser_headers(HeaderType::HeadlessBrowser);
        for header in custom_headers {
            if let Some((key, value)) = header.split_once(':')
                && let Ok(header_name) = http::header::HeaderName::from_bytes(key.trim().as_bytes())
                && let Ok(header_value) = http::header::HeaderValue::from_str(value.trim())
            {
                headers.insert(header_name, header_value);
            }
        }

        let tab_manager = TabManager::new(Tab::new(TabId(0)));

        HeadlessBrowser {
            tab_manager,
            _emitter: emitter,
            network: NetworkService::new(http_client, cookie_jar, Arc::new(headers)),
        }
    }

    pub fn print_headers(&mut self) {
        for header in self.network_service().browser_headers().iter() {
            println!("{}: {}", header.0, header.1.to_str().unwrap_or(""));
        }
    }

    pub fn print_body(&self) {
        if let Some(active_tab) = self.tab_manager.active_tab() {
            println!("{}", active_tab.page().document());
        } else {
            println!("No active tab.");
        }
    }

    pub fn print_cookies(&mut self, domain: Option<&str>) {
        if domain.is_none() {
            for cookie in self.network_service().cookie_jar().cookies() {
                println!("{}", cookie);
            }
            return;
        }

        let domain = domain.unwrap();

        for cookie in self
            .network_service()
            .cookie_jar()
            .get_cookies_for_domain(domain)
        {
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

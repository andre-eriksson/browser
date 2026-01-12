use std::{
    sync::{Arc, Mutex},
    vec,
};

use assets::{ASSETS, constants::DEFAULT_CSS};
use async_trait::async_trait;
use cookies::cookie_store::CookieJar;
use css_cssom::{CSSStyleSheet, StylesheetOrigin};
use network::clients::reqwest::ReqwestClient;
use url::Url;

use crate::{
    commands::{
        navigate::navigate,
        tab::{add_tab, change_active_tab, close_tab},
    },
    events::{BrowserCommand, BrowserEvent, Commandable, Emitter},
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

pub struct Browser {
    tab_manager: TabManager,
    default_stylesheet: CSSStyleSheet,
    _emitter: Box<dyn Emitter<BrowserEvent> + Send + Sync>,
    network: NetworkService,
}

impl Browser {
    pub fn new(emitter: Box<dyn Emitter<BrowserEvent> + Send + Sync>) -> Self {
        let http_client = Box::new(ReqwestClient::new());
        // TODO: Load cookies from persistent storage
        let cookie_jar = Arc::new(Mutex::new(CookieJar::new()));
        let headers = Arc::new(DefaultHeaders::create_browser_headers(HeaderType::Browser));

        let user_agent_css = ASSETS.read().unwrap().load_embedded(DEFAULT_CSS);

        // TODO: Load the CSSStyleSheet from cache before parsing it again
        let stylesheet = CSSStyleSheet::from_css(
            std::str::from_utf8(&user_agent_css).unwrap_or_default(),
            StylesheetOrigin::UserAgent,
        );

        let tab_manager = TabManager::new(Tab::new(TabId(0)));

        Browser {
            tab_manager,
            default_stylesheet: stylesheet,
            _emitter: emitter,
            network: NetworkService::new(http_client, cookie_jar, headers),
        }
    }
}

impl ScriptExecutor for Browser {
    fn execute_script(&self, _script: &str) {
        //debug!("Executing script: {}", script);
    }
}

impl StyleProcessor for Browser {
    fn process_css(&self, css: &str, stylesheets: &mut Vec<CSSStyleSheet>) {
        let stylesheet = CSSStyleSheet::from_css(css, StylesheetOrigin::Author);
        stylesheets.push(stylesheet);
    }
}

impl NavigationContext for Browser {
    fn script_executor(&self) -> &dyn ScriptExecutor {
        self
    }

    fn style_processor(&self) -> &dyn StyleProcessor {
        self
    }

    fn network_service(&mut self) -> &mut NetworkService {
        &mut self.network
    }

    fn tab_manager(&mut self) -> &mut TabManager {
        &mut self.tab_manager
    }
}

#[async_trait]
impl Commandable for Browser {
    async fn execute(&mut self, command: BrowserCommand) -> Result<BrowserEvent, String> {
        match command {
            BrowserCommand::Navigate { tab_id, url } => {
                let stylesheets = vec![self.default_stylesheet.clone()];

                let qualified_url =
                    Url::parse(&url).map_err(|e| format!("Failed to parse URL: {}", e))?;

                let page = navigate(self, tab_id, &qualified_url, stylesheets).await?;

                let tab = self
                    .tab_manager
                    .get_tab_mut(tab_id)
                    .ok_or_else(|| format!("Tab with id {:?} not found in TabManager", tab_id))?;

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

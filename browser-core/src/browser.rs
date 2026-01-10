use std::sync::{Arc, Mutex};

use assets::{ASSETS, constants::DEFAULT_CSS};
use async_trait::async_trait;
use cookies::cookie_store::CookieJar;
use css_cssom::{CSSStyleSheet, StylesheetOrigin};
use http::HeaderMap;
use network::http::client::HttpClient;

use crate::{
    commands::{
        navigate::navigate_to,
        tab::{add_tab, change_active_tab, close_tab},
    },
    events::{BrowserCommand, BrowserEvent, Commandable, Emitter},
    tab::{Tab, TabId, TabManager},
};

pub struct Browser {
    tab_manager: TabManager,

    default_stylesheet: CSSStyleSheet,

    emitter: Box<dyn Emitter<BrowserEvent> + Send + Sync>,

    http_client: Box<dyn HttpClient>,
    _cookie_jar: Arc<Mutex<CookieJar>>,
    _headers: Arc<HeaderMap>,
}

impl Browser {
    pub fn new(
        emitter: Box<dyn Emitter<BrowserEvent> + Send + Sync>,
        http_client: Box<dyn HttpClient>,
        cookie_jar: Arc<Mutex<CookieJar>>,
        headers: Arc<HeaderMap>,
    ) -> Self {
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
            emitter,
            http_client,
            _cookie_jar: cookie_jar,
            _headers: headers,
        }
    }

    pub(crate) fn execute_script(&mut self, _script: &str) {
        //debug!("Executing script: {}", script);
    }

    pub(crate) fn process_css(&mut self, css: &str, stylesheets: &mut Vec<CSSStyleSheet>) {
        let stylesheet = CSSStyleSheet::from_css(css, StylesheetOrigin::Author);
        stylesheets.push(stylesheet);
    }

    pub(crate) fn emit_event(&self, event: BrowserEvent) {
        self.emitter.emit(event);
    }

    pub(crate) fn http_client(&self) -> &dyn HttpClient {
        self.http_client.as_ref()
    }

    pub(crate) fn tab_manager(&mut self) -> &mut TabManager {
        &mut self.tab_manager
    }

    pub(crate) fn default_stylesheet(&self) -> &CSSStyleSheet {
        &self.default_stylesheet
    }
}

#[async_trait]
impl Commandable for Browser {
    async fn execute(&mut self, command: BrowserCommand) -> Result<BrowserEvent, String> {
        match command {
            BrowserCommand::Navigate { tab_id, url } => navigate_to(self, tab_id, url).await,
            BrowserCommand::AddTab => Ok(add_tab(self)),
            BrowserCommand::CloseTab { tab_id } => close_tab(self, tab_id),
            BrowserCommand::ChangeActiveTab { tab_id } => change_active_tab(self, tab_id),
        }
    }
}

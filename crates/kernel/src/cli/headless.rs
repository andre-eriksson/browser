use std::sync::{Arc, Mutex};

use crate::{
    commands::load_image,
    errors::{BrowserError, TabError},
    header::{DefaultHeaders, HeaderType},
};
use async_trait::async_trait;
use cli::args::BrowserArgs;
use cookies::CookieJar;
use network::{HeaderMap, HeaderName, HeaderValue, client::HttpClient, clients::reqwest::ReqwestClient};

use crate::{
    BrowserCommand, BrowserEvent, Commandable, Emitter,
    commands::{add_tab, change_active_tab, close_tab, navigate},
    navigation::{NavigationContext, ScriptExecutor},
    tab::{
        manager::TabManager,
        tabs::{Tab, TabId},
    },
};

pub struct HeadlessBrowser {
    tab_manager: TabManager,
    _emitter: Box<dyn Emitter<BrowserEvent> + Send + Sync>,
    cookie_jar: Arc<Mutex<CookieJar>>,
    http_client: Box<dyn HttpClient>,
    headers: Arc<HeaderMap>,
}

impl HeadlessBrowser {
    pub fn new(args: &BrowserArgs, emitter: Box<dyn Emitter<BrowserEvent> + Send + Sync>) -> Self {
        let http_client = Box::new(ReqwestClient::new());
        let cookie_jar = Arc::new(Mutex::new(CookieJar::load()));

        let mut headers = DefaultHeaders::create_browser_headers(HeaderType::HeadlessBrowser);
        for header in args.headers.iter() {
            if let Some((key, value)) = header.split_once(':')
                && let Ok(header_name) = HeaderName::from_bytes(key.trim().as_bytes())
                && let Ok(header_value) = HeaderValue::from_str(value.trim())
            {
                headers.insert(header_name, header_value);
            }
        }

        let tab_manager = TabManager::new(Tab::new(TabId(0)));

        HeadlessBrowser {
            tab_manager,
            _emitter: emitter,
            cookie_jar,
            http_client,
            headers: Arc::new(headers),
        }
    }

    pub fn print_headers(&mut self) {
        for header in self.headers.iter() {
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
        let jar = self.cookie_jar.lock().unwrap();

        if domain.is_none() {
            for cookie in jar.cookies() {
                println!("{}", cookie);
            }
            return;
        }

        let domain = domain.unwrap();

        for cookie in jar.get_cookies_for_domain(domain) {
            println!("{}", cookie);
        }
    }
}

impl ScriptExecutor for HeadlessBrowser {
    fn execute_script(&self, _script: &str) {
        // TODO: Implement script execution in headless browser since it can modify the DOM.
    }
}

impl NavigationContext for HeadlessBrowser {
    fn script_executor(&self) -> &dyn ScriptExecutor {
        self
    }

    fn cookie_jar(&mut self) -> &mut Arc<Mutex<CookieJar>> {
        &mut self.cookie_jar
    }

    fn headers(&self) -> &Arc<HeaderMap> {
        &self.headers
    }

    fn http_client(&self) -> &dyn HttpClient {
        self.http_client.as_ref()
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

                let page = Arc::new(navigate(self, tab_id, &url, stylesheets).await?);

                let tab = self
                    .tab_manager
                    .get_tab_mut(tab_id)
                    .ok_or_else(|| BrowserError::TabError(TabError::TabNotFound(tab_id.0)))?;

                tab.set_page(Arc::clone(&page));

                Ok(BrowserEvent::NavigateSuccess(tab_id, page))
            }
            BrowserCommand::AddTab => Ok(add_tab(&mut self.tab_manager)),
            BrowserCommand::CloseTab { tab_id } => close_tab(&mut self.tab_manager, tab_id),
            BrowserCommand::ChangeActiveTab { tab_id } => change_active_tab(&mut self.tab_manager, tab_id),
            BrowserCommand::FetchImage { tab_id, url } => load_image(self, tab_id, &url).await,
        }
    }
}

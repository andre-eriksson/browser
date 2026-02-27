use std::{
    sync::{Arc, Mutex},
    vec,
};

use crate::{
    commands::load_image,
    errors::{BrowserError, TabError},
    header::{DefaultHeaders, HeaderType},
};
use async_trait::async_trait;
use cli::args::BrowserArgs;
use cookies::CookieJar;
use css_cssom::{CSSStyleSheet, StylesheetOrigin};
use io::{Resource, embeded::DEFAULT_CSS, files::CACHE_USER_AGENT};
use network::{
    HeaderMap, HeaderName, HeaderValue, client::HttpClient, clients::reqwest::ReqwestClient,
};
use postcard::{from_bytes, to_stdvec};
use tracing::instrument;

use crate::{
    commands::{add_tab, change_active_tab, close_tab, navigate},
    events::{BrowserCommand, BrowserEvent, Commandable, Emitter},
    navigation::{NavigationContext, ScriptExecutor},
    tab::{
        manager::TabManager,
        tabs::{Tab, TabId},
    },
};

pub struct Browser {
    tab_manager: TabManager,
    default_stylesheet: Option<CSSStyleSheet>,
    cookie_jar: Arc<Mutex<CookieJar>>,
    http_client: Box<dyn HttpClient>,
    headers: Arc<HeaderMap>,
    _emitter: Box<dyn Emitter<BrowserEvent> + Send + Sync>,
}

impl Browser {
    pub fn new(args: &BrowserArgs, emitter: Box<dyn Emitter<BrowserEvent> + Send + Sync>) -> Self {
        let http_client = Box::new(ReqwestClient::new());
        let cookie_jar = Arc::new(Mutex::new(CookieJar::load()));

        let mut headers = DefaultHeaders::create_browser_headers(HeaderType::Browser);
        for header in args.headers.iter() {
            if let Some((key, value)) = header.split_once(':')
                && let Ok(header_name) = HeaderName::from_bytes(key.trim().as_bytes())
                && let Ok(header_value) = HeaderValue::from_str(value.trim())
            {
                headers.insert(header_name, header_value);
            }
        }

        let user_agent_css = Resource::load_embedded(DEFAULT_CSS);

        let stylesheet = if args.enable_ua_css {
            match Resource::load(CACHE_USER_AGENT) {
                Ok(data) => {
                    let out: CSSStyleSheet = from_bytes(data.as_slice()).unwrap_or_else(|_| {
                        CSSStyleSheet::from_css(
                            std::str::from_utf8(&user_agent_css).unwrap_or_default(),
                            StylesheetOrigin::UserAgent,
                            false,
                        )
                    });

                    Some(out)
                }
                Err(_) => {
                    let parsed = CSSStyleSheet::from_css(
                        std::str::from_utf8(&user_agent_css).unwrap_or_default(),
                        StylesheetOrigin::UserAgent,
                        false,
                    );

                    let serialized = to_stdvec(&parsed).unwrap();

                    Resource::write(CACHE_USER_AGENT, serialized.as_slice()).ok();

                    Some(parsed)
                }
            }
        } else {
            None
        };

        let tab_manager = TabManager::new(Tab::new(TabId(0)));

        Browser {
            tab_manager,
            default_stylesheet: stylesheet,
            _emitter: emitter,
            cookie_jar,
            http_client,
            headers: Arc::new(headers),
        }
    }
}

impl ScriptExecutor for Browser {
    fn execute_script(&self, _script: &str) {
        //debug!("Executing script: {}", script);
    }
}

impl NavigationContext for Browser {
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
impl Commandable for Browser {
    #[instrument(skip(self))]
    async fn execute(&mut self, command: BrowserCommand) -> Result<BrowserEvent, BrowserError> {
        match command {
            BrowserCommand::Navigate { tab_id, url } => {
                let stylesheets = if let Some(default) = &self.default_stylesheet {
                    vec![default.clone()]
                } else {
                    vec![]
                };

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
            BrowserCommand::ChangeActiveTab { tab_id } => {
                change_active_tab(&mut self.tab_manager, tab_id)
            }
            BrowserCommand::FetchImage { tab_id, url } => load_image(self, tab_id, &url).await,
        }
    }
}

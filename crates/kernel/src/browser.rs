use std::{
    sync::{Arc, RwLock},
    vec,
};

use crate::errors::{BrowserError, TabError};
use async_trait::async_trait;
use cli::args::BrowserArgs;
use constants::files::CACHE_USER_AGENT;
use cookies::CookieJar;
use css_cssom::{CSSStyleSheet, StylesheetOrigin};
use io::{ASSETS, constants::DEFAULT_CSS};
use network::{HeaderName, HeaderValue, clients::reqwest::ReqwestClient};
use postcard::{from_bytes, to_stdvec};
use storage::files::{read_file_from_cache, write_file_to_cache};
use tracing::instrument;

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
    default_stylesheet: Option<CSSStyleSheet>,
    _emitter: Box<dyn Emitter<BrowserEvent> + Send + Sync>,
    network: NetworkService,
}

impl Browser {
    pub fn new(args: &BrowserArgs, emitter: Box<dyn Emitter<BrowserEvent> + Send + Sync>) -> Self {
        let http_client = Box::new(ReqwestClient::new());
        let cookie_jar = RwLock::new(CookieJar::load());

        let mut headers = DefaultHeaders::create_browser_headers(HeaderType::Browser);
        for header in args.headers.iter() {
            if let Some((key, value)) = header.split_once(':')
                && let Ok(header_name) = HeaderName::from_bytes(key.trim().as_bytes())
                && let Ok(header_value) = HeaderValue::from_str(value.trim())
            {
                headers.insert(header_name, header_value);
            }
        }

        let user_agent_css = ASSETS.read().unwrap().load_embedded(DEFAULT_CSS);

        let stylesheet = if args.enable_ua_css {
            match read_file_from_cache(CACHE_USER_AGENT) {
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

                    write_file_to_cache(CACHE_USER_AGENT, serialized.as_slice()).ok();

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
            network: NetworkService::new(http_client, cookie_jar, Arc::new(headers)),
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
        let stylesheet = CSSStyleSheet::from_css(css, StylesheetOrigin::Author, true);
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
    #[instrument(skip(self))]
    async fn execute(&mut self, command: BrowserCommand) -> Result<BrowserEvent, BrowserError> {
        match command {
            BrowserCommand::Navigate { tab_id, url } => {
                let stylesheets = if let Some(default) = &self.default_stylesheet {
                    vec![default.clone()]
                } else {
                    vec![]
                };

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

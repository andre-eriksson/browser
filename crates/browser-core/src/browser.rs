use std::{
    sync::{Arc, Mutex},
    vec,
};

use crate::{
    DevtoolsPage,
    commands::{load_image, parse_devtools_html},
    errors::KernelError,
};
use async_trait::async_trait;
use browser_config::BrowserConfig;
use cookies::CookieJar;
use css_cssom::{CSSStyleSheet, StylesheetOrigin};
use io::{
    Resource,
    embeded::{DEFAULT_CSS, DEVTOOLS_CSS},
    files::CACHE_USER_AGENT,
};
use network::{HeaderMap, client::HttpClient, clients::reqwest::ReqwestClient};
use postcard::{from_bytes, to_stdvec};
use tracing::instrument;

use crate::{
    commands::navigate,
    events::{Commandable, EngineCommand, EngineResponse},
    navigation::{NavigationContext, ScriptExecutor},
};

pub struct Browser {
    default_stylesheet: Option<CSSStyleSheet>,
    cookie_jar: Arc<Mutex<CookieJar>>,
    http_client: Box<dyn HttpClient>,
    headers: &'static HeaderMap,
}

impl Browser {
    pub fn new(config: &'static BrowserConfig) -> Self {
        let http_client = Box::new(ReqwestClient::new());
        let cookie_jar = Arc::new(Mutex::new(CookieJar::load()));

        let user_agent_css = Resource::load_embedded(DEFAULT_CSS);

        let stylesheet = if config.args().enable_ua_css {
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

        Browser {
            default_stylesheet: stylesheet,
            cookie_jar,
            http_client,
            headers: config.headers(),
        }
    }

    pub fn headers(&self) -> &HeaderMap {
        self.headers
    }

    pub fn cookie_jar(&mut self) -> &mut Arc<Mutex<CookieJar>> {
        &mut self.cookie_jar
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

    fn headers(&self) -> &HeaderMap {
        self.headers
    }

    fn http_client(&self) -> &dyn HttpClient {
        self.http_client.as_ref()
    }
}

#[async_trait]
impl Commandable for Browser {
    #[instrument(skip(self))]
    async fn execute(&mut self, command: EngineCommand) -> Result<EngineResponse, KernelError> {
        match command {
            EngineCommand::Navigate { url } => {
                let stylesheets = if let Some(default) = &self.default_stylesheet {
                    vec![default.clone()]
                } else {
                    vec![]
                };

                let page = Arc::new(navigate(self, &url, stylesheets).await?);

                Ok(EngineResponse::NavigateSuccess(page))
            }
            EngineCommand::GetDevtoolsPage { document } => {
                let default_css = {
                    let css_resource = Resource::load_embedded(DEFAULT_CSS);
                    CSSStyleSheet::from_css(
                        // SAFETY: The CSS is ASCII and embedded in the binary, so it should always be valid UTF-8.
                        unsafe { str::from_utf8_unchecked(css_resource.as_slice()) },
                        StylesheetOrigin::UserAgent,
                        false,
                    )
                };
                let devtools_css = {
                    let css_resource = Resource::load_embedded(DEVTOOLS_CSS);
                    CSSStyleSheet::from_css(
                        // SAFETY: The CSS is ASCII and embedded in the binary, so it should always be valid UTF-8.
                        unsafe { str::from_utf8_unchecked(css_resource.as_slice()) },
                        StylesheetOrigin::Author,
                        false,
                    )
                };

                let stylesheets = vec![default_css, devtools_css];
                let dom =
                    parse_devtools_html(&document).map_err(|e| KernelError::DevtoolsGenerationError(e.to_string()))?;

                let devtools_page = DevtoolsPage::new(dom, stylesheets);

                Ok(EngineResponse::DevtoolsPageReady(devtools_page))
            }
            EngineCommand::FetchImage {
                request_url,
                request_policies,
                image_url,
            } => load_image(self, request_url, request_policies, &image_url).await,
        }
    }
}

use std::vec;

use crate::{Document, commands::parse_devtools_html, database::Databases, errors::CoreError};
use async_trait::async_trait;
use browser_args::BrowserArgs;
use browser_config::BrowserConfig;
use cookies::CookieJar;
use css_cssom::{CSSStyleSheet, StylesheetOrigin};
use io::{
    HttpCache, Resource,
    embeded::{DEFAULT_CSS, DEVTOOLS_CSS},
    files::CACHE_USER_AGENT,
};
use network::{HeaderMap, client::HttpClient, clients::reqwest::ReqwestClient};
use postcard::{from_bytes, to_stdvec};
use tracing::{Instrument, instrument, trace, warn};

use crate::{
    events::{Commandable, EngineCommand, EngineResponse},
    navigation::ScriptExecutor,
};

#[derive(Debug)]
pub struct Browser {
    config: BrowserConfig,
    databases: Databases,
    default_stylesheet: Option<CSSStyleSheet>,
    http_client: Box<dyn HttpClient>,
}

impl Browser {
    /// Maximum allowed size for the user agent stylesheet, set to 50 KiB.
    const MAX_USER_AGENT_CSS_SIZE: Option<usize> = Some(50 * 1024);

    /// Creates a new instance of the `Browser` struct, initializing the HTTP client, cookie jar, and user agent stylesheet.
    ///
    /// # Panics
    /// * This function will panic if the embedded user agent CSS is not valid UTF-8, which should never happen since it's embedded in the binary.
    pub fn new(args: &BrowserArgs) -> Self {
        let config = BrowserConfig::new(args);
        let databases = Databases::init().expect("Failed to initialize databases, which is required for the browser to function. Please ensure you have enough disk space and permissions to create necessary files.");
        let http_client = Box::new(ReqwestClient::new());
        let user_agent_css = Resource::load_embedded(DEFAULT_CSS);

        let stylesheet = if args.enable_ua_css {
            match Resource::load(CACHE_USER_AGENT, Self::MAX_USER_AGENT_CSS_SIZE) {
                Ok(data) => {
                    trace!("Loaded user agent stylesheet from cache");

                    let out: CSSStyleSheet = from_bytes(data.as_slice()).unwrap_or_else(|_| {
                        CSSStyleSheet::from_css(
                            std::str::from_utf8(&user_agent_css).unwrap_or_default(),
                            StylesheetOrigin::UserAgent,
                            false,
                        )
                    });

                    Some(out)
                }
                Err(err) => {
                    trace!("Failed to load user agent stylesheet from cache: {}, parsing embedded CSS", err);

                    let parsed = CSSStyleSheet::from_css(
                        std::str::from_utf8(&user_agent_css).unwrap_or_default(),
                        StylesheetOrigin::UserAgent,
                        false,
                    );

                    let serialized = to_stdvec(&parsed).unwrap();

                    if Resource::write(CACHE_USER_AGENT, serialized.as_slice()).is_err() {
                        warn!("Failed to write user agent stylesheet to cache");
                    }

                    Some(parsed)
                }
            }
        } else {
            None
        };

        Self {
            config,
            databases,
            default_stylesheet: stylesheet,
            http_client,
        }
    }

    pub fn config(&self) -> &BrowserConfig {
        &self.config
    }

    #[must_use]
    pub fn headers(&self) -> &HeaderMap {
        self.config.headers()
    }

    pub const fn http_client(&self) -> &dyn HttpClient {
        &*self.http_client
    }

    pub const fn http_cache(&self) -> &HttpCache {
        &self.databases.http_cache
    }

    pub const fn cookie_jar(&self) -> &CookieJar {
        &self.databases.cookie_jar
    }
}

impl ScriptExecutor for Browser {
    fn execute_script(&self, _script: &str) {
        //debug!("Executing script: {}", script);
    }
}

#[async_trait]
impl Commandable for Browser {
    #[instrument(skip(self), level = "trace")]
    async fn execute(&self, command: EngineCommand) -> Result<EngineResponse, CoreError> {
        match command {
            EngineCommand::Navigate {
                url,
                navigation_type,
            } => {
                let span = tracing::debug_span!("Browser::Navigate");

                let stylesheets = self
                    .default_stylesheet
                    .as_ref()
                    .map_or_else(Vec::new, |default| vec![default.clone()]);

                let (page, metadata) = self.navigate(&url, stylesheets).instrument(span).await?;

                Ok(EngineResponse::NavigateSuccess(page, metadata, navigation_type))
            }
            EngineCommand::GetDevtoolsPage { document } => {
                let span = tracing::debug_span!("Browser::GetDevtoolsPage");
                let _enter = span.enter();

                let default_css = {
                    let css_resource = Resource::load_embedded(DEFAULT_CSS);
                    CSSStyleSheet::from_css(
                        str::from_utf8(css_resource.as_slice()).expect("Embedded default CSS should be valid UTF-8"),
                        StylesheetOrigin::UserAgent,
                        false,
                    )
                };
                let devtools_css = {
                    let css_resource = Resource::load_embedded(DEVTOOLS_CSS);
                    CSSStyleSheet::from_css(
                        str::from_utf8(css_resource.as_slice()).expect("Embedded DevTools CSS should be valid UTF-8"),
                        StylesheetOrigin::Author,
                        false,
                    )
                };

                let stylesheets = vec![default_css, devtools_css];
                let dom = parse_devtools_html(&document).map_err(|e| CoreError::DevtoolsGeneration(e.to_string()))?;

                let devtools_page = Document::new(dom, stylesheets);

                Ok(EngineResponse::DevtoolsPageReady(devtools_page))
            }
            EngineCommand::FetchImage {
                node_ids,
                request_url,
                request_policies,
                image_url,
            } => {
                let span = tracing::debug_span!("Browser::FetchImage");

                self.load_image(node_ids, request_url, request_policies, &image_url)
                    .instrument(span)
                    .await
            }
        }
    }
}

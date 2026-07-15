use std::{collections::HashMap, vec};

use crate::{Document, commands::parse_devtools_html, errors::CoreError, profile::Profile};
use async_trait::async_trait;
use browser_args::BrowserArgs;
use css_cssom::{CSSStyleSheet, StylesheetOrigin};
use http_fetch::{client::HttpClient, clients::ReqwestClient};
use io::{
    Readable, Writable,
    embedded::{DEFAULT_CSS, DEVTOOLS_CSS},
    entries::PROFILE_CACHE_USER_AGENT,
};
use postcard::{from_bytes, to_stdvec};
use tracing::{Instrument, instrument, trace, warn};

use crate::{
    events::{Commandable, EngineCommand, EngineResponse},
    navigation::ScriptExecutor,
};

#[derive(Debug)]
pub struct Browser {
    profile: Profile,
    default_stylesheet: Option<CSSStyleSheet>,
    http_client: Box<dyn HttpClient>,
}

impl Browser {
    /// Maximum allowed size for the user agent stylesheet, set to 50 KiB.
    const MAX_USER_AGENT_CSS_SIZE: Option<u64> = Some(50 * 1024);

    /// Creates a new instance of the `Browser` struct, initializing the HTTP client, cookie jar, and user agent stylesheet.
    ///
    /// # Panics
    /// * This function will panic if the embedded user agent CSS is not valid UTF-8, which should never happen since it's embedded in the binary.
    pub fn new(args: &BrowserArgs) -> Self {
        let profile = Profile::new(args);
        let http_client = Box::new(ReqwestClient::new());
        let user_agent_css = DEFAULT_CSS.load();

        let stylesheet = if args.enable_ua_css {
            match PROFILE_CACHE_USER_AGENT.read(&profile.dirs().into(), Self::MAX_USER_AGENT_CSS_SIZE) {
                Ok(data) => {
                    trace!("Loaded user agent stylesheet from cache");
                    let data: &[u8] = &data;

                    let out: CSSStyleSheet = from_bytes(data).unwrap_or_else(|_| {
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

                    if PROFILE_CACHE_USER_AGENT
                        .write(serialized.as_slice(), &profile.dirs().into())
                        .is_err()
                    {
                        warn!("Failed to write user agent stylesheet to cache");
                    }

                    Some(parsed)
                }
            }
        } else {
            None
        };

        Self {
            profile,
            default_stylesheet: stylesheet,
            http_client,
        }
    }

    pub fn profile(&self) -> &Profile {
        &self.profile
    }

    pub const fn http_client(&self) -> &dyn HttpClient {
        &*self.http_client
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
            EngineCommand::GetDevtoolsPage { title, document } => {
                let span = tracing::debug_span!("Browser::GetDevtoolsPage");
                let _enter = span.enter();

                let default_css = {
                    let css_resource = DEFAULT_CSS.load();
                    CSSStyleSheet::from_css(
                        str::from_utf8(&css_resource).expect("Embedded default CSS should be valid UTF-8"),
                        StylesheetOrigin::UserAgent,
                        false,
                    )
                };
                let devtools_css = {
                    let css_resource = DEVTOOLS_CSS.load();
                    CSSStyleSheet::from_css(
                        str::from_utf8(&css_resource).expect("Embedded DevTools CSS should be valid UTF-8"),
                        StylesheetOrigin::Author,
                        false,
                    )
                };

                let stylesheets = vec![default_css, devtools_css];
                let dom =
                    parse_devtools_html(&title, &document).map_err(|e| CoreError::DevtoolsGeneration(e.to_string()))?;

                let devtools_page = Document::new(dom, HashMap::new(), stylesheets);

                Ok(EngineResponse::DevtoolsPageReady(devtools_page))
            }
            EngineCommand::FetchImage {
                node_ids,
                request_url,
                image_url,
            } => {
                let span = tracing::debug_span!("Browser::FetchImage");

                self.load_image(node_ids, request_url, &image_url)
                    .instrument(span)
                    .await
            }
        }
    }
}

use std::sync::Arc;

use http::HeaderMap;
use tokio::task::JoinHandle;
use tracing::{Instrument, debug, warn};
use url::Url;

use cookies::CookieJar;
use css_cssom::{CSSStyleSheet, StylesheetOrigin};
use html_parser::{BlockedReason, HtmlStreamParser, ParserState, ResourceType, Script};
use http_cache::{block::MAX_BLOCK_SIZE, http::HttpCache};
use http_fetch::{
    client::HttpClient,
    errors::{FetchError, NetworkError},
    request::{FetchResult, fetch},
};
use http_types::{
    errors::RequestError,
    properties::{Destination, RequestMode},
    request::Request,
};
use io::{DocumentPolicy, Entry, Resource};
use storage::Directory;

use crate::{
    Browser,
    context::page::{Favicon, PageMetadata},
    errors::NavigationError,
    navigation::ScriptExecutor,
};

use crate::context::{collector::TabCollector, page::Document};

/// A list of allowed "about:" URLs that the browser can load.
/// This is a security measure to prevent loading potentially harmful or
/// unintended content through "about:" URLs. Only the URLs specified in
/// this list will be allowed to be loaded by the browser.
const ALLOWED_ABOUT_URLS: &[&str] = &["blank"];

impl Browser {
    /// Navigates the specified tab to the given URL, fetching and parsing the content.
    /// Executes any scripts and processes stylesheets found during parsing.
    pub async fn navigate(
        &self,
        url: &str,
        mut stylesheets: Vec<CSSStyleSheet>,
    ) -> Result<(Document, PageMetadata), NavigationError> {
        let page = Document::blank();

        let client = self.http_client();
        let headers = Arc::new(self.profile().config().headers().clone());
        let cookie_jar = self.profile().cookie_jar();

        let navigation_request = Request::builder(url)
            .destination(Destination::Document)
            .request_mode(RequestMode::Navigate)
            .build();

        let request_url = navigation_request.context.url.clone();

        let response_result = fetch(
            None,
            navigation_request,
            client,
            &headers,
            &self.profile().dirs().into(),
            cookie_jar,
            self.profile().http_cache(),
        )
        .await;

        let response_handle = match response_result {
            FetchResult::Success(response) => response,
            FetchResult::ClientError(response) | FetchResult::ServerError(response) => {
                return Err(NavigationError::Request {
                    source: FetchError::Network(NetworkError::HttpStatus(response.head().status_code)),
                    url: url.to_string(),
                });
            }
            FetchResult::Failed(error) => {
                return Err(NavigationError::Request {
                    source: error,
                    url: url.to_string(),
                });
            }
        };

        let response = match response_handle.response().await {
            Ok(resp) => resp,
            Err(error) => {
                return Err(NavigationError::Request {
                    source: FetchError::Network(error),
                    url: url.to_string(),
                });
            }
        };

        // let (request_url, response) = self
        //     .resolve_navigation_request(url, None, &DocumentPolicy::default(), &headers, client)
        //     .await?;

        // TODO: Use stream!
        let Some(body) = response.body.into_complete(MAX_BLOCK_SIZE as usize).await else {
            return Err(NavigationError::Request {
                source: FetchError::Request(RequestError::InvalidBody("Either too large or emtpy".to_string())),
                url: url.to_string(),
            });
        };

        let mut favicon = Favicon::default();
        let mut style_handles: Vec<JoinHandle<Option<CSSStyleSheet>>> = Vec::new();
        let mut favicon_handle: Option<JoinHandle<Option<Vec<u8>>>> = None;

        let reader: &[u8] = &body.0;
        let mut parser = HtmlStreamParser::new(reader).with_collector(TabCollector::default());

        let result = loop {
            let state = parser.step().map_err(|e| NavigationError::Parsing {
                url: url.to_string(),
                source: e,
            })?;

            match state {
                ParserState::Running => {}
                ParserState::Blocked(reason) => {
                    match reason {
                        BlockedReason::WaitingForScript { script } => {
                            match script {
                                Script::Inline { data, type_attr: _ } => {
                                    let script_content = data.map_err(|e| NavigationError::Parsing {
                                        url: url.to_string(),
                                        source: e,
                                    })?;

                                    self.execute_script(&script_content);
                                }
                                Script::External { .. } => {
                                    // TODO: external script and async/defer handling
                                }
                            }
                        }
                        BlockedReason::WaitingForStyle {
                            data,
                            attributes: _attributes,
                        } => {
                            let css_content = data.map_err(|e| NavigationError::Parsing {
                                url: url.to_string(),
                                source: e,
                            })?;

                            let current_span = tracing::Span::current();
                            let handle = tokio::task::spawn_blocking(move || {
                                let _span = current_span.enter();
                                Some(CSSStyleSheet::from_css(&css_content, StylesheetOrigin::Author, true))
                            });
                            style_handles.push(handle);
                        }
                        BlockedReason::WaitingForResource(resource_type, href, metadata) => match resource_type {
                            ResourceType::Style => {
                                let relative_url =
                                    request_url
                                        .join(&href)
                                        .map_err(|error| NavigationError::Request {
                                            source: FetchError::Network(NetworkError::InvalidUrl(error)),
                                            url: href.clone(),
                                        })?;

                                let handle = Self::spawn_style_fetch_and_parse(
                                    self.profile().dirs().into(),
                                    relative_url,
                                    &request_url,
                                    self.profile().http_cache(),
                                    client.box_clone(),
                                    Arc::clone(&headers),
                                    cookie_jar,
                                );
                                style_handles.push(handle);
                            }
                            ResourceType::Favicon => {
                                let relative_url =
                                    request_url
                                        .join(&href)
                                        .map_err(|error| NavigationError::Request {
                                            source: FetchError::Network(NetworkError::InvalidUrl(error)),
                                            url: href.clone(),
                                        })?;

                                favicon.content_type = metadata.content_type;
                                favicon.size = metadata.sizes;

                                let page_url = request_url.clone();
                                let client_clone = client.box_clone();
                                let headers_clone = Arc::clone(&headers);
                                let http_cache = self.profile().http_cache().clone();
                                let dirs = self.profile().dirs().into();
                                let cookie_jar = cookie_jar.clone();

                                let handle = tokio::spawn(
                                    async move {
                                        if relative_url.scheme() != "http" && relative_url.scheme() != "https" {
                                            match Resource::load(
                                                io::ResourceType::Path(Entry::absolute(
                                                    relative_url.to_file_path().unwrap().to_str().unwrap(),
                                                )),
                                                dirs,
                                                Resource::DEFAULT_MAX_FILE_SIZE,
                                            ) {
                                                Ok(b) => Some(b),
                                                Err(error) => {
                                                    debug!(%error, "Failed to load favicon {}", relative_url);
                                                    None
                                                }
                                            }
                                        } else {
                                            let request = Request::builder(relative_url.as_str())
                                                .destination(Destination::Image)
                                                .request_mode(RequestMode::Cors)
                                                .build();

                                            match fetch(
                                                Some(&page_url),
                                                request,
                                                client_clone.as_ref(),
                                                &headers_clone,
                                                &dirs,
                                                &cookie_jar,
                                                &http_cache,
                                            )
                                            .await
                                            {
                                                FetchResult::Success(response_handle) => {
                                                    match response_handle.response().await {
                                                        Ok(response) => {
                                                            if let Some(body) =
                                                                response.body.into_complete(2 * 1024 * 1024).await
                                                            {
                                                                Some(body.0.into())
                                                            } else {
                                                                debug!("Empty body for favicon {}", relative_url);
                                                                None
                                                            }
                                                        }
                                                        Err(error) => {
                                                            debug!(%error, "Failed to fetch favicon {}", relative_url);
                                                            None
                                                        }
                                                    }
                                                }
                                                FetchResult::Failed(error) => {
                                                    debug!(%error, "Failed to fetch favicon {}", relative_url);
                                                    None
                                                }
                                                FetchResult::ClientError(resp) | FetchResult::ServerError(resp) => {
                                                    debug!(
                                                        "Failed to fetch favicon {}: status code {}",
                                                        relative_url,
                                                        resp.head().status_code
                                                    );
                                                    None
                                                }
                                            }
                                        }
                                    }
                                    .in_current_span(),
                                );

                                favicon_handle = Some(handle);
                            }
                        },
                        BlockedReason::SVGContent { data } => {
                            let _svg_content = data.map_err(|e| NavigationError::Parsing {
                                url: url.to_string(),
                                source: e,
                            })?;

                            // TODO: Process SVG content
                        }
                    }
                }
                ParserState::Completed(build_result) => {
                    break build_result;
                }
            }
        };

        for handle in style_handles {
            match handle.await {
                Ok(Some(stylesheet)) => {
                    stylesheets.push(stylesheet);
                }
                Ok(None) => {}
                Err(e) => {
                    warn!("Style fetch+parse task panicked: {}", e);
                }
            }
        }

        let result_metadata = result.metadata.unwrap();
        let mut page_metadata = PageMetadata {
            url: request_url,
            title: result_metadata
                .title
                .clone()
                .unwrap_or_else(|| "Untitled".to_string()),
            favicon: None,
            policies: DocumentPolicy::default(),
        };

        if let Some(favicon_handle) = favicon_handle {
            match favicon_handle.await {
                Ok(Some(favicon_bytes)) => {
                    favicon.data = favicon_bytes;
                    page_metadata.favicon = Some(favicon);
                }
                Ok(None) => {}
                Err(e) => {
                    warn!("Favicon fetch task panicked: {}", e);
                }
            }
        }

        Ok((page.load(result.dom_tree, result_metadata.images, stylesheets), page_metadata))
    }

    /// Spawns a task to fetch and parse a stylesheet from the given URL, returning a handle to the resulting stylesheet.
    /// The task will handle cookies and headers appropriately, and will return `None` if fetching or parsing fails.
    fn spawn_style_fetch_and_parse(
        dirs: Directory,
        style_url: Url,
        page_url: &Url,
        cache: &HttpCache,
        client: Box<dyn HttpClient>,
        headers: Arc<HeaderMap>,
        cookie_jar: &CookieJar,
    ) -> JoinHandle<Option<CSSStyleSheet>> {
        let page_url = page_url.clone();
        let cache = cache.clone();
        let cookie_jar = cookie_jar.clone();

        tokio::spawn(
            async move {
                if style_url.scheme() != "http" && style_url.scheme() != "https" {
                    let res = Resource::load(
                        io::ResourceType::Path(Entry::absolute(style_url.to_file_path().unwrap().to_str().unwrap())),
                        dirs,
                        Resource::DEFAULT_MAX_FILE_SIZE,
                    );

                    let body = match res {
                        Ok(b) => b,
                        Err(error) => {
                            debug!(%error, "Failed to load stylesheet {}", style_url);
                            return None;
                        }
                    };

                    let stylesheet_url = style_url.clone();
                    let current_span = tracing::Span::current();
                    match tokio::task::spawn_blocking(move || {
                        let _span = current_span.enter();
                        let css_str = String::from_utf8_lossy(&body);
                        CSSStyleSheet::from_css(&css_str, StylesheetOrigin::Author, true)
                    })
                    .await
                    {
                        Ok(stylesheet) => Some(stylesheet),
                        Err(error) => {
                            warn!(%error, "CSS parse task panicked for {}", stylesheet_url);
                            None
                        }
                    }
                } else {
                    let request = Request::builder(style_url.as_str())
                        .request_mode(RequestMode::Cors)
                        .destination(Destination::Style)
                        .build();

                    let response_result =
                        fetch(Some(&page_url), request, client.as_ref(), &headers, &dirs, &cookie_jar, &cache).await;

                    let response_handle = match response_result {
                        FetchResult::Success(response) => response,
                        FetchResult::ClientError(response) | FetchResult::ServerError(response) => {
                            debug!(
                                "Failed to fetch stylesheet {}: status code {}",
                                style_url,
                                response.head().status_code
                            );
                            return None;
                        }
                        FetchResult::Failed(error) => {
                            debug!(%error, "Failed to fetch stylesheet {}", style_url);
                            return None;
                        }
                    };

                    let response = match response_handle.response().await {
                        Ok(resp) => resp,
                        Err(error) => {
                            debug!(%error, "Failed to read body for stylesheet {}", style_url);
                            return None;
                        }
                    };

                    let body = match response.body.into_complete(MAX_BLOCK_SIZE as usize).await {
                        Some(b) => b,
                        None => {
                            debug!("Empty body for stylesheet {}", style_url);
                            return None;
                        }
                    };

                    let body_bytes = body.0.to_vec();

                    let stylesheet_url = style_url.clone();
                    let current_span = tracing::Span::current();
                    match tokio::task::spawn_blocking(move || {
                        let _span = current_span.enter();

                        let css_str = String::from_utf8_lossy(&body_bytes);
                        CSSStyleSheet::from_css(&css_str, StylesheetOrigin::Author, true)
                    })
                    .await
                    {
                        Ok(stylesheet) => Some(stylesheet),
                        Err(error) => {
                            warn!(%error, "CSS parse task panicked for {}", stylesheet_url);
                            None
                        }
                    }
                }
            }
            .in_current_span(),
        )
    }
}

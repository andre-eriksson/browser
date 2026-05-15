use std::sync::Arc;

use crate::{
    Browser,
    context::page::{Favicon, PageMetadata},
    errors::NavigationError,
    navigation::ScriptExecutor,
};
use cookies::CookieJar;
use css_cssom::{CSSStyleSheet, StylesheetOrigin};
use html_escape::decode_html_entities;
use html_parser::{BlockedReason, HtmlStreamParser, ParserState, ResourceType, Script};
use io::{CookieMiddleware, DecodingMiddleware, DocumentPolicy, HttpCache, Resource};
use network::{
    HeaderMap, SET_COOKIE,
    client::HttpClient,
    errors::{NetworkError, RequestError},
    response::Response,
};
use tokio::task::JoinHandle;
use tracing::{Instrument, debug, warn};
use url::Url;

use crate::context::{collector::TabCollector, page::Page};

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
    ) -> Result<(Page, PageMetadata), NavigationError> {
        let page = Page::blank();

        let client = self.http_client();
        let headers = Arc::new(self.headers().clone());
        let cookie_jar = self.cookie_jar();

        let (request_url, response) = self
            .resolve_navigation_request(url, None, &DocumentPolicy::default(), &headers, client)
            .await?;

        let Some(body) = response.body else {
            return Err(NavigationError::Request {
                source: RequestError::EmptyBody,
                url: url.to_string(),
            });
        };

        let decoded = DecodingMiddleware::decode(&response.headers, body).await?;

        let mut favicon = Favicon::default();
        let mut style_handles: Vec<JoinHandle<Option<CSSStyleSheet>>> = Vec::new();
        let mut favicon_handle: Option<JoinHandle<Option<Vec<u8>>>> = None;
        let mut parser = HtmlStreamParser::new(decoded.as_slice()).with_collector(TabCollector::default());

        let result = loop {
            let state = parser.step().map_err(|e| NavigationError::Parsing {
                url: url.to_string(),
                source: e,
            })?;

            match state {
                ParserState::Running => {}
                ParserState::Blocked(reason) => match reason {
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
                            let relative_url = request_url
                                .join(&href)
                                .map_err(|error| NavigationError::Request {
                                    source: RequestError::Network(NetworkError::InvalidUrl(error)),
                                    url: href.clone(),
                                })?;

                            let handle = Self::spawn_style_fetch_and_parse(
                                relative_url,
                                &request_url,
                                self.http_cache(),
                                client.box_clone(),
                                Arc::clone(&headers),
                                cookie_jar,
                            );
                            style_handles.push(handle);
                        }
                        ResourceType::Favicon => {
                            let relative_url = request_url
                                .join(&href)
                                .map_err(|error| NavigationError::Request {
                                    source: RequestError::Network(NetworkError::InvalidUrl(error)),
                                    url: href.clone(),
                                })?;

                            favicon.content_type = metadata.content_type;
                            favicon.size = metadata.sizes;

                            let page_url = request_url.clone();
                            let policies = DocumentPolicy::default();
                            let client_clone = client.box_clone();
                            let headers_clone = Arc::clone(&headers);
                            let http_cache = self.http_cache().clone();

                            let cookies = if let Some(host) = request_url.host() {
                                self.cookie_jar()
                                    .get_cookies(&host, request_url.path(), true)
                            } else {
                                vec![]
                            };

                            let handle = tokio::spawn(
                                async move {
                                    if relative_url.scheme() != "http" && relative_url.scheme() != "https" {
                                        match Resource::load(
                                            io::ResourceType::Absolute {
                                                protocol: relative_url.scheme(),
                                                location: relative_url.path(),
                                            },
                                            Resource::DEFAULT_MAX_FILE_SIZE,
                                        ) {
                                            Ok(b) => Some(b),
                                            Err(error) => {
                                                debug!(%error, "Failed to load favicon {}", relative_url);
                                                None
                                            }
                                        }
                                    } else {
                                        match Resource::from_remote(
                                            relative_url.as_str(),
                                            &http_cache,
                                            client_clone.as_ref(),
                                            &cookies,
                                            &headers_clone,
                                            Some(page_url),
                                            &policies,
                                        )
                                        .await
                                        {
                                            Ok(r) => match r.response().await {
                                                Ok(body_resp) => {
                                                    if let Some(body) = body_resp.body {
                                                        let Ok(decoded) =
                                                            DecodingMiddleware::decode(&body_resp.headers, body).await
                                                        else {
                                                            return None;
                                                        };

                                                        Some(decoded)
                                                    } else {
                                                        None
                                                    }
                                                }
                                                Err(error) => {
                                                    debug!(%error, "Failed to read body for favicon {}", relative_url);
                                                    None
                                                }
                                            },
                                            Err(error) => {
                                                debug!(%error, "Failed to fetch favicon {}", relative_url);
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
                },
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

    /// Resolves the body of the document to be navigated to, handling both "about:" URLs and regular HTTP/HTTPS URLs.
    /// For "about:" URLs, it loads the corresponding embedded resource. For HTTP/HTTPS URLs, it performs a network
    /// request to fetch the content, applying cookies and headers as needed.
    ///
    /// Returns the resolved URL and the body content as a byte vector, or an error if the URL is invalid or the content
    /// cannot be fetched.
    async fn resolve_navigation_request(
        &self,
        raw_url: &str,
        page_url: Option<Url>,
        policies: &DocumentPolicy,
        headers: &HeaderMap,
        client: &dyn HttpClient,
    ) -> Result<(Url, Response), NavigationError> {
        if let Some(location) = raw_url.strip_prefix("about:") {
            if ALLOWED_ABOUT_URLS
                .iter()
                .all(|&allowed| allowed != location)
            {
                return Err(NavigationError::Forbidden("Disallowed about URL".to_string()));
            }

            let resp = Response::from(
                Resource::load(
                    io::ResourceType::Absolute {
                        protocol: "about",
                        location: format!("{location}.html").as_str(),
                    },
                    Resource::DEFAULT_MAX_FILE_SIZE,
                )
                .map_err(NavigationError::Resource)?,
            );

            let url = Url::parse(&format!("about:{location}")).unwrap_or_else(|_| Url::parse("about:blank").unwrap());

            return Ok((url, resp));
        }

        let decoded_url = decode_html_entities(raw_url);

        let url = page_url
            .as_ref()
            .map_or_else(|| Url::parse(&decoded_url), |u| u.join(&decoded_url))
            .map_err(|e| NavigationError::Request {
                source: RequestError::Network(NetworkError::InvalidUrl(e)),
                url: raw_url.to_string(),
            })?;

        let resp = self
            .resolve_request(url, None, policies, headers, client)
            .await?;

        Ok(resp)
    }

    pub async fn resolve_request(
        &self,
        url: Url,
        page_url: Option<Url>,
        policies: &DocumentPolicy,
        headers: &HeaderMap,
        client: &dyn HttpClient,
    ) -> Result<(Url, Response), NavigationError> {
        let resp = if url.scheme() == "file" {
            match page_url {
                Some(base) => {
                    if base.scheme() == "file" {
                        Response::from(
                            Resource::load(
                                io::ResourceType::Absolute {
                                    protocol: url.scheme(),
                                    location: url.path(),
                                },
                                Resource::DEFAULT_MAX_FILE_SIZE,
                            )
                            .map_err(NavigationError::Resource)?,
                        )
                    } else {
                        return Err(NavigationError::Forbidden(
                            "Cannot resolve file URL with a non-file base URL for security reasons".to_string(),
                        ));
                    }
                }
                None => Response::from(
                    Resource::load(
                        io::ResourceType::Absolute {
                            protocol: url.scheme(),
                            location: url.path(),
                        },
                        Resource::DEFAULT_MAX_FILE_SIZE,
                    )
                    .map_err(NavigationError::Resource)?,
                ),
            }
        } else {
            let cookies = if let Some(host) = url.host() {
                self.cookie_jar().get_cookies(&host, url.path(), true)
            } else {
                vec![]
            };

            Resource::from_remote(url.as_str(), self.http_cache(), client, &cookies, headers, page_url, policies)
                .await
                .map_err(|e| NavigationError::Request {
                    source: e,
                    url: url.to_string(),
                })?
                .response()
                .await
                .map_err(|e| NavigationError::Request {
                    source: RequestError::Network(e),
                    url: url.to_string(),
                })?
        };

        for header in &resp.headers {
            if header.0 == SET_COOKIE {
                CookieMiddleware::handle_response_cookie(self.cookie_jar(), &url, header.1);
            }
        }

        Ok((url, resp))
    }

    /// Spawns a task to fetch and parse a stylesheet from the given URL, returning a handle to the resulting stylesheet.
    /// The task will handle cookies and headers appropriately, and will return `None` if fetching or parsing fails.
    fn spawn_style_fetch_and_parse(
        style_url: Url,
        page_url: &Url,
        cache: &HttpCache,
        client: Box<dyn HttpClient>,
        headers: Arc<HeaderMap>,
        cookie_jar: &CookieJar,
    ) -> JoinHandle<Option<CSSStyleSheet>> {
        let page_url = page_url.clone();
        let policies = DocumentPolicy::default();
        let cache = cache.clone();
        let cookie_jar = cookie_jar.clone();

        tokio::spawn(
            async move {
                if style_url.scheme() != "http" && style_url.scheme() != "https" {
                    let res = Resource::load(
                        io::ResourceType::Absolute {
                            protocol: style_url.scheme(),
                            location: style_url.path(),
                        },
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
                    let cookies = if let Some(host) = style_url.host() {
                        cookie_jar.get_cookies(&host, style_url.path(), true)
                    } else {
                        vec![]
                    };

                    let resp = match Resource::from_remote(
                        style_url.as_str(),
                        &cache,
                        client.as_ref(),
                        &cookies,
                        &headers,
                        Some(page_url),
                        &policies,
                    )
                    .await
                    {
                        Ok(r) => r,
                        Err(error) => {
                            debug!(%error, "Failed to fetch stylesheet {}", style_url);
                            return None;
                        }
                    };

                    for header in &resp.metadata().headers {
                        if header.0 == SET_COOKIE {
                            CookieMiddleware::handle_response_cookie(&cookie_jar, &style_url, header.1);
                        }
                    }

                    let body_bytes = match resp.response().await {
                        Ok(body_resp) => {
                            if let Some(b) = body_resp.body {
                                let Ok(decoded) = DecodingMiddleware::decode(&body_resp.headers, b).await else {
                                    return None;
                                };

                                decoded
                            } else {
                                debug!("Empty body for stylesheet {}", style_url);
                                return None;
                            }
                        }
                        Err(error) => {
                            debug!(%error, "Failed to read body for stylesheet {}", style_url);
                            return None;
                        }
                    };

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

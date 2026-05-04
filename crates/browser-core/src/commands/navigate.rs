use std::sync::{Arc, Mutex};

use crate::{
    context::page::{Favicon, PageMetadata},
    errors::NavigationError,
};
use cookies::{Cookie, CookieJar};
use css_cssom::{CSSStyleSheet, StylesheetOrigin};
use html_escape::decode_html_entities;
use html_parser::{BlockedReason, HtmlStreamParser, ParserState, ResourceType};
use io::{CookieMiddleware, DocumentPolicy, Resource};
use network::{
    HeaderMap, SET_COOKIE,
    client::HttpClient,
    errors::{NetworkError, RequestError},
    response::Response,
};
use tokio::task::JoinHandle;
use tracing::{Instrument, debug, warn};
use url::Url;

use crate::{
    context::{collector::TabCollector, page::Page},
    navigation::NavigationContext,
};

/// A list of allowed "about:" URLs that the browser can load.
/// This is a security measure to prevent loading potentially harmful or
/// unintended content through "about:" URLs. Only the URLs specified in
/// this list will be allowed to be loaded by the browser.
const ALLOWED_ABOUT_URLS: &[&str] = &["blank"];

/// Navigates the specified tab to the given URL, fetching and parsing the content.
/// Executes any scripts and processes stylesheets found during parsing.
pub async fn navigate(
    ctx: &mut dyn NavigationContext,
    url: &str,
    mut stylesheets: Vec<CSSStyleSheet>,
) -> Result<(Page, PageMetadata), NavigationError> {
    let page = Page::blank();

    let client = ctx.http_client().box_clone();
    let headers = Arc::new(ctx.headers().clone());
    let cookies = ctx
        .cookie_jar()
        .lock()
        .map_err(|_| NavigationError::CookieJarLocked)?
        .cookies()
        .clone();
    let cookie_jar = Arc::clone(ctx.cookie_jar());

    let (url, response) =
        resolve_navigation_request(url, ctx, None, &DocumentPolicy::default(), &cookies, &headers, client.as_ref())
            .await?;

    let Some(body) = response.body else {
        return Err(NavigationError::Request {
            source: RequestError::EmptyBody,
            url: url.to_string(),
        });
    };

    let mut favicon = Favicon::default();

    let mut style_handles: Vec<JoinHandle<Option<CSSStyleSheet>>> = Vec::new();
    let mut favicon_handle: Option<JoinHandle<Option<Vec<u8>>>> = None;
    let mut parser = HtmlStreamParser::new(body.as_slice(), None, Some(TabCollector::default()));

    loop {
        parser.step().map_err(|e| NavigationError::Parsing {
            url: url.to_string(),
            source: e,
        })?;

        match parser.get_state() {
            ParserState::Running => {}
            ParserState::Blocked(reason) => match reason {
                BlockedReason::WaitingForScript(attributes) => {
                    if let Some(src) = attributes.as_ref().and_then(|attrs| attrs.get("src")) {
                        match url.join(src) {
                            Ok(_url) => {
                                // TODO: Uncomment when we have script execution implemented
                                //
                                // let script_request = RequestBuilder::from(url).build();
                                // let script_resp =
                                //     ctx.network_service().fetch(&mut page, script_request);
                                // let script_response = match script_resp.await {
                                //     RequestResult::Failed(err) => {
                                //         return Err(NavigationError::RequestError(err));
                                //     }
                                //     RequestResult::ClientError(resp)
                                //     | RequestResult::ServerError(resp)
                                //     | RequestResult::Success(resp) => resp,
                                // };
                                // let script_body = match script_response.body().await {
                                //     Ok(resp) => match resp.body {
                                //         Some(b) => b,
                                //         None => {
                                //             return Err(NavigationError::RequestError(
                                //                 RequestError::EmptyBody,
                                //             ));
                                //         }
                                //     },
                                //     Err(e) => {
                                //         return Err(NavigationError::RequestError(
                                //             RequestError::Network(e),
                                //         ));
                                //     }
                                // };
                                //
                                // let script_text =
                                //     String::from_utf8_lossy(script_body.as_slice()).to_string();
                                // let _ = parser.extract_script_content()?;
                                // ctx.script_executor().execute_script(&script_text);
                            }
                            Err(error) => {
                                warn!(
                                    "{}",
                                    NavigationError::Request {
                                        source: RequestError::Network(NetworkError::InvalidUrl(error)),
                                        url: src.clone(),
                                    }
                                );
                            }
                        }
                    } else {
                        let script_content = parser
                            .extract_script_content()
                            .map_err(|e| NavigationError::Parsing {
                                url: url.to_string(),
                                source: e,
                            })?;
                        ctx.script_executor().execute_script(&script_content);
                    }

                    parser.resume().map_err(|e| NavigationError::Parsing {
                        url: url.to_string(),
                        source: e,
                    })?;
                }
                BlockedReason::WaitingForStyle(_attributes) => {
                    let css_content = parser
                        .extract_style_content()
                        .map_err(|e| NavigationError::Parsing {
                            url: url.to_string(),
                            source: e,
                        })?;

                    let current_span = tracing::Span::current();
                    let handle = tokio::task::spawn_blocking(move || {
                        let _span = current_span.enter();
                        Some(CSSStyleSheet::from_css(&css_content, StylesheetOrigin::Author, true))
                    });
                    style_handles.push(handle);

                    parser.resume().map_err(|e| NavigationError::Parsing {
                        url: url.to_string(),
                        source: e,
                    })?;
                }
                BlockedReason::WaitingForResource(resource_type, href, metadata) => match resource_type {
                    ResourceType::Style => {
                        let relative_url = url.join(href).map_err(|error| NavigationError::Request {
                            source: RequestError::Network(NetworkError::InvalidUrl(error)),
                            url: href.clone(),
                        })?;

                        let handle = spawn_style_fetch_and_parse(
                            relative_url,
                            &url,
                            client.box_clone(),
                            Arc::clone(&headers),
                            cookies.clone(),
                            Arc::clone(&cookie_jar),
                        );
                        style_handles.push(handle);

                        parser.resume().map_err(|e| NavigationError::Parsing {
                            url: url.to_string(),
                            source: e,
                        })?;
                    }
                    ResourceType::Favicon => {
                        let relative_url = url.join(href).map_err(|error| NavigationError::Request {
                            source: RequestError::Network(NetworkError::InvalidUrl(error)),
                            url: href.clone(),
                        })?;

                        favicon.content_type = metadata.content_type.clone();
                        favicon.size = metadata.sizes;

                        let page_url = url.clone();
                        let policies = DocumentPolicy::default();
                        let client_clone = client.box_clone();
                        let headers_clone = Arc::clone(&headers);
                        let cookies_clone = cookies.clone();

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
                                        client_clone.as_ref(),
                                        &cookies_clone,
                                        &headers_clone,
                                        Some(page_url),
                                        &policies,
                                    )
                                    .await
                                    {
                                        Ok(r) => match r.response().await {
                                            Ok(body_resp) => body_resp.body,
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

                        parser.resume().map_err(|e| NavigationError::Parsing {
                            url: url.to_string(),
                            source: e,
                        })?;
                    }
                },
                BlockedReason::SVGContent => {
                    let _svg_content = parser
                        .extract_svg_content()
                        .map_err(|e| NavigationError::Parsing {
                            url: url.to_string(),
                            source: e,
                        })?;

                    // TODO: Process SVG content

                    parser.resume().map_err(|e| NavigationError::Parsing {
                        url: url.to_string(),
                        source: e,
                    })?;
                }
            },
            ParserState::Completed => {
                break;
            }
        }
    }

    let result = parser.finalize();

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

    let mut metadata = PageMetadata {
        url,
        title: result
            .metadata
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

                metadata.favicon = Some(favicon);
            }
            Ok(None) => {}
            Err(e) => {
                warn!("Favicon fetch task panicked: {}", e);
            }
        }
    }

    Ok((page.load(result.dom_tree, result.metadata.images, stylesheets), metadata))
}

/// Resolves the body of the document to be navigated to, handling both "about:" URLs and regular HTTP/HTTPS URLs.
/// For "about:" URLs, it loads the corresponding embedded resource. For HTTP/HTTPS URLs, it performs a network
/// request to fetch the content, applying cookies and headers as needed.
///
/// Returns the resolved URL and the body content as a byte vector, or an error if the URL is invalid or the content
/// cannot be fetched.
async fn resolve_navigation_request(
    raw_url: &str,
    ctx: &mut dyn NavigationContext,
    page_url: Option<Url>,
    policies: &DocumentPolicy,
    cookies: &[Cookie],
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

    let resp = resolve_request(url, ctx, None, policies, cookies, headers, client).await?;

    Ok(resp)
}

pub async fn resolve_request(
    url: Url,
    ctx: &mut dyn NavigationContext,
    page_url: Option<Url>,
    policies: &DocumentPolicy,
    cookies: &[Cookie],
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
        Resource::from_remote(url.as_str(), client, cookies, headers, page_url, policies)
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
        if header.0 == SET_COOKIE
            && let Ok(mut cookie_jar) = ctx.cookie_jar().lock()
        {
            CookieMiddleware::handle_response_cookie(&mut cookie_jar, &url, header.1);
        }
    }

    Ok((url, resp))
}

/// Spawns a task to fetch and parse a stylesheet from the given URL, returning a handle to the resulting stylesheet.
/// The task will handle cookies and headers appropriately, and will return `None` if fetching or parsing fails.
fn spawn_style_fetch_and_parse(
    style_url: Url,
    page_url: &Url,
    client: Box<dyn HttpClient>,
    headers: Arc<HeaderMap>,
    cookies: Vec<Cookie>,
    cookie_jar: Arc<Mutex<CookieJar>>,
) -> JoinHandle<Option<CSSStyleSheet>> {
    let page_url = page_url.clone();
    let policies = DocumentPolicy::default();

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
                let resp = match Resource::from_remote(
                    style_url.as_str(),
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
                    if header.0 == SET_COOKIE
                        && let Ok(mut cookie_jar) = cookie_jar.lock()
                    {
                        CookieMiddleware::handle_response_cookie(&mut cookie_jar, &style_url, header.1);
                    }
                }

                let body_bytes = match resp.response().await {
                    Ok(body_resp) => {
                        if let Some(b) = body_resp.body {
                            b
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

use std::sync::{Arc, Mutex};

use crate::errors::NavigationError;
use cookies::{Cookie, CookieJar};
use css_cssom::{CSSStyleSheet, StylesheetOrigin};
use html_parser::{BlockedReason, HtmlStreamParser, ParserState, ResourceType};
use io::{CookieMiddleware, DocumentPolicy, Resource, files::ALLOWED_ABOUT_URLS};
use network::{
    HeaderMap, SET_COOKIE,
    client::HttpClient,
    errors::{NetworkError, RequestError},
    response::Response,
};
use tokio::task::JoinHandle;
use tracing::{debug, warn};
use url::Url;

use crate::{
    TabId,
    navigation::NavigationContext,
    tab::{collector::TabCollector, page::Page},
};

/// Navigates the specified tab to the given URL, fetching and parsing the content.
/// Executes any scripts and processes stylesheets found during parsing.
pub(crate) async fn navigate(
    ctx: &mut dyn NavigationContext,
    tab_id: TabId,
    url: &str,
    mut stylesheets: Vec<CSSStyleSheet>,
) -> Result<Page, NavigationError> {
    let mut page = Page::blank();

    let client = ctx.http_client().box_clone();
    let headers = Arc::clone(ctx.headers());
    let cookies = ctx
        .cookie_jar()
        .lock()
        .map_err(|_| NavigationError::CookieJarLocked)?
        .cookies()
        .clone();
    let cookie_jar = Arc::clone(ctx.cookie_jar());

    let (url, response) = resolve_navigation_request(
        url,
        ctx,
        &page.document_url,
        page.policies(),
        &cookies,
        &headers,
        client.as_ref(),
    )
    .await?;

    let body = match response.body {
        Some(b) => b,
        None => {
            return Err(NavigationError::RequestError(RequestError::EmptyBody));
        }
    };

    page.document_url = Some(url.clone());

    let mut style_handles: Vec<JoinHandle<Option<CSSStyleSheet>>> = Vec::new();
    let mut parser = HtmlStreamParser::<_, TabCollector>::new(
        body.as_slice(),
        None,
        Some(TabCollector::default()),
    );

    loop {
        parser.step()?;

        match parser.get_state() {
            ParserState::Running => continue,
            ParserState::Blocked(reason) => match reason {
                BlockedReason::WaitingForScript(attributes) => {
                    if let Some(src) = attributes.get("src") {
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
                            Err(e) => {
                                warn!(
                                    "{}",
                                    NavigationError::RequestError(RequestError::Network(
                                        NetworkError::InvalidUrl(e.to_string()),
                                    ))
                                );
                            }
                        }
                    } else {
                        let script_content = parser.extract_script_content()?;
                        ctx.script_executor().execute_script(&script_content);
                    }

                    parser.resume()?;
                }
                BlockedReason::WaitingForStyle(_attributes) => {
                    let css_content = parser.extract_style_content()?;

                    let handle = tokio::task::spawn_blocking(move || {
                        Some(CSSStyleSheet::from_css(
                            &css_content,
                            StylesheetOrigin::Author,
                            true,
                        ))
                    });
                    style_handles.push(handle);

                    parser.resume()?;
                }
                BlockedReason::WaitingForResource(resource_type, href) => match resource_type {
                    ResourceType::Style => {
                        let relative_url = url.join(href).map_err(|e| {
                            NavigationError::RequestError(RequestError::Network(
                                NetworkError::InvalidUrl(e.to_string()),
                            ))
                        })?;

                        let handle = spawn_style_fetch_and_parse(
                            relative_url,
                            &page,
                            client.box_clone(),
                            Arc::clone(&headers),
                            cookies.clone(),
                            Arc::clone(&cookie_jar),
                        );
                        style_handles.push(handle);

                        parser.resume()?;
                    }
                },
                BlockedReason::SVGContent => {
                    let _svg_content = parser.extract_svg_content()?;

                    // TODO: Process SVG content

                    parser.resume()?;
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

    let policies = match ctx.tab_manager().get_tab_mut(tab_id) {
        Some(tab) => *tab.policies(),
        None => DocumentPolicy::default(),
    };

    Ok(page.load(
        result.metadata.title.unwrap_or(url.to_string()),
        Some(url.clone()),
        result.dom_tree,
        stylesheets,
        Arc::new(policies),
        result.metadata.images,
    ))
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
    page_url: &Option<Url>,
    policies: &Arc<DocumentPolicy>,
    cookies: &[Cookie],
    headers: &Arc<HeaderMap>,
    client: &dyn HttpClient,
) -> Result<(Url, Response), NavigationError> {
    if let Some(location) = raw_url.strip_prefix("about:") {
        if ALLOWED_ABOUT_URLS
            .iter()
            .all(|&allowed| allowed != location)
        {
            return Err(NavigationError::RequestError(RequestError::Network(
                NetworkError::InvalidUrl(format!("Disallowed about URL: {}", location)),
            )));
        }

        let resp = Response::from(
            Resource::load(io::ResourceType::Absolute {
                protocol: "about",
                location: format!("{}.html", location).as_str(),
            })
            .map_err(|e| {
                NavigationError::RequestError(RequestError::Network(NetworkError::InvalidUrl(
                    e.to_string(),
                )))
            })?,
        );

        let url = Url::parse(&format!("about:{}", location))
            .unwrap_or_else(|_| Url::parse("about:blank").unwrap());

        return Ok((url, resp));
    }

    let url = match page_url.as_ref() {
        Some(u) => u.join(raw_url),
        None => Url::parse(raw_url),
    }
    .map_err(|e| RequestError::Network(NetworkError::InvalidUrl(e.to_string())))?;

    let resp = if url.scheme() != "http" && url.scheme() != "https" {
        Response::from(
            Resource::load(io::ResourceType::Absolute {
                protocol: url.scheme(),
                location: url.path(),
            })
            .map_err(|e| {
                NavigationError::RequestError(RequestError::Network(NetworkError::InvalidUrl(
                    e.to_string(),
                )))
            })?,
        )
    } else {
        return resolve_request_inner(url, ctx, page_url, policies, cookies, headers, client).await;
    };

    Ok((url, resp))
}

/// The safer version of `resolve_navigation_request` that can be used by other commands
/// like image loading, which also need to resolve URLs and fetch content while respecting
/// cookies and headers. This function performs the same URL resolution and fetching logic,
/// but returns the resolved URL along with the response body, allowing callers to handle
/// the content as needed.
pub(crate) async fn resolve_request(
    raw_url: &str,
    ctx: &mut dyn NavigationContext,
    page_url: &Option<Url>,
    policies: &Arc<DocumentPolicy>,
    cookies: &[Cookie],
    headers: &Arc<HeaderMap>,
    client: &dyn HttpClient,
) -> Result<(Url, Response), NavigationError> {
    let url = match page_url.as_ref() {
        Some(u) => u.join(raw_url),
        None => Url::parse(raw_url),
    }
    .map_err(|e| RequestError::Network(NetworkError::InvalidUrl(e.to_string())))?;

    resolve_request_inner(url, ctx, page_url, policies, cookies, headers, client).await
}

async fn resolve_request_inner(
    url: Url,
    ctx: &mut dyn NavigationContext,
    page_url: &Option<Url>,
    policies: &Arc<DocumentPolicy>,
    cookies: &[Cookie],
    headers: &Arc<HeaderMap>,
    client: &dyn HttpClient,
) -> Result<(Url, Response), NavigationError> {
    let resp =
        Resource::from_remote(url.as_str(), client, cookies, headers, page_url, policies).await?;

    for header in resp.metadata().headers.iter() {
        if header.0 == SET_COOKIE
            && let Ok(mut cookie_jar) = ctx.cookie_jar().lock()
        {
            CookieMiddleware::handle_response_cookie(&mut cookie_jar, &url, header.1);
        }
    }

    let response = resp
        .body()
        .await
        .map_err(|e| RequestError::Network(NetworkError::RuntimeError(e.to_string())))?;

    Ok((url, response))
}

/// Spawns a task to fetch and parse a stylesheet from the given URL, returning a handle to the resulting stylesheet.
/// The task will handle cookies and headers appropriately, and will return `None` if fetching or parsing fails.
fn spawn_style_fetch_and_parse(
    url: Url,
    page: &Page,
    client: Box<dyn HttpClient>,
    headers: Arc<HeaderMap>,
    cookies: Vec<Cookie>,
    cookie_jar: Arc<Mutex<CookieJar>>,
) -> JoinHandle<Option<CSSStyleSheet>> {
    let page_url = page.document_url.clone();
    let policies = page.policies().clone();

    tokio::spawn(async move {
        if url.scheme() != "http" && url.scheme() != "https" {
            let res = Resource::load(io::ResourceType::Absolute {
                protocol: url.scheme(),
                location: url.path(),
            });

            let body = match res {
                Ok(b) => b,
                Err(e) => {
                    debug!("Failed to load stylesheet {}: {}", url, e);
                    return None;
                }
            };

            let stylesheet_url = url.clone();
            match tokio::task::spawn_blocking(move || {
                let css_str = String::from_utf8_lossy(&body);
                CSSStyleSheet::from_css(&css_str, StylesheetOrigin::Author, true)
            })
            .await
            {
                Ok(stylesheet) => Some(stylesheet),
                Err(e) => {
                    warn!("CSS parse task panicked for {}: {}", stylesheet_url, e);
                    None
                }
            }
        } else {
            let resp = match Resource::from_remote(
                url.as_str(),
                client.as_ref(),
                &cookies,
                &headers,
                &page_url,
                &policies,
            )
            .await
            {
                Ok(r) => r,
                Err(e) => {
                    debug!("Failed to fetch stylesheet {}: {}", url, e);
                    return None;
                }
            };

            for header in resp.metadata().headers.iter() {
                if header.0 == SET_COOKIE
                    && let Ok(mut cookie_jar) = cookie_jar.lock()
                {
                    CookieMiddleware::handle_response_cookie(&mut cookie_jar, &url, header.1);
                }
            }

            let body_bytes = match resp.body().await {
                Ok(body_resp) => match body_resp.body {
                    Some(b) => b,
                    None => {
                        debug!("Empty body for stylesheet {}", url);
                        return None;
                    }
                },
                Err(e) => {
                    debug!("Failed to read body for stylesheet {}: {}", url, e);
                    return None;
                }
            };

            let stylesheet_url = url.clone();
            match tokio::task::spawn_blocking(move || {
                let css_str = String::from_utf8_lossy(&body_bytes);
                CSSStyleSheet::from_css(&css_str, StylesheetOrigin::Author, true)
            })
            .await
            {
                Ok(stylesheet) => Some(stylesheet),
                Err(e) => {
                    warn!("CSS parse task panicked for {}: {}", stylesheet_url, e);
                    None
                }
            }
        }
    })
}

use std::{collections::HashMap, sync::Arc};

use crate::errors::NavigationError;
use css_cssom::CSSStyleSheet;
use html_parser::{BlockedReason, HtmlStreamParser, ParserState, ResourceType};
use io::{
    DocumentPolicy,
    manager::{RequestContext, Resource},
};
use network::errors::{NetworkError, RequestError};
use tracing::warn;
use url::Url;

use crate::{
    TabId,
    navigation::NavigationContext,
    tab::{collector::TabCollector, page::Page},
};

/// Navigates the specified tab to the given URL, fetching and parsing the content.
/// Executes any scripts and processes stylesheets found during parsing.
pub async fn navigate(
    ctx: &mut dyn NavigationContext,
    tab_id: TabId,
    url: &str,
    mut stylesheets: Vec<CSSStyleSheet>,
) -> Result<Page, NavigationError> {
    let mut page = Page::blank();

    let url = Url::parse(url).map_err(|e| {
        NavigationError::RequestError(RequestError::Network(NetworkError::InvalidUrl(
            e.to_string(),
        )))
    })?;

    let protocol = url.scheme();

    let body = {
        let client = ctx.http_client().box_clone();
        let headers = Arc::clone(ctx.headers());
        let cookie_jar = ctx
            .cookie_jar()
            .get_mut()
            .map_err(|_| NavigationError::CookieJarLocked)?;

        Resource::load_async(
            io::manager::ResourceType::Absolute {
                protocol,
                location: url.as_str(),
                ctx: &mut RequestContext {
                    client: client.as_ref(),
                    cookie_jar,
                    browser_headers: &headers,
                    page_url: &page.document_url,
                    policies: page.policies(),
                },
            },
            &mut HashMap::default(),
        )
        .await
    };

    let body = match body {
        Ok(b) => b,
        Err(e) => {
            return Err(NavigationError::RequestError(RequestError::Network(
                NetworkError::RuntimeError(e.to_string()),
            )));
        }
    };

    page.document_url = Some(url.clone());

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
                    ctx.style_processor()
                        .process_css(&css_content, &mut stylesheets);

                    parser.resume()?;
                }
                BlockedReason::WaitingForResource(resource_type, href) => match resource_type {
                    ResourceType::Style => {
                        let style_resp = {
                            let client = ctx.http_client().box_clone();
                            let headers = Arc::clone(ctx.headers());
                            let cookie_jar = ctx
                                .cookie_jar()
                                .get_mut()
                                .map_err(|_| NavigationError::CookieJarLocked)?;

                            let relative_url = url.join(href).map_err(|e| {
                                NavigationError::RequestError(RequestError::Network(
                                    NetworkError::InvalidUrl(e.to_string()),
                                ))
                            })?;

                            Resource::load_async(
                                io::manager::ResourceType::Absolute {
                                    protocol,
                                    location: relative_url.as_str(),
                                    ctx: &mut RequestContext {
                                        client: client.as_ref(),
                                        cookie_jar,
                                        browser_headers: &headers,
                                        page_url: &page.document_url,
                                        policies: page.policies(),
                                    },
                                },
                                &mut HashMap::default(),
                            )
                            .await
                        };

                        let style_body = match style_resp {
                            Ok(resp) => resp,
                            Err(e) => {
                                return Err(NavigationError::RequestError(RequestError::Network(
                                    NetworkError::RuntimeError(e.to_string()),
                                )));
                            }
                        };

                        let css_content =
                            String::from_utf8_lossy(style_body.as_slice()).to_string();
                        ctx.style_processor()
                            .process_css(&css_content, &mut stylesheets);

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
    ))
}

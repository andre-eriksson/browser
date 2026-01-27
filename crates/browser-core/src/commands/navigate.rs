use std::sync::Arc;

use css_cssom::CSSStyleSheet;
use errors::{
    browser::NavigationError,
    network::{NetworkError, RequestError},
};
use html_parser::{BlockedReason, HtmlStreamParser, ParserState, ResourceType};
use network::http::request::RequestBuilder;
use url::Url;

use crate::{
    TabId,
    navigation::NavigationContext,
    service::network::{policy::DocumentPolicy, request::RequestResult},
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

    // TODO: Handle exceptions like about:blank or about:config
    let request = RequestBuilder::from(url.clone()).build();
    let header_response = match ctx.network_service().fetch(&mut page, request).await {
        RequestResult::Failed(err) => return Err(NavigationError::RequestError(err)),
        RequestResult::ClientError(resp)
        | RequestResult::ServerError(resp)
        | RequestResult::Success(resp) => resp,
    };
    let body = match header_response.body().await {
        Ok(resp) => match resp.body {
            Some(b) => b,
            None => {
                return Err(NavigationError::RequestError(RequestError::EmptyBody));
            }
        },
        Err(e) => {
            return Err(NavigationError::RequestError(RequestError::Network(e)));
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
                    if attributes.get("src").is_some() {
                        let src = attributes.get("src").unwrap();

                        let src_url = match url.join(src) {
                            Ok(u) => u,
                            Err(e) => {
                                return Err(NavigationError::RequestError(RequestError::Network(
                                    NetworkError::InvalidUrl(e.to_string()),
                                )));
                            }
                        };

                        let script_request = RequestBuilder::from(src_url).build();
                        let script_resp = ctx.network_service().fetch(&mut page, script_request);
                        let script_response = match script_resp.await {
                            RequestResult::Failed(err) => {
                                return Err(NavigationError::RequestError(err));
                            }
                            RequestResult::ClientError(resp)
                            | RequestResult::ServerError(resp)
                            | RequestResult::Success(resp) => resp,
                        };
                        let script_body = match script_response.body().await {
                            Ok(resp) => match resp.body {
                                Some(b) => b,
                                None => {
                                    return Err(NavigationError::RequestError(
                                        RequestError::EmptyBody,
                                    ));
                                }
                            },
                            Err(e) => {
                                return Err(NavigationError::RequestError(RequestError::Network(
                                    e,
                                )));
                            }
                        };

                        let script_text =
                            String::from_utf8_lossy(script_body.as_slice()).to_string();
                        let _ = parser.extract_script_content()?;
                        ctx.script_executor().execute_script(&script_text);
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
                        let style_url = match url.join(href) {
                            Ok(u) => u,
                            Err(e) => {
                                return Err(NavigationError::RequestError(RequestError::Network(
                                    NetworkError::InvalidUrl(e.to_string()),
                                )));
                            }
                        };

                        let style_request = RequestBuilder::from(style_url).build();
                        let style_resp = ctx.network_service().fetch(&mut page, style_request);
                        let style_response = match style_resp.await {
                            RequestResult::Failed(err) => {
                                return Err(NavigationError::RequestError(err));
                            }
                            RequestResult::ClientError(resp)
                            | RequestResult::ServerError(resp)
                            | RequestResult::Success(resp) => resp,
                        };
                        let style_body = match style_response.body().await {
                            Ok(resp) => match resp.body {
                                Some(b) => b,
                                None => {
                                    return Err(NavigationError::RequestError(
                                        RequestError::EmptyBody,
                                    ));
                                }
                            },
                            Err(e) => {
                                return Err(NavigationError::RequestError(RequestError::Network(
                                    e,
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
                BlockedReason::ParsingSVG => {
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
        Some(tab) => tab.policies().clone(),
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

use std::sync::Arc;

use css_cssom::CSSStyleSheet;
use errors::network::{HttpError, NetworkError};
use html_parser::{BlockedReason, HtmlStreamParser, ParserState};
use network::http::request::RequestBuilder;
use url::Url;

use crate::{
    TabId,
    navigation::NavigationContext,
    service::network::{policy::DocumentPolicy, service::NetworkService},
    tab::{collector::TabCollector, page::Page},
};

async fn fetch_url(
    page: &mut Page,
    network: &mut NetworkService,
    url: &Url,
) -> Result<Vec<u8>, NetworkError> {
    let request = RequestBuilder::from(url.clone()).build();
    let header_response = network.fetch(page, request).await?;

    let response = header_response.body().await?;

    let body = match response.body {
        Some(b) => b,
        None => return Err(NetworkError::Http(HttpError::MissingBody)),
    };

    Ok(body)
}

/// Navigates the specified tab to the given URL, fetching and parsing the content.
/// Executes any scripts and processes stylesheets found during parsing.
pub async fn navigate(
    ctx: &mut dyn NavigationContext,
    tab_id: TabId,
    url: &Url,
    mut stylesheets: Vec<CSSStyleSheet>,
) -> Result<Page, String> {
    let mut network_service = ctx.network_service().clone();
    let mut page = Page::blank();

    // TODO: Handle exceptions like about:blank or about:config
    let body = match fetch_url(&mut page, &mut network_service, url).await {
        Ok(b) => b,
        Err(e) => {
            return Err(format!("Failed to fetch URL {}: {}", url, e));
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
                    //println!("Script attributes: {:?}", attributes);

                    if attributes.get("src").is_some() {
                        let src = attributes.get("src").unwrap();

                        let src_url = match url.join(src) {
                            Ok(u) => u,
                            Err(e) => {
                                return Err(format!("Failed to resolve script URL {}: {}", src, e));
                            }
                        };

                        let script_body =
                            match fetch_url(&mut page, &mut network_service, &src_url).await {
                                Ok(b) => b,
                                Err(e) => {
                                    return Err(format!("Failed to fetch script {}: {}", src, e));
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
                _ => {
                    break;
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

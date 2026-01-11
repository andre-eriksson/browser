use css_cssom::CSSStyleSheet;
use errors::network::{HttpError, NetworkError};
use html_dom::BuildResult;
use html_parser::{BlockedReason, HtmlStreamParser, ParserState};
use network::http::request::RequestBuilder;
use url::Url;

use crate::{
    TabId,
    navigation::NavigationContext,
    service::network::{context::NetworkContext, service::NetworkService},
    tab::TabCollector,
};

async fn fetch_url(
    ctx: &mut NetworkContext,
    network: &mut NetworkService,
    url: &str,
) -> Result<Vec<u8>, NetworkError> {
    let url = if ctx.document_url.is_some() {
        let base_url = ctx.document_url.as_ref().unwrap();
        base_url
            .join(url)
            .map_err(|e| NetworkError::Http(HttpError::InvalidURL(e.to_string())))?
    } else {
        Url::parse(url).map_err(|e| NetworkError::Http(HttpError::InvalidURL(e.to_string())))?
    };

    let request = RequestBuilder::from(url.clone()).build();
    let header_response = network.fetch(ctx, request).await?;

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
    url: &str,
    stylesheets: &mut Vec<CSSStyleSheet>,
) -> Result<BuildResult<TabCollector>, String> {
    let mut network_service = ctx.network_service().clone();
    let mut network_ctx = ctx
        .tab_manager()
        .get_tab_mut(tab_id)
        .map(|tab| tab.network_context().clone())
        .ok_or_else(|| format!("Tab with id {:?} not found in TabManager", tab_id))?;

    // TODO: Handle exceptions like about:blank or about:config
    let body = match fetch_url(&mut network_ctx, &mut network_service, url).await {
        Ok(b) => b,
        Err(e) => {
            return Err(format!("Failed to fetch URL {}: {}", url, e));
        }
    };

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
                        let script_body =
                            match fetch_url(&mut network_ctx, &mut network_service, src).await {
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
                    ctx.style_processor().process_css(&css_content, stylesheets);

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

    Ok(parser.finalize())
}

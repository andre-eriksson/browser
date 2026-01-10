use html_parser::{
    parser::HtmlStreamParser,
    state::{BlockedReason, ParserState},
};
use network::http::request::RequestBuilder;
use tracing::debug;
use url::Url;

use crate::{
    browser::Browser,
    events::BrowserEvent,
    tab::{TabCollector, TabId, TabMetadata},
};

/// Navigates the specified tab to the given URL, fetching and parsing the content.
/// Executes any scripts and processes stylesheets found during parsing.
pub async fn navigate_to(
    browser: &mut Browser,
    tab_id: TabId,
    url: String,
) -> Result<BrowserEvent, String> {
    debug!("Navigating tab {:?} to URL: {}", tab_id, url);

    // TODO: Handle exceptions like about:blank or about:config
    let url = match Url::parse(&url) {
        Ok(parsed_url) => parsed_url,
        Err(e) => return Err(format!("Invalid URL: {}", e)),
    };

    if !browser.tab_manager().tabs().iter().any(|t| t.id == tab_id) {
        return Err(format!("Tab with ID {:?} does not exist", tab_id));
    }

    let request = RequestBuilder::from(url.clone()).build();
    let header_response = match browser.http_client().send(request).await {
        Ok(resp) => resp,
        Err(e) => return Err(format!("HTTP request failed: {}", e)),
    };

    let response = match header_response.body().await {
        Ok(b) => b,
        Err(e) => return Err(format!("Failed to read response body: {}", e)),
    };

    let body = match response.body {
        Some(b) => b,
        None => return Err("Response body is empty".to_string()),
    };

    let mut parser = HtmlStreamParser::<_, TabCollector>::new(
        body.as_slice(),
        None,
        Some(TabCollector::default()),
    );

    let mut stylesheets = Vec::new();

    loop {
        parser.step()?;

        match parser.get_state() {
            ParserState::Running => continue,
            ParserState::Blocked(reason) => match reason {
                BlockedReason::WaitingForScript(attributes) => {
                    //println!("Script attributes: {:?}", attributes);

                    if attributes.get("src").is_some() {
                        let src = attributes.get("src").unwrap();
                        let script_request = match RequestBuilder::from_relative_url(&url, src) {
                            Ok(req) => req.build(),
                            Err(e) => {
                                return Err(format!(
                                    "Failed to build script request for {}: {}",
                                    src, e
                                ));
                            }
                        };

                        let script_response = match browser.http_client().send(script_request).await
                        {
                            Ok(resp) => resp,
                            Err(e) => {
                                return Err(format!("Failed to fetch script from {}: {}", src, e));
                            }
                        };

                        let response = match script_response.body().await {
                            Ok(b) => b,
                            Err(e) => {
                                return Err(format!(
                                    "Failed to read script body from {}: {}",
                                    src, e
                                ));
                            }
                        };

                        let body = match response.body {
                            Some(b) => b,
                            None => {
                                return Err(format!("Script body from {} is empty", src));
                            }
                        };

                        let script_text = String::from_utf8_lossy(body.as_slice()).to_string();
                        let _ = parser.extract_script_content()?;
                        browser.execute_script(&script_text);
                    } else {
                        let script_content = parser.extract_script_content()?;
                        browser.execute_script(&script_content);
                    }

                    parser.resume()?;
                }
                BlockedReason::WaitingForStyle(_attributes) => {
                    let css_content = parser.extract_style_content()?;
                    browser.process_css(&css_content, &mut stylesheets);

                    parser.resume()?;
                }
                _ => {
                    debug!(
                        "Parser for tab {:?} is blocked for reason: {:?}",
                        tab_id, reason
                    );
                    break;
                }
            },
            ParserState::Completed => {
                break;
            }
        }
    }

    let parser_result = parser.finalize();
    let default_stylesheet = browser.default_stylesheet().clone();

    let tab = browser
        .tab_manager()
        .tabs_mut()
        .iter_mut()
        .find(|t| t.id == tab_id)
        .ok_or_else(|| format!("Tab with ID {:?} does not exist", tab_id))?;

    tab.clear_stylesheets();
    tab.add_stylesheet(default_stylesheet);

    for stylesheet in stylesheets {
        tab.add_stylesheet(stylesheet);
    }

    Ok(BrowserEvent::NavigateSuccess(TabMetadata {
        id: tab_id,
        title: parser_result.metadata.title.unwrap_or(url.to_string()),
        document: parser_result.dom_tree,
        stylesheets: tab.stylesheets().clone(),
    }))
}

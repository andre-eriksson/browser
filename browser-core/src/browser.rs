use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use cookies::cookie_store::CookieJar;
use html_parser::{
    parser::HtmlStreamParser,
    state::{BlockedReason, ParserState},
};
use http::HeaderMap;
use network::http::{client::HttpClient, request::RequestBuilder};
use tracing::debug;
use url::Url;

use crate::{
    commands::BrowserCommand,
    events::{BrowserEvent, Emitter},
    tab::{Tab, TabCollector, TabId, TabMetadata},
};

#[async_trait]
pub trait Commandable {
    async fn execute(&mut self, command: BrowserCommand) -> Result<BrowserEvent, String>;
}

pub struct Browser {
    active_tab: TabId,
    tabs: Vec<Tab>,
    next_tab_id: usize,

    emitter: Box<dyn Emitter<BrowserEvent> + Send + Sync>,

    http_client: Box<dyn HttpClient>,
    cookie_jar: Arc<Mutex<CookieJar>>,
    headers: Arc<HeaderMap>,
}

impl Browser {
    pub fn new(
        emitter: Box<dyn Emitter<BrowserEvent> + Send + Sync>,
        http_client: Box<dyn HttpClient>,
        cookie_jar: Arc<Mutex<CookieJar>>,
        headers: Arc<HeaderMap>,
    ) -> Self {
        Browser {
            active_tab: TabId(0),
            tabs: vec![Tab::new(TabId(0), None)],
            next_tab_id: 1,
            emitter,
            http_client,
            cookie_jar,
            headers,
        }
    }

    pub fn print_body(&self, tab_id: TabId) {
        if let Some(tab) = self.tabs.iter().find(|t| t.id == tab_id) {
            if let Some(document) = &tab.document {
                debug!("Tab {:?} Document:\n{}", tab_id, document);
            } else {
                debug!("Tab {:?} has no document loaded.", tab_id);
            }
        } else {
            debug!("Tab {:?} does not exist.", tab_id);
        }
    }

    fn execute_script(&mut self, script: &str) {
        debug!("Executing script: {}", script);
    }

    fn process_css(&mut self, css: &str) {
        let stylesheet = CSSStyleSheet::from_css(css);

        println!("Parsed CSS Stylesheet: {:?}", stylesheet);
    }
}

#[async_trait]
impl Commandable for Browser {
    async fn execute(&mut self, command: BrowserCommand) -> Result<BrowserEvent, String> {
        match command {
            BrowserCommand::Navigate { tab_id, url } => {
                debug!("Navigating tab {:?} to URL: {}", tab_id, url);

                // TODO: Handle exceptions like about:blank or about:config
                let url = match Url::parse(&url) {
                    Ok(parsed_url) => parsed_url,
                    Err(e) => return Err(format!("Invalid URL: {}", e)),
                };

                if !self.tabs.iter().any(|t| t.id == tab_id) {
                    return Err(format!("Tab with ID {:?} does not exist", tab_id));
                }

                let request = RequestBuilder::from(url.clone()).build();
                let header_response = match self.http_client.send(request).await {
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

                loop {
                    parser.step()?;

                    match parser.get_state() {
                        ParserState::Running => continue,
                        ParserState::Blocked(reason) => match reason {
                            BlockedReason::WaitingForScript(attributes) => {
                                //println!("Script attributes: {:?}", attributes);

                                if attributes.get("src").is_some() {
                                    let src = attributes.get("src").unwrap();
                                    let script_request =
                                        match RequestBuilder::from_relative_url(&url, src) {
                                            Ok(req) => req.build(),
                                            Err(e) => {
                                                return Err(format!(
                                                    "Failed to build script request for {}: {}",
                                                    src, e
                                                ));
                                            }
                                        };

                                    let script_response =
                                        match self.http_client.send(script_request).await {
                                            Ok(resp) => resp,
                                            Err(e) => {
                                                return Err(format!(
                                                    "Failed to fetch script from {}: {}",
                                                    src, e
                                                ));
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
                                            return Err(format!(
                                                "Script body from {} is empty",
                                                src
                                            ));
                                        }
                                    };

                                    let script_text =
                                        String::from_utf8_lossy(body.as_slice()).to_string();
                                    let _ = parser.extract_script_content()?;
                                    self.execute_script(&script_text);
                                } else {
                                    let script_content = parser.extract_script_content()?;
                                    self.execute_script(&script_content);
                                }

                                parser.resume()?;
                            }
                            BlockedReason::WaitingForStyle(_attributes) => {
                                let css_content = parser.extract_style_content()?;
                                self.process_css(&css_content);

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

                let tab = self
                    .tabs
                    .iter_mut()
                    .find(|t| t.id == tab_id)
                    .ok_or_else(|| format!("Tab with ID {:?} does not exist", tab_id))?;
                tab.document = Some(parser_result.dom_tree);

                let metadata = TabMetadata {
                    tab_id,
                    title: parser_result.metadata.title.unwrap_or(url.to_string()),
                };

                return Ok(BrowserEvent::NavigateSuccess(metadata));
            }
            BrowserCommand::AddTab { url } => {
                let new_tab_id = TabId(self.next_tab_id);

                if let Some(url) = url {
                    let parsed_url = match Url::parse(&url) {
                        Ok(u) => u,
                        Err(e) => return Err(format!("Invalid URL: {}", e)),
                    };
                    let new_tab = Tab::new(new_tab_id, Some(parsed_url));
                    self.tabs.push(new_tab);
                } else {
                    let new_tab = Tab::new(new_tab_id, None);
                    self.tabs.push(new_tab);
                }

                self.next_tab_id += 1;
                debug!("Added new tab with ID {:?}", new_tab_id);

                return Ok(BrowserEvent::TabAdded(new_tab_id));
            }
            BrowserCommand::CloseTab { tab_id } => {
                debug!("Closing tab with ID {:?}", tab_id);
                if let Some(pos) = self.tabs.iter().position(|t| t.id == tab_id) {
                    self.tabs.remove(pos);
                    debug!("Closed tab with ID {:?}", tab_id);

                    if self.active_tab == tab_id
                        && let Some(first_tab) = self.tabs.first()
                    {
                        self.active_tab = first_tab.id;
                        debug!("Changed active tab to {:?}", first_tab.id);
                        self.emitter
                            .emit(BrowserEvent::ActiveTabChanged(first_tab.id));
                    }

                    return Ok(BrowserEvent::TabClosed(tab_id));
                } else {
                    return Err(format!("Tab with ID {:?} does not exist", tab_id));
                }
            }
            BrowserCommand::ChangeActiveTab { tab_id } => {
                if !self.tabs.iter().any(|t| t.id == tab_id) {
                    return Err(format!("Tab with ID {:?} does not exist", tab_id));
                }

                self.active_tab = tab_id;
                debug!("Changed active tab to {:?}", tab_id);

                return Ok(BrowserEvent::ActiveTabChanged(tab_id));
            }
        }
    }
}

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use cookies::cookie_store::CookieJar;
use event::{Emitter, browser::BrowserEvent};
use html_parser::{
    parser::HtmlStreamParser,
    state::{BlockedReason, ParserState},
};
use html_syntax::collector::DefaultCollector;
use http::HeaderMap;
use network::http::{client::HttpClient, request::RequestBuilder};
use tracing::debug;
use url::Url;

use crate::{commands::BrowserCommand, tab::Tab};

#[async_trait]
pub trait Commandable {
    async fn execute(&mut self, command: BrowserCommand) -> Result<BrowserEvent, String>;
}

pub struct Browser {
    emitter: Box<dyn Emitter<BrowserEvent> + Send + Sync>,

    http_client: Box<dyn HttpClient>,
    cookie_jar: Arc<Mutex<CookieJar>>,
    headers: Arc<HeaderMap>,

    tabs: Vec<Tab>,
}

impl Browser {
    pub fn new(
        emitter: Box<dyn Emitter<BrowserEvent> + Send + Sync>,
        http_client: Box<dyn HttpClient>,
        cookie_jar: Arc<Mutex<CookieJar>>,
        headers: Arc<HeaderMap>,
    ) -> Self {
        Browser {
            emitter,
            http_client,
            cookie_jar,
            headers,
            tabs: vec![Tab::new(0, None)],
        }
    }

    fn execute_script(&self, script: &str) {
        debug!("Executing script: {}", script);
    }

    fn process_css(&self, css: &str) {
        debug!("Processing CSS: {}", css);
    }
}

#[async_trait]
impl Commandable for Browser {
    async fn execute(&mut self, command: BrowserCommand) -> Result<BrowserEvent, String> {
        match command {
            BrowserCommand::Navigate { tab_id, url } => {
                debug!("Navigating tab {} to URL: {}", tab_id, url);

                // TODO: Handle exceptions like about:blank or about:config
                let url = match Url::parse(&url) {
                    Ok(parsed_url) => parsed_url,
                    Err(e) => return Err(format!("Invalid URL: {}", e)),
                };

                let _tab = match self.tabs.iter_mut().find(|t| t.id == tab_id) {
                    Some(t) => t,
                    None => return Err(format!("Tab with ID {} not found", tab_id)),
                };

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

                let mut parser = HtmlStreamParser::<_, DefaultCollector>::simple(body.as_slice());

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
                                    "Parser for tab {} is blocked for reason: {:?}",
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

                let _ = parser.finalize();

                return Ok(BrowserEvent::NavigateSuccess);
            }
            _ => unimplemented!("Command not implemented yet"),
        }
    }
}

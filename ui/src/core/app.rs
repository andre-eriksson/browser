use std::sync::Arc;
use std::sync::Mutex;

use cookies::cookie_store::CookieJar;
use http::HeaderMap;
use iced::{Renderer, Subscription, Task, Theme, window};
use network::client::reqwest::ReqwestClient;
use network::http::client::HttpClient;
use network::http::request::RequestBuilder;
use network::session::network::NetworkSession;
use telemetry::events::ui::EVENT_NEW_TAB;
use telemetry::events::ui::EVENT_TAB_CLOSED;
use telemetry::keys::EVENT;
use tracing::error;
use tracing::info;

use html_parser::parser::streaming::HtmlStreamParser;
use url::Url;

use crate::{
    api::{
        message::Message,
        tabs::{BrowserTab, TabCollector},
        window::WindowType,
    },
    manager::WindowController,
    views::{browser, devtools},
};

/// Represents the main application state, including the current window, tabs, and client.
pub struct Application {
    pub id: window::Id,
    window_controller: WindowController<Message, Theme, iced::Renderer>,
    pub tabs: Vec<BrowserTab>,
    pub current_tab_id: usize,
    pub next_tab_id: usize,
    browser_headers: Arc<HeaderMap>,
    cookie_jar: Arc<Mutex<CookieJar>>,
}

impl Application {
    /// Creates a new instance of the `Application` with an initial window and a default tab.
    pub fn new(
        browser_headers: Arc<HeaderMap>,
        cookie_jar: Arc<Mutex<CookieJar>>,
    ) -> (Self, Task<Message>) {
        let mut window_controller = WindowController::new();
        let (main_window_id, browser_task) =
            window_controller.new_window(Box::new(browser::window::BrowserWindow::default()));

        let tasks = vec![browser_task.map(|_| Message::None)];

        let first_tab = BrowserTab::empty(0);

        let app = Application {
            id: main_window_id,
            window_controller,
            tabs: vec![first_tab],
            current_tab_id: 0,
            next_tab_id: 1,
            browser_headers,
            cookie_jar,
        };

        (app, Task::batch(tasks))
    }

    /// Updates the application state based on the received message.
    ///
    /// # Arguments
    /// * `message` - The message containing the action to perform.
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::None => {}

            // === Window Management ===
            Message::NewWindow(window_type) => match window_type {
                WindowType::Devtools => {
                    let (_, window_task) = self
                        .window_controller
                        .new_window(Box::new(devtools::window::DevtoolsWindow));
                    return window_task.map(|_| Message::None);
                }
            },
            Message::CloseWindow(window_id) => {
                self.window_controller.close(window_id);

                if window_id == self.id {
                    if self.window_controller.open_windows.is_empty() {
                        return iced::exit();
                    } else {
                        return self
                            .window_controller
                            .close_all_windows()
                            .map(|_| Message::None);
                    }
                }

                if self.window_controller.open_windows.is_empty() {
                    return iced::exit();
                }
            }

            // === Tab Management ===
            Message::OpenNewTab => {
                let new_tab = BrowserTab::empty(self.next_tab_id);
                self.tabs.push(new_tab);
                self.current_tab_id = self.tabs.len() - 1;
                info!({ EVENT } = EVENT_NEW_TAB, tab_id = self.next_tab_id);
                self.next_tab_id += 1;
            }
            Message::ChangeTab(index) => {
                if index < self.tabs.len() {
                    self.current_tab_id = index;
                }
            }
            Message::CloseTab(index) => {
                if self.tabs.len() <= 1 {
                    return Task::none();
                }

                if let Some(pos) = self.tabs.iter().position(|t| t.id == index) {
                    self.tabs.remove(pos);
                }

                if self.current_tab_id >= self.tabs.len() {
                    self.current_tab_id = self.tabs.len().saturating_sub(1);
                }

                info!({ EVENT } = EVENT_TAB_CLOSED, tab_id = index);
            }
            Message::ChangeURL(new_url) => {
                self.tabs[self.current_tab_id].temp_url = new_url;
            }

            // === Navigation ===
            Message::NavigateTo(new_url) => {
                //let resolved_url =
                //    resolve_path(&self.web_client.origin.ascii_serialization(), &url);
                //self.tabs[self.current_tab_id].temp_url = resolved_url.clone();
                //let mut client_clone = self.web_client.clone();

                let url = Url::parse(&new_url);
                if let Err(err) = url {
                    error!("Invalid URL: {}", err);
                    return Task::none();
                }
                let url = url.unwrap();

                let network_client = Box::new(ReqwestClient::new()) as Box<dyn HttpClient>;
                let browser_headers = Arc::clone(&self.browser_headers);
                let cookie_jar = Arc::clone(&self.cookie_jar);

                let mut session =
                    NetworkSession::new(network_client, browser_headers, cookie_jar, None);

                session.set_current_url(url.clone());

                return Task::perform(
                    async move {
                        let url_for_error = url.clone();
                        let req = RequestBuilder::from(url).build();
                        let res = session.send(req).await;

                        match res {
                            Ok(response_handle) => {
                                let body = response_handle.body().await;
                                match body {
                                    Ok(resp) => {
                                        if let Some(content) = resp.body {
                                            let html =
                                                String::from_utf8_lossy(&content).to_string();
                                            return Ok((html, session));
                                        }
                                        Err("Response body is empty".to_string())
                                    }
                                    Err(err) => {
                                        Err(format!("Failed to read response body, {}", err))
                                    }
                                }
                            }
                            Err(_) => Err(format!("{} took too long to response", url_for_error)),
                        }
                    },
                    |result| match result {
                        Ok((html, session)) => Message::NavigateSuccess(html, session),
                        Err(err) => Message::NavigateError(err),
                    },
                );
            }
            Message::NavigateSuccess(html, session) => {
                let parser = HtmlStreamParser::new(html.as_bytes(), None);

                let url = Url::parse(&self.tabs[self.current_tab_id].temp_url).unwrap();

                let parsing_result = parser.parse(Some(TabCollector {
                    url: Some(url),
                    ..Default::default()
                }));

                match parsing_result {
                    Ok(result) => {
                        let dom_tree = result.dom_tree;

                        self.tabs[self.current_tab_id].network_session = Some(session);

                        self.tabs[self.current_tab_id].metadata =
                            Some(Arc::new(Mutex::new(result.metadata)));

                        self.tabs[self.current_tab_id].html_content = self.tabs
                            [self.current_tab_id]
                            .html_content
                            .convert(dom_tree);

                        return Task::done(Message::RefreshContent);
                    }
                    Err(err) => {
                        error!("Failed to parse HTML: {}", err);
                    }
                }
            }
            Message::NavigateError(err) => {
                error!("Navigation error: {}", err);
            }

            // === UI Updates ===
            Message::RefreshContent => {
                // This message exists purely to trigger a UI refresh
                // No additional action needed as the state has already been updated
            }
        }
        Task::none()
    }

    /// Returns the current subscriptions for the application.
    pub fn subscriptions(&self) -> iced::Subscription<Message> {
        Subscription::batch([window::close_events().map(Message::CloseWindow)])
    }

    /// Renders the application UI for a specific window.
    pub fn view(&self, window_id: window::Id) -> iced::Element<'_, Message, Theme, Renderer> {
        self.window_controller.render(self, window_id)
    }

    /// Returns the title of the application window.
    pub fn title(&self, window_id: window::Id) -> String {
        self.window_controller.title(window_id)
    }
}

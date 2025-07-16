use iced::{Renderer, Subscription, Task, Theme, window};
use tracing::error;
use tracing::info;

use api::{
    dom::ConvertDom,
    logging::{EVENT, EVENT_NEW_TAB, EVENT_TAB_CLOSED},
};
use html_parser::parser::streaming::HtmlStreamParser;
use network::web::client::WebClient;

use crate::{
    api::{
        message::Message,
        tabs::{BrowserTab, TabCollector},
        window::WindowType,
    },
    manager::WindowController,
    network::client::setup_new_client,
    views::{browser, devtools},
};

/// Represents the main application state, including the current window, tabs, and client.
///
/// # Fields
/// * `id` - The unique identifier for the main application window.
/// * `window_controller` - A controller for managing application windows.
/// * `tabs` - A vector of `BrowserTab` representing the open tabs in the browser.
/// * `current_tab_id` - The index of the currently active tab.
/// * `next_tab_id` - A counter for generating unique IDs for new tabs.
/// * `client` - An instance of `WebClient` used for making network requests.
pub struct Application {
    pub id: window::Id,
    window_controller: WindowController<Message, Theme, iced::Renderer>,
    pub tabs: Vec<BrowserTab>,
    pub current_tab_id: usize,
    pub next_tab_id: usize,
    web_client: WebClient,
}

impl Application {
    /// Creates a new instance of the `Application` with an initial window and a default tab.
    pub fn new() -> (Self, Task<Message>) {
        let mut window_controller = WindowController::new();
        let (main_window_id, browser_task) =
            window_controller.new_window(Box::new(browser::window::BrowserWindow::default()));

        let tasks = vec![browser_task.map(|_| Message::None)];

        let first_tab = BrowserTab::default();

        let app = Application {
            id: main_window_id,
            window_controller,
            web_client: setup_new_client(),
            tabs: vec![first_tab],
            current_tab_id: 0,
            next_tab_id: 1,
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
                let new_tab = BrowserTab::default();
                self.tabs.push(new_tab);
                self.current_tab_id = self.tabs.len() - 1;
                info!({ EVENT } = EVENT_NEW_TAB, tab_id = self.next_tab_id);
                self.next_tab_id += 1;
            }
            Message::ChangeTab(index) => {
                if index < self.tabs.len() {
                    self.current_tab_id = index;
                    println!("Switched to tab: {}", index);
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
            Message::ChangeURL(url) => {
                self.tabs[self.current_tab_id].temp_url = url;
            }

            // === Navigation ===
            Message::NavigateTo(url) => {
                let mut client_clone = self.web_client.clone();
                return Task::perform(
                    async move {
                        let res = client_clone.setup_client_from_url(url.as_str()).await;

                        match res {
                            Ok(response) => match response.text().await {
                                Ok(html) => Ok(html),
                                Err(err) => Err(format!("Failed to read response body: {}", err)),
                            },
                            Err(_) => Err(format!("{} took too long to response", url)),
                        }
                    },
                    |result| match result {
                        Ok(html) => Message::NavigateSuccess(html),
                        Err(err) => Message::NavigateError(err),
                    },
                );
            }
            Message::NavigateSuccess(html) => {
                let parser = HtmlStreamParser::new(html.as_bytes(), None);

                let parsing_result = parser.parse(Some(TabCollector {
                    url: self.tabs[self.current_tab_id].temp_url.clone(),
                    ..Default::default()
                }));

                match parsing_result {
                    Ok(result) => {
                        let dom_tree = result.dom_tree;

                        self.tabs[self.current_tab_id].url =
                            self.tabs[self.current_tab_id].temp_url.clone();

                        let mut metadata = self.tabs[self.current_tab_id].metadata.lock().unwrap();
                        *metadata = result.metadata;

                        let mut html_content =
                            self.tabs[self.current_tab_id].html_content.lock().unwrap();
                        *html_content = dom_tree.convert().lock().unwrap().clone();

                        info!(
                            "Successfully parsed HTML content for tab {}",
                            self.current_tab_id
                        );

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
    pub fn view(&self, window_id: window::Id) -> iced::Element<Message, Theme, Renderer> {
        self.window_controller.render(self, window_id)
    }

    /// Returns the title of the application window.
    pub fn title(&self, window_id: window::Id) -> String {
        self.window_controller.title(window_id)
    }
}

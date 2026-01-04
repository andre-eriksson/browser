use std::hash::{Hash, Hasher};
use std::pin::Pin;
use std::sync::Arc;

use browser_core::browser::Browser;
use browser_core::browser::Commandable;
use browser_core::commands::BrowserCommand;
use browser_core::events::BrowserEvent;
use browser_core::tab::TabId;
use errors::network::NetworkError;
use iced::Subscription;
use iced::futures::Stream;
use iced::futures::stream::unfold;
use iced::{Renderer, Task, Theme, window};
use tokio::sync::Mutex;
use tokio::sync::mpsc::UnboundedReceiver;
use tracing::error;

use crate::api::window::WindowType;
use crate::core::tabs::UiTab;
use crate::events::UiEvent;
use crate::{
    manager::WindowController,
    views::{browser, devtools},
};

#[derive(Debug, Clone)]
pub enum Event {
    None,
    Browser(BrowserEvent),
    Ui(UiEvent),
}

/// Represents the main application state, including the current window, tabs, and client.
pub struct Application {
    pub id: window::Id,
    pub tabs: Vec<UiTab>,
    pub active_tab: TabId,
    pub current_url: String,

    window_controller: WindowController<Event, Theme, iced::Renderer>,

    event_receiver: Arc<Mutex<UnboundedReceiver<BrowserEvent>>>,
    browser: Arc<Mutex<Browser>>,
}

impl Application {
    /// Creates a new instance of the `Application` with an initial window and a default tab.
    pub fn new(
        event_receiver: UnboundedReceiver<BrowserEvent>,
        browser: Arc<Mutex<Browser>>,
    ) -> (Self, Task<Event>) {
        let mut window_controller = WindowController::new();
        let (main_window_id, browser_task) =
            window_controller.new_window(Box::new(browser::window::BrowserWindow::default()));

        let tasks = vec![browser_task.map(|_| Event::None)];
        let first_tab = UiTab::new(TabId(0));

        let app = Application {
            id: main_window_id,
            tabs: vec![first_tab],
            active_tab: TabId(0),
            current_url: "http://127.0.0.1:8000/test.html".to_string(),
            window_controller,
            event_receiver: Arc::new(Mutex::new(event_receiver)),
            browser,
        };

        (app, Task::batch(tasks))
    }

    /// Updates the application state based on the received message.
    ///
    /// # Arguments
    /// * `message` - The message containing the action to perform.
    pub fn update(&mut self, event: Event) -> Task<Event> {
        match event {
            Event::None => {}

            // === UI Events ===
            Event::Ui(ui_event) => match ui_event {
                UiEvent::NewWindow(window_type) => match window_type {
                    WindowType::Devtools => {
                        let (_, window_task) = self
                            .window_controller
                            .new_window(Box::new(devtools::window::DevtoolsWindow));
                        return window_task.map(|_| Event::None);
                    }
                },
                UiEvent::CloseWindow(window_id) => {
                    self.window_controller.close(window_id);

                    if window_id == self.id {
                        if self.window_controller.open_windows.is_empty() {
                            return iced::exit();
                        } else {
                            return self
                                .window_controller
                                .close_all_windows()
                                .map(|_| Event::None);
                        }
                    }

                    if self.window_controller.open_windows.is_empty() {
                        return iced::exit();
                    }
                }

                UiEvent::NewTab => {
                    let browser = self.browser.clone();

                    return Task::perform(
                        async move {
                            let mut lock = browser.lock().await;
                            lock.execute(BrowserCommand::AddTab { url: None }).await
                        },
                        |result| match result {
                            Ok(task) => Event::Browser(task),
                            Err(err) => Event::Browser(BrowserEvent::NavigateError(
                                NetworkError::RequestFailed(err),
                            )),
                        },
                    );
                }
                UiEvent::CloseTab(tab_id) => {
                    let browser = self.browser.clone();

                    return Task::perform(
                        async move {
                            let mut lock = browser.lock().await;
                            lock.execute(BrowserCommand::CloseTab { tab_id }).await
                        },
                        |result| match result {
                            Ok(task) => Event::Browser(task),
                            Err(err) => Event::Browser(BrowserEvent::NavigateError(
                                NetworkError::RequestFailed(err),
                            )),
                        },
                    );
                }
                UiEvent::ChangeActiveTab(tab_id) => {
                    let browser = self.browser.clone();

                    return Task::perform(
                        async move {
                            let mut lock = browser.lock().await;
                            lock.execute(BrowserCommand::ChangeActiveTab { tab_id })
                                .await
                        },
                        |result| match result {
                            Ok(task) => Event::Browser(task),
                            Err(err) => Event::Browser(BrowserEvent::NavigateError(
                                NetworkError::RequestFailed(err),
                            )),
                        },
                    );
                }

                UiEvent::ChangeURL(url) => {
                    self.current_url = url;
                }
            },

            // === Browser Events ===
            Event::Browser(browser_event) => match browser_event {
                BrowserEvent::TabAdded(new_tab_id) => {
                    let new_tab = UiTab::new(new_tab_id);
                    self.tabs.push(new_tab);
                }

                BrowserEvent::TabClosed(tab_id) => {
                    self.tabs.retain(|tab| tab.id != tab_id);
                }

                BrowserEvent::ActiveTabChanged(tab_id) => {
                    self.active_tab = tab_id;
                }

                BrowserEvent::NavigateTo(new_url) => {
                    let browser = self.browser.clone();
                    let active_tab = self.active_tab;

                    return Task::perform(
                        async move {
                            let mut lock = browser.lock().await;
                            lock.execute(BrowserCommand::Navigate {
                                tab_id: active_tab,
                                url: new_url,
                            })
                            .await
                        },
                        |result| match result {
                            Ok(task) => Event::Browser(task),
                            Err(err) => Event::Browser(BrowserEvent::NavigateError(
                                NetworkError::RequestFailed(err),
                            )),
                        },
                    );
                }
                BrowserEvent::NavigateSuccess(metadata) => {
                    let current_tab = self.tabs.iter_mut().find(|tab| tab.id == metadata.tab_id);

                    if let Some(tab) = current_tab {
                        tab.title = Some(metadata.title);
                    }
                }
                BrowserEvent::NavigateError(err) => {
                    error!("Navigation error: {}", err);
                }
            },
        }
        Task::none()
    }

    /// Returns the current subscriptions for the application.
    pub fn subscriptions(&self) -> iced::Subscription<Event> {
        let receiver = self.event_receiver.clone();

        Subscription::batch([
            window::close_events().map(|window_id| Event::Ui(UiEvent::CloseWindow(window_id))),
            Subscription::run_with(ReceiverHandle::new(receiver), create_browser_event_stream),
        ])
    }

    /// Renders the application UI for a specific window.
    pub fn view(&self, window_id: window::Id) -> iced::Element<'_, Event, Theme, Renderer> {
        self.window_controller.render(self, window_id)
    }

    /// Returns the title of the application window.
    pub fn title(&self, window_id: window::Id) -> String {
        self.window_controller.title(window_id)
    }

    /// Returns the theme for the application window.
    pub fn theme(&self, _window_id: window::Id) -> Theme {
        Theme::CatppuccinMocha
    }
}

/// A hashable wrapper around the browser event receiver.
/// The hash is based on a static ID since there's only one receiver.
struct ReceiverHandle {
    receiver: Arc<Mutex<UnboundedReceiver<BrowserEvent>>>,
}

impl ReceiverHandle {
    fn new(receiver: Arc<Mutex<UnboundedReceiver<BrowserEvent>>>) -> Self {
        Self { receiver }
    }
}

impl Hash for ReceiverHandle {
    fn hash<H: Hasher>(&self, state: &mut H) {
        "browser-core-events".hash(state);
    }
}

/// Creates a stream that receives browser events and converts them to UI events.
fn create_browser_event_stream(
    handle: &ReceiverHandle,
) -> Pin<Box<dyn Stream<Item = Event> + Send>> {
    let receiver = handle.receiver.clone();
    Box::pin(unfold(receiver, |receiver| async move {
        let event = {
            let mut lock = receiver.lock().await;
            lock.recv().await
        };

        event.map(|browser_event| (Event::Browser(browser_event), receiver))
    }))
}

use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use assets::ASSETS;
use assets::constants::{DEFAULT_FONT, MONOSPACE_FONT};
use browser_config::Config;
use browser_core::{Browser, BrowserCommand, BrowserEvent, Commandable, TabId};
use css_style::StyleTree;
use errors::browser::{BrowserError, NavigationError};
use iced::advanced::graphics::text::cosmic_text::FontSystem;
use iced::advanced::graphics::text::cosmic_text::fontdb::Source;
use iced::theme::{Custom, Palette};
use iced::{Color, Subscription};
use iced::{Renderer, Task, Theme, window};
use layout::{LayoutEngine, Rect, TextContext};
use tokio::sync::Mutex;
use tokio::sync::mpsc::UnboundedReceiver;
use tracing::info;

use crate::core::{ReceiverHandle, UiTab, WindowType, create_browser_event_stream};
use crate::events::UiEvent;
use crate::views::browser::window::BrowserWindow;
use crate::{manager::WindowController, views::devtools};

/// Represents the different types of events that can occur in the application.
#[derive(Debug, Clone)]
pub enum Event {
    None,
    Browser(BrowserEvent),
    Ui(UiEvent),
}

/// Represents the main application state, including the current window, tabs, and client.
pub struct Application {
    /// The application config.
    pub config: Config,

    /// The unique identifier for the application window.
    pub id: window::Id,

    /// The list of tabs currently open in the application.
    pub tabs: Vec<UiTab>,

    /// The identifier of the currently active tab.
    pub active_tab: TabId,

    /// The current URL displayed in the address bar.
    pub current_url: String,

    /// The viewport rectangle defining the visible area of the window.
    pub viewports: HashMap<window::Id, (f32, f32)>,

    /// The window controller managing multiple windows.
    window_controller: WindowController<Event, Theme, iced::Renderer>,

    /// The receiver for browser events.
    event_receiver: Arc<Mutex<UnboundedReceiver<BrowserEvent>>>,

    /// The shared browser instance.
    browser: Arc<Mutex<Browser>>,

    /// The shared text context for text rendering.
    text_context: TextContext,
}

impl Application {
    /// Creates a new instance of the `Application` with an initial window and a default tab.
    pub fn new(
        event_receiver: UnboundedReceiver<BrowserEvent>,
        browser: Arc<Mutex<Browser>>,
        config: Config,
    ) -> (Self, Task<Event>) {
        let default_font = ASSETS.read().unwrap().load_embedded(DEFAULT_FONT);
        let monospace_font = ASSETS.read().unwrap().load_embedded(MONOSPACE_FONT);

        let first_tab = UiTab::new(TabId(0));

        let mut window_controller = WindowController::new();
        let (main_window_id, browser_task) = window_controller.new_window(Box::new(BrowserWindow));

        let tasks = vec![browser_task.map(|_| Event::None)];

        let mut viewports = HashMap::new();

        let width = window_controller
            .get_window(main_window_id)
            .settings()
            .size
            .width;
        let height = window_controller
            .get_window(main_window_id)
            .settings()
            .size
            .height;

        viewports.insert(main_window_id, (width, height));

        let text_context = TextContext::new(FontSystem::new_with_fonts(vec![
            Source::Binary(Arc::new(default_font)),
            Source::Binary(Arc::new(monospace_font)),
        ]));

        let app = Application {
            id: main_window_id,
            tabs: vec![first_tab],
            active_tab: TabId(0),
            current_url: "http://127.0.0.1:5000/cookies/set-cookie".to_string(),
            window_controller,
            event_receiver: Arc::new(Mutex::new(event_receiver)),
            browser,
            viewports,
            text_context,
            config,
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
                UiEvent::WindowResized(window_id, width, height) => {
                    self.viewports.insert(window_id, (width, height));

                    if window_id == self.id
                        && let Some(tab) =
                            self.tabs.iter_mut().find(|tab| tab.id == self.active_tab)
                    {
                        let style_tree = StyleTree::build(&tab.document, &tab.stylesheets);

                        let layout_tree = LayoutEngine::compute_layout(
                            &style_tree,
                            Rect {
                                x: 0.0,
                                y: 0.0,
                                width,
                                height,
                            },
                            &mut self.text_context,
                        );

                        tab.layout_tree = layout_tree;
                    }
                }

                UiEvent::NewTab => {
                    let browser = self.browser.clone();

                    return Task::perform(
                        async move {
                            let mut lock = browser.lock().await;
                            lock.execute(BrowserCommand::AddTab).await
                        },
                        |result| match result {
                            Ok(task) => Event::Browser(task),
                            Err(_) => Event::None,
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
                            Err(_) => Event::None,
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
                            Err(_) => Event::None,
                        },
                    );
                }

                UiEvent::ChangeURL(url) => {
                    self.current_url = url;
                }
                UiEvent::ContentScrolled(x, y) => {
                    if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == self.active_tab) {
                        tab.scroll_offset.x = x;
                        tab.scroll_offset.y = y;
                    }
                }
            },

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
                            Err(err) => match err {
                                BrowserError::NavigationError(NavigationError::RequestError(
                                    err,
                                )) => Event::Browser(BrowserEvent::NavigateError(err)),

                                _ => Event::None,
                            },
                        },
                    );
                }
                BrowserEvent::NavigateSuccess(tab_id, page) => {
                    let current_tab = self.tabs.iter_mut().find(|tab| tab.id == tab_id);

                    if let Some(tab) = current_tab {
                        let style_tree = StyleTree::build(page.document(), page.stylesheets());

                        let layout_tree = LayoutEngine::compute_layout(
                            &style_tree,
                            self.viewports
                                .get(&self.id)
                                .map(|(w, h)| Rect {
                                    x: 0.0,
                                    y: 0.0,
                                    width: *w,
                                    height: *h,
                                })
                                .unwrap_or(Rect {
                                    x: 0.0,
                                    y: 0.0,
                                    width: 800.0,
                                    height: 600.0,
                                }),
                            &mut self.text_context,
                        );

                        tab.document = page.document().clone();
                        tab.stylesheets = page.stylesheets().clone();
                        tab.current_url = Some(self.current_url.parse().unwrap());
                        tab.layout_tree = layout_tree;
                        tab.title = Some(page.title().to_string());
                    }
                }
                BrowserEvent::NavigateError(err) => {
                    info!("{}", err);
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
            window::resize_events().map(|(window_id, size)| {
                Event::Ui(UiEvent::WindowResized(window_id, size.width, size.height))
            }),
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
        let app_theme = self.config.theme();

        let palette = Palette {
            background: Color::from_str(app_theme.background.as_str()).unwrap_or(Color::WHITE),
            text: Color::from_str(app_theme.text.as_str()).unwrap_or(Color::from_rgb8(10, 10, 10)),
            primary: Color::from_str(app_theme.primary.as_str())
                .unwrap_or(Color::from_rgb8(0, 187, 249)),
            success: Color::from_str(app_theme.success.as_str())
                .unwrap_or(Color::from_rgb8(144, 190, 109)),
            warning: Color::from_str(app_theme.warning.as_str())
                .unwrap_or(Color::from_rgb8(248, 150, 30)),
            danger: Color::from_str(app_theme.danger.as_str())
                .unwrap_or(Color::from_rgb8(249, 65, 68)),
        };

        let custom = Custom::new(String::from("Settings"), palette);

        Theme::Custom(Arc::new(custom))
    }
}

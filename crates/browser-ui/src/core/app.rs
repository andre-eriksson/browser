use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use browser_config::BrowserConfig;
use browser_core::Browser;
use iced::keyboard::key;
use iced::theme::{Custom, Palette};
use iced::widget::text;
use iced::window::Id;
use iced::{Color, Subscription, event, keyboard};
use iced::{Renderer, Task, Theme, window};
use renderer::image::ImageCache;
use tokio::sync::Mutex;

use crate::core::WindowType;
use crate::events::kernel::EngineRequest;
use crate::events::window::WindowEvent;
use crate::events::{Event, EventHandler};
use crate::manager::WindowController;
use crate::views::browser::window::BrowserContext;

/// Represents the main application state, including the current window, tabs, and client.
pub struct Application {
    /// The application config.
    pub config: &'static BrowserConfig,

    /// The browser contexts for each open window, keyed by window ID.
    pub browser_windows: HashMap<Id, BrowserContext>,

    /// The window controller managing multiple windows.
    pub window_controller: WindowController,

    /// The shared browser instance.
    pub browser: Arc<Mutex<Browser>>,

    /// Shared image cache for loaded images.
    pub image_cache: Option<ImageCache>,
}

impl Application {
    /// Creates a new instance of the `Application` with an initial window and a default tab.
    pub fn new(browser: Arc<Mutex<Browser>>, config: &'static BrowserConfig) -> (Self, Task<Event>) {
        let mut window_controller = WindowController::new();
        let (main_window_id, browser_task) = window_controller.new_window(None, WindowType::Browser);

        let tasks = vec![browser_task.discard()];

        let app = Application {
            browser_windows: HashMap::from([(main_window_id, BrowserContext::new(config.args()))]),
            window_controller,
            browser,
            config,
            image_cache: None,
        };

        (app, Task::batch(tasks))
    }

    /// Updates the application state based on the received message.
    ///
    /// # Arguments
    /// * `message` - The message containing the action to perform.
    pub fn update(&mut self, event: Event) -> Task<Event> {
        match event {
            Event::Window(window_event) => self.handle(window_event),
            Event::EngineResponse(window_id, engine_response) => self.handle((window_id, engine_response)),
            Event::EngineRequest(engine_request) => self.handle(engine_request),
            Event::Browser(browser_event) => self.handle(browser_event),
            Event::Devtools(devtools_event) => self.handle(devtools_event),
        }
    }

    /// Returns the current subscriptions for the application.
    ///
    /// Global subscriptions (e.g. close events) are declared here. Per-window
    /// subscriptions (e.g. resize events scoped to a specific window type) are
    /// collected from each open window via [`WindowController::subscriptions`].
    pub fn subscriptions(&self) -> iced::Subscription<Event> {
        Subscription::batch([
            window::close_events().map(|window_id| Event::Window(WindowEvent::CloseWindow(window_id))),
            event::listen_with(|event, _status, window_id| match event {
                iced::Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(key::Named::F5),
                    ..
                }) => Some(Event::EngineRequest(EngineRequest::Refresh(window_id))),
                iced::Event::Keyboard(keyboard::Event::KeyPressed {
                    physical_key: key::Physical::Code(key::Code::KeyR),
                    modifiers,
                    ..
                }) if modifiers.control() => Some(Event::EngineRequest(EngineRequest::Refresh(window_id))),
                _ => None,
            }),
            self.window_controller.subscriptions(),
        ])
    }

    /// Renders the application UI for a specific window.
    pub fn view(&self, window_id: window::Id) -> iced::Element<'_, Event, Theme, Renderer> {
        self.window_controller
            .render(self, window_id)
            .unwrap_or_else(|| text("Window not found").into())
    }

    /// Returns the title of the application window.
    pub fn title(&self, window_id: window::Id) -> String {
        self.window_controller
            .title(window_id)
            .unwrap_or_else(|| "Browser".to_string())
    }

    /// Returns the theme for the application window.
    pub fn theme(&self, _window_id: window::Id) -> Theme {
        let app_theme = self.config.preferences().active_theme();

        let palette = Palette {
            background: Color::from_str(app_theme.background.as_str()).unwrap_or(Color::WHITE),
            text: Color::from_str(app_theme.text.as_str())
                .unwrap_or(Color::from_str(&browser_preferences::Theme::default().text).unwrap()),
            primary: Color::from_str(app_theme.primary.as_str())
                .unwrap_or(Color::from_str(&browser_preferences::Theme::default().primary).unwrap()),
            success: Color::from_str(app_theme.success.as_str())
                .unwrap_or(Color::from_str(&browser_preferences::Theme::default().success).unwrap()),
            warning: Color::from_str(app_theme.warning.as_str())
                .unwrap_or(Color::from_str(&browser_preferences::Theme::default().warning).unwrap()),
            danger: Color::from_str(app_theme.danger.as_str())
                .unwrap_or(Color::from_str(&browser_preferences::Theme::default().danger).unwrap()),
        };

        let custom = Custom::new(String::from("Settings"), palette);

        Theme::Custom(Arc::new(custom))
    }
}

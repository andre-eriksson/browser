use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, Mutex as StdMutex};

use iced::advanced::graphics::text::cosmic_text::FontSystem;
use iced::theme::{Custom, Palette};
use iced::{Color, Subscription};
use iced::{Renderer, Task, Theme, window};
use kernel::{Browser, BrowserEvent, TabId};
use layout::TextContext;
use preferences::BrowserConfig;
use renderer::image::ImageCache;
use tokio::sync::Mutex;
use tokio::sync::mpsc::UnboundedReceiver;

use crate::core::{ReceiverHandle, UiTab, create_browser_event_stream};
use crate::events::{Event, EventHandler, UiEvent};
use crate::manager::WindowController;
use crate::util::fonts::load_fallback_fonts;
use crate::views::browser::window::BrowserWindow;

/// Represents the main application state, including the current window, tabs, and client.
pub struct Application {
    /// The application config.
    pub config: BrowserConfig,

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
    pub window_controller: WindowController<Event, Theme, iced::Renderer>,

    /// The receiver for browser events.
    pub event_receiver: Arc<Mutex<UnboundedReceiver<BrowserEvent>>>,

    /// The shared browser instance.
    pub browser: Arc<Mutex<Browser>>,

    /// The shared text context for text rendering.
    pub text_context: Arc<StdMutex<TextContext>>,

    /// Shared image cache for loaded images.
    pub image_cache: Option<ImageCache>,
}

impl Application {
    /// Creates a new instance of the `Application` with an initial window and a default tab.
    pub fn new(
        event_receiver: UnboundedReceiver<BrowserEvent>,
        browser: Arc<Mutex<Browser>>,
        config: BrowserConfig,
        initial_url: Option<String>,
    ) -> (Self, Task<Event>) {
        let first_tab = UiTab::new(TabId(0));

        let mut window_controller = WindowController::new();
        let (main_window_id, browser_task) = window_controller.new_window(Box::new(BrowserWindow));

        let tasks = vec![browser_task.discard()];

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

        let font_system = FontSystem::new_with_fonts(load_fallback_fonts());

        let text_context = Arc::new(StdMutex::new(TextContext::new(font_system)));

        let app = Application {
            id: main_window_id,
            tabs: vec![first_tab],
            active_tab: TabId(0),
            current_url: initial_url.unwrap_or("https://www.google.com".to_string()),
            window_controller,
            event_receiver: Arc::new(Mutex::new(event_receiver)),
            browser,
            viewports,
            text_context,
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
            Event::Ui(ui_event) => self.handle(ui_event),
            Event::Browser(browser_event) => self.handle(browser_event),
        }
    }

    /// Returns the current subscriptions for the application.
    pub fn subscriptions(&self) -> iced::Subscription<Event> {
        let receiver = self.event_receiver.clone();

        Subscription::batch([
            window::close_events().map(|window_id| Event::Ui(UiEvent::CloseWindow(window_id))),
            window::resize_events()
                .map(|(window_id, size)| Event::Ui(UiEvent::WindowResized(window_id, size.width, size.height))),
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
            text: Color::from_str(app_theme.text.as_str())
                .unwrap_or(Color::from_str(&preferences::Theme::default().text).unwrap()),
            primary: Color::from_str(app_theme.primary.as_str())
                .unwrap_or(Color::from_str(&preferences::Theme::default().primary).unwrap()),
            success: Color::from_str(app_theme.success.as_str())
                .unwrap_or(Color::from_str(&preferences::Theme::default().success).unwrap()),
            warning: Color::from_str(app_theme.warning.as_str())
                .unwrap_or(Color::from_str(&preferences::Theme::default().warning).unwrap()),
            danger: Color::from_str(app_theme.danger.as_str())
                .unwrap_or(Color::from_str(&preferences::Theme::default().danger).unwrap()),
        };

        let custom = Custom::new(String::from("Settings"), palette);

        Theme::Custom(Arc::new(custom))
    }
}

use std::{borrow::Cow, sync::Arc};

use assets::{
    ASSETS,
    constants::{DEFAULT_FONT, MONOSPACE_FONT},
};
use browser_core::{Browser, BrowserEvent};
use errors::subsystem::SubsystemError;
use iced::{Font, Settings};
use tokio::sync::{Mutex, mpsc::UnboundedReceiver};

use crate::core::Application;

/// The main runtime for the UI, responsible for initializing and running the application.
pub struct Ui {
    browser: Arc<Mutex<Browser>>,
    event_receiver: UnboundedReceiver<BrowserEvent>,
}

impl Ui {
    /// Creates a new instance of the `UiRuntime`.
    pub fn new(
        browser: Arc<Mutex<Browser>>,
        event_receiver: UnboundedReceiver<BrowserEvent>,
    ) -> Self {
        Ui {
            browser,
            event_receiver,
        }
    }

    /// Runs the UI runtime, initializing the application and starting the event loop.
    pub fn run(self) -> Result<(), SubsystemError> {
        let default_font = ASSETS.read().unwrap().load_embedded(DEFAULT_FONT);
        let monospace_font = ASSETS.read().unwrap().load_embedded(MONOSPACE_FONT);
        let browser = self.browser;
        let event_receiver = Arc::new(std::sync::Mutex::new(Some(self.event_receiver)));

        let result = iced::daemon(
            move || {
                let receiver = event_receiver
                    .lock()
                    .unwrap()
                    .take()
                    .expect("Boot function called more than once");
                Application::new(receiver, browser.clone())
            },
            Application::update,
            Application::view,
        )
        .subscription(Application::subscriptions)
        .settings(Settings {
            fonts: vec![Cow::Owned(default_font), Cow::Owned(monospace_font)],
            default_font: Font::with_name("Open Sans"),
            ..Default::default()
        })
        .theme(Application::theme)
        .title(Application::title)
        .run();

        match result {
            Ok(_) => Ok(()),
            Err(e) => match e {
                iced::Error::ExecutorCreationFailed(msg) => Err(SubsystemError::RuntimeError(
                    format!("UI Executor Creation Failed: {}", msg),
                )),
                iced::Error::GraphicsCreationFailed(msg) => Err(SubsystemError::RuntimeError(
                    format!("UI Graphics Creation Failed: {}", msg),
                )),
                iced::Error::WindowCreationFailed(msg) => Err(SubsystemError::RuntimeError(
                    format!("UI Window Creation Failed: {}", msg),
                )),
            },
        }
    }
}

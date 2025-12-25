use std::{borrow::Cow, sync::Arc};

use assets::{
    ASSETS,
    constants::{DEFAULT_FONT, MONOSPACE_FONT},
};
use browser_core::{browser::Browser, events::BrowserEvent};
use errors::subsystem::SubsystemError;
use iced::{Font, Settings};
use tokio::sync::{Mutex, mpsc::UnboundedReceiver};

use crate::core::app::Application;

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
        let event_receiver = self.event_receiver;
        let browser = self.browser;

        let result = iced::daemon(Application::title, Application::update, Application::view)
            .settings(Settings {
                fonts: vec![Cow::Owned(default_font), Cow::Owned(monospace_font)],
                default_font: Font::with_name("Open Sans"),
                ..Default::default()
            })
            .subscription(Application::subscriptions)
            .run_with(move || Application::new(event_receiver, browser.clone()));

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

use std::{borrow::Cow, sync::Arc};

use cli::args::BrowserArgs;
use iced::{Font, Settings};
use io::{
    Resource,
    embeded::{OPEN_SANS_REGULAR, ROBOTO_MONO_REGULAR},
};
use kernel::{Browser, BrowserEvent};
use preferences::BrowserConfig;
use tokio::sync::{Mutex, mpsc::UnboundedReceiver};

use crate::{core::Application, errors::UiError};

/// The main runtime for the UI, responsible for initializing and running the application.
pub struct Ui {
    browser: Arc<Mutex<Browser>>,
    event_receiver: UnboundedReceiver<BrowserEvent>,
    args: BrowserArgs,
    config: BrowserConfig,
}

impl Ui {
    /// Creates a new instance of the `UiRuntime`.
    pub fn new(
        browser: Arc<Mutex<Browser>>,
        event_receiver: UnboundedReceiver<BrowserEvent>,
        args: BrowserArgs,
        config: BrowserConfig,
    ) -> Self {
        Ui {
            browser,
            event_receiver,
            args,
            config,
        }
    }

    /// Runs the UI runtime, initializing the application and starting the event loop.
    pub fn run(self) -> Result<(), UiError> {
        let default_font = Resource::load_embedded(OPEN_SANS_REGULAR);
        let monospace_font = Resource::load_embedded(ROBOTO_MONO_REGULAR);
        let browser = self.browser;
        let config = self.config;
        let initial_url = self.args.url;
        let event_receiver = Arc::new(std::sync::Mutex::new(Some(self.event_receiver)));

        let result = iced::daemon(
            move || {
                let receiver = event_receiver
                    .lock()
                    .unwrap()
                    .take()
                    .expect("Boot function called more than once");
                Application::new(receiver, browser.clone(), config.clone(), initial_url.clone())
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
                iced::Error::ExecutorCreationFailed(msg) => {
                    Err(UiError::RuntimeError(format!("Executor Creation Failed: {}", msg)))
                }
                iced::Error::GraphicsCreationFailed(msg) => {
                    Err(UiError::RuntimeError(format!("Graphics Creation Failed: {}", msg)))
                }
                iced::Error::WindowCreationFailed(msg) => {
                    Err(UiError::RuntimeError(format!("Window Creation Failed: {}", msg)))
                }
            },
        }
    }
}

use std::{
    borrow::Cow,
    sync::{Arc, Mutex},
};

use assets::{
    ASSETS,
    constants::{DEFAULT_FONT, MONOSPACE_FONT},
};
use cookies::cookie_store::CookieJar;
use errors::subsystem::SubsystemError;
use http::HeaderMap;
use iced::{Font, Settings};

use crate::core::app::Application;

/// The main runtime for the UI, responsible for initializing and running the application.
pub struct UiRuntime {
    browser_headers: Arc<HeaderMap>,
    cookie_jar: Arc<Mutex<CookieJar>>,
}

impl UiRuntime {
    /// Creates a new instance of the `UiRuntime`.
    pub fn new(browser_headers: Arc<HeaderMap>, cookie_jar: Arc<Mutex<CookieJar>>) -> Self {
        UiRuntime {
            browser_headers,
            cookie_jar,
        }
    }

    /// Runs the UI runtime, initializing the application and starting the event loop.
    pub fn run(self) -> Result<(), SubsystemError> {
        let default_font = ASSETS.read().unwrap().load_embedded(DEFAULT_FONT);
        let monospace_font = ASSETS.read().unwrap().load_embedded(MONOSPACE_FONT);
        let browser_headers = self.browser_headers;
        let cookie_jar = self.cookie_jar;

        let result = iced::daemon(Application::title, Application::update, Application::view)
            .settings(Settings {
                fonts: vec![Cow::Owned(default_font), Cow::Owned(monospace_font)],
                default_font: Font::with_name("Open Sans"),
                ..Default::default()
            })
            .subscription(Application::subscriptions)
            .run_with(|| Application::new(browser_headers, cookie_jar));

        return match result {
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
        };
    }
}

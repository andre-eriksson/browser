use std::{borrow::Cow, sync::Arc};

use browser_config::BrowserConfig;
use browser_core::Browser;
use iced::{Font, Pixels, Settings};
use io::{
    Resource,
    embeded::{OPEN_SANS_REGULAR, ROBOTO_MONO_REGULAR},
};
use tokio::sync::Mutex;

use crate::{core::Application, errors::UiError};

pub struct Ui;

impl Ui {
    /// Runs the UI runtime, initializing the application and starting the event loop.
    ///
    /// # Errors
    /// If the application fails to run, a `UiError::Runtime` is returned with the underlying error.
    pub fn run(browser: Arc<Mutex<Browser>>, config: &'static BrowserConfig) -> Result<(), UiError> {
        let default_font = Resource::load_embedded(OPEN_SANS_REGULAR);
        let monospace_font = Resource::load_embedded(ROBOTO_MONO_REGULAR);
        let theme = config.preferences().theme();

        let result =
            iced::daemon(move || Application::new(browser.clone(), config), Application::update, Application::view)
                .subscription(Application::subscriptions)
                .settings(Settings {
                    fonts: vec![Cow::Owned(default_font), Cow::Owned(monospace_font)],
                    default_font: Font::with_name(&theme.typography.ui.name),
                    default_text_size: Pixels(theme.typography.ui.size),
                    ..Default::default()
                })
                .theme(Application::theme)
                .title(Application::title)
                .run();

        match result {
            Ok(()) => Ok(()),
            Err(error) => Err(UiError::Runtime(error)),
        }
    }
}

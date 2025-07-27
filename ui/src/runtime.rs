use std::borrow::Cow;

use assets::{
    ASSETS,
    constants::{DEFAULT_FONT, MONOSPACE_FONT},
};
use iced::{Font, Settings};
use tracing::{error, info};

use crate::core::app::Application;

/// The main runtime for the UI, responsible for initializing and running the application.
pub struct UiRuntime;

impl UiRuntime {
    /// Runs the UI runtime, initializing the application and starting the event loop.
    pub fn run() {
        let default_font = ASSETS.lock().unwrap().get(DEFAULT_FONT);
        let monospace_font = ASSETS.lock().unwrap().get(MONOSPACE_FONT);
        let result = iced::daemon(Application::title, Application::update, Application::view)
            .settings(Settings {
                fonts: vec![Cow::Owned(default_font), Cow::Owned(monospace_font)],
                default_font: Font::with_name("Open Sans"),
                ..Default::default()
            })
            .subscription(Application::subscriptions)
            .run_with(Application::new);

        if let Err(e) = result {
            error!("Error running the application: {}", e);
        }
        info!("Application has exited successfully.");
    }
}

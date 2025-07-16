use tracing::{error, info};

use crate::core::app::Application;

/// The main runtime for the UI, responsible for initializing and running the application.
pub struct UiRuntime;

impl UiRuntime {
    /// Runs the UI runtime, initializing the application and starting the event loop.
    pub fn run() {
        let result = iced::daemon(Application::title, Application::update, Application::view)
            .subscription(Application::subscriptions)
            .run_with(Application::new);

        if let Err(e) = result {
            error!("Error running the application: {}", e);
        }
        info!("Application has exited successfully.");
    }
}

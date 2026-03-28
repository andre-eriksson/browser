use iced::{Task, window::Id};

use crate::{
    core::Application,
    events::{
        Event, EventHandler,
        devtools::window::{on_resized, on_scrolled},
    },
};

mod window;

/// Represents the different types of Devtool-related events that can occur in the application.
///
/// These events are specific to the Devtools window and are handled by the main application logic when they are triggered from the Devtools UI.
#[derive(Debug, Clone)]
pub enum DevtoolEvent {
    /// Handle devtools scroll event with new scroll offset.
    Scroll(f32, f32),

    /// Handle browser resize event with new width and height.
    Resize(Id, f32, f32),
}

impl EventHandler<DevtoolEvent> for Application {
    fn handle(&mut self, event: DevtoolEvent) -> Task<Event> {
        match event {
            DevtoolEvent::Scroll(x, y) => on_scrolled(self, x, y),
            DevtoolEvent::Resize(window_id, width, height) => on_resized(self, window_id, width, height),
        }
    }
}

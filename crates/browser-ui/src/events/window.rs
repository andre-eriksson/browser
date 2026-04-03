use iced::{Task, window::Id};

use crate::{
    core::{Application, WindowType},
    events::{
        Event, EventHandler,
        window::events::{close_window, create_window},
    },
};

/// Represents the different types of Window-related events that can occur in the application.
///
/// These are global events that can be triggered from any window and are handled by the main application logic.
#[derive(Debug, Clone)]
pub enum WindowEvent {
    /// Create a new window of the specified type.
    NewWindow(Id, WindowType),

    /// Close the window with the specified ID.
    CloseWindow(Id),
}

mod events;

impl EventHandler<WindowEvent> for Application {
    fn handle(&mut self, event: WindowEvent) -> Task<Event> {
        match event {
            WindowEvent::NewWindow(window_id, window_type) => create_window(self, window_id, window_type),
            WindowEvent::CloseWindow(window_id) => close_window(self, window_id),
        }
    }
}

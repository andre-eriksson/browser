use iced::{Task, window::Id};

use crate::{
    core::{Application, WindowType},
    events::{
        Event, EventHandler,
        window::events::{close_window, create_window, on_window_resized},
    },
};

/// Represents the different types of Window-related events that can occur in the application.
///
/// These are global events that can be triggered from any window and are handled by the main application logic.
#[derive(Debug, Clone)]
pub enum WindowEvent {
    /// Create a new window of the specified type.
    NewWindow(WindowType),

    /// Close the window with the specified ID.
    CloseWindow(Id),

    /// Handle window resize event with new width and height.
    WindowResized(Id, f32, f32),
}

mod events;

impl EventHandler<WindowEvent> for Application {
    fn handle(&mut self, event: WindowEvent) -> Task<Event> {
        match event {
            WindowEvent::NewWindow(window_type) => create_window(self, window_type),
            WindowEvent::CloseWindow(window_id) => close_window(self, window_id),
            WindowEvent::WindowResized(window_id, width, height) => on_window_resized(self, window_id, width, height),
        }
    }
}

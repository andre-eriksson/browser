use ::browser_core::EngineResponse;
use iced::Task;

use crate::events::{browser::BrowserEvent, devtools::DevtoolEvent, kernel::EngineRequest, window::WindowEvent};

pub mod browser;
pub mod devtools;
pub mod kernel;
pub mod window;

/// Represents the different types of events that can occur in the application.
#[derive(Debug, Clone)]
pub enum Event {
    EngineResponse(EngineResponse),
    EngineRequest(EngineRequest),
    Window(WindowEvent),
    Browser(BrowserEvent),
    Devtools(DevtoolEvent),
}

/// A trait for handling events of a specific type. Implementors of this trait can define how to
/// handle events and return a Task that may produce new events as a result.
pub trait EventHandler<E> {
    /// Handle an event of type E and return a Task that may produce new events as a result.
    fn handle(&mut self, event: E) -> Task<Event>;
}

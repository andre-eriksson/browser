use iced::{Renderer, Subscription, Theme, window};
use window::Id;

use crate::{core::Application, events::Event};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WindowType {
    /// Represents the main browser window, which displays the web content and user interface.
    Browser,

    /// Represents a developer tools window, which provides debugging and inspection capabilities for web developers.
    Devtools,
}

/// A trait that defines the interface for a window in the application.
pub trait ApplicationWindow {
    /// Constructs the window with the given window Id.
    fn new(parent_id: Option<Id>, id: Id) -> Self
    where
        Self: Sized;

    /// Renders the window's content for the current application state.
    fn render<'window>(&'window self, app: &'window Application) -> iced::Element<'window, Event, Theme, Renderer>;

    /// Returns the settings of the window, such as its initial size, resizability, etc.
    fn settings() -> window::Settings
    where
        Self: Sized;

    /// Returns the window title.
    fn title(&self) -> String;

    /// Returns the unique identifier of the parent window, if this window is a child of another window.
    fn parent_id(&self) -> Option<Id>;

    /// Returns subscriptions scoped to this window.
    fn subscription(&self) -> Subscription<Event> {
        Subscription::none()
    }
}

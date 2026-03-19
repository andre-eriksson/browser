use iced::{Renderer, Subscription, Theme, window};
use window::Id;

use crate::events::Event;

#[derive(Debug, Clone)]
pub enum WindowType {
    /// Represents the main browser window, which displays the web content and user interface.
    Browser,

    /// Represents a developer tools window, which provides debugging and inspection capabilities for web developers.
    Devtools,
}

/// A trait that defines the interface for a window in the application.
///
/// All windows in the browser use iced's concrete [`Theme`] and [`Renderer`] — there
/// is no benefit to keeping those generic since every impl will be `iced::Theme` /
/// `iced::Renderer` and the generics were the root cause of trait-object casting
/// failures in [`WindowController`].
///
/// [`WindowController`]: crate::manager::WindowController
pub trait ApplicationWindow<App> {
    /// Constructs the window with the OS-assigned [`Id`].
    ///
    /// Called by [`WindowController::new_window`] immediately after [`window::open`]
    /// returns, so the window always knows its own ID from the moment it exists.
    fn new(id: Id) -> Self
    where
        Self: Sized;

    /// Renders the window's content for the current application state.
    fn render<'window>(&'window self, app: &'window App) -> iced::Element<'window, Event, Theme, Renderer>;

    /// Returns the iced [`window::Settings`] used when opening this window.
    ///
    /// This is a static method because it must be called *before* the instance
    /// is constructed (we need the settings to call [`window::open`]).
    fn settings() -> window::Settings
    where
        Self: Sized;

    /// Returns the window title.
    fn title(&self) -> String;

    /// Returns the OS-assigned [`Id`] for this window.
    fn id(&self) -> Id;

    /// Returns subscriptions scoped to this window.
    ///
    /// The window's own ID is available via [`Self::id`], so implementations
    /// can filter events (e.g. resize) to only those targeting this window.
    /// The default implementation produces no subscriptions.
    fn subscription(&self) -> Subscription<Event> {
        Subscription::none()
    }
}

use iced::window;

#[derive(Debug, Clone)]
pub enum WindowType {
    /// Represents a target for a new browser window.
    ///
    /// NOTE: Not used yet, but can be implemented in the future.
    // Browser,

    /// Represents a target for a new Devtools window.
    ///
    /// This is used for debugging and inspecting the app and the HTML content.
    Devtools,
}

/// A trait that defines the interface for a window in the application.
///
/// # Generic Parameters
/// * `Application` - The type representing the application.
/// * `Message` - The type of messages that the window can send.
/// * `Theme` - The theme type used for styling the window.
/// * `Renderer` - The renderer type used for drawing the window's content.
pub trait ApplicationWindow<Application, Message, Theme, Renderer> {
    /// The entrypoint for rendering the window's content.
    ///
    /// # Arguments
    /// * `app` - The application state that the window can access
    ///
    /// # Returns
    /// * `Element<'window, Message, Theme, Renderer>` - The rendered content of the window
    fn render<'window>(
        &'window self,
        app: &'window Application,
    ) -> iced::Element<'window, Message, Theme, Renderer>;

    /// Returns the settings for the window.
    fn settings(&self) -> window::Settings;
    fn title(&self) -> String;
}

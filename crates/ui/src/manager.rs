use std::collections::HashMap;

use iced::{
    Task,
    window::{self, Id},
};

use crate::core::{Application, ApplicationWindow};

/// WindowController manages multiple application windows, allowing for rendering and interaction
///
/// # Fields
/// * `open_windows` - A map of currently open windows, keyed by their unique ID
pub struct WindowController<Message, Theme, Renderer> {
    pub open_windows:
        HashMap<window::Id, Box<dyn ApplicationWindow<Application, Message, Theme, Renderer>>>,
}

impl<Message, Theme, Renderer> WindowController<Message, Theme, Renderer> {
    pub fn new() -> Self {
        Self {
            open_windows: HashMap::new(),
        }
    }

    /// Retrieves a reference to the window with the specified ID.
    ///
    /// # Arguments
    /// * `id` - The ID of the window to retrieve
    pub fn get_window(
        &self,
        id: Id,
    ) -> &dyn ApplicationWindow<Application, Message, Theme, Renderer> {
        self.open_windows
            .get(&id)
            .expect("Window not found")
            .as_ref()
    }

    /// Renders the content of the window with the specified ID.
    ///
    /// # Arguments
    /// * `app` - The application instance to use for rendering
    /// * `id` - The ID of the window to render
    pub fn render<'window>(
        &'window self,
        app: &'window Application,
        id: Id,
    ) -> iced::Element<'window, Message, Theme, Renderer> {
        self.get_window(id).render(app)
    }

    /// Returns the title of the window with the specified ID.
    ///
    /// # Arguments
    /// * `id` - The ID of the window whose title is requested
    pub fn title(&self, id: Id) -> String {
        self.get_window(id).title()
    }

    /// Opens a new window with the specified settings and returns its ID and a task to track its state.
    ///
    /// # Arguments
    /// * `window` - A boxed window that implements the ApplicationWindow trait
    pub fn new_window(
        &mut self,
        window: Box<dyn ApplicationWindow<Application, Message, Theme, Renderer>>,
    ) -> (Id, Task<Id>) {
        let (id, task) = window::open(window.settings());
        self.open_windows.insert(id, window);

        (id, task)
    }

    /// Closes all open windows and returns a task that completes when all windows are closed.
    pub fn close_all_windows(&mut self) -> Task<()> {
        let tasks: Vec<_> = self
            .open_windows
            .keys()
            .map(|id| window::close(*id))
            .collect();

        Task::batch(tasks)
    }

    /// Closes a specific window by its ID.
    ///
    /// # Arguments
    /// * `id` - The ID of the window to close
    pub fn close(&mut self, id: Id) {
        self.open_windows.remove(&id);
    }
}

use std::collections::HashMap;

use iced::{
    Renderer, Subscription, Task, Theme,
    window::{self, Id},
};

use crate::{
    core::{Application, ApplicationWindow, WindowType},
    events::Event,
    views::{browser::window::BrowserWindow, devtools::window::DevtoolsWindow},
};

/// `WindowController` manages multiple application windows, allowing for rendering and interaction.
///
/// # Fields
/// * `open_windows` - A map of currently open windows, keyed by their unique ID.
pub struct WindowController {
    pub open_windows: HashMap<Id, Box<dyn ApplicationWindow>>,
}

impl WindowController {
    pub fn new() -> Self {
        Self {
            open_windows: HashMap::new(),
        }
    }

    /// Retrieves a reference to the window with the specified ID.
    ///
    /// # Arguments
    /// * `id` - The ID of the window to retrieve.
    pub fn get_window(&self, id: Id) -> Option<&dyn ApplicationWindow> {
        self.open_windows.get(&id).map(AsRef::as_ref)
    }

    /// Renders the content of the window with the specified ID.
    ///
    /// # Arguments
    /// * `app` - The application instance to use for rendering.
    /// * `id` - The ID of the window to render.
    pub fn render<'window>(
        &'window self,
        app: &'window Application,
        id: Id,
    ) -> Option<iced::Element<'window, Event, Theme, Renderer>> {
        self.get_window(id).map(|window| window.render(app))
    }

    /// Returns the title of the window with the specified ID.
    ///
    /// # Arguments
    /// * `id` - The ID of the window whose title is requested.
    pub fn title(&self, id: Id) -> Option<String> {
        self.get_window(id).map(ApplicationWindow::title)
    }

    /// Opens a new window of the given type, constructs its instance with the
    /// OS-assigned ID, and registers it with the controller.
    ///
    /// # Arguments
    /// * `window_type` - The type of window to open.
    pub fn new_window(&mut self, parent_id: Option<Id>, window_type: WindowType) -> (Id, Task<Id>) {
        let (id, task, window): (Id, Task<Id>, Box<dyn ApplicationWindow>) = match window_type {
            WindowType::Browser => {
                let (id, task) = window::open(BrowserWindow::settings());
                (id, task, Box::new(BrowserWindow::new(parent_id, id)))
            }
            WindowType::Devtools => {
                let (id, task) = window::open(DevtoolsWindow::settings());
                (id, task, Box::new(DevtoolsWindow::new(parent_id, id)))
            }
        };

        self.open_windows.insert(id, window);

        (id, task)
    }

    /// Closes all open windows and returns a task that completes when all windows are closed.
    pub fn close_all_windows(&self) -> Task<()> {
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
    /// * `id` - The ID of the window to close.
    pub fn close(&mut self, id: Id) {
        self.open_windows.remove(&id);
    }

    /// Returns a merged subscription from all open windows.
    ///
    /// Each window can declare its own subscriptions (e.g. resize events scoped
    /// to that window type). This collects them all into a single
    /// `Subscription::batch` so the caller only needs one call site.
    pub fn subscriptions(&self) -> Subscription<Event> {
        Subscription::batch(
            self.open_windows
                .values()
                .map(|window| window.subscription()),
        )
    }
}

use std::collections::HashMap;

use iced::{
    Task,
    window::{self, Id},
};

use crate::{api::window::ApplicationWindow, core::app::Application};

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

    pub fn get_window(
        &self,
        id: Id,
    ) -> &dyn ApplicationWindow<Application, Message, Theme, Renderer> {
        self.open_windows
            .get(&id)
            .expect("Window not found")
            .as_ref()
    }

    pub fn render<'window>(
        &'window self,
        app: &'window Application,
        id: Id,
    ) -> iced::Element<'window, Message, Theme, Renderer> {
        self.get_window(id).render(app)
    }

    pub fn title(&self, id: Id) -> String {
        self.get_window(id).title()
    }

    pub fn new_window(
        &mut self,
        window: Box<dyn ApplicationWindow<Application, Message, Theme, Renderer>>,
    ) -> (Id, Task<Id>) {
        let (id, task) = window::open(window.settings());
        self.open_windows.insert(id, window);

        (id, task)
    }

    pub fn close_all_windows(&mut self) -> Task<()> {
        let tasks: Vec<_> = self
            .open_windows
            .keys()
            .map(|id| window::close(*id))
            .collect();

        Task::batch(tasks)
    }

    pub fn close(&mut self, id: Id) {
        self.open_windows.remove(&id);
    }
}

use iced::{
    Length, Renderer, Size, Theme,
    widget::{column, container, text},
    window::{Position, Settings},
};

use crate::{
    api::{message::Message, window::ApplicationWindow}, core::app::Application,
};

#[derive(Debug, Default)]
pub struct DevtoolsWindow;

impl ApplicationWindow<Application, Message, Theme, Renderer> for DevtoolsWindow {
    fn render<'window>(
        &'window self,
        _app: &'window Application,
    ) -> iced::Element<'window, Message, Theme, Renderer> {
        let ui = container(column![text("Devtools!")].spacing(10.0))
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(10.0);

        ui.into()
    }

    fn settings(&self) -> iced::window::Settings {
        Settings {
            size: Size::new(800.0, 800.0),
            position: Position::Centered,
            ..Default::default()
        }
    }

    fn title(&self) -> String {
        "Devtools".to_string()
    }
}

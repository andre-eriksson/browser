use iced::{
    Background, Color, Length, Renderer, Size, Theme,
    widget::{
        column, container,
        scrollable::{self, Direction, Scrollbar},
        text,
    },
    window::{Position, Settings},
};

use crate::{
    api::{message::Message, window::ApplicationWindow},
    core::app::Application,
    views::devtools::components::tree::render_dom_tree,
};

/// DevtoolsWindow is a window for displaying developer tools in the application.
#[derive(Debug, Default)]
pub struct DevtoolsWindow;

impl ApplicationWindow<Application, Message, Theme, Renderer> for DevtoolsWindow {
    fn render<'window>(
        &'window self,
        app: &'window Application,
    ) -> iced::Element<'window, Message, Theme, Renderer> {
        let dom_tree = match render_dom_tree(app) {
            Ok(content) => content,
            Err(e) => container(text(format!("Error rendering content: {}", e)))
                .width(Length::Fill)
                .padding(10.0)
                .style(|_| container::Style {
                    background: Some(Background::Color(Color::from_rgb(0.95, 0.95, 0.95))),
                    text_color: Some(Color::BLACK),
                    ..Default::default()
                }),
        };

        let ui = container(
            column![
                scrollable::Scrollable::new(dom_tree)
                    .direction(Direction::Vertical(Scrollbar::new()))
                    .height(Length::Fill),
            ]
            .spacing(10.0),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(10.0)
        .style(|_| container::Style {
            background: Some(Background::Color(Color::from_rgb(0.95, 0.95, 0.95))),
            text_color: Some(Color::BLACK),
            ..Default::default()
        });

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

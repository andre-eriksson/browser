use assets::{ASSETS, constants::WINDOW_ICON};
use iced::{
    Background, Color, Length, Renderer, Size, Theme,
    widget::{
        button, column, container,
        scrollable::{self, Direction, Scrollbar},
        text,
    },
    window::{Position, Settings},
};

use crate::{
    api::{
        message::Message,
        window::{ApplicationWindow, WindowType},
    },
    core::app::Application,
    util::image::load_icon,
    views::browser::components::{content::render_content, header::render_header},
};

/// BrowserWindow is the "main" application window for the browser UI.
#[derive(Debug, Default)]
pub struct BrowserWindow {}

impl ApplicationWindow<Application, Message, Theme, Renderer> for BrowserWindow {
    fn render<'window>(
        &'window self,
        app: &'window Application,
    ) -> iced::Element<'window, Message, Theme, Renderer> {
        let header = render_header(app);
        let content = match render_content(app) {
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

        let debug_footer = container(
            button("Open DevTools")
                .on_press(Message::NewWindow(WindowType::Devtools))
                .padding(10),
        )
        .style(|_| container::background(Background::Color(Color::WHITE)))
        .padding(10.0)
        .width(Length::Fill)
        .height(Length::Shrink);

        let ui = container(
            column![
                header,
                scrollable::Scrollable::new(content)
                    .direction(Direction::Vertical(Scrollbar::new()))
                    .height(Length::Fill),
                debug_footer
            ]
            .spacing(10.0),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(10.0)
        .style(|_| container::Style {
            background: Some(Background::Color(Color::WHITE)),
            text_color: Some(Color::BLACK),
            ..Default::default()
        });

        ui.into()
    }

    fn settings(&self) -> iced::window::Settings {
        let icon = ASSETS.lock().unwrap().get(WINDOW_ICON);

        let browser_icon = load_icon(icon);

        Settings {
            size: Size::new(1920.0, 1080.0),
            position: Position::Centered,
            icon: Some(browser_icon),
            ..Default::default()
        }
    }

    fn title(&self) -> String {
        "Browser".to_string()
    }
}

use assets::{ASSETS, constants::WINDOW_ICON};
use constants::APP_NAME;
use iced::{
    Length, Renderer, Size, Theme,
    widget::{Shader, column, container, shader},
    window::{Position, Settings},
};

use crate::{
    api::window::ApplicationWindow,
    core::app::{Application, Event},
    util::image::load_icon,
    views::browser::components::{
        footer::render_footer, header::render_header, shader::HtmlRenderer,
    },
};

/// BrowserWindow is the "main" application window for the browser UI.
#[derive(Debug, Default)]
pub struct BrowserWindow {}

impl ApplicationWindow<Application, Event, Theme, Renderer> for BrowserWindow {
    fn render<'window>(
        &'window self,
        app: &'window Application,
    ) -> iced::Element<'window, Event, Theme, Renderer> {
        let header = render_header(app);
        let footer = render_footer();
        let shader: Shader<Event, HtmlRenderer> = shader(HtmlRenderer)
            .width(Length::Fill)
            .height(Length::Fill);

        container(column![header, shader, footer].spacing(10.0))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn settings(&self) -> iced::window::Settings {
        let icon = ASSETS.read().unwrap().load_embedded(WINDOW_ICON);

        let browser_icon = load_icon(icon);

        Settings {
            size: Size::new(1920.0, 1080.0),
            position: Position::Centered,
            icon: Some(browser_icon),
            ..Default::default()
        }
    }

    fn title(&self) -> String {
        APP_NAME.to_string()
    }
}

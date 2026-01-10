use assets::{ASSETS, constants::DEVTOOLS_ICON};
use constants::APP_NAME;
use iced::{
    Length, Renderer, Size, Theme,
    widget::{column, container},
    window::{Position, Settings},
};

use crate::{
    core::{Application, ApplicationWindow, Event},
    util::image::load_icon,
};

/// DevtoolsWindow is a window for displaying developer tools in the application.
#[derive(Debug, Default)]
pub struct DevtoolsWindow;

impl ApplicationWindow<Application, Event, Theme, Renderer> for DevtoolsWindow {
    fn render<'window>(
        &'window self,
        _app: &'window Application,
    ) -> iced::Element<'window, Event, Theme, Renderer> {
        //let dom_tree = match render_dom_tree(app) {
        //    Ok(content) => content,
        //    Err(e) => container(text(format!("Error rendering content: {}", e)))
        //        .width(Length::Fill)
        //        .padding(10.0)
        //        .style(|_| container::Style {
        //            background: Some(Background::Color(Color::from_rgb(0.95, 0.95, 0.95))),
        //            text_color: Some(Color::BLACK),
        //            ..Default::default()
        //        }),
        //};

        let ui = container(
            column![
                //scrollable::Scrollable::new(dom_tree)
                //    .direction(Direction::Vertical(Scrollbar::new()))
                //    .height(Length::Fill),
            ]
            .spacing(10.0),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(10.0);

        ui.into()
    }

    fn settings(&self) -> iced::window::Settings {
        let icon = ASSETS.read().unwrap().load_embedded(DEVTOOLS_ICON);

        let devtools_icon = load_icon(icon);

        Settings {
            size: Size::new(800.0, 800.0),
            position: Position::Centered,
            icon: Some(devtools_icon),
            ..Default::default()
        }
    }

    fn title(&self) -> String {
        format!("{} - DevTools", APP_NAME)
    }
}

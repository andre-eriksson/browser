use std::cell::Cell;

use constants::APP_NAME;
use iced::{
    Length, Renderer, Size, Theme,
    widget::{column, container},
    window::{self, Position, Settings},
};
use io::{Resource, embeded::DEVTOOLS_ICON};

use crate::{
    core::{Application, ApplicationWindow},
    events::Event,
    util::image::load_icon,
};

/// DevtoolsWindow is a window for displaying developer tools in the application.
#[derive(Debug)]
pub struct DevtoolsWindow {
    id: Cell<window::Id>,
}

impl ApplicationWindow<Application> for DevtoolsWindow {
    fn new(id: window::Id) -> Self
    where
        Self: Sized,
    {
        Self { id: Cell::new(id) }
    }

    fn render<'window>(&'window self, _app: &'window Application) -> iced::Element<'window, Event, Theme, Renderer> {
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

    fn settings() -> iced::window::Settings {
        let icon = Resource::load_embedded(DEVTOOLS_ICON);

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

    fn id(&self) -> window::Id {
        self.id.get()
    }
}

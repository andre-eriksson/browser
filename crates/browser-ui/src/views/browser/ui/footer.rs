use std::str::FromStr;

use iced::{
    Background, Border, Color, Length,
    border::Radius,
    widget::{button, container},
    window::Id,
};

use crate::{
    core::{Application, WindowType},
    events::{Event, window::WindowEvent},
};

pub struct BrowserFooter;

impl BrowserFooter {
    /// Renders the footer of the browser window.
    ///
    /// Contains a button to open the devtools!
    pub fn render(app: &Application, window_id: Id) -> container::Container<'_, Event> {
        let theme = app.config.preferences().theme();

        let ctx = app
            .browser_windows
            .get(&window_id)
            .expect("Browser context should exist for the window");

        let active_tab = ctx
            .tab_manager
            .active_tab()
            .expect("Active tab should always be present when rendering the browser window");

        let toggle_devtools_event = active_tab.devtools.as_ref().map_or_else(
            || Event::Window(WindowEvent::NewWindow(window_id, WindowType::Devtools)),
            |devtools| Event::Window(WindowEvent::CloseWindow(devtools.window_id)),
        );

        container(
            button("Open DevTools")
                .style(|_, status| button::Style {
                    background: match status {
                        button::Status::Hovered => {
                            Some(Background::Color(Color::from_str(&theme.colors.primary).unwrap()).scale_alpha(0.8))
                        }
                        _ => Some(Background::Color(Color::from_str(&theme.colors.primary).unwrap())),
                    },
                    border: Border {
                        radius: Radius::new(theme.style.border_radius),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .on_press(toggle_devtools_event)
                .padding(10),
        )
        .style(|_| {
            container::background(Background::Color(
                Color::from_str(app.config.preferences().theme().colors.foreground.as_str()).unwrap(),
            ))
        })
        .padding(10.0)
        .width(Length::Fill)
        .height(Length::Shrink)
    }
}

use std::str::FromStr;

use iced::{
    Background, Color, Length,
    widget::{button, container},
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
    pub fn render(app: &Application) -> container::Container<'_, Event> {
        container(
            button("Open DevTools")
                .on_press(Event::Window(WindowEvent::NewWindow(WindowType::Devtools)))
                .padding(10),
        )
        .style(|_| {
            container::background(Background::Color(
                Color::from_str(app.config.preferences().active_theme().foreground.as_str())
                    .unwrap_or(Color::from_str(&browser_preferences::Theme::default().foreground).unwrap()),
            ))
        })
        .padding(10.0)
        .width(Length::Fill)
        .height(Length::Shrink)
    }
}

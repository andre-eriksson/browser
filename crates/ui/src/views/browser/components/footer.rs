use iced::{
    Background, Color, Length,
    widget::{button, container},
};

use crate::{
    core::{Application, Event, WindowType},
    events::UiEvent,
};

pub struct BrowserFooter;

impl BrowserFooter {
    /// Renders the footer of the browser window.
    ///
    /// Contains a button to open the devtools!
    pub fn render(app: &Application) -> container::Container<'_, Event> {
        let color = &app.config.theme().foreground;

        container(
            button("Open DevTools")
                .on_press(Event::Ui(UiEvent::NewWindow(WindowType::Devtools)))
                .padding(10),
        )
        .style(|_| {
            container::background(Background::Color(Color::from_rgb8(
                color[0], color[1], color[2],
            )))
        })
        .padding(10.0)
        .width(Length::Fill)
        .height(Length::Shrink)
    }
}

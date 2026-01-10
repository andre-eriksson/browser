use iced::{
    Background, Color, Length,
    widget::{button, container},
};

use crate::{
    core::{Event, WindowType},
    events::UiEvent,
};

/// Renders the footer of the browser window.
///
/// Contains a button to open the devtools!
pub fn render_footer<'window>() -> container::Container<'window, Event> {
    container(
        button("Open DevTools")
            .on_press(Event::Ui(UiEvent::NewWindow(WindowType::Devtools)))
            .padding(10),
    )
    .style(|_| container::background(Background::Color(Color::from_rgb8(49, 50, 68))))
    .padding(10.0)
    .width(Length::Fill)
    .height(Length::Shrink)
}

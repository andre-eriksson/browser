use iced::{
    Background, Color, Length,
    widget::{button, container},
};

use crate::api::{message::Message, window::WindowType};

/// Renders the footer of the browser window.
///
/// Contains a button to open the devtools!
pub fn render_footer<'window>() -> container::Container<'window, Message> {
    container(
        button("Open DevTools")
            .on_press(Message::NewWindow(WindowType::Devtools))
            .padding(10),
    )
    .style(|_| container::background(Background::Color(Color::WHITE)))
    .padding(10.0)
    .width(Length::Fill)
    .height(Length::Shrink)
}

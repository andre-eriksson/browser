use std::str::FromStr;

use iced::{
    Background, Color, Length,
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
        let ctx = app
            .browser_windows
            .get(&window_id)
            .expect("Browser context should exist for the window");

        let active_tab = ctx
            .tabs
            .iter()
            .find(|tab| tab.id == ctx.active_tab_id)
            .expect("Active tab should always be present when rendering the browser window");

        let toggle_devtools_event = match &active_tab.devtools {
            Some(devtools) => Event::Window(WindowEvent::CloseWindow(devtools.window_id)),
            None => Event::Window(WindowEvent::NewWindow(window_id, WindowType::Devtools)),
        };

        container(
            button("Open DevTools")
                .on_press(toggle_devtools_event)
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

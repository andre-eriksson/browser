use std::str::FromStr;

use iced::{
    Background, Border, Color, Theme,
    border::Radius,
    widget::{TextInput, text_input},
    window::Id,
};

use crate::events::{Event, browser::BrowserEvent, kernel::EngineRequest};

pub struct SearchInput;

impl SearchInput {
    pub fn render<'app>(
        window_id: Id,
        theme: &'app browser_config::Theme,
        current_url: &str,
    ) -> TextInput<'app, Event> {
        text_input("Search", current_url)
            .style(|t: &Theme, _| text_input::Style {
                border: Border {
                    color: Color::from_str(&theme.colors.primary).unwrap(),
                    width: 0.5,
                    radius: Radius::new(theme.style.border_radius),
                },
                background: Background::Color(Color::from_str(&theme.colors.background).unwrap()),
                icon: Color::BLACK,
                placeholder: t.palette().text.scale_alpha(0.6),
                selection: Color::from_str(&theme.colors.tertiary)
                    .unwrap()
                    .scale_alpha(0.3),
                value: t.palette().text,
            })
            .on_input(move |text| Event::Browser(BrowserEvent::ChangeURL(window_id, text)))
            .on_submit(Event::EngineRequest(EngineRequest::NavigateTo(window_id, current_url.to_string())))
    }
}

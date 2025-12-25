use browser_core::events::BrowserEvent;
use iced::{
    Background, Border, Color, Length,
    widget::{button, column, container, mouse_area, row, text, text_input},
};

use crate::{
    core::app::{Application, Event},
    events::UiEvent,
};

/// Renders the header for the browser window, including tabs and a search bar.
pub fn render_header(app: &Application) -> container::Container<'_, Event> {
    let tabs = row(app
        .tabs
        .iter()
        .map(|tab| {
            let tab_title = tab.title.as_ref();

            mouse_area(
                button(text(format!(
                    "{} - {:?}",
                    tab_title.unwrap_or(&"N/A".to_string()),
                    tab.id
                )))
                .on_press(Event::Ui(UiEvent::ChangeActiveTab(tab.id))),
            )
            .on_right_press(Event::Ui(UiEvent::CloseTab(tab.id)))
            .into()
        })
        .chain(std::iter::once(
            button(text("+"))
                .on_press(Event::Ui(UiEvent::NewTab))
                .into(),
        ))
        .collect::<Vec<_>>())
    .width(Length::Shrink)
    .spacing(10.0);

    let search_bar = text_input("Search", &app.current_url)
        .on_input(|text| Event::Ui(UiEvent::ChangeURL(text)))
        .style(|_, _| text_input::Style {
            background: Background::Color(Color::from_rgb(0.95, 0.95, 0.95)),
            value: Color::BLACK,
            placeholder: Color::from_rgb(0.7, 0.7, 0.7),
            border: Border::default(),
            selection: Color::from_rgb(0.2, 0.6, 1.0),
            icon: Color::from_rgb(0.5, 0.5, 0.5),
        });

    let search_field = row![
        search_bar,
        button("Go").on_press(Event::Browser(BrowserEvent::NavigateTo(
            app.current_url.clone()
        )))
    ]
    .spacing(10.0);

    container(column![tabs, search_field].spacing(6.0))
        .width(Length::Fill)
        .style(|_| container::Style {
            background: Some(Background::Color(Color::WHITE)),
            text_color: Some(Color::BLACK),
            ..Default::default()
        })
}

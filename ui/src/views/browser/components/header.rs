use iced::{
    Background, Border, Color, Length,
    widget::{button, column, container, mouse_area, row, text, text_input},
};

use crate::{api::message::Message, core::app::Application};

pub fn render_header(app: &Application) -> container::Container<'_, Message> {
    let tabs = row(app
        .tabs
        .iter()
        .map(|tab| {
            let metadata = tab.metadata.lock().unwrap();
            let tab_title = metadata
                .title
                .as_ref()
                .map_or_else(|| "Untitled".to_string(), |t| t.clone());
            mouse_area(button(text(tab_title)).on_press(Message::ChangeTab(tab.id)))
                .on_right_press(Message::CloseTab(tab.id))
                .into()
        })
        .chain(std::iter::once(
            button(text("+")).on_press(Message::OpenNewTab).into(),
        ))
        .collect::<Vec<_>>())
    .width(Length::Shrink)
    .spacing(10.0);

    let search_bar = text_input("Search", &app.tabs[app.current_tab_id].temp_url)
        .on_input(Message::ChangeURL)
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
        button("Go").on_press(Message::NavigateTo(
            app.tabs[app.current_tab_id].temp_url.clone()
        ))
    ]
    .spacing(10.0);

    let top_bar = container(column![tabs, search_field].spacing(6.0))
        .width(Length::Fill)
        .style(|_| container::Style {
            background: Some(Background::Color(Color::WHITE)),
            text_color: Some(Color::BLACK),
            ..Default::default()
        });
    top_bar
}

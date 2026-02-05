use std::str::FromStr;

use iced::{
    Background, Color, Length, Theme,
    widget::{
        button, column, container, mouse_area, row,
        scrollable::{self, Direction, Scrollbar},
        text, text_input,
    },
};
use kernel::BrowserEvent;

use crate::{
    core::{Application, Event},
    events::UiEvent,
};

pub struct BrowserHeader;

impl BrowserHeader {
    /// Renders the header for the browser window, including tabs and a search bar.
    pub fn render(app: &Application) -> container::Container<'_, Event> {
        let theme = app.config.theme();

        let all_tabs = row(app
            .tabs
            .iter()
            .map(|tab| {
                let active_tab_id = app.active_tab;

                mouse_area(
                    button(text(tab.title.as_deref().unwrap_or("New Tab")))
                        .on_press(Event::Ui(UiEvent::ChangeActiveTab(tab.id)))
                        .style(move |t: &Theme, _| {
                            if tab.id == active_tab_id {
                                button::Style {
                                    background: Some(Background::Color(t.palette().primary)),
                                    ..Default::default()
                                }
                            } else {
                                button::Style {
                                    background: Some(Background::Color(
                                        Color::from_str(theme.secondary.as_str()).unwrap_or(
                                            Color::from_str(
                                                &preferences::Theme::default().secondary,
                                            )
                                            .unwrap(),
                                        ),
                                    )),
                                    ..Default::default()
                                }
                            }
                        }),
                )
                .on_right_press(Event::Ui(UiEvent::CloseTab(tab.id)))
                .into()
            })
            .chain(std::iter::once(
                button(text("+"))
                    .on_press(Event::Ui(UiEvent::NewTab))
                    .style(|_, _| button::Style {
                        background: Some(Background::Color(
                            Color::from_str(theme.tertiary.as_str()).unwrap_or(
                                Color::from_str(&preferences::Theme::default().tertiary).unwrap(),
                            ),
                        )),
                        ..Default::default()
                    })
                    .into(),
            ))
            .collect::<Vec<_>>())
        .width(Length::Shrink)
        .spacing(10.0);

        let tabs = scrollable::Scrollable::new(all_tabs)
            .direction(Direction::Horizontal(Scrollbar::new()))
            .height(Length::Fixed(40.0))
            .width(Length::FillPortion(2));

        let search_bar = text_input("Search", &app.current_url)
            .on_input(|text| Event::Ui(UiEvent::ChangeURL(text)))
            .on_submit(Event::Browser(BrowserEvent::NavigateTo(
                app.current_url.clone(),
            )));

        let search_field = row![
            search_bar,
            button("Go").on_press(Event::Browser(BrowserEvent::NavigateTo(
                app.current_url.clone()
            )))
        ]
        .spacing(10.0);

        container(column![tabs, search_field].spacing(6.0))
            .width(Length::Fill)
            .padding(10.0)
            .style(|_| container::Style {
                background: Some(Background::Color(
                    Color::from_str(theme.foreground.as_str()).unwrap_or(
                        Color::from_str(&preferences::Theme::default().foreground).unwrap(),
                    ),
                )),
                ..Default::default()
            })
    }
}

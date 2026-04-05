use std::str::FromStr;

use browser_core::TabId;
use iced::{
    Background, Border, Color, Length, Theme,
    alignment::Vertical,
    border::Radius,
    widget::{Button, MouseArea, Row, button, image, mouse_area, svg, text},
    window::Id,
};
use io::{Resource, embeded::PLUS_ICON};

use crate::{
    core::UiTab,
    events::{Event, browser::BrowserEvent},
};

pub struct TabButton;

impl TabButton {
    pub fn render<'a>(
        window_id: Id,
        theme: &'a browser_config::Theme,
        tab: &'a UiTab,
        active_tab_id: TabId,
    ) -> MouseArea<'a, Event> {
        let tab_title = text(tab.page.title().trim())
            .width(Length::Shrink)
            .height(Length::Shrink);

        let mut tab_title_row = Row::new();

        if let Some(favicon) = &tab.page.favicon {
            match favicon.content_type.as_deref() {
                Some("image/svg+xml") => {
                    tab_title_row = tab_title_row.push(
                        svg(iced::widget::svg::Handle::from_memory(favicon.data.clone()))
                            .width(Length::Fixed(16.0))
                            .height(Length::Fixed(16.0)),
                    );
                }
                _ => {
                    tab_title_row = tab_title_row.push(
                        image(iced::widget::image::Handle::from_bytes(favicon.data.clone()))
                            .width(Length::Fixed(16.0))
                            .height(Length::Fixed(16.0)),
                    );
                }
            }
        }

        tab_title_row = tab_title_row
            .push(tab_title)
            .align_y(Vertical::Center)
            .spacing(5.0);

        mouse_area(
            button(tab_title_row)
                .on_press(Event::Browser(BrowserEvent::ChangeActiveTab(window_id, tab.id)))
                .style(move |_theme: &Theme, status| {
                    if tab.id == active_tab_id {
                        button::Style {
                            background: Some(Background::Color(Color::from_str(&theme.colors.primary).unwrap())),
                            border: Border {
                                color: Color::from_str(&theme.colors.primary)
                                    .unwrap()
                                    .scale_alpha(0.1),
                                radius: Radius::new(theme.style.border_radius),
                                width: 1.0,
                            },
                            text_color: Color::from_str(&theme.colors.text).unwrap().inverse(),
                            ..Default::default()
                        }
                    } else {
                        match status {
                            button::Status::Hovered => button::Style {
                                background: Some(
                                    Background::Color(Color::from_str(&theme.colors.primary).unwrap()).scale_alpha(0.3),
                                ),
                                border: Border {
                                    color: Color::from_str(&theme.colors.primary)
                                        .unwrap()
                                        .scale_alpha(0.1),
                                    width: 1.0,
                                    radius: Radius::new(theme.style.border_radius),
                                },
                                text_color: Color::from_str(&theme.colors.text).unwrap(),
                                ..Default::default()
                            },
                            _ => button::Style {
                                background: Some(
                                    Background::Color(Color::from_str(&theme.colors.primary).unwrap()).scale_alpha(0.2),
                                ),
                                border: Border {
                                    color: Color::from_str(&theme.colors.primary)
                                        .unwrap()
                                        .scale_alpha(0.1),
                                    width: 1.0,
                                    radius: Radius::new(theme.style.border_radius),
                                },
                                text_color: Color::from_str(&theme.colors.text).unwrap(),
                                ..Default::default()
                            },
                        }
                    }
                }),
        )
        .on_right_press(Event::Browser(BrowserEvent::CloseTab(window_id, tab.id)))
    }
}

pub struct NewTabButton;

impl NewTabButton {
    pub fn render<'a>(window_id: Id, theme: &'a browser_config::Theme) -> Button<'a, Event> {
        let plus_icon = Resource::load_embedded(PLUS_ICON);

        button(
            svg(iced::widget::svg::Handle::from_memory(plus_icon))
                .width(Length::Fixed(16.0))
                .height(Length::Fixed(16.0)),
        )
        .padding(5.0)
        .style(|_, status| button::Style {
            background: match status {
                button::Status::Hovered => {
                    Some(Background::Color(Color::from_str(&theme.colors.tertiary).unwrap()).scale_alpha(0.8))
                }
                _ => Some(Background::Color(Color::from_str(&theme.colors.tertiary).unwrap())),
            },
            border: Border {
                radius: Radius::new(theme.style.border_radius),
                ..Default::default()
            },
            ..Default::default()
        })
        .on_press(Event::Browser(BrowserEvent::NewTab(window_id)))
    }
}

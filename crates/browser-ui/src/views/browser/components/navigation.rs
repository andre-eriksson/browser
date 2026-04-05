use std::str::FromStr;

use iced::{
    Background, Border, Color, Length,
    border::Radius,
    widget::{Button, button, svg},
    window::Id,
};
use io::{
    Resource,
    embeded::{LEFT_CHEVRON_ICON, REFRESH_ICON, RIGHT_CHEVRON_ICON},
};

use crate::{
    core::UiTab,
    events::{Event, kernel::EngineRequest},
};

pub struct BackButton;

impl BackButton {
    pub fn render<'a>(
        window_id: Id,
        theme: &'a browser_config::Theme,
        current_tab: Option<&'a UiTab>,
    ) -> Button<'a, Event> {
        let left_chevron_icon = Resource::load_embedded(LEFT_CHEVRON_ICON);

        button(
            svg(iced::widget::svg::Handle::from_memory(left_chevron_icon))
                .width(Length::Fixed(16.0))
                .height(Length::Fixed(16.0)),
        )
        .style(move |_, status| button::Style {
            background: if current_tab.is_some_and(|tab| tab.history_state.can_go_back) {
                match status {
                    button::Status::Hovered => {
                        Some(Background::Color(Color::from_str(&theme.colors.primary).unwrap()).scale_alpha(0.8))
                    }
                    _ => Some(Background::Color(Color::from_str(&theme.colors.primary).unwrap())),
                }
            } else {
                Some(Background::Color(Color::from_str(&theme.colors.primary).unwrap()).scale_alpha(0.5))
            },
            border: Border {
                radius: Radius::new(theme.style.border_radius),
                ..Default::default()
            },
            ..Default::default()
        })
        .on_press_maybe(if current_tab.is_some_and(|tab| tab.history_state.can_go_back) {
            Some(Event::EngineRequest(EngineRequest::NavigateBack(window_id)))
        } else {
            None
        })
    }
}

pub struct ForwardButton;

impl ForwardButton {
    pub fn render<'a>(
        window_id: Id,
        theme: &'a browser_config::Theme,
        current_tab: Option<&'a UiTab>,
    ) -> Button<'a, Event> {
        let right_chevron_icon = Resource::load_embedded(RIGHT_CHEVRON_ICON);

        button(
            svg(iced::widget::svg::Handle::from_memory(right_chevron_icon))
                .width(Length::Fixed(16.0))
                .height(Length::Fixed(16.0)),
        )
        .style(move |_, status| button::Style {
            background: if current_tab.is_some_and(|tab| tab.history_state.can_go_forward) {
                match status {
                    button::Status::Hovered => {
                        Some(Background::Color(Color::from_str(&theme.colors.primary).unwrap()).scale_alpha(0.8))
                    }
                    _ => Some(Background::Color(Color::from_str(&theme.colors.primary).unwrap())),
                }
            } else {
                Some(Background::Color(Color::from_str(&theme.colors.primary).unwrap()).scale_alpha(0.5))
            },
            border: Border {
                radius: Radius::new(theme.style.border_radius),
                ..Default::default()
            },
            ..Default::default()
        })
        .on_press_maybe(if current_tab.is_some_and(|tab| tab.history_state.can_go_forward) {
            Some(Event::EngineRequest(EngineRequest::NavigateForward(window_id)))
        } else {
            None
        })
    }
}

pub struct RefreshButton;

impl RefreshButton {
    pub fn render<'a>(window_id: Id, theme: &'a browser_config::Theme) -> Button<'a, Event> {
        let refresh_icon = Resource::load_embedded(REFRESH_ICON);

        button(
            svg(iced::widget::svg::Handle::from_memory(refresh_icon))
                .width(Length::Fixed(18.0))
                .height(Length::Fixed(18.0)),
        )
        .style(|_, status| button::Style {
            background: match status {
                button::Status::Hovered => {
                    Some(Background::Color(Color::from_str(&theme.colors.primary).unwrap()).scale_alpha(0.8))
                }
                _ => Some(Background::Color(Color::from_str(&theme.colors.primary).unwrap())),
            },
            border: Border {
                radius: Radius::new(theme.style.border_radius),
                ..Default::default()
            },
            ..Default::default()
        })
        .on_press(Event::EngineRequest(EngineRequest::Refresh(window_id)))
    }
}

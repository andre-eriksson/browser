use std::str::FromStr;

use iced::{
    Background, Color, Length, Theme,
    alignment::Vertical,
    widget::{
        button, column, container, mouse_area, row,
        scrollable::{self, Direction, Scrollbar},
        svg,
        svg::Handle,
        text, text_input,
    },
};
use io::{
    Resource,
    embeded::{LEFT_CHEVRON_ICON, PLUS_ICON, REFRESH_ICON, RIGHT_CHEVRON_ICON},
};

use crate::{
    core::Application,
    events::{Event, browser::BrowserEvent, kernel::KernelRequest},
};

pub struct BrowserHeader;

impl BrowserHeader {
    /// Renders the header for the browser window, including tabs and a search bar.
    pub fn render(app: &Application) -> container::Container<'_, Event> {
        let plus_icon = Resource::load_embedded(PLUS_ICON);
        let left_chevron_icon = Resource::load_embedded(LEFT_CHEVRON_ICON);
        let right_chevron_icon = Resource::load_embedded(RIGHT_CHEVRON_ICON);
        let refresh_icon = Resource::load_embedded(REFRESH_ICON);

        let current_tab = app.tabs.iter().find(|tab| tab.id == app.active_tab);

        let theme = app.config.active_theme();

        let all_tabs = row(app
            .tabs
            .iter()
            .map(|tab| {
                let active_tab_id = app.active_tab;

                mouse_area(
                    button(text(tab.page.title().trim()))
                        .on_press(Event::Browser(BrowserEvent::ChangeActiveTab(tab.id)))
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
                                            Color::from_str(&preferences::Theme::default().secondary).unwrap(),
                                        ),
                                    )),
                                    ..Default::default()
                                }
                            }
                        }),
                )
                .on_right_press(Event::Browser(BrowserEvent::CloseTab(tab.id)))
                .into()
            })
            .chain(std::iter::once(
                button(
                    svg(Handle::from_memory(plus_icon))
                        .width(Length::Fixed(21.0))
                        .height(Length::Fixed(21.0)),
                )
                .on_press(Event::Browser(BrowserEvent::NewTab))
                .style(|_, _| button::Style {
                    background: Some(Background::Color(
                        Color::from_str(theme.tertiary.as_str())
                            .unwrap_or(Color::from_str(&preferences::Theme::default().tertiary).unwrap()),
                    )),
                    ..Default::default()
                })
                .into(),
            ))
            .collect::<Vec<_>>())
        .width(Length::Shrink)
        .spacing(10.0);

        let back_navigation = button(
            svg(Handle::from_memory(left_chevron_icon))
                .width(Length::Fixed(18.0))
                .height(Length::Fixed(18.0)),
        )
        .on_press_maybe(if current_tab.is_some_and(|tab| tab.history_state.can_go_back) {
            Some(Event::KernelRequest(KernelRequest::NavigateBack))
        } else {
            None
        });

        let forward_navigation = button(
            svg(Handle::from_memory(right_chevron_icon))
                .width(Length::Fixed(18.0))
                .height(Length::Fixed(18.0)),
        )
        .on_press_maybe(if current_tab.is_some_and(|tab| tab.history_state.can_go_forward) {
            Some(Event::KernelRequest(KernelRequest::NavigateForward))
        } else {
            None
        });

        let refresh_navigation = button(
            svg(Handle::from_memory(refresh_icon))
                .width(Length::Fixed(18.0))
                .height(Length::Fixed(18.0)),
        )
        .on_press(Event::KernelRequest(KernelRequest::Refresh));

        let tabs = scrollable::Scrollable::new(all_tabs)
            .direction(Direction::Horizontal(Scrollbar::new()))
            .width(Length::FillPortion(2));

        let search_bar = text_input("Search", &app.current_url)
            .on_input(|text| Event::Browser(BrowserEvent::ChangeURL(text)))
            .on_submit(Event::KernelRequest(KernelRequest::NavigateTo(app.current_url.clone())));

        let search_field = row![
            back_navigation,
            forward_navigation,
            refresh_navigation,
            search_bar,
        ]
        .align_y(Vertical::Center)
        .spacing(10.0);

        container(column![tabs, search_field].spacing(6.0))
            .width(Length::Fill)
            .padding(10.0)
            .style(|_| container::Style {
                background: Some(Background::Color(
                    Color::from_str(theme.foreground.as_str())
                        .unwrap_or(Color::from_str(&preferences::Theme::default().foreground).unwrap()),
                )),
                ..Default::default()
            })
    }
}

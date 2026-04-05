use std::str::FromStr;

use iced::{
    Background, Color, Length, Theme,
    alignment::Vertical,
    widget::{
        Row, button, column, container, image, mouse_area, row,
        scrollable::{self, Direction, Scrollbar},
        svg, text, text_input,
    },
    window::Id,
};
use io::{
    Resource,
    embeded::{LEFT_CHEVRON_ICON, PLUS_ICON, REFRESH_ICON, RIGHT_CHEVRON_ICON},
};

use crate::{
    core::Application,
    events::{Event, browser::BrowserEvent, kernel::EngineRequest},
};

pub struct BrowserHeader;

impl BrowserHeader {
    /// Renders the header for the browser window, including tabs and a search bar.
    pub fn render(app: &Application, window_id: Id) -> container::Container<'_, Event> {
        let plus_icon = Resource::load_embedded(PLUS_ICON);
        let left_chevron_icon = Resource::load_embedded(LEFT_CHEVRON_ICON);
        let right_chevron_icon = Resource::load_embedded(RIGHT_CHEVRON_ICON);
        let refresh_icon = Resource::load_embedded(REFRESH_ICON);

        let ctx = app
            .browser_windows
            .get(&window_id)
            .expect("Browser context should exist for the window");

        let current_tab = ctx.tabs.iter().find(|tab| tab.id == ctx.active_tab_id);

        let theme = app.config.preferences().theme();

        let all_tabs = row(ctx
            .tabs
            .iter()
            .map(|tab| {
                let active_tab_id = ctx.active_tab_id;
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
                        .style(move |t: &Theme, _| {
                            if tab.id == active_tab_id {
                                button::Style {
                                    background: Some(Background::Color(t.palette().primary)),
                                    ..Default::default()
                                }
                            } else {
                                button::Style {
                                    background: Some(Background::Color(
                                        Color::from_str(theme.colors.secondary.as_str()).unwrap(),
                                    )),
                                    ..Default::default()
                                }
                            }
                        }),
                )
                .on_right_press(Event::Browser(BrowserEvent::CloseTab(window_id, tab.id)))
                .into()
            })
            .chain(std::iter::once(
                button(
                    svg(iced::widget::svg::Handle::from_memory(plus_icon))
                        .width(Length::Fixed(21.0))
                        .height(Length::Fixed(21.0)),
                )
                .on_press(Event::Browser(BrowserEvent::NewTab(window_id)))
                .style(|_, _| button::Style {
                    background: Some(Background::Color(Color::from_str(theme.colors.tertiary.as_str()).unwrap())),
                    ..Default::default()
                })
                .into(),
            ))
            .collect::<Vec<_>>())
        .width(Length::Shrink)
        .spacing(10.0);

        let back_navigation = button(
            svg(iced::widget::svg::Handle::from_memory(left_chevron_icon))
                .width(Length::Fixed(18.0))
                .height(Length::Fixed(18.0)),
        )
        .on_press_maybe(if current_tab.is_some_and(|tab| tab.history_state.can_go_back) {
            Some(Event::EngineRequest(EngineRequest::NavigateBack(window_id)))
        } else {
            None
        });

        let forward_navigation = button(
            svg(iced::widget::svg::Handle::from_memory(right_chevron_icon))
                .width(Length::Fixed(18.0))
                .height(Length::Fixed(18.0)),
        )
        .on_press_maybe(if current_tab.is_some_and(|tab| tab.history_state.can_go_forward) {
            Some(Event::EngineRequest(EngineRequest::NavigateForward(window_id)))
        } else {
            None
        });

        let refresh_navigation = button(
            svg(iced::widget::svg::Handle::from_memory(refresh_icon))
                .width(Length::Fixed(18.0))
                .height(Length::Fixed(18.0)),
        )
        .on_press(Event::EngineRequest(EngineRequest::Refresh(window_id)));

        let tabs = scrollable::Scrollable::new(all_tabs)
            .direction(Direction::Horizontal(Scrollbar::new()))
            .width(Length::FillPortion(2));

        let search_bar = text_input("Search", &ctx.current_url)
            .on_input(move |text| Event::Browser(BrowserEvent::ChangeURL(window_id, text)))
            .on_submit(Event::EngineRequest(EngineRequest::NavigateTo(window_id, ctx.current_url.clone())));

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
                background: Some(Background::Color(Color::from_str(theme.colors.foreground.as_str()).unwrap())),
                ..Default::default()
            })
    }
}

use std::str::FromStr;

use iced::{
    Background, Color, Length,
    alignment::Vertical,
    widget::{
        column, container, row,
        scrollable::{self, Direction, Scrollbar},
    },
    window::Id,
};

use crate::{
    core::Application,
    events::Event,
    views::browser::components::{
        navigation::{BackButton, ForwardButton, RefreshButton},
        search::SearchInput,
        tab::{NewTabButton, TabButton},
    },
};

pub struct BrowserHeader;

impl BrowserHeader {
    /// Renders the header for the browser window, including tabs and a search bar.
    pub fn render(app: &Application, window_id: Id) -> container::Container<'_, Event> {
        let ctx = app
            .browser_windows
            .get(&window_id)
            .expect("Browser context should exist for the window");

        let current_tab = ctx.tab_manager.active_tab();
        let theme = app.config.preferences().theme();

        let all_tabs = row(ctx
            .tab_manager
            .tabs()
            .iter()
            .map(|tab| TabButton::render(window_id, theme, tab, ctx.tab_manager.active_tab_id()).into())
            .chain(std::iter::once(NewTabButton::render(window_id, theme).into()))
            .collect::<Vec<_>>())
        .width(Length::Shrink)
        .align_y(Vertical::Center)
        .spacing(10.0);

        let tabs = scrollable::Scrollable::new(all_tabs)
            .direction(Direction::Horizontal(Scrollbar::new()))
            .width(Length::FillPortion(2));

        let search_field = row![
            BackButton::render(window_id, theme, current_tab),
            ForwardButton::render(window_id, theme, current_tab),
            RefreshButton::render(window_id, theme),
            SearchInput::render(window_id, theme, &ctx.current_url)
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

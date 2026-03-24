use std::cell::Cell;

use constants::APP_NAME;
use iced::{
    Length, Renderer, Size, Theme,
    widget::container,
    window::{self, Position, Settings},
};
use io::{Resource, embeded::DEVTOOLS_ICON};
use layout::Rect;

use crate::{
    core::{Application, ApplicationWindow, WindowType},
    events::{Event, devtools::DevtoolEvent},
    renderer::program::HtmlRenderer,
    util::image::load_icon,
    views::devtools::components::html::DevtoolsHtml,
};

/// DevtoolsWindow is a window for displaying developer tools in the application.
#[derive(Debug)]
pub struct DevtoolsWindow {
    id: Cell<window::Id>,
}

impl ApplicationWindow<Application> for DevtoolsWindow {
    fn new(id: window::Id) -> Self
    where
        Self: Sized,
    {
        Self { id: Cell::new(id) }
    }

    fn render<'window>(
        &'window self,
        application: &'window Application,
    ) -> iced::Element<'window, Event, Theme, Renderer> {
        let tab = application
            .tabs
            .iter()
            .find(|tab| tab.id == application.active_tab);

        let (viewport_width, viewport_height) = application
            .viewports
            .get(&self.id())
            .copied()
            .unwrap_or((800.0, 600.0));

        // NOTE: Varies depending on UI elements around the content.
        let content_viewport_height = (viewport_height + 50.0).max(100.0);

        match tab.and_then(|t| t.devtools_page.as_ref()) {
            Some(devtools) => {
                let renderer = HtmlRenderer::new(
                    devtools.document(),
                    devtools.layout_tree(),
                    devtools.scroll_offset,
                    WindowType::Devtools,
                );
                let html = DevtoolsHtml::new(
                    renderer,
                    devtools.layout_tree(),
                    Rect::new(0.0, 0.0, viewport_width, content_viewport_height),
                    devtools.scroll_offset,
                );
                html.render(application)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
            }
            None => container("DevTools page not available")
                .width(Length::Fill)
                .height(Length::Fill)
                .into(),
        }
    }

    fn settings() -> iced::window::Settings {
        let icon = Resource::load_embedded(DEVTOOLS_ICON);

        let devtools_icon = load_icon(icon);

        Settings {
            size: Size::new(800.0, 800.0),
            position: Position::Centered,
            icon: Some(devtools_icon),
            ..Default::default()
        }
    }

    fn title(&self) -> String {
        format!("{} - DevTools", APP_NAME)
    }

    fn id(&self) -> window::Id {
        self.id.get()
    }

    fn subscription(&self) -> iced::Subscription<Event> {
        window::resize_events()
            .with(self.id())
            .filter_map(|(id, (window_id, size))| {
                if id == window_id {
                    Some(Event::Devtools(DevtoolEvent::Resize(window_id, size.width, size.height)))
                } else {
                    None
                }
            })
    }
}

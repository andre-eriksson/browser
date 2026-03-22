use std::cell::Cell;

use constants::APP_NAME;
use iced::{
    Length, Renderer, Size, Theme,
    widget::container,
    window::{self, Position, Settings},
};
use io::{Resource, embeded::DEVTOOLS_ICON};

use crate::{
    core::{Application, ApplicationWindow},
    events::Event,
    util::image::load_icon,
    views::{
        browser::components::shader::{HtmlRenderer, ScrollEventTarget, ViewportBounds},
        devtools::html::DevtoolsHtml,
    },
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

        let (_, viewport_height) = application
            .viewports
            .get(&self.id())
            .copied()
            .unwrap_or((800.0, 600.0));

        // NOTE: Varies depending on UI elements around the content.
        let content_viewport_height = (viewport_height + 50.0).max(100.0);

        match tab.and_then(|t| t.devtools_page.as_ref()) {
            Some(devtools) => {
                let viewport_bounds = ViewportBounds::new(devtools.scroll_offset.y, content_viewport_height);
                let mut renderer = HtmlRenderer::new(devtools.document(), devtools.layout_tree());
                renderer.set_scroll_event_target(ScrollEventTarget::DevtoolsContent);
                let html = DevtoolsHtml::new(
                    devtools.scroll_offset,
                    viewport_bounds,
                    renderer,
                    devtools.document(),
                    devtools.layout_tree(),
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
}

use iced::{
    Length, Renderer, Size, Theme,
    widget::{container, text},
    window::{self, Id, Position, Settings, settings::PlatformSpecific},
};
use io::embedded::DEVTOOLS_ICON;
use layout::Rect;
use manifest::{DEVTOOLS_ID, DEVTOOLS_NAME};

use crate::{
    core::{Application, ApplicationWindow, WindowType},
    events::{DevtoolEvent, Event},
    renderer::program::HtmlRenderer,
    util::image::load_icon,
    windows::devtools::components::html::DevtoolsHtml,
};

/// `DevtoolsWindow` is a window for displaying developer tools in the application.
#[derive(Debug)]
pub struct DevtoolsWindow {
    parent_id: Id,
    id: Id,
}

impl DevtoolsWindow {
    pub(crate) const DEFAULT_VIEWPORT_SIZE: Size = Size::new(800.0, 600.0);
}

impl ApplicationWindow for DevtoolsWindow {
    fn new(parent_id: Option<Id>, id: Id) -> Self
    where
        Self: Sized,
    {
        Self {
            parent_id: parent_id.expect("Devtools window should have a parent window"),
            id,
        }
    }

    fn render<'window>(
        &'window self,
        application: &'window Application,
    ) -> iced::Element<'window, Event, Theme, Renderer> {
        let Some(ctx) = application.browser_windows.get(&self.parent_id) else {
            return container("Browser context not found for the window")
                .width(Length::Fill)
                .height(Length::Fill)
                .into();
        };

        let tab = ctx
            .tab_manager
            .active_tab()
            .expect("There should always be an active tab in the browser");

        let viewport = tab
            .devtools
            .as_ref()
            .map_or(Self::DEFAULT_VIEWPORT_SIZE, |d| d.context.viewport);

        // NOTE: Varies depending on UI elements around the content.
        let content_viewport_height = (viewport.height + 50.0).max(100.0);

        let Some(devtools) = tab.devtools.as_ref() else {
            return container(text("DevTools not available for this tab"))
                .width(Length::Fill)
                .height(Length::Fill)
                .into();
        };

        let Some(devtools_page) = &devtools.context.page else {
            return container(text("DevTools page not loaded"))
                .width(Length::Fill)
                .height(Length::Fill)
                .into();
        };

        let renderer = HtmlRenderer::new(
            self.id,
            devtools_page.dom(),
            devtools_page.layout_tree(),
            devtools_page.scroll_offset,
            WindowType::Devtools,
        );

        let html = DevtoolsHtml::new(
            renderer,
            devtools_page.layout_tree(),
            Rect::new(0.0, 0.0, f64::from(viewport.width), f64::from(content_viewport_height)),
            devtools_page.scroll_offset,
        );

        html.render(application, &devtools.context)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn settings() -> iced::window::Settings {
        let icon = DEVTOOLS_ICON.load();

        let devtools_icon = load_icon(&icon);

        Settings {
            size: Self::DEFAULT_VIEWPORT_SIZE,
            position: Position::Centered,
            icon: Some(devtools_icon),
            platform_specific: PlatformSpecific {
                application_id: DEVTOOLS_ID.to_string(),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    fn title(&self) -> String {
        DEVTOOLS_NAME.to_string()
    }

    fn parent_id(&self) -> Option<Id> {
        Some(self.parent_id)
    }

    fn subscription(&self) -> iced::Subscription<Event> {
        window::resize_events()
            .with(self.id)
            .filter_map(|(id, (window_id, new_viewport))| {
                if id == window_id {
                    Some(Event::Devtools(DevtoolEvent::Resize(window_id, new_viewport)))
                } else {
                    None
                }
            })
    }
}

use constants::{DEVTOOLS_ID, DEVTOOLS_NAME};
use iced::{
    Length, Renderer, Size, Theme,
    widget::{container, text},
    window::{self, Id, Position, Settings, settings::PlatformSpecific},
};
use io::{Resource, embeded::DEVTOOLS_ICON};
use layout::Rect;

use crate::{
    core::{Application, ApplicationWindow, UiDevtools, WindowType},
    events::{Event, devtools::DevtoolEvent},
    renderer::program::HtmlRenderer,
    util::image::load_icon,
    views::devtools::components::html::DevtoolsHtml,
};

#[derive(Debug, Clone)]
pub struct DevtoolsContext {
    pub viewport: Size,
    pub page: Option<UiDevtools>,
}

impl Default for DevtoolsContext {
    fn default() -> Self {
        Self {
            viewport: DevtoolsWindow::DEFAULT_VIEWPORT_SIZE,
            page: None,
        }
    }
}

/// DevtoolsWindow is a window for displaying developer tools in the application.
#[derive(Debug)]
pub struct DevtoolsWindow {
    parent_id: Id,
    id: Id,
}

impl DevtoolsWindow {
    pub const DEFAULT_VIEWPORT_SIZE: Size = Size::new(800.0, 600.0);
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

        let tab = ctx.tab_manager.active_tab();

        let viewport = tab
            .and_then(|t| t.devtools.as_ref())
            .map(|d| d.context.viewport)
            .unwrap_or(Self::DEFAULT_VIEWPORT_SIZE);

        // NOTE: Varies depending on UI elements around the content.
        let content_viewport_height = (viewport.height + 50.0).max(100.0);

        match tab
            .and_then(|t| t.devtools.as_ref())
            .and_then(|d| d.context.page.as_ref())
        {
            Some(devtools) => {
                let renderer = HtmlRenderer::new(
                    self.id,
                    devtools.document(),
                    devtools.layout_tree(),
                    devtools.scroll_offset,
                    WindowType::Devtools,
                );
                let html = DevtoolsHtml::new(
                    renderer,
                    devtools.layout_tree(),
                    Rect::new(0.0, 0.0, viewport.width, content_viewport_height),
                    devtools.scroll_offset,
                );
                html.render(application)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
            }
            None => container(text("DevTools page not available"))
                .width(Length::Fill)
                .height(Length::Fill)
                .into(),
        }
    }

    fn settings() -> iced::window::Settings {
        let icon = Resource::load_embedded(DEVTOOLS_ICON);

        let devtools_icon = load_icon(icon);

        Settings {
            size: DevtoolsWindow::DEFAULT_VIEWPORT_SIZE,
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

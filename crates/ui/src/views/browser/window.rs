use std::cell::Cell;

use constants::APP_NAME;
use iced::{
    Length, Renderer, Size, Subscription, Theme, event, mouse,
    widget::{column, container},
    window::{self, Position, Settings},
};
use io::{Resource, embeded::WINDOW_ICON};
use layout::Rect;

use crate::{
    core::{Application, ApplicationWindow, WindowType},
    events::{Event, browser::BrowserEvent, kernel::KernelRequest},
    renderer::program::HtmlRenderer,
    util::image::load_icon,
    views::browser::components::{footer::BrowserFooter, header::BrowserHeader, html::BrowserHtml},
};

pub const TOP_UI_OFFSET: f32 = 87.0;

/// BrowserWindow is the "main" application window for the browser UI.
#[derive(Debug)]
pub struct BrowserWindow {
    id: Cell<window::Id>,
}

impl ApplicationWindow<Application> for BrowserWindow {
    fn new(id: window::Id) -> Self
    where
        Self: Sized,
    {
        Self { id: Cell::new(id) }
    }

    fn render<'window>(&'window self, app: &'window Application) -> iced::Element<'window, Event, Theme, Renderer> {
        let header = BrowserHeader::render(app);
        let footer = BrowserFooter::render(app);

        let active_tab = app
            .tabs
            .iter()
            .find(|tab| tab.id == app.active_tab)
            .expect("Active tab should always be present when rendering the browser window");

        let (viewport_width, viewport_height) = app
            .viewports
            .get(&self.id())
            .copied()
            .unwrap_or((800.0, 600.0));

        // NOTE: Varies depending on UI elements around the content.
        let content_viewport_height = (viewport_height - 100.0).max(100.0);

        let renderer = HtmlRenderer::new(
            active_tab.page.document(),
            &active_tab.layout_tree,
            active_tab.scroll_offset,
            WindowType::Browser,
        );
        let html = BrowserHtml::new(
            renderer,
            &active_tab.layout_tree,
            Rect::new(0.0, TOP_UI_OFFSET, viewport_width, content_viewport_height),
            active_tab.scroll_offset,
        );
        let html_content = html.render(app);

        container(column![header, html_content, footer])
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn settings() -> iced::window::Settings {
        let icon = Resource::load_embedded(WINDOW_ICON);

        let browser_icon = load_icon(icon);

        Settings {
            size: Size::new(1920.0, 1080.0),
            position: Position::Centered,
            icon: Some(browser_icon),
            ..Default::default()
        }
    }

    fn title(&self) -> String {
        APP_NAME.to_string()
    }

    fn id(&self) -> window::Id {
        self.id.get()
    }

    fn subscription(&self) -> Subscription<Event> {
        let resize = window::resize_events()
            .with(self.id())
            .filter_map(|(id, (window_id, size))| {
                if id == window_id {
                    Some(Event::Browser(BrowserEvent::Resize(window_id, size.width, size.height)))
                } else {
                    None
                }
            });

        let mouse_nav = event::listen_with(|event, _status, _window| match event {
            iced::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Back)) => {
                Some(Event::KernelRequest(KernelRequest::NavigateBack))
            }
            iced::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Forward)) => {
                Some(Event::KernelRequest(KernelRequest::NavigateForward))
            }
            _ => None,
        })
        .with(self.id())
        .filter_map(|(id, event)| {
            let _ = id;
            Some(event)
        });

        Subscription::batch([resize, mouse_nav])
    }
}

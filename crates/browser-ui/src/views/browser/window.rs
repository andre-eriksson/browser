use std::sync::{Arc, Mutex};

use browser_config::BrowserArgs;
use browser_core::TabId;
use constants::{BROWSER_ID, BROWSER_NAME};
use iced::{
    Length, Renderer, Size, Subscription, Theme,
    advanced::graphics::text::cosmic_text::FontSystem,
    event, mouse,
    widget::{column, container},
    window::{self, Id, Position, Settings, settings::PlatformSpecific},
};
use io::{Resource, embeded::WINDOW_ICON};
use layout::{Rect, TextContext};

use crate::{
    core::{Application, ApplicationWindow, UiTab, WindowType},
    events::{Event, browser::BrowserEvent, kernel::EngineRequest},
    load_fallback_fonts,
    renderer::program::HtmlRenderer,
    util::image::load_icon,
    views::browser::components::{footer::BrowserFooter, header::BrowserHeader, html::BrowserHtml},
};

pub const TOP_UI_OFFSET: f32 = 87.0;
pub const DEFAULT_URL: &str = "https://www.google.com";

#[derive(Debug, Clone)]
pub struct BrowserContext {
    pub viewport: Size,
    pub current_url: String,
    pub tabs: Vec<UiTab>,
    pub active_tab_id: TabId,
    pub text_context: Arc<Mutex<TextContext>>,
}

impl BrowserContext {
    pub fn new(args: &BrowserArgs) -> Self {
        let font_system = FontSystem::new_with_fonts(load_fallback_fonts());
        let text_context = Arc::new(Mutex::new(TextContext::new(font_system)));

        Self {
            viewport: BrowserWindow::DEFAULT_VIEWPORT_SIZE,
            current_url: args.url.clone().unwrap_or(DEFAULT_URL.to_string()),
            tabs: vec![UiTab::new(TabId(0))],
            active_tab_id: TabId(0),
            text_context,
        }
    }
}

/// BrowserWindow is the "main" application window for the browser UI.
#[derive(Debug)]
pub struct BrowserWindow {
    id: Id,
}

impl BrowserWindow {
    pub const DEFAULT_VIEWPORT_SIZE: Size = Size::new(1920.0, 1080.0);
}

impl ApplicationWindow for BrowserWindow {
    fn new(_parent_id: Option<Id>, id: Id) -> Self
    where
        Self: Sized,
    {
        Self { id }
    }

    fn render<'window>(&'window self, app: &'window Application) -> iced::Element<'window, Event, Theme, Renderer> {
        let header = BrowserHeader::render(app, self.id);
        let footer = BrowserFooter::render(app, self.id);

        let ctx = app
            .browser_windows
            .get(&self.id)
            .expect("Browser context should exist for the window");

        let active_tab = ctx
            .tabs
            .iter()
            .find(|tab| tab.id == ctx.active_tab_id)
            .expect("Active tab should always be present when rendering the browser window");

        let viewport = ctx.viewport;

        // NOTE: Varies depending on UI elements around the content.
        let content_viewport_height = (viewport.height - 100.0).max(100.0);

        let renderer = HtmlRenderer::new(
            self.id,
            active_tab.page.document(),
            &active_tab.layout_tree,
            active_tab.scroll_offset,
            WindowType::Browser,
        );
        let html = BrowserHtml::new(
            renderer,
            &active_tab.layout_tree,
            Rect::new(0.0, TOP_UI_OFFSET, viewport.width, content_viewport_height),
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
            size: BrowserWindow::DEFAULT_VIEWPORT_SIZE,
            position: Position::Centered,
            icon: Some(browser_icon),
            platform_specific: PlatformSpecific {
                application_id: BROWSER_ID.to_string(),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    fn title(&self) -> String {
        BROWSER_NAME.to_string()
    }

    fn parent_id(&self) -> Option<Id> {
        None
    }

    fn subscription(&self) -> Subscription<Event> {
        let resize = window::resize_events()
            .with(self.id)
            .filter_map(|(id, (window_id, size))| {
                if id == window_id {
                    Some(Event::Browser(BrowserEvent::Resize(window_id, size)))
                } else {
                    None
                }
            });

        let mouse_nav = event::listen_with(|event, _status, window_id| match event {
            iced::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Back)) => {
                Some(Event::EngineRequest(EngineRequest::NavigateBack(window_id)))
            }
            iced::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Forward)) => {
                Some(Event::EngineRequest(EngineRequest::NavigateForward(window_id)))
            }
            _ => None,
        })
        .with(self.id)
        .filter_map(|(id, event)| {
            let _ = id;
            Some(event)
        });

        Subscription::batch([resize, mouse_nav])
    }
}

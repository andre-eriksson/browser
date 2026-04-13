use std::sync::{Arc, Mutex};

use browser_config::BrowserConfig;
use iced::{
    Length, Renderer, Size, Subscription, Theme,
    advanced::graphics::text::cosmic_text::FontSystem,
    event, mouse,
    widget::{Column, container},
    window::{self, Id, Position, Settings, settings::PlatformSpecific},
};
use io::{Resource, embeded::WINDOW_ICON};
use layout::{Rect, TextContext};
use manifest::{APP_ID, APP_NAME};

use crate::{
    core::{Application, ApplicationWindow, TabManager, WindowType},
    events::{Event, browser::BrowserEvent},
    load_fallback_fonts,
    renderer::program::HtmlRenderer,
    util::image::load_icon,
    views::browser::ui::{footer::BrowserFooter, header::BrowserHeader, html::BrowserHtml},
};

#[derive(Debug, Clone)]
pub struct BrowserContext {
    pub viewport: Size,
    pub current_url: String,
    pub tab_manager: TabManager,
    pub text_context: Arc<Mutex<TextContext>>,
}

impl BrowserContext {
    pub const DEFAULT_URL: &str = "https://www.google.com";

    pub fn new(config: &BrowserConfig) -> Self {
        let mut font_system = FontSystem::new_with_fonts(load_fallback_fonts());
        font_system.db_mut().set_serif_family("Roboto Serif");
        font_system.db_mut().set_sans_serif_family("Open Sans");
        font_system.db_mut().set_monospace_family("Roboto Mono");

        let text_context = Arc::new(Mutex::new(TextContext::new(font_system)));

        Self {
            viewport: BrowserWindow::DEFAULT_VIEWPORT_SIZE,
            current_url: config
                .args()
                .url
                .clone()
                .unwrap_or_else(|| Self::DEFAULT_URL.to_string()),
            tab_manager: TabManager::new(),
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
            .tab_manager
            .active_tab()
            .expect("There should always be an active tab in the browser");

        let viewport = ctx.viewport;

        // NOTE: Varies depending on UI elements around the content.
        let content_viewport_height = (viewport.height - 100.0).max(100.0);

        let mut column = Column::new();
        column = column.push(header);

        if let Some(page_ctx) = &active_tab.page_ctx
            && let Some(layout_tree) = &active_tab.layout_tree
        {
            let renderer = HtmlRenderer::new(
                self.id,
                page_ctx.page.document(),
                layout_tree,
                active_tab.scroll_offset,
                WindowType::Browser,
            );
            let html = BrowserHtml::new(
                renderer,
                layout_tree,
                Rect::new(0.0, 87.0, viewport.width, content_viewport_height),
                active_tab.scroll_offset,
            );
            let html_content = html.render(app);
            column = column.push(html_content);
        } else {
            let blank_page = container("").width(Length::Fill).height(Length::Fill);
            column = column.push(blank_page);
        }

        container(column.push(footer))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn settings() -> iced::window::Settings {
        let icon = Resource::load_embedded(WINDOW_ICON);

        let browser_icon = load_icon(icon);

        Settings {
            size: Self::DEFAULT_VIEWPORT_SIZE,
            position: Position::Centered,
            icon: Some(browser_icon),
            platform_specific: PlatformSpecific {
                application_id: APP_ID.to_string(),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    fn title(&self) -> String {
        APP_NAME.to_string()
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
                Some(Event::Browser(BrowserEvent::NavigateBack(window_id)))
            }
            iced::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Forward)) => {
                Some(Event::Browser(BrowserEvent::NavigateForward(window_id)))
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

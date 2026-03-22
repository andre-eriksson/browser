use std::cell::Cell;

use constants::APP_NAME;
use iced::{
    Length, Renderer, Size, Subscription, Theme, event, mouse,
    widget::{column, container},
    window::{self, Position, Settings},
};
use io::{Resource, embeded::WINDOW_ICON};

use crate::{
    core::{Application, ApplicationWindow},
    events::{Event, browser::BrowserEvent, kernel::KernelRequest},
    util::image::load_icon,
    views::browser::components::{
        footer::BrowserFooter, header::BrowserHeader, html::BrowserHtml, shader::HtmlRenderer,
    },
};

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

        let (dom, layout) = match app.tabs.iter().find(|tab| tab.id == app.active_tab) {
            Some(tab) => (&tab.page.document(), &tab.layout_tree),
            None => {
                return container(column![header, footer].spacing(10.0))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into();
            }
        };

        let mut renderer = HtmlRenderer::new(dom, layout);

        let active_tab = match app.tabs.iter().find(|tab| tab.id == app.active_tab) {
            Some(tab) => tab,
            None => {
                renderer.clear();
                return container(column![header, footer].spacing(10.0))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into();
            }
        };

        let html = BrowserHtml::new(renderer);
        let html_content = html.render(app, active_tab);

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

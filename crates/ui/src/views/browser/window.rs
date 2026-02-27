use constants::APP_NAME;
use iced::{
    Length, Renderer, Size, Theme,
    widget::{column, container},
    window::{Position, Settings},
};
use io::{Resource, embeded::WINDOW_ICON};

use crate::{
    core::{Application, ApplicationWindow},
    events::Event,
    util::image::load_icon,
    views::browser::components::{
        footer::BrowserFooter, header::BrowserHeader, html::BrowserHtml, shader::HtmlRenderer,
    },
};

/// BrowserWindow is the "main" application window for the browser UI.
#[derive(Debug, Default)]
pub struct BrowserWindow;

impl ApplicationWindow<Application, Event, Theme, Renderer> for BrowserWindow {
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

    fn settings(&self) -> iced::window::Settings {
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
}

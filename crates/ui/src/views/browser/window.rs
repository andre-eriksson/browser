use assets::{ASSETS, constants::WINDOW_ICON};
use constants::APP_NAME;
use iced::{
    Background, Color, Length, Renderer, Size, Theme,
    widget::{
        Shader, Space, column, container,
        scrollable::{self, Direction, Scrollbar, Viewport},
        shader, stack,
    },
    window::{Position, Settings},
};

use crate::{
    api::window::ApplicationWindow,
    core::app::{Application, Event},
    events::UiEvent,
    util::image::load_icon,
    views::browser::components::{
        footer::render_footer,
        header::render_header,
        shader::{HtmlRenderer, ViewportBounds, collect_render_data_from_layout},
    },
};

/// BrowserWindow is the "main" application window for the browser UI.
#[derive(Debug, Default)]
pub struct BrowserWindow;

impl ApplicationWindow<Application, Event, Theme, Renderer> for BrowserWindow {
    fn render<'window>(
        &'window self,
        app: &'window Application,
    ) -> iced::Element<'window, Event, Theme, Renderer> {
        let header = render_header(app);
        let footer = render_footer();
        let mut renderer = HtmlRenderer::default();

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

        let (_, viewport_height) = app
            .viewports
            .get(&app.id)
            .copied()
            .unwrap_or((800.0, 600.0));

        let content_viewport_height = (viewport_height - 100.0).max(100.0);

        let viewport_bounds =
            ViewportBounds::new(active_tab.scroll_offset.y, content_viewport_height);

        let render_data =
            collect_render_data_from_layout(&active_tab.layout_tree, Some(viewport_bounds));
        renderer.set_rects(render_data.rects);
        renderer.set_text_blocks(render_data.text_blocks);
        renderer.set_scroll_offset(active_tab.scroll_offset);

        let shader: Shader<Event, HtmlRenderer> =
            shader(renderer).width(Length::Fill).height(Length::Fill);

        let scroll_spacer = Space::new()
            .width(Length::Fill)
            .height(Length::Fixed(active_tab.layout_tree.content_height));

        let scrollable_layer = scrollable::Scrollable::new(scroll_spacer)
            .direction(Direction::Vertical(Scrollbar::new()))
            .width(Length::Fill)
            .height(Length::Fill)
            .on_scroll(|viewport: Viewport| {
                Event::Ui(UiEvent::ContentScrolled(
                    viewport.absolute_offset().x,
                    viewport.absolute_offset().y,
                ))
            });

        let content_stack = stack![scrollable_layer, shader]
            .width(Length::Fill)
            .height(Length::Fill);

        let content = container(content_stack)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_| container::Style {
                background: Some(Background::Color(Color::from_rgb8(0xFF, 0xF5, 0xEE))),
                ..Default::default()
            });

        container(column![header, content, footer])
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn settings(&self) -> iced::window::Settings {
        let icon = ASSETS.read().unwrap().load_embedded(WINDOW_ICON);

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

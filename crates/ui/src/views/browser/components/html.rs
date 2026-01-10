use iced::{
    Background, Color, Length,
    widget::{
        Shader, Space, container,
        scrollable::{self, Direction, Scrollbar, Viewport},
        shader, stack,
    },
};

use crate::{
    core::{Application, Event, UiTab},
    events::UiEvent,
    views::browser::components::shader::{
        HtmlRenderer, ViewportBounds, collect_render_data_from_layout,
    },
};

pub struct BrowserHtml {
    renderer: HtmlRenderer,
}

impl BrowserHtml {
    pub fn new(renderer: HtmlRenderer) -> Self {
        Self { renderer }
    }

    pub fn render<'a>(
        mut self,
        app: &Application,
        active_tab: &UiTab,
    ) -> container::Container<'a, Event> {
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
        self.renderer.set_rects(render_data.rects);
        self.renderer.set_text_blocks(render_data.text_blocks);
        self.renderer.set_scroll_offset(active_tab.scroll_offset);

        let shader: Shader<Event, HtmlRenderer> = shader(self.renderer)
            .width(Length::Fill)
            .height(Length::Fill);

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

        container(content_stack)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_| container::Style {
                background: Some(Background::Color(Color::from_rgb8(0xFF, 0xF5, 0xEE))),
                ..Default::default()
            })
    }
}

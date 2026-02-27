use std::str::FromStr;

use iced::{
    Background, Color, Length,
    widget::{
        Shader, Space, container,
        scrollable::{self, Direction, Scrollbar, Viewport},
        shader, stack,
    },
};

use crate::{
    core::{Application, UiTab},
    events::{Event, UiEvent},
    views::browser::components::shader::{
        HtmlRenderer, ViewportBounds, collect_render_data_from_layout,
    },
};

pub struct BrowserHtml<'renderer> {
    renderer: HtmlRenderer<'renderer>,
}

impl<'renderer> BrowserHtml<'renderer> {
    pub fn new(renderer: HtmlRenderer<'renderer>) -> Self {
        Self { renderer }
    }

    pub fn render<'application>(
        mut self,
        app: &'application Application,
        active_tab: &'application UiTab,
    ) -> container::Container<'application, Event>
    where
        'renderer: 'application,
    {
        let (_, viewport_height) = app
            .viewports
            .get(&app.id)
            .copied()
            .unwrap_or((800.0, 600.0));

        let content_viewport_height = (viewport_height - 100.0).max(100.0);

        let viewport_bounds =
            ViewportBounds::new(active_tab.scroll_offset.y, content_viewport_height);

        let render_data = collect_render_data_from_layout(
            active_tab.page.document(),
            &active_tab.layout_tree,
            Some(viewport_bounds),
            app.image_cache.as_ref(),
        );

        self.renderer.set_rects(render_data.rects);
        self.renderer.set_tris(render_data.tris);
        self.renderer.set_text_blocks(render_data.text_blocks);
        self.renderer.set_images(render_data.images);
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
            .style(move |_| container::Style {
                background: Some(Background::Color(
                    Color::from_str(app.config.theme().background.as_str()).unwrap_or(
                        Color::from_str(&preferences::Theme::default().background).unwrap(),
                    ),
                )),
                ..Default::default()
            })
    }
}

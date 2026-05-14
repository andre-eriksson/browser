use std::str::FromStr;

use iced::{
    Background, Color, Length,
    widget::{Shader, container, shader},
};
use layout::{LayoutTree, Rect};

use crate::{
    core::{Application, PageContext, ScrollOffset},
    events::Event,
    renderer::{program::HtmlRenderer, viewport::collect_render_data_from_layout},
};

pub struct BrowserHtml<'renderer> {
    renderer: HtmlRenderer<'renderer>,
    layout_tree: &'renderer LayoutTree,
    initial_bounds: Rect,
    scroll_offset: ScrollOffset,
}

impl<'renderer> BrowserHtml<'renderer> {
    pub const fn new(
        renderer: HtmlRenderer<'renderer>,
        layout_tree: &'renderer LayoutTree,
        initial_bounds: Rect,
        scroll_offset: ScrollOffset,
    ) -> Self {
        Self {
            renderer,
            layout_tree,
            initial_bounds,
            scroll_offset,
        }
    }

    pub fn render(
        mut self,
        app: &'renderer Application,
        page_ctx: &PageContext,
    ) -> container::Container<'renderer, Event> {
        let image_ctx = page_ctx.image_context();
        let image_ctx = image_ctx.lock().unwrap();

        collect_render_data_from_layout(
            &image_ctx,
            &mut self.renderer,
            self.layout_tree,
            self.initial_bounds,
            self.scroll_offset,
        );

        let shader: Shader<Event, HtmlRenderer> = shader(self.renderer)
            .width(Length::Fill)
            .height(Length::Fill);

        container(shader)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_| {
                let background_color = if app.config.args().preferences.force_dark {
                    Color::from_str(app.config.preferences().theme().colors.background.as_str()).unwrap()
                } else {
                    Color::from_rgb8(255, 255, 255)
                };

                container::Style {
                    background: Some(Background::Color(background_color)),
                    ..Default::default()
                }
            })
    }
}

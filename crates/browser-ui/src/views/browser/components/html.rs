use std::str::FromStr;

use iced::{
    Background, Color, Length,
    widget::{Shader, container, shader},
};
use layout::{LayoutTree, Rect};

use crate::{
    core::{Application, ScrollOffset},
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
    pub fn new(
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

    pub fn render(mut self, app: &'renderer Application) -> container::Container<'renderer, Event> {
        collect_render_data_from_layout(
            &mut self.renderer,
            self.layout_tree,
            app.image_cache.as_ref(),
            self.initial_bounds,
            self.scroll_offset,
        );

        let shader: Shader<Event, HtmlRenderer> = shader(self.renderer)
            .width(Length::Fill)
            .height(Length::Fill);

        container(shader)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_| container::Style {
                background: Some(Background::Color(
                    Color::from_str(app.config.preferences().active_theme().background.as_str())
                        .unwrap_or(Color::from_str(&browser_preferences::Theme::default().background).unwrap()),
                )),
                ..Default::default()
            })
    }
}

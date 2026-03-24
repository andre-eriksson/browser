use iced::{
    Length,
    widget::{Shader, container, shader},
};
use layout::{LayoutTree, Rect};

use crate::{
    core::{Application, ScrollOffset},
    events::Event,
    renderer::{program::HtmlRenderer, viewport::collect_render_data_from_layout},
};

pub struct DevtoolsHtml<'renderer> {
    renderer: HtmlRenderer<'renderer>,
    layout_tree: &'renderer LayoutTree,
    initial_bounds: Rect,
    scroll_offset: ScrollOffset,
}

impl<'renderer> DevtoolsHtml<'renderer> {
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

    pub fn render<'application>(mut self, _app: &'application Application) -> container::Container<'application, Event>
    where
        'renderer: 'application,
    {
        collect_render_data_from_layout(
            &mut self.renderer,
            self.layout_tree,
            None,
            self.initial_bounds,
            self.scroll_offset,
        );

        let shader: Shader<Event, HtmlRenderer> = shader(self.renderer)
            .width(Length::Fill)
            .height(Length::Fill);

        container(shader).width(Length::Fill).height(Length::Fill)
    }
}

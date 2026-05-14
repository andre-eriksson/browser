use iced::{
    Length,
    widget::{Shader, container, shader},
};
use layout::{LayoutTree, Rect};

use crate::{
    core::{Application, ScrollOffset},
    events::Event,
    renderer::{program::HtmlRenderer, viewport::collect_render_data_from_layout},
    views::devtools::window::DevtoolsContext,
};

pub struct DevtoolsHtml<'renderer> {
    renderer: HtmlRenderer<'renderer>,
    layout_tree: &'renderer LayoutTree,
    initial_bounds: Rect,
    scroll_offset: ScrollOffset,
}

impl<'renderer> DevtoolsHtml<'renderer> {
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

    pub fn render<'app>(
        mut self,
        _app: &'app Application,
        devtools_ctx: &DevtoolsContext,
    ) -> container::Container<'app, Event>
    where
        'renderer: 'app,
    {
        let image_ctx = devtools_ctx.image_context();
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

        container(shader).width(Length::Fill).height(Length::Fill)
    }
}

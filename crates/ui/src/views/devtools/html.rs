use html_dom::DocumentRoot;
use iced::{
    Length,
    widget::{Shader, container, shader},
};
use layout::LayoutTree;

use crate::{
    core::{Application, ScrollOffset},
    events::Event,
    views::browser::components::shader::{HtmlRenderer, ViewportBounds, collect_render_data_from_layout},
};

pub struct DevtoolsHtml<'renderer> {
    scroll: ScrollOffset,
    viewport_bounds: ViewportBounds,
    renderer: HtmlRenderer<'renderer>,
    dom: &'renderer DocumentRoot,
    layout_tree: &'renderer LayoutTree,
}

impl<'renderer> DevtoolsHtml<'renderer> {
    pub fn new(
        scroll: ScrollOffset,
        viewport_bounds: ViewportBounds,
        renderer: HtmlRenderer<'renderer>,
        dom: &'renderer DocumentRoot,
        layout_tree: &'renderer LayoutTree,
    ) -> Self {
        Self {
            scroll,
            viewport_bounds,
            renderer,
            dom,
            layout_tree,
        }
    }

    pub fn render<'application>(mut self, _app: &'application Application) -> container::Container<'application, Event>
    where
        'renderer: 'application,
    {
        let render_data = collect_render_data_from_layout(self.dom, self.layout_tree, Some(self.viewport_bounds), None);
        self.renderer.set_rects(render_data.rects);
        self.renderer.set_tris(render_data.tris);
        self.renderer.set_text_blocks(render_data.text_blocks);
        self.renderer.set_images(render_data.images);
        self.renderer.set_scroll_offset(self.scroll);
        self.renderer
            .set_viewport_height(self.viewport_bounds.viewport_height);

        let shader: Shader<Event, HtmlRenderer> = shader(self.renderer)
            .width(Length::Fill)
            .height(Length::Fixed(self.viewport_bounds.viewport_height));

        container(shader)
            .width(Length::Fill)
            .height(Length::Fixed(self.viewport_bounds.viewport_height))
    }
}

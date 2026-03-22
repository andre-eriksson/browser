use std::str::FromStr;

use iced::{
    Background, Color, Length,
    widget::{Shader, container, shader},
};
use layout::SideOffset;

use crate::{
    core::{Application, UiTab},
    events::Event,
    views::browser::components::shader::{
        HtmlRenderer, RendererViewport, ViewportBounds, collect_render_data_from_layout,
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
        let (viewport_width, viewport_height) = app
            .viewports
            .get(&app.id)
            .copied()
            .unwrap_or((800.0, 600.0));

        // NOTE: Varies depending on UI elements around the content.
        let content_viewport_height = (viewport_height - 100.0).max(100.0);

        let viewport_bounds = ViewportBounds::new(
            RendererViewport {
                scroll_offset: active_tab.scroll_offset,
                width: viewport_width,
                height: content_viewport_height,
            },
            SideOffset {
                top: 100.0,
                bottom: 100.0,
                ..SideOffset::zero()
            },
        );

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
        self.renderer.set_viewport(viewport_bounds.viewport);

        let shader: Shader<Event, HtmlRenderer> = shader(self.renderer)
            .width(Length::Fill)
            .height(Length::Fill);

        container(shader)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_| container::Style {
                background: Some(Background::Color(
                    Color::from_str(app.config.active_theme().background.as_str())
                        .unwrap_or(Color::from_str(&preferences::Theme::default().background).unwrap()),
                )),
                ..Default::default()
            })
    }
}

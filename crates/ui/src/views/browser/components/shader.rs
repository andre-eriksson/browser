use std::sync::Arc;

use assets::{
    ASSETS,
    constants::{DEFAULT_FONT, MONOSPACE_FONT},
};
use cosmic_text::{FontSystem, fontdb::Source};
use iced::{
    Rectangle,
    mouse::Cursor,
    wgpu::{self, RenderPass},
    widget::shader::{Pipeline, Primitive, Program, Viewport},
};
use layout::{Color4f, LayoutNode, LayoutTree, Rect};
use renderer::{GlyphAtlas, RectPipeline, RenderRect, TextBlockInfo, TexturePipeline};

use crate::core::app::Event;
use crate::core::tabs::ScrollOffset;

/// The primitive that carries render data from draw() to prepare()/render()
#[derive(Debug, Clone)]
pub struct HtmlPrimitive {
    pub rects: Vec<RenderRect>,
    pub text_blocks: Vec<TextBlockInfo>,
    pub viewport_width: f32,
    pub viewport_height: f32,
    pub scroll_offset: ScrollOffset,
}

impl HtmlPrimitive {
    pub fn new(viewport_width: f32, viewport_height: f32, scroll_offset: ScrollOffset) -> Self {
        Self {
            rects: Vec::new(),
            text_blocks: Vec::new(),
            viewport_width,
            viewport_height,
            scroll_offset,
        }
    }

    /// Add a rectangle to be rendered
    pub fn push_rect(&mut self, rect: Rect, background: Color4f) {
        self.rects.push(RenderRect { rect, background });
    }

    /// Add a text block to be rendered
    pub fn push_text_block(&mut self, text_block: TextBlockInfo) {
        self.text_blocks.push(text_block);
    }
}

/// Pipeline wrapper that implements iced's Pipeline trait
pub struct HtmlPipeline {
    rect_pipeline: RectPipeline,
    text_pipeline: TexturePipeline,
    glyph_atlas: GlyphAtlas,
    font_system: FontSystem,
}

impl Pipeline for HtmlPipeline {
    fn new(device: &wgpu::Device, _queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self
    where
        Self: Sized,
    {
        let glyph_atlas = GlyphAtlas::new(device);

        let text_pipeline =
            TexturePipeline::new_text(device, format, glyph_atlas.bind_group_layout());

        let default_font = ASSETS.read().unwrap().load_embedded(DEFAULT_FONT);
        let monospace_font = ASSETS.read().unwrap().load_embedded(MONOSPACE_FONT);

        let font_system = FontSystem::new_with_fonts(vec![
            Source::Binary(Arc::new(default_font)),
            Source::Binary(Arc::new(monospace_font)),
        ]);

        Self {
            rect_pipeline: RectPipeline::new(device, format),
            text_pipeline,
            glyph_atlas,
            font_system,
        }
    }
}

impl Primitive for HtmlPrimitive {
    type Pipeline = HtmlPipeline;

    fn prepare(
        &self,
        pipeline: &mut Self::Pipeline,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        _bounds: &Rectangle,
        _viewport: &Viewport,
    ) {
        pipeline
            .rect_pipeline
            .update_globals(queue, self.viewport_width, self.viewport_height);
        pipeline
            .text_pipeline
            .update_globals(queue, self.viewport_width, self.viewport_height);

        pipeline.rect_pipeline.clear();
        pipeline.text_pipeline.clear();

        for render_rect in &self.rects {
            let offset_rect = Rect {
                x: render_rect.rect.x - self.scroll_offset.x,
                y: render_rect.rect.y - self.scroll_offset.y,
                width: render_rect.rect.width,
                height: render_rect.rect.height,
            };
            pipeline
                .rect_pipeline
                .push_quad(offset_rect, render_rect.background);
        }

        let (atlas_width, atlas_height) = pipeline.glyph_atlas.size();

        for text_block in &self.text_blocks {
            for glyph_info in &text_block.glyphs {
                let region = match pipeline.glyph_atlas.cache_glyph(
                    &mut pipeline.font_system,
                    queue,
                    glyph_info.cache_key,
                ) {
                    Some(region) => region,
                    None => continue,
                };

                if region.width == 0 || region.height == 0 {
                    continue;
                }

                let screen_x =
                    glyph_info.x as f32 + region.placement_left as f32 - self.scroll_offset.x;
                let screen_y =
                    glyph_info.y as f32 - region.placement_top as f32 - self.scroll_offset.y;

                let uv_rect = region.uv_rect(atlas_width, atlas_height);

                let screen_rect = Rect {
                    x: screen_x,
                    y: screen_y,
                    width: region.width as f32,
                    height: region.height as f32,
                };

                pipeline
                    .text_pipeline
                    .push_quad(screen_rect, uv_rect, glyph_info.text_color);
            }
        }

        pipeline.rect_pipeline.flush(queue);
        pipeline.text_pipeline.flush(queue);
    }

    fn draw(&self, pipeline: &Self::Pipeline, render_pass: &mut RenderPass<'_>) -> bool {
        let has_rects = pipeline.rect_pipeline.has_content();
        let has_text = pipeline.text_pipeline.has_content();

        if !has_rects && !has_text {
            return false;
        }

        if has_rects {
            render_pass.set_pipeline(pipeline.rect_pipeline.pipeline());
            render_pass.set_bind_group(0, pipeline.rect_pipeline.bind_group(), &[]);
            render_pass.set_vertex_buffer(0, pipeline.rect_pipeline.vertex_buffer().slice(..));
            render_pass.draw(0..pipeline.rect_pipeline.vertex_count(), 0..1);
        }

        if has_text {
            render_pass.set_pipeline(pipeline.text_pipeline.pipeline());
            render_pass.set_bind_group(0, pipeline.text_pipeline.bind_group(), &[]);
            render_pass.set_bind_group(1, pipeline.glyph_atlas.bind_group(), &[]);
            render_pass.set_vertex_buffer(0, pipeline.text_pipeline.vertex_buffer().slice(..));
            render_pass.draw(0..pipeline.text_pipeline.vertex_count(), 0..1);
        }

        true
    }
}

/// HTML/CSS renderer using wgpu
#[derive(Debug, Clone, Default)]
pub struct HtmlRenderer {
    /// Rectangles to render (populated by layout engine)
    pub rects: Vec<RenderRect>,
    /// Text blocks to render (populated by layout engine)
    pub text_blocks: Vec<TextBlockInfo>,
    /// The scroll offset for viewport-based rendering
    pub scroll_offset: ScrollOffset,
}

impl HtmlRenderer {
    pub fn clear(&mut self) {
        self.rects.clear();
        self.text_blocks.clear();
    }

    pub fn set_rects(&mut self, rects: Vec<RenderRect>) {
        self.rects = rects;
    }

    pub fn set_text_blocks(&mut self, text_blocks: Vec<TextBlockInfo>) {
        self.text_blocks = text_blocks;
    }

    pub fn set_scroll_offset(&mut self, scroll_offset: ScrollOffset) {
        self.scroll_offset = scroll_offset;
    }
}

/// State for the shader widget
#[derive(Default)]
pub struct ShaderState;

impl Program<Event> for HtmlRenderer {
    type Primitive = HtmlPrimitive;
    type State = ShaderState;

    fn draw(&self, _state: &Self::State, _cursor: Cursor, bounds: Rectangle) -> Self::Primitive {
        let mut primitive = HtmlPrimitive::new(bounds.width, bounds.height, self.scroll_offset);

        for render_rect in &self.rects {
            primitive.push_rect(render_rect.rect, render_rect.background);
        }

        for text_block in &self.text_blocks {
            primitive.push_text_block(text_block.clone());
        }

        primitive
    }
}

/// Viewport bounds for culling off-screen content
#[derive(Debug, Clone, Copy)]
pub struct ViewportBounds {
    pub scroll_y: f32,
    pub viewport_height: f32,
    pub margin: f32,
}

impl ViewportBounds {
    pub fn new(scroll_y: f32, viewport_height: f32) -> Self {
        Self {
            scroll_y,
            viewport_height,
            margin: 200.0,
        }
    }

    /// Check if a node is visible within the viewport (with margin)
    fn is_visible(&self, node_y: f32, node_height: f32) -> bool {
        let viewport_top = self.scroll_y - self.margin;
        let viewport_bottom = self.scroll_y + self.viewport_height + self.margin;
        let node_bottom = node_y + node_height;

        node_bottom >= viewport_top && node_y <= viewport_bottom
    }
}

/// Helper function to collect all render data from a layout tree with viewport culling
pub fn collect_render_data_from_layout(
    layout_tree: &LayoutTree,
    viewport: Option<ViewportBounds>,
) -> HtmlRenderer {
    let mut data = HtmlRenderer::default();

    fn collect_node(node: &LayoutNode, data: &mut HtmlRenderer, viewport: Option<&ViewportBounds>) {
        if let Some(vp) = viewport
            && !vp.is_visible(node.dimensions.y, node.dimensions.height)
        {
            return;
        }

        let bg = node.colors.background_color;

        if bg.a > 0.0 {
            data.rects.push(RenderRect {
                rect: Rect {
                    x: node.dimensions.x,
                    y: node.dimensions.y,
                    width: node.dimensions.width,
                    height: node.dimensions.height,
                },
                background: bg,
            });
        }

        if let Some(buffer) = &node.text_buffer {
            let text_block = TextBlockInfo::from_arc_buffer(
                buffer,
                node.dimensions.x,
                node.dimensions.y,
                node.colors.color,
            );
            if !text_block.glyphs.is_empty() {
                data.text_blocks.push(text_block);
            }
        }

        for child in &node.children {
            collect_node(child, data, viewport);
        }
    }

    for root in &layout_tree.root_nodes {
        collect_node(root, &mut data, viewport.as_ref());
    }

    data
}

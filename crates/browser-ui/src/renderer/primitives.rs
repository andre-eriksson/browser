use iced::{
    Rectangle,
    wgpu::{self, RenderPass},
    widget::shader::{Primitive, Viewport},
};
use layout::{Color4f, Rect};
use renderer::{ImageRenderInfo, RenderRect, RenderTri, TextBlockInfo};

use crate::{core::ScrollOffset, renderer::pipeline::HtmlPipeline};

/// The primitive that carries render data from `draw()` to `prepare()`/`render()`
#[derive(Debug, Clone)]
pub struct HtmlPrimitive {
    pub rects: Vec<RenderRect>,
    pub tris: Vec<RenderTri>,
    pub text_blocks: Vec<TextBlockInfo>,
    pub images: Vec<ImageRenderInfo>,
    pub scroll_offset: ScrollOffset,
}

impl HtmlPrimitive {
    pub const fn new(scroll_offset: ScrollOffset) -> Self {
        Self {
            rects: Vec::new(),
            tris: Vec::new(),
            text_blocks: Vec::new(),
            images: Vec::new(),
            scroll_offset,
        }
    }

    /// Add a rectangle to be rendered
    pub fn push_rect(&mut self, rect: Rect, background: Color4f) {
        self.rects.push(RenderRect { rect, background });
    }

    /// Add a triangle to be rendered
    pub fn push_triangle(&mut self, p0: [f32; 2], p1: [f32; 2], p2: [f32; 2], color: Color4f) {
        self.tris.push(RenderTri { p0, p1, p2, color });
    }

    /// Add a text block to be rendered
    pub fn push_text_block(&mut self, text_block: TextBlockInfo) {
        self.text_blocks.push(text_block);
    }

    /// Add an image to be rendered
    pub fn push_image(&mut self, image: ImageRenderInfo) {
        self.images.push(image);
    }
}

impl Primitive for HtmlPrimitive {
    type Pipeline = HtmlPipeline;

    fn prepare(
        &self,
        pipeline: &mut Self::Pipeline,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bounds: &Rectangle,
        _viewport: &Viewport,
    ) {
        pipeline
            .rect_pipeline
            .update_globals(queue, bounds.width, bounds.height);
        pipeline
            .text_pipeline
            .update_globals(queue, bounds.width, bounds.height);
        pipeline
            .image_pipeline
            .update_globals(queue, bounds.width, bounds.height);

        pipeline.rect_pipeline.clear();
        pipeline.text_pipeline.clear();
        pipeline.image_pipeline.clear();

        for render_rect in &self.rects {
            let offset_rect = Rect::new(
                render_rect.rect.x as f32 - self.scroll_offset.x,
                render_rect.rect.y as f32 - self.scroll_offset.y,
                render_rect.rect.width as f32,
                render_rect.rect.height as f32,
            );
            pipeline
                .rect_pipeline
                .push_quad(offset_rect, render_rect.background);
        }

        for tri in &self.tris {
            let p0 = [
                tri.p0[0] - self.scroll_offset.x,
                tri.p0[1] - self.scroll_offset.y,
            ];
            let p1 = [
                tri.p1[0] - self.scroll_offset.x,
                tri.p1[1] - self.scroll_offset.y,
            ];
            let p2 = [
                tri.p2[0] - self.scroll_offset.x,
                tri.p2[1] - self.scroll_offset.y,
            ];
            pipeline.rect_pipeline.push_triangle(p0, p1, p2, tri.color);
        }

        let (atlas_width, atlas_height) = pipeline.glyph_atlas.size();

        for text_block in &self.text_blocks {
            for glyph_info in &text_block.glyphs {
                let Some(region) =
                    pipeline
                        .glyph_atlas
                        .cache_glyph(&mut pipeline.font_system, queue, glyph_info.cache_key)
                else {
                    continue;
                };

                if region.width == 0 || region.height == 0 {
                    continue;
                }

                let screen_x = glyph_info.x + region.placement_left as f32 - self.scroll_offset.x;
                let screen_y = glyph_info.y - region.placement_top as f32 - self.scroll_offset.y;

                let uv_rect = region.uv_rect(atlas_width as f32, atlas_height as f32);

                let screen_rect = Rect::new(screen_x, screen_y, region.width as f32, region.height as f32);

                pipeline
                    .text_pipeline
                    .push_quad(screen_rect, uv_rect, glyph_info.text_color);
            }
        }

        for image_info in &self.images {
            pipeline
                .gpu_image_cache
                .ensure_uploaded(device, queue, &image_info.src, &image_info.data);

            let screen_rect = Rect::new(
                image_info.screen_rect.x as f32 - self.scroll_offset.x,
                image_info.screen_rect.y as f32 - self.scroll_offset.y,
                image_info.screen_rect.width as f32,
                image_info.screen_rect.height as f32,
            );
            let full_uv = Rect::new(0.0, 0.0, 1.0, 1.0);

            pipeline
                .image_pipeline
                .push_quad(screen_rect, full_uv, Color4f::WHITE);
        }

        pipeline.rect_pipeline.flush(queue);
        pipeline.text_pipeline.flush(queue);
        pipeline.image_pipeline.flush(queue);
    }

    fn draw(&self, pipeline: &Self::Pipeline, render_pass: &mut RenderPass<'_>) -> bool {
        let has_rects = pipeline.rect_pipeline.has_content();
        let has_text = pipeline.text_pipeline.has_content();
        let has_images = pipeline.image_pipeline.has_content();

        if !has_rects && !has_text && !has_images {
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

        if has_images {
            render_pass.set_pipeline(pipeline.image_pipeline.pipeline());
            render_pass.set_bind_group(0, pipeline.image_pipeline.bind_group(), &[]);
            render_pass.set_vertex_buffer(0, pipeline.image_pipeline.vertex_buffer().slice(..));

            let mut vertex_offset: u32 = 0;
            for image_info in &self.images {
                if let Some(bind_group) = pipeline.gpu_image_cache.get_bind_group(&image_info.src) {
                    render_pass.set_bind_group(1, bind_group, &[]);
                    render_pass.draw(vertex_offset..vertex_offset + 6, 0..1);
                }
                vertex_offset += 6;
            }
        }

        true
    }
}

use iced::{
    advanced::graphics::text::cosmic_text::FontSystem,
    wgpu::{self},
    widget::shader::Pipeline,
};
use renderer::{GlyphAtlas, GpuImageCache, RectPipeline, TexturePipeline};

use crate::util::fonts::load_fallback_fonts;

/// Pipeline wrapper that implements iced's Pipeline trait
pub struct HtmlPipeline {
    pub rect_pipeline: RectPipeline,
    pub text_pipeline: TexturePipeline,
    pub image_pipeline: TexturePipeline,
    pub glyph_atlas: GlyphAtlas,
    pub font_system: FontSystem,
    pub gpu_image_cache: GpuImageCache,
}

impl Pipeline for HtmlPipeline {
    fn new(device: &wgpu::Device, _queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self
    where
        Self: Sized,
    {
        let rect_pipeline = RectPipeline::new(device, format);
        let glyph_atlas = GlyphAtlas::new(device);
        let text_pipeline = TexturePipeline::new_text(device, format, glyph_atlas.bind_group_layout());
        let gpu_image_cache = GpuImageCache::new(device);
        let image_pipeline = TexturePipeline::new_image(device, format, gpu_image_cache.bind_group_layout());
        let font_system = FontSystem::new_with_fonts(load_fallback_fonts());

        Self {
            rect_pipeline,
            text_pipeline,
            image_pipeline,
            glyph_atlas,
            font_system,
            gpu_image_cache,
        }
    }
}

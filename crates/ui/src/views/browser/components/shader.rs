use std::sync::Arc;

use html_dom::{DocumentRoot, HtmlTag, Tag};
use iced::{
    Rectangle,
    advanced::graphics::text::cosmic_text::FontSystem,
    mouse::{self, Cursor},
    wgpu::{self, RenderPass},
    widget::{
        Action,
        shader::{Pipeline, Primitive, Program, Viewport},
    },
};
use io::{CacheEntry, CacheRead};
use kernel::BrowserEvent;
use layout::{Color4f, LayoutNode, LayoutTree, Rect};
use renderer::{
    DecodedImageData, GlyphAtlas, GpuImageCache, ImageRenderInfo, RectPipeline, RenderRect,
    RenderTri, TextBlockInfo, TexturePipeline, image::ImageCache,
};

use crate::{
    core::{Event, ScrollOffset},
    util::fonts::load_fallback_fonts,
};

pub const UI_VERTICAL_OFFSET: f32 = 88.0;

/// The primitive that carries render data from draw() to prepare()/render()
#[derive(Debug, Clone)]
pub struct HtmlPrimitive {
    pub rects: Vec<RenderRect>,
    pub tris: Vec<RenderTri>,
    pub text_blocks: Vec<TextBlockInfo>,
    pub images: Vec<ImageRenderInfo>,
    pub viewport_width: f32,
    pub viewport_height: f32,
    pub scroll_offset: ScrollOffset,
}

impl HtmlPrimitive {
    pub fn new(viewport_width: f32, viewport_height: f32, scroll_offset: ScrollOffset) -> Self {
        Self {
            rects: Vec::new(),
            tris: Vec::new(),
            text_blocks: Vec::new(),
            images: Vec::new(),
            viewport_width,
            viewport_height,
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

/// Pipeline wrapper that implements iced's Pipeline trait
pub struct HtmlPipeline {
    rect_pipeline: RectPipeline,
    text_pipeline: TexturePipeline,
    image_pipeline: TexturePipeline,
    glyph_atlas: GlyphAtlas,
    font_system: FontSystem,
    gpu_image_cache: GpuImageCache,
}

impl Pipeline for HtmlPipeline {
    fn new(device: &wgpu::Device, _queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self
    where
        Self: Sized,
    {
        let glyph_atlas = GlyphAtlas::new(device);

        let text_pipeline =
            TexturePipeline::new_text(device, format, glyph_atlas.bind_group_layout());

        let gpu_image_cache = GpuImageCache::new(device);

        let image_pipeline =
            TexturePipeline::new_image(device, format, gpu_image_cache.bind_group_layout());

        let font_system = FontSystem::new_with_fonts(load_fallback_fonts());

        Self {
            rect_pipeline: RectPipeline::new(device, format),
            text_pipeline,
            image_pipeline,
            glyph_atlas,
            font_system,
            gpu_image_cache,
        }
    }
}

impl Primitive for HtmlPrimitive {
    type Pipeline = HtmlPipeline;

    fn prepare(
        &self,
        pipeline: &mut Self::Pipeline,
        device: &wgpu::Device,
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
        pipeline
            .image_pipeline
            .update_globals(queue, self.viewport_width, self.viewport_height);

        pipeline.rect_pipeline.clear();
        pipeline.text_pipeline.clear();
        pipeline.image_pipeline.clear();

        for render_rect in &self.rects {
            let offset_rect = Rect::new(
                render_rect.rect.x - self.scroll_offset.x,
                render_rect.rect.y - self.scroll_offset.y,
                render_rect.rect.width,
                render_rect.rect.height,
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

                let screen_rect = Rect::new(
                    screen_x,
                    screen_y,
                    region.width as f32,
                    region.height as f32,
                );

                pipeline
                    .text_pipeline
                    .push_quad(screen_rect, uv_rect, glyph_info.text_color);
            }
        }

        for image_info in &self.images {
            pipeline.gpu_image_cache.ensure_uploaded(
                device,
                queue,
                &image_info.src,
                &image_info.data,
            );

            let screen_rect = Rect::new(
                image_info.screen_rect.x - self.scroll_offset.x,
                image_info.screen_rect.y - self.scroll_offset.y,
                image_info.screen_rect.width,
                image_info.screen_rect.height,
            );
            let full_uv = Rect::new(0.0, 0.0, 1.0, 1.0);
            let white = Color4f {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            };
            pipeline
                .image_pipeline
                .push_quad(screen_rect, full_uv, white);
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

/// HTML/CSS renderer using wgpu
#[derive(Debug, Clone)]
pub struct HtmlRenderer<'a> {
    /// Rectangles to render (populated by layout engine)
    pub rects: Vec<RenderRect>,

    /// Triangles to render (populated by layout engine)
    pub tris: Vec<RenderTri>,

    /// Text blocks to render (populated by layout engine)
    pub text_blocks: Vec<TextBlockInfo>,

    /// Images to render (populated by layout engine + image cache)
    pub images: Vec<ImageRenderInfo>,

    /// The scroll offset for viewport-based rendering
    pub scroll_offset: ScrollOffset,

    /// The DOM tree being rendered
    pub dom_tree: &'a DocumentRoot,

    /// The layout tree being rendered
    pub layout_tree: &'a LayoutTree,

    /// Shared image cache for checking loaded images
    pub image_cache: Option<ImageCache>,
}

impl<'html> HtmlRenderer<'html> {
    pub fn new(dom_tree: &'html DocumentRoot, layout_tree: &'html LayoutTree) -> Self {
        Self {
            rects: Vec::new(),
            tris: Vec::new(),
            text_blocks: Vec::new(),
            images: Vec::new(),
            scroll_offset: ScrollOffset { x: 0.0, y: 0.0 },
            dom_tree,
            layout_tree,
            image_cache: None,
        }
    }

    pub fn clear(&mut self) {
        self.rects.clear();
        self.tris.clear();
        self.text_blocks.clear();
        self.images.clear();
    }

    pub fn set_rects(&mut self, rects: Vec<RenderRect>) {
        self.rects = rects;
    }

    pub fn set_tris(&mut self, tris: Vec<RenderTri>) {
        self.tris = tris;
    }

    pub fn set_text_blocks(&mut self, text_blocks: Vec<TextBlockInfo>) {
        self.text_blocks = text_blocks;
    }

    pub fn set_images(&mut self, images: Vec<ImageRenderInfo>) {
        self.images = images;
    }

    pub fn set_scroll_offset(&mut self, scroll_offset: ScrollOffset) {
        self.scroll_offset = scroll_offset;
    }

    pub fn set_image_cache(&mut self, cache: ImageCache) {
        self.image_cache = Some(cache);
    }
}

/// State for the shader widget
#[derive(Default)]
pub struct HtmlProgram;

impl<'a> Program<Event> for HtmlRenderer<'a> {
    type Primitive = HtmlPrimitive;
    type State = HtmlProgram;

    fn draw(&self, _state: &Self::State, _cursor: Cursor, bounds: Rectangle) -> Self::Primitive {
        let mut primitive = HtmlPrimitive::new(bounds.width, bounds.height, self.scroll_offset);

        for tri in &self.tris {
            primitive.push_triangle(tri.p0, tri.p1, tri.p2, tri.color);
        }

        for render_rect in &self.rects {
            primitive.push_rect(render_rect.rect, render_rect.background);
        }

        for text_block in &self.text_blocks {
            primitive.push_text_block(text_block.clone());
        }

        for image in &self.images {
            primitive.push_image(image.clone());
        }

        primitive
    }

    fn update(
        &self,
        _state: &mut Self::State,
        event: &iced::Event,
        _bounds: Rectangle,
        cursor: iced::advanced::mouse::Cursor,
    ) -> Option<iced::widget::Action<Event>> {
        if let Some(position) = cursor.position() {
            let x = position.x + self.scroll_offset.x;
            let y = position.y + self.scroll_offset.y - UI_VERTICAL_OFFSET;

            let nodes = self.layout_tree.resolve(x, y);
            let mut found_link = false;
            let mut element = None;

            'outer: for node in nodes {
                let dom_node = if let Some(dom_node) = self.dom_tree.get_node(&node.node_id) {
                    dom_node
                } else {
                    continue;
                };

                if let Some(n) = dom_node.data.as_element()
                    && n.tag == Tag::Html(HtmlTag::A)
                {
                    element = Some(n);
                    found_link = true;
                    break;
                }

                for ancestor in self.dom_tree.ancestors(&node.node_id) {
                    if let Some(n) = ancestor.data.as_element()
                        && n.tag == Tag::Html(HtmlTag::A)
                    {
                        element = Some(n);
                        found_link = true;
                        break 'outer;
                    }
                }
            }

            if found_link
                && let Some(element) = element
                && let iced::Event::Mouse(e) = event
                && let mouse::Event::ButtonReleased(_) = e
            {
                return Some(Action::publish(Event::Browser(BrowserEvent::NavigateTo(
                    element.attributes.get("href").cloned().unwrap_or_default(),
                ))));
            }
        }

        None
    }

    fn mouse_interaction(
        &self,
        _state: &Self::State,
        _bounds: Rectangle,
        cursor: iced::advanced::mouse::Cursor,
    ) -> iced::advanced::mouse::Interaction {
        if let Some(position) = cursor.position() {
            let x = position.x + self.scroll_offset.x;
            let y = position.y + self.scroll_offset.y - UI_VERTICAL_OFFSET;

            let nodes = self.layout_tree.resolve(x, y);
            let mut found_link = false;

            'outer: for node in nodes {
                let dom_node = if let Some(dom_node) = self.dom_tree.get_node(&node.node_id) {
                    dom_node
                } else {
                    continue;
                };

                if let Some(n) = dom_node.data.as_element()
                    && n.tag == Tag::Html(HtmlTag::A)
                {
                    found_link = true;
                    break;
                }

                for ancestor in self.dom_tree.ancestors(&node.node_id) {
                    if let Some(n) = ancestor.data.as_element()
                        && n.tag == Tag::Html(HtmlTag::A)
                    {
                        found_link = true;
                        break 'outer;
                    }
                }
            }

            if found_link {
                return iced::advanced::mouse::Interaction::Pointer;
            }
        }

        iced::advanced::mouse::Interaction::default()
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
pub fn collect_render_data_from_layout<'html>(
    dom_tree: &'html DocumentRoot,
    layout_tree: &'html LayoutTree,
    viewport: Option<ViewportBounds>,
    image_cache: Option<&ImageCache>,
) -> HtmlRenderer<'html> {
    let mut data = HtmlRenderer::new(dom_tree, layout_tree);
    if let Some(cache) = image_cache {
        data.set_image_cache(cache.clone());
    }

    fn collect_node(
        node: &LayoutNode,
        data: &mut HtmlRenderer,
        viewport: Option<&ViewportBounds>,
        image_cache: Option<&ImageCache>,
    ) {
        if let Some(vp) = viewport
            && !vp.is_visible(node.dimensions.y, node.dimensions.height)
        {
            return;
        }

        let bg = node.colors.background_color;

        let border = node.resolved_border;
        let border_color = &node.colors.border_color;
        if (border.top > 0.0 || border.right > 0.0 || border.bottom > 0.0 || border.left > 0.0)
            && node.dimensions.width > 0.0
            && node.dimensions.height > 0.0
        {
            let x = node.dimensions.x;
            let y = node.dimensions.y;
            let w = node.dimensions.width;
            let h = node.dimensions.height;

            let inner_x = x + border.left;
            let inner_y = y + border.top;
            let inner_w = (w - border.horizontal()).max(0.0);
            let inner_h = (h - border.vertical()).max(0.0);
            let inner_right = inner_x + inner_w;
            let inner_bottom = inner_y + inner_h;

            let outer_right = x + w;
            let outer_bottom = y + h;

            if border.top > 0.0 && border_color.top.a > 0.0 {
                data.tris.push(RenderTri {
                    p0: [x, y],
                    p1: [outer_right, y],
                    p2: [inner_right, inner_y],
                    color: border_color.top,
                });
                data.tris.push(RenderTri {
                    p0: [x, y],
                    p1: [inner_right, inner_y],
                    p2: [inner_x, inner_y],
                    color: border_color.top,
                });
            }

            if border.right > 0.0 && border_color.right.a > 0.0 {
                data.tris.push(RenderTri {
                    p0: [outer_right, y],
                    p1: [outer_right, outer_bottom],
                    p2: [inner_right, inner_bottom],
                    color: border_color.right,
                });
                data.tris.push(RenderTri {
                    p0: [outer_right, y],
                    p1: [inner_right, inner_bottom],
                    p2: [inner_right, inner_y],
                    color: border_color.right,
                });
            }

            if border.bottom > 0.0 && border_color.bottom.a > 0.0 {
                data.tris.push(RenderTri {
                    p0: [outer_right, outer_bottom],
                    p1: [x, outer_bottom],
                    p2: [inner_x, inner_bottom],
                    color: border_color.bottom,
                });
                data.tris.push(RenderTri {
                    p0: [outer_right, outer_bottom],
                    p1: [inner_x, inner_bottom],
                    p2: [inner_right, inner_bottom],
                    color: border_color.bottom,
                });
            }

            if border.left > 0.0 && border_color.left.a > 0.0 {
                data.tris.push(RenderTri {
                    p0: [x, outer_bottom],
                    p1: [x, y],
                    p2: [inner_x, inner_y],
                    color: border_color.left,
                });
                data.tris.push(RenderTri {
                    p0: [x, outer_bottom],
                    p1: [inner_x, inner_y],
                    p2: [inner_x, inner_bottom],
                    color: border_color.left,
                });
            }
        }

        if bg.a > 0.0 {
            let border = node.resolved_border;
            let inner_x = node.dimensions.x + border.left;
            let inner_y = node.dimensions.y + border.top;
            let inner_width = (node.dimensions.width - border.horizontal()).max(0.0);
            let inner_height = (node.dimensions.height - border.vertical()).max(0.0);
            data.rects.push(RenderRect {
                rect: Rect::new(inner_x, inner_y, inner_width, inner_height),
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

        if let Some(image_data) = &node.image_data
            && let Some(cache) = image_cache
        {
            let src = &image_data.image_src;
            let vary_key = &image_data.vary_key;
            if let Ok(CacheEntry::Loaded(decoded)) = cache.get_with_vary(src, vary_key)
                && let CacheRead::Hit(decoded) = (*decoded).clone()
            {
                data.images.push(ImageRenderInfo {
                    src: src.clone(),
                    screen_rect: node.dimensions,
                    data: Arc::new(DecodedImageData {
                        rgba: decoded.rgba.clone(),
                        width: decoded.width,
                        height: decoded.height,
                    }),
                });
            } else {
                const IMAGE_PLACEHOLDER_COLOR: Color4f = Color4f {
                    r: 0.9,
                    g: 0.9,
                    b: 0.9,
                    a: 1.0,
                };
                data.rects.push(RenderRect {
                    rect: node.dimensions,
                    background: IMAGE_PLACEHOLDER_COLOR,
                });
            }
        }

        for child in &node.children {
            collect_node(child, data, viewport, image_cache);
        }
    }

    for root in &layout_tree.root_nodes {
        collect_node(root, &mut data, viewport.as_ref(), image_cache);
    }

    data
}

use iced::{
    Rectangle,
    mouse::Cursor,
    wgpu::{self, RenderPass},
    widget::shader::{Pipeline, Primitive, Program, Viewport},
};
use layout::{Color4f, LayoutNode, LayoutTree, Rect};
use renderer::rect::RectPipeline;

use crate::core::app::Event;

/// Data for rectangles to be rendered
#[derive(Debug, Clone)]
pub struct RenderRect {
    pub rect: Rect,
    pub background: Color4f,
}

/// The primitive that carries render data from draw() to prepare()/render()
#[derive(Debug, Clone)]
pub struct HtmlPrimitive {
    /// Rectangles to render
    pub rects: Vec<RenderRect>,

    pub viewport_width: f32,
    pub viewport_height: f32,
}

impl HtmlPrimitive {
    pub fn new(viewport_width: f32, viewport_height: f32) -> Self {
        Self {
            rects: Vec::new(),
            viewport_width,
            viewport_height,
        }
    }

    /// Add a rectangle to be rendered
    pub fn push_rect(&mut self, rect: Rect, background: Color4f) {
        self.rects.push(RenderRect { rect, background });
    }
}

/// Pipeline wrapper that implements iced's Pipeline trait
pub struct HtmlPipeline {
    rect_pipeline: RectPipeline,
}

impl Pipeline for HtmlPipeline {
    fn new(device: &wgpu::Device, _queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self
    where
        Self: Sized,
    {
        Self {
            rect_pipeline: RectPipeline::new(device, format),
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
            .update_viewport(queue, self.viewport_width, self.viewport_height);

        pipeline.rect_pipeline.clear();

        for render_rect in &self.rects {
            pipeline
                .rect_pipeline
                .push_quad(render_rect.rect, render_rect.background);
        }

        pipeline.rect_pipeline.flush(queue);
    }

    fn draw(&self, pipeline: &Self::Pipeline, render_pass: &mut RenderPass<'_>) -> bool {
        if !pipeline.rect_pipeline.has_content() {
            return false;
        }

        render_pass.set_pipeline(pipeline.rect_pipeline.pipeline());
        render_pass.set_bind_group(0, pipeline.rect_pipeline.bind_group(), &[]);
        render_pass.set_vertex_buffer(0, pipeline.rect_pipeline.vertex_buffer().slice(..));
        render_pass.draw(0..pipeline.rect_pipeline.vertex_count(), 0..1);
        true
    }
}

/// HTML/CSS renderer using wgpu
#[derive(Debug, Clone)]
pub struct HtmlRenderer {
    /// Rectangles to render (populated by layout engine)
    rects: Vec<RenderRect>,
}

impl Default for HtmlRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl HtmlRenderer {
    pub fn new() -> Self {
        Self { rects: Vec::new() }
    }

    /// Clear all rectangles
    pub fn clear(&mut self) {
        self.rects.clear();
    }

    /// Set rectangles from a layout tree
    pub fn set_rects(&mut self, rects: Vec<RenderRect>) {
        self.rects = rects;
    }
}

/// State for the shader widget
#[derive(Default)]
pub struct ShaderState;

impl Program<Event> for HtmlRenderer {
    type Primitive = HtmlPrimitive;
    type State = ShaderState;

    fn draw(&self, _state: &Self::State, _cursor: Cursor, bounds: Rectangle) -> Self::Primitive {
        let mut primitive = HtmlPrimitive::new(bounds.width, bounds.height);

        // Copy rectangles to primitive
        for render_rect in &self.rects {
            primitive.push_rect(render_rect.rect, render_rect.background);
        }

        primitive
    }
}

/// Helper function to collect render rects from a layout tree
pub fn collect_rects_from_layout(layout_tree: &LayoutTree) -> Vec<RenderRect> {
    let mut rects = Vec::new();

    fn collect_node(node: &LayoutNode, rects: &mut Vec<RenderRect>) {
        let bg = node.colors.background_color;

        if bg.a > 0.0 {
            rects.push(RenderRect {
                rect: Rect {
                    x: node.dimensions.x,
                    y: node.dimensions.y,
                    width: node.dimensions.width,
                    height: node.dimensions.height,
                },
                background: bg,
            });
        }

        // Recurse into children
        for child in &node.children {
            collect_node(child, rects);
        }
    }

    for root in &layout_tree.root_nodes {
        collect_node(root, &mut rects);
    }

    rects
}

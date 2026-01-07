use std::time::Instant;

use iced::{
    Rectangle,
    mouse::Cursor,
    wgpu::{CommandEncoder, Device, Queue, RenderPass, TextureFormat, TextureView},
    widget::{
        Action,
        shader::{Pipeline, Primitive, Program, Viewport},
    },
};

use crate::core::app::Event;

/// A simple triangle renderer using wgpu
pub struct HtmlRenderer {
    start_time: Instant,
}

impl Default for HtmlRenderer {
    fn default() -> Self {
        Self {
            start_time: Instant::now(),
        }
    }
}

/// The primitive that will be rendered
#[derive(Debug)]
pub struct TrianglePrimitive {
    elapsed_time: f32,
}

/// State for the shader widget
#[derive(Default)]
pub struct ShaderState;

/// Pipeline wrapper that implements iced's Pipeline trait
pub struct HtmlPipeline {
    inner: renderer::TestPipeline,
}

impl Pipeline for HtmlPipeline {
    fn new(device: &Device, _queue: &Queue, format: TextureFormat) -> Self
    where
        Self: Sized,
    {
        Self {
            inner: renderer::TestPipeline::new(device, format),
        }
    }
}

impl Primitive for TrianglePrimitive {
    type Pipeline = HtmlPipeline;

    fn prepare(
        &self,
        pipeline: &mut Self::Pipeline,
        _device: &Device,
        queue: &Queue,
        _bounds: &Rectangle,
        _viewport: &Viewport,
    ) {
        pipeline.inner.update_time(queue, self.elapsed_time);
    }

    fn draw(&self, pipeline: &Self::Pipeline, render_pass: &mut RenderPass<'_>) -> bool {
        render_pass.set_pipeline(pipeline.inner.pipeline());
        render_pass.set_bind_group(0, pipeline.inner.time_bind_group(), &[]);
        render_pass.draw(0..3, 0..1);
        true
    }

    fn render(
        &self,
        _pipeline: &Self::Pipeline,
        _encoder: &mut CommandEncoder,
        _target: &TextureView,
        _clip_bounds: &Rectangle<u32>,
    ) {
        // Rendering is handled in the draw method.
    }
}

impl Program<Event> for HtmlRenderer {
    type Primitive = TrianglePrimitive;
    type State = ShaderState;

    fn draw(&self, _state: &Self::State, _cursor: Cursor, _bounds: Rectangle) -> Self::Primitive {
        let elapsed = self.start_time.elapsed();
        let elapsed_time = elapsed.as_secs_f32();
        TrianglePrimitive { elapsed_time }
    }

    fn update(
        &self,
        _state: &mut Self::State,
        _event: &iced::Event,
        _bounds: Rectangle,
        _cursor: iced::advanced::mouse::Cursor,
    ) -> Option<iced::widget::Action<Event>> {
        Some(Action::request_redraw())
    }
}

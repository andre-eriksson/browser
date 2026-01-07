use std::borrow::Cow;

use assets::{ASSETS, constants::TEST_SHADER};
// Re-export wgpu so consumers use the same version
pub use wgpu;

use wgpu::{
    CommandEncoder, Device, RenderPipeline, ShaderModuleDescriptor, ShaderSource, TextureFormat,
    TextureView,
};

pub struct TestPipeline {
    pipeline: RenderPipeline,
    time_buffer: wgpu::Buffer,
    time_bind_group: wgpu::BindGroup,
}

impl TestPipeline {
    /// Creates a new TestPipeline by loading and compiling the test.wgsl shader
    pub fn new(device: &Device, format: TextureFormat) -> Self {
        let shader_bytes = ASSETS.read().unwrap().load_embedded(TEST_SHADER);
        let shader_str = std::str::from_utf8(&shader_bytes).expect("Shader is not valid UTF-8");
        let shader = shader_str.to_string();

        let shader_module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Test Shader"),
            source: ShaderSource::Wgsl(Cow::Borrowed(&shader)),
        });

        let time_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Time Buffer"),
            size: 32,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Time Bind Group Layout"),
            entries: &[
                // @binding(0) Uniform Buffer for Time
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let time_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Time Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                // @binding(0) Uniform Buffer for Time
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: time_buffer.as_entire_binding(),
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Test Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Test Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        Self {
            pipeline,
            time_buffer,
            time_bind_group,
        }
    }

    pub fn update_time(&self, queue: &wgpu::Queue, time: f32) {
        let time_data = time.to_le_bytes();
        queue.write_buffer(&self.time_buffer, 0, &time_data);
    }

    /// Renders the test triangle
    pub fn render(
        &self,
        encoder: &mut CommandEncoder,
        target: &TextureView,
        clip_bounds: (u32, u32, u32, u32),
    ) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Test Render Pass"),
            color_attachments: &[
                // @location(0) Color Attachment
                Some(wgpu::RenderPassColorAttachment {
                    view: target,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                }),
            ],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_scissor_rect(clip_bounds.0, clip_bounds.1, clip_bounds.2, clip_bounds.3);
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.time_bind_group, &[]);
        render_pass.draw(0..3, 0..1);
    }

    /// Returns a reference to the underlying render pipeline
    pub fn pipeline(&self) -> &RenderPipeline {
        &self.pipeline
    }

    pub fn time_bind_group(&self) -> &wgpu::BindGroup {
        &self.time_bind_group
    }
}

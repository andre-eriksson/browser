use assets::{ASSETS, constants::TEXTURE_SHADER};
use bytemuck::{Pod, Zeroable};
use layout::{Color4f, Rect};
use wgpu::RenderPipeline;

use crate::{globals::Globals2D, vertex::VertexBuffer};

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct TextureVertex {
    /// `@location(0)`
    position: [f32; 2],

    /// `@location(1)`
    uv: [f32; 2],

    /// `@location(2)`
    color: [f32; 4],
}

/// A GPU pipeline for rendering textured quads, text and images using position, uv and color attributes
pub struct TexturePipeline {
    pipeline: RenderPipeline,
    globals: Globals2D,
    vertex_buffer: wgpu::Buffer,
    vertices: Vec<TextureVertex>,
    vertex_count: u32,
    max_vertices: usize,
}

impl TexturePipeline {
    pub const MAX_QUADS: usize = 10_000;

    fn start_pipeline(device: &wgpu::Device, label: &str) -> (Globals2D, wgpu::ShaderModule) {
        let shader_bytes = ASSETS.read().unwrap().load_embedded(TEXTURE_SHADER);
        let shader = std::str::from_utf8(&shader_bytes).expect("Shader is not valid UTF-8");

        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(&format!("{} Shader", label)),
            source: wgpu::ShaderSource::Wgsl(shader.into()),
        });

        let globals = Globals2D::new(device, &format!("{} Pipeline", label));

        (globals, shader_module)
    }

    /// Create a new TexturePipeline for rendering images
    pub fn new_image(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let (globals, shader_module) = Self::start_pipeline(device, "Texture");

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Texture Pipeline Layout"),
            bind_group_layouts: &[&globals.layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Texture Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[TextureVertex::layout()],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                module: &shader_module,
                entry_point: Some("fs_image"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
            cache: None,
        });

        let max_vertices = Self::MAX_QUADS * 6;
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Texture Vertex Buffer"),
            size: (max_vertices * std::mem::size_of::<TextureVertex>()) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            globals,
            vertex_buffer,
            vertices: Vec::with_capacity(max_vertices),
            vertex_count: 0,
            max_vertices,
        }
    }

    /// Create a new TexturePipeline for rendering text
    pub fn new_text(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        atlas_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let (globals, shader_module) = Self::start_pipeline(device, "Text");

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Text Pipeline Layout"),
            bind_group_layouts: &[&globals.layout, atlas_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Text Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[TextureVertex::layout()],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: Some("fs_text"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
            cache: None,
        });

        let max_vertices = Self::MAX_QUADS * 6;
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Text Vertex Buffer"),
            size: (max_vertices * std::mem::size_of::<TextureVertex>()) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            globals,
            vertex_buffer,
            vertices: Vec::with_capacity(max_vertices),
            vertex_count: 0,
            max_vertices,
        }
    }

    pub fn push_quad(&mut self, rect: Rect, uv_rect: Rect, color: Color4f) {
        if self.vertex_count as usize + 6 > self.max_vertices {
            return;
        }

        let x = rect.x;
        let y = rect.y;
        let w = rect.width;
        let h = rect.height;

        let u0 = uv_rect.x;
        let v0 = uv_rect.y;
        let u1 = uv_rect.x + uv_rect.width;
        let v1 = uv_rect.y + uv_rect.height;

        let c = color.to_array();

        let vertices = [
            TextureVertex {
                position: [x, y],
                uv: [u0, v0],
                color: c,
            },
            TextureVertex {
                position: [x + w, y],
                uv: [u1, v0],
                color: c,
            },
            TextureVertex {
                position: [x + w, y + h],
                uv: [u1, v1],
                color: c,
            },
            TextureVertex {
                position: [x, y],
                uv: [u0, v0],
                color: c,
            },
            TextureVertex {
                position: [x + w, y + h],
                uv: [u1, v1],
                color: c,
            },
            TextureVertex {
                position: [x, y + h],
                uv: [u0, v1],
                color: c,
            },
        ];

        self.vertices.extend_from_slice(&vertices);
        self.vertex_count += 6;
    }

    pub fn update_globals(&self, queue: &wgpu::Queue, width: f32, height: f32) {
        self.globals.update(queue, width, height);
    }

    pub fn flush(&mut self, queue: &wgpu::Queue) {
        if self.vertices.is_empty() {
            self.vertex_count = 0;
            return;
        }

        let vertex_data = bytemuck::cast_slice(&self.vertices);
        queue.write_buffer(&self.vertex_buffer, 0, vertex_data);
        self.vertex_count = self.vertices.len() as u32;
    }

    pub fn clear(&mut self) {
        self.vertices.clear();
        self.vertex_count = 0;
    }

    pub fn has_content(&self) -> bool {
        self.vertex_count > 0
    }

    pub fn vertex_buffer(&self) -> &wgpu::Buffer {
        &self.vertex_buffer
    }

    pub fn pipeline(&self) -> &wgpu::RenderPipeline {
        &self.pipeline
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.globals.bind_group
    }

    pub fn vertex_count(&self) -> u32 {
        self.vertex_count
    }
}

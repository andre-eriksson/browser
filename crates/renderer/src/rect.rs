use assets::{ASSETS, constants::SOLID_SHADER};
use bytemuck::{Pod, Zeroable};
use layout::{Color4f, Rect};
use wgpu::{Device, Queue, RenderPipeline, TextureFormat};

use crate::{globals::Globals2D, vertex::VertexBuffer};

/// A single vertex with position and color attributes
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct SolidVertex {
    /// `@location(0)`
    position: [f32; 2],

    /// `@location(1)`
    color: [f32; 4],
}

/// A GPU pipeline for rendering solid-colored rectangles using position and color attributes
pub struct RectPipeline {
    pipeline: RenderPipeline,
    globals: Globals2D,
    vertex_buffer: wgpu::Buffer,
    vertices: Vec<SolidVertex>,
    vertex_count: u32,
    max_vertices: usize,
}

impl RectPipeline {
    pub const MAX_QUADS: usize = 10_000;

    /// Creates a new RectPipeline
    pub fn new(device: &Device, format: TextureFormat) -> Self {
        let shader_bytes = ASSETS.read().unwrap().load_embedded(SOLID_SHADER);
        let shader = std::str::from_utf8(&shader_bytes).expect("Shader is not valid UTF-8");

        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Solid Rect Shader"),
            source: wgpu::ShaderSource::Wgsl(shader.into()),
        });

        let globals = Globals2D::new(device, "Rect Pipeline");

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Rect Pipeline Layout"),
            bind_group_layouts: &[&globals.layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Rect Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[SolidVertex::layout()],
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
                entry_point: Some("fs_main"),
                module: &shader_module,
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
            label: Some("Rect Vertex Buffer"),
            size: (std::mem::size_of::<SolidVertex>() * max_vertices) as wgpu::BufferAddress,
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

    /// Updates the viewport/screen size uniform
    pub fn update_globals(&self, queue: &Queue, width: f32, height: f32) {
        self.globals.update(queue, width, height);
    }

    /// Clears all queued vertices
    pub fn clear(&mut self) {
        self.vertices.clear();
        self.vertex_count = 0;
    }

    /// Pushes a solid-colored rectangle to be rendered
    pub fn push_quad(&mut self, rect: Rect, background: Color4f) {
        if self.vertices.len() + 6 > self.max_vertices {
            eprintln!("RectPipeline: max vertex capacity reached, skipping quad");
            return;
        }

        if background.a <= 0.0 {
            return;
        }

        let x = rect.x;
        let y = rect.y;
        let w = rect.width;
        let h = rect.height;

        let color = background.to_array();

        let quad_vertices = [
            SolidVertex {
                position: [x, y],
                color,
            },
            SolidVertex {
                position: [x + w, y],
                color,
            },
            SolidVertex {
                position: [x, y + h],
                color,
            },
            SolidVertex {
                position: [x + w, y],
                color,
            },
            SolidVertex {
                position: [x + w, y + h],
                color,
            },
            SolidVertex {
                position: [x, y + h],
                color,
            },
        ];

        self.vertices.extend_from_slice(&quad_vertices);
    }

    /// Flushes all queued vertices to the GPU
    pub fn flush(&mut self, queue: &Queue) {
        if self.vertices.is_empty() {
            self.vertex_count = 0;
            return;
        }

        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&self.vertices));
        self.vertex_count = self.vertices.len() as u32;
    }

    /// Returns true if there are vertices to draw
    pub fn has_content(&self) -> bool {
        self.vertex_count > 0
    }

    /// Returns a reference to the render pipeline
    pub fn pipeline(&self) -> &RenderPipeline {
        &self.pipeline
    }

    /// Returns a reference to the globals bind group
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.globals.bind_group
    }

    /// Returns the vertex buffer
    pub fn vertex_buffer(&self) -> &wgpu::Buffer {
        &self.vertex_buffer
    }

    /// Returns the number of vertices to draw
    pub fn vertex_count(&self) -> u32 {
        self.vertex_count
    }
}

/// Data for rectangles to be rendered
#[derive(Debug, Clone)]
pub struct RenderRect {
    pub rect: Rect,
    pub background: Color4f,
}

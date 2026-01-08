use std::borrow::Cow;

use assets::{ASSETS, constants::SOLID_SHADER};
use bytemuck::{Pod, Zeroable};
use layout::{Color4f, Rect};
use wgpu::{Device, Queue, RenderPipeline, TextureFormat};

/// A single vertex with position and color attributes
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    position: [f32; 2], // @location(0)
    color: [f32; 4],    // @location(1)
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![
        0 => Float32x2,  // position
        1 => Float32x4,  // color
    ];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

/// A GPU pipeline for rendering solid-colored rectangles
pub struct RectPipeline {
    pipeline: RenderPipeline,
    global_buffer: wgpu::Buffer,
    global_bind_group: wgpu::BindGroup,
    vertex_buffer: wgpu::Buffer,
    vertices: Vec<Vertex>,
    vertex_count: u32,
    max_vertices: usize,
}

impl RectPipeline {
    /// Creates a new RectPipeline
    pub fn new(device: &Device, format: TextureFormat) -> Self {
        let shader_bytes = ASSETS.read().unwrap().load_embedded(SOLID_SHADER);
        let shader_str = std::str::from_utf8(&shader_bytes).expect("Shader is not valid UTF-8");
        let shader = shader_str.to_string();

        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Solid Rect Shader"),
            source: wgpu::ShaderSource::Wgsl(shader.into()),
        });

        // Global uniform buffer for screen size
        let global_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Globals Buffer"),
            mapped_at_creation: false,
            size: 16, // vec2<f32> screen_size + vec2<f32> padding
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Globals Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                count: None,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
            }],
        });

        let global_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Globals Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: global_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Rect Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Rect Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[Vertex::desc()],
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

        let max_quads = 10_000;
        let max_vertices = max_quads * 6;
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Rect Vertex Buffer"),
            size: (std::mem::size_of::<Vertex>() * max_vertices) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            global_buffer,
            global_bind_group,
            vertex_buffer,
            vertices: Vec::with_capacity(max_vertices),
            vertex_count: 0,
            max_vertices,
        }
    }

    /// Updates the viewport/screen size uniform
    pub fn update_viewport(&self, queue: &Queue, width: f32, height: f32) {
        let viewport_data: [f32; 4] = [width, height, 0.0, 0.0];
        queue.write_buffer(&self.global_buffer, 0, bytemuck::cast_slice(&viewport_data));
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

        // Two triangles to form a quad (counter-clockwise winding)
        let quad_vertices = [
            // Triangle 1
            Vertex {
                position: [x, y],
                color,
            }, // Top-left
            Vertex {
                position: [x + w, y],
                color,
            }, // Top-right
            Vertex {
                position: [x, y + h],
                color,
            }, // Bottom-left
            // Triangle 2
            Vertex {
                position: [x + w, y],
                color,
            }, // Top-right
            Vertex {
                position: [x + w, y + h],
                color,
            }, // Bottom-right
            Vertex {
                position: [x, y + h],
                color,
            }, // Bottom-left
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
        &self.global_bind_group
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

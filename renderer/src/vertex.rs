use crate::{rect::SolidVertex, texture::TextureVertex};

pub trait VertexBuffer {
    fn layout() -> wgpu::VertexBufferLayout<'static>;
}

impl VertexBuffer for SolidVertex {
    fn layout() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBUTES: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![
            0 => Float32x2,  // position
            1 => Float32x4,  // color
        ];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<SolidVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }
}

impl VertexBuffer for TextureVertex {
    fn layout() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBUTES: [wgpu::VertexAttribute; 3] = wgpu::vertex_attr_array![
            0 => Float32x2,  // position
            1 => Float32x2,  // uv
            2 => Float32x4,  // color
        ];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<TextureVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }
}

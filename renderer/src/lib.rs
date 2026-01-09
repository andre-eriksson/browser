//! HTML Renderer Library using wgpu

/// The atlas module handles glyph atlases for text rendering
mod atlas;

/// The globals module manages global uniform buffers and bind groups
mod globals;

/// The pipeline module defines rendering pipelines for rectangles
mod rect;

/// The texture module defines rendering pipelines for textured quads and text
mod texture;

/// The vertex module defines vertex structures and layouts
mod vertex;

pub use atlas::{GlyphAtlas, TextBlockInfo};
pub use rect::{RectPipeline, RenderRect};
pub use texture::TexturePipeline;

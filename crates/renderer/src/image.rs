//! GPU-side image texture management.
//!
//! This module provides types for passing image render information from the
//! layout/collection phase to the GPU pipeline, and a cache for managing
//! per-image wgpu textures and bind groups.

use std::collections::HashMap;
use std::sync::Arc;

use io::MemoryCache;
use layout::Rect;
use serde::{Deserialize, Serialize};
use wgpu;

/// Information needed to render a single image on screen.
///
/// Carried from the render data collection phase (main thread) through
/// `HtmlPrimitive` to the GPU pipeline's `prepare()` method.
#[derive(Debug, Clone)]
pub struct ImageRenderInfo {
    /// The source URL of the image (used as cache key)
    pub src: String,
    /// Screen-space rectangle where the image should be drawn
    pub screen_rect: Rect,
    /// Decoded RGBA image data (width, height, pixels)
    pub data: Arc<DecodedImageData>,
}

/// Decoded RGBA image data ready for GPU upload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecodedImageData {
    /// Raw RGBA pixel data (4 bytes per pixel)
    pub rgba: Vec<u8>,
    /// Image width in pixels
    pub width: u32,
    /// Image height in pixels
    pub height: u32,
}

/// Cache of decoded image data, keyed by source URL.
pub type ImageCache = MemoryCache<String, DecodedImageData>;

/// A single GPU-resident image with its bind group.
struct GpuImage {
    bind_group: wgpu::BindGroup,
}

/// Cache of GPU textures for images, keyed by source URL.
///
/// Lives inside `HtmlPipeline` and persists across frames. New images are
/// uploaded to the GPU on first encounter in `prepare()`, then the cached
/// bind group is reused on subsequent frames.
pub struct GpuImageCache {
    cache: HashMap<String, GpuImage>,
    bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
}

impl GpuImageCache {
    /// Creates a new GPU image cache, including the shared bind group layout
    /// and sampler that will be used for all image textures.
    pub fn new(device: &wgpu::Device) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Image Texture Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Image Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            cache: HashMap::new(),
            bind_group_layout,
            sampler,
        }
    }

    /// Returns the bind group layout used for image textures.
    /// This is needed when creating the image render pipeline.
    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    /// Ensures the given image is uploaded to the GPU and returns its bind group.
    ///
    /// If the image is already cached, returns the existing bind group.
    /// Otherwise, creates a new GPU texture, uploads the RGBA data, and
    /// creates a bind group for it.
    pub fn ensure_uploaded(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        src: &str,
        data: &DecodedImageData,
    ) -> &wgpu::BindGroup {
        if !self.cache.contains_key(src) {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some(&format!("Image Texture: {}", src)),
                size: wgpu::Extent3d {
                    width: data.width,
                    height: data.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });

            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &data.rgba,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * data.width),
                    rows_per_image: Some(data.height),
                },
                wgpu::Extent3d {
                    width: data.width,
                    height: data.height,
                    depth_or_array_layers: 1,
                },
            );

            let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(&format!("Image Bind Group: {}", src)),
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                ],
            });

            self.cache.insert(src.to_string(), GpuImage { bind_group });
        }

        &self.cache[src].bind_group
    }

    /// Returns the bind group for a cached image, if it exists.
    pub fn get_bind_group(&self, src: &str) -> Option<&wgpu::BindGroup> {
        self.cache.get(src).map(|img| &img.bind_group)
    }

    /// Returns true if the given image URL has a GPU texture cached.
    pub fn contains(&self, src: &str) -> bool {
        self.cache.contains_key(src)
    }

    /// Clears all cached GPU textures and bind groups.
    pub fn clear(&mut self) {
        self.cache.clear();
    }
}

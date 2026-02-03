use std::sync::Arc;
use std::{collections::HashMap, fmt::Debug};

use cosmic_text::{Buffer, CacheKey, FontSystem, SwashCache, SwashContent, SwashImage};
use layout::{Color4f, Rect};

/// A region in the atlas where a glyph is stored
#[derive(Debug, Clone, Copy)]
pub struct GlyphRegion {
    /// X position in the atlas texture (pixels)
    pub x: u32,
    /// Y position in the atlas texture (pixels)
    pub y: u32,
    /// Width of the glyph in the atlas (pixels)
    pub width: u32,
    /// Height of the glyph in the atlas (pixels)
    pub height: u32,
    /// Placement offset from swash (left offset from glyph origin)
    pub placement_left: i32,
    /// Placement offset from swash (top offset from glyph origin)
    pub placement_top: i32,
    /// Whether this is a color glyph (emoji)
    pub is_color: bool,
}

impl GlyphRegion {
    /// Calculate UV coordinates for this region in a texture of given size
    pub fn uv_rect(&self, atlas_width: u32, atlas_height: u32) -> Rect {
        Rect::new(
            self.x as f32 / atlas_width as f32,
            self.y as f32 / atlas_height as f32,
            self.width as f32 / atlas_width as f32,
            self.height as f32 / atlas_height as f32,
        )
    }
}

/// Simple row-based atlas packer
struct AtlasPacker {
    cursor_x: u32,
    cursor_y: u32,
    row_height: u32,
    width: u32,
    height: u32,
    padding: u32,
}

impl AtlasPacker {
    fn new(width: u32, height: u32) -> Self {
        Self {
            cursor_x: 0,
            cursor_y: 0,
            row_height: 0,
            width,
            height,
            padding: 1,
        }
    }

    /// Try to allocate a region for a glyph of given size
    fn allocate(&mut self, width: u32, height: u32) -> Option<(u32, u32)> {
        if width == 0 || height == 0 {
            return Some((0, 0));
        }

        let padded_width = width + self.padding;
        let padded_height = height + self.padding;

        if self.cursor_x + padded_width > self.width {
            self.cursor_y += self.row_height + self.padding;
            self.cursor_x = 0;
            self.row_height = 0;
        }

        if self.cursor_y + padded_height > self.height {
            return None;
        }

        let x = self.cursor_x;
        let y = self.cursor_y;

        self.cursor_x += padded_width;
        self.row_height = self.row_height.max(padded_height);

        Some((x, y))
    }
}

/// A GPU-backed texture atlas for caching rasterized glyphs
pub struct GlyphAtlas {
    /// The GPU texture
    texture: wgpu::Texture,
    /// Bind group for the texture
    bind_group: wgpu::BindGroup,
    /// Bind group layout (needed for pipeline creation)
    bind_group_layout: wgpu::BindGroupLayout,
    /// Atlas width
    width: u32,
    /// Atlas height
    height: u32,
    /// Packer for allocating glyph regions
    packer: AtlasPacker,
    /// Cache mapping CacheKey to glyph regions
    glyph_cache: HashMap<CacheKey, GlyphRegion>,
    /// SwashCache for rasterizing glyphs
    swash_cache: SwashCache,
}

impl Debug for GlyphAtlas {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GlyphAtlas")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("glyph_count", &self.glyph_cache.len())
            .finish()
    }
}

impl GlyphAtlas {
    pub const DEFAULT_SIZE: u32 = 2048;

    /// Create a new glyph atlas
    pub fn new(device: &wgpu::Device) -> Self {
        Self::with_size(device, Self::DEFAULT_SIZE, Self::DEFAULT_SIZE)
    }

    /// Create a new glyph atlas with specified dimensions
    pub fn with_size(device: &wgpu::Device, width: u32, height: u32) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Glyph Atlas Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Glyph Atlas Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Glyph Atlas Bind Group Layout"),
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

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Glyph Atlas Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        Self {
            texture,
            bind_group,
            bind_group_layout,
            width,
            height,
            packer: AtlasPacker::new(width, height),
            glyph_cache: HashMap::new(),
            swash_cache: SwashCache::new(),
        }
    }

    /// Get the bind group layout for pipeline creation
    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    /// Get the bind group for rendering
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    /// Get atlas dimensions
    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Upload glyph image data to the atlas texture
    fn upload_glyph_data(
        &self,
        queue: &wgpu::Queue,
        image: &SwashImage,
        atlas_x: u32,
        atlas_y: u32,
    ) {
        let data = match image.content {
            SwashContent::Mask => image.data.as_slice(),
            SwashContent::Color => {
                // TODO: Support color emoji with RGBA atlas
                return;
            }
            SwashContent::SubpixelMask => {
                // TODO: Subpixel rendering not supported yet
                return;
            }
        };

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: atlas_x,
                    y: atlas_y,
                    z: 0,
                },
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(image.placement.width),
                rows_per_image: Some(image.placement.height),
            },
            wgpu::Extent3d {
                width: image.placement.width,
                height: image.placement.height,
                depth_or_array_layers: 1,
            },
        );
    }

    /// Rasterize and cache a glyph, returning its region in the atlas
    pub fn cache_glyph(
        &mut self,
        font_system: &mut FontSystem,
        queue: &wgpu::Queue,
        cache_key: CacheKey,
    ) -> Option<GlyphRegion> {
        // TODO: Cache atlases on disk
        if let Some(region) = self.glyph_cache.get(&cache_key) {
            return Some(*region);
        }

        let image = self
            .swash_cache
            .get_image_uncached(font_system, cache_key)?;

        if image.placement.width == 0 || image.placement.height == 0 {
            let region = GlyphRegion {
                x: 0,
                y: 0,
                width: 0,
                height: 0,
                placement_left: image.placement.left,
                placement_top: image.placement.top,
                is_color: matches!(image.content, SwashContent::Color),
            };

            self.glyph_cache.insert(cache_key, region);

            return Some(region);
        }

        let (atlas_x, atlas_y) = self
            .packer
            .allocate(image.placement.width, image.placement.height)?;

        self.upload_glyph_data(queue, &image, atlas_x, atlas_y);

        let region = GlyphRegion {
            x: atlas_x,
            y: atlas_y,
            width: image.placement.width,
            height: image.placement.height,
            placement_left: image.placement.left,
            placement_top: image.placement.top,
            is_color: matches!(image.content, SwashContent::Color),
        };

        self.glyph_cache.insert(cache_key, region);
        Some(region)
    }
}

/// Information needed to render a single glyph
#[derive(Debug, Clone)]
pub struct GlyphRenderInfo {
    pub cache_key: CacheKey,
    pub x: i32,
    pub y: i32,
    pub text_color: Color4f,
}

/// A text block with all its glyphs ready for rendering
#[derive(Debug, Clone, Default)]
pub struct TextBlockInfo {
    pub glyphs: Vec<GlyphRenderInfo>,
}

impl TextBlockInfo {
    /// Extract glyph render info from a cosmic-text Buffer
    fn from_buffer(buffer: &Buffer, base_x: f32, base_y: f32, text_color: Color4f) -> Self {
        let mut info = Self::default();

        for run in buffer.layout_runs() {
            let line_y = run.line_y;

            for glyph in run.glyphs.iter() {
                let physical = glyph.physical((base_x, base_y), 1.0);

                info.glyphs.push(GlyphRenderInfo {
                    cache_key: physical.cache_key,
                    x: physical.x,
                    y: line_y as i32 + physical.y,
                    text_color,
                });
            }
        }

        info
    }

    /// Extract from an Arc<Buffer>
    pub fn from_arc_buffer(
        buffer: &Arc<Buffer>,
        base_x: f32,
        base_y: f32,
        text_color: Color4f,
    ) -> Self {
        Self::from_buffer(buffer.as_ref(), base_x, base_y, text_color)
    }
}

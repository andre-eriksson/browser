use cosmic_text::{Align, Attrs, Buffer, Family, FontSystem, Metrics, Shaping, Wrap};
use css_style::types::{
    font::{FontFamily, FontFamilyName, GenericName},
    line_height::LineHeight,
};

pub struct TextOffsetContext {
    pub offset_x: f32,
    pub available_width: f32,
}

#[derive(Debug)]
pub struct Text {
    pub width: f32,
    pub last_line_width: f32,
    pub height: f32,
    pub total_width: f32,
    pub buffer: Buffer,
}

/// TextContext provides functionality to measure and render text.
#[derive(Debug)]
pub struct TextContext {
    /// The font system used for text rendering.
    font_system: FontSystem,
}

impl Default for TextContext {
    fn default() -> Self {
        Self {
            font_system: FontSystem::new(),
        }
    }
}

impl TextContext {
    pub fn new(font_system: FontSystem) -> Self {
        Self { font_system }
    }

    /// Get a mutable reference to the font system for glyph rasterization
    pub fn font_system_mut(&mut self) -> &mut FontSystem {
        &mut self.font_system
    }

    /// Measures the rendered size of the given text with specified styles and constraints.
    pub fn measure_text(
        &mut self,
        text: &str,
        font_size_px: f32,
        line_height: &LineHeight,
        font_family: &FontFamily,
        available_width: f32,
    ) -> Text {
        let line_height_px = line_height.to_px(font_size_px);

        let metrics = Metrics::new(font_size_px, line_height_px);
        let mut buffer = Buffer::new(&mut self.font_system, metrics);

        buffer.set_wrap(&mut self.font_system, Wrap::Word);
        buffer.set_size(&mut self.font_system, Some(available_width), None);

        let family = resolve_font_family(font_family);
        let attrs = Attrs::new().family(family);

        if text.trim().is_empty() {
            return Text {
                width: 0.0,
                last_line_width: 0.0,
                height: 0.0,
                total_width: 0.0,
                buffer,
            };
        }

        buffer.set_text(
            &mut self.font_system,
            text,
            &attrs,
            Shaping::Advanced,
            Some(Align::Left),
        );

        buffer.shape_until_scroll(&mut self.font_system, false);

        let mut max_width: f32 = 0.0;
        let mut last_line_width: f32 = 0.0;
        let mut line_count: usize = 0;

        for run in buffer.layout_runs() {
            max_width = max_width.max(run.line_w);
            last_line_width = run.line_w;
            line_count += 1;
        }

        let total_height = if line_count > 0 {
            line_count as f32 * line_height_px
        } else if !text.is_empty() {
            line_height_px
        } else {
            0.0
        };

        Text {
            width: max_width,
            last_line_width,
            height: total_height,
            total_width: max_width,
            buffer,
        }
    }
}

fn resolve_font_family(font_family: &FontFamily) -> Family<'_> {
    for name in &font_family.names {
        match name {
            FontFamilyName::Generic(generic) => {
                return match generic {
                    GenericName::Serif => Family::Serif,
                    GenericName::SansSerif => Family::SansSerif,
                    GenericName::Monospace => Family::Monospace,
                    GenericName::Cursive => Family::Cursive,
                    GenericName::Fantasy => Family::Fantasy,
                    _ => Family::SansSerif,
                };
            }
            FontFamilyName::Specific(_name) => {
                continue; // Specific font handling would go here
            }
        }
    }

    Family::SansSerif
}

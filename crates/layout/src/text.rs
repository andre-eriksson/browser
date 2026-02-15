use cosmic_text::{
    Align, Attrs, Buffer, Family, FontSystem, Metrics, Shaping, Stretch, Weight, Wrap,
};
use css_style::{FontFamily, FontFamilyName, Whitespace, font::GenericName};

#[derive(Debug)]
pub struct Text {
    pub width: f32,
    pub last_line_width: f32,
    pub height: f32,
    pub total_width: f32,
    pub buffer: Buffer,
}

pub struct TextDescription<'a> {
    pub whitespace: &'a Whitespace,
    pub line_height: f32,
    pub font_family: &'a FontFamily,
    pub font_weight: u16,
    pub font_size_px: f32,
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

    pub fn measure_text_that_fits<'a>(
        &mut self,
        text: &'a str,
        text_description: &TextDescription,
        max_width: f32,
    ) -> (Text, Option<&'a str>) {
        let line_height_px = text_description.line_height;

        let metrics = Metrics::new(text_description.font_size_px, line_height_px);
        let family = Self::resolve_font_family(text_description.font_family);
        let weight = Self::resolve_font_weight(text_description.font_weight);
        let attrs = Attrs::new()
            .family(family)
            .weight(weight)
            .stretch(Stretch::Normal);

        let wrap_mode = match text_description.whitespace {
            Whitespace::Normal | Whitespace::PreLine | Whitespace::PreWrap => Wrap::Word,
            Whitespace::Pre => Wrap::None,
            _ => Wrap::Word,
        };

        let mut temp_buffer = Buffer::new(&mut self.font_system, metrics);

        temp_buffer.set_size(&mut self.font_system, Some(max_width), None);
        temp_buffer.set_wrap(&mut self.font_system, wrap_mode);

        temp_buffer.set_text(
            &mut self.font_system,
            text,
            &attrs,
            Shaping::Advanced,
            Some(Align::Left),
        );

        temp_buffer.shape_until_scroll(&mut self.font_system, false);

        let runs: Vec<_> = temp_buffer.layout_runs().collect();

        if runs.is_empty() {
            return (
                self.measure_text(text, text_description, max_width, wrap_mode),
                None,
            );
        }

        if runs.len() > 1 {
            let first_run = &runs[0];

            let split_index = first_run.glyphs.last().map(|g| g.end).unwrap_or(text.len());

            let split_index = split_index.min(text.len());

            let fitted_text = &text[..split_index];
            let remaining_text = &text[split_index..];

            return (
                self.measure_text(fitted_text, text_description, max_width, wrap_mode),
                Some(remaining_text),
            );
        }

        (
            self.measure_text(text, text_description, max_width, wrap_mode),
            None,
        )
    }

    /// Measures the rendered size of the given text with specified styles and constraints.
    fn measure_text(
        &mut self,
        text: &str,
        text_description: &TextDescription,
        available_width: f32,
        wrap_mode: Wrap,
    ) -> Text {
        let line_height_px = text_description.line_height;

        let metrics = Metrics::new(text_description.font_size_px, line_height_px);
        let mut buffer = Buffer::new(&mut self.font_system, metrics);

        buffer.set_wrap(&mut self.font_system, wrap_mode);
        buffer.set_size(&mut self.font_system, Some(available_width), None);

        let family = Self::resolve_font_family(text_description.font_family);
        let weight = Self::resolve_font_weight(text_description.font_weight);

        let attrs = Attrs::new()
            .family(family)
            .weight(weight)
            .stretch(Stretch::Normal);

        let preserve_whitespace = matches!(
            text_description.whitespace,
            Whitespace::Pre | Whitespace::PreWrap
        );

        let is_whitespace_only = text.trim().is_empty() && !preserve_whitespace;

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
            let w = if is_whitespace_only {
                // For whitespace-only text, `line_w` excludes trailing
                // whitespace so it reports 0.  Use glyph advances instead
                // to obtain the real space width.
                run.glyphs.last().map(|g| g.x + g.w).unwrap_or(0.0)
            } else {
                run.line_w
            };
            max_width = max_width.max(w);
            last_line_width = w;
            line_count += 1;
        }

        // Fallback: if cosmic-text produced no layout runs for the
        // whitespace (or all glyph advances were zero), approximate the
        // space width from the font size (~0.25 em is a reasonable default
        // for most fonts) so that inter-element spacing is preserved.
        if is_whitespace_only && max_width == 0.0 {
            let space_count = text.chars().filter(|c| *c == ' ').count().max(1);
            max_width = space_count as f32 * text_description.font_size_px * 0.25;
            last_line_width = max_width;
            if line_count == 0 {
                line_count = 1;
            }
        }

        let mut total_height = line_count as f32 * line_height_px;

        if preserve_whitespace && text.ends_with('\n') {
            total_height += line_height_px;
            last_line_width = 0.0;
        }

        Text {
            width: max_width,
            last_line_width,
            height: total_height,
            total_width: max_width,
            buffer,
        }
    }

    fn resolve_font_family(font_family: &FontFamily) -> Family<'_> {
        match &font_family.names()[0] {
            FontFamilyName::Generic(generic) => match generic {
                GenericName::Serif => Family::Serif,
                GenericName::SansSerif => Family::SansSerif,
                GenericName::Monospace => Family::Monospace,
                GenericName::Cursive => Family::Cursive,
                GenericName::Fantasy => Family::Fantasy,
                _ => Family::SansSerif,
            },
            FontFamilyName::Specific(name) => Family::Name(name.as_str()),
        }
    }

    fn resolve_font_weight(font_weight: u16) -> Weight {
        match font_weight {
            100 => Weight::THIN,
            200 => Weight::EXTRA_LIGHT,
            300 => Weight::LIGHT,
            400 => Weight::NORMAL,
            500 => Weight::MEDIUM,
            600 => Weight::SEMIBOLD,
            700 => Weight::BOLD,
            800 => Weight::EXTRA_BOLD,
            900 => Weight::BLACK,
            _ => Weight::NORMAL,
        }
    }
}

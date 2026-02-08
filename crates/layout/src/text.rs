use cosmic_text::{
    Align, Attrs, Buffer, Family, FontSystem, Metrics, Shaping, Stretch, Weight, Wrap,
};
use css_style::{
    AbsoluteContext, FontFamily, FontFamilyName, FontWeight, LineHeight, Whitespace,
    font::GenericName,
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

pub struct TextDescription<'a> {
    pub whitespace: &'a Whitespace,
    pub line_height: &'a LineHeight,
    pub font_family: &'a FontFamily,
    pub font_weight: &'a FontWeight,
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

    pub fn measure_multiline_text(
        &mut self,
        absolute_ctx: &AbsoluteContext,
        text: &str,
        text_description: &TextDescription,
        available_width: f32,
        offset_ctx: TextOffsetContext,
    ) -> (Text, Option<Text>) {
        let wrap_mode = match text_description.whitespace {
            Whitespace::Normal | Whitespace::PreLine | Whitespace::PreWrap => Wrap::Word,
            Whitespace::Pre => Wrap::None,
            _ => Wrap::Word,
        };

        let is_pre = matches!(text_description.whitespace, Whitespace::Pre);

        if offset_ctx.offset_x == 0.0 || is_pre {
            return (
                self.measure_text(
                    absolute_ctx,
                    text,
                    text_description,
                    available_width,
                    wrap_mode,
                ),
                None,
            );
        }

        let line_height_px = text_description
            .line_height
            .to_px(absolute_ctx, text_description.font_size_px);

        let metrics = Metrics::new(text_description.font_size_px, line_height_px);
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
        if text.trim().is_empty() && !preserve_whitespace {
            let buffer = Buffer::new(&mut self.font_system, metrics);
            return (
                Text {
                    width: 0.0,
                    last_line_width: 0.0,
                    height: 0.0,
                    total_width: 0.0,
                    buffer,
                },
                None,
            );
        }

        let mut temp_buffer = Buffer::new(&mut self.font_system, metrics);
        temp_buffer.set_wrap(&mut self.font_system, wrap_mode);
        temp_buffer.set_size(
            &mut self.font_system,
            Some(offset_ctx.available_width),
            None,
        );
        temp_buffer.set_text(
            &mut self.font_system,
            text,
            &attrs,
            Shaping::Advanced,
            Some(Align::Left),
        );
        temp_buffer.shape_until_scroll(&mut self.font_system, false);

        let first_line_end = if let Some(run) = temp_buffer.layout_runs().next() {
            run.glyphs.last().map(|g| g.end).unwrap_or(0)
        } else if preserve_whitespace && text.starts_with('\n') {
            1
        } else {
            return (
                self.measure_text(
                    absolute_ctx,
                    text,
                    text_description,
                    available_width,
                    wrap_mode,
                ),
                None,
            );
        };

        let first_line_text = &text[..first_line_end];
        let remaining_text = match text_description.whitespace {
            Whitespace::Normal | Whitespace::PreLine => text[first_line_end..].trim_start(),
            Whitespace::Pre | Whitespace::PreWrap => &text[first_line_end..],
            _ => &text[first_line_end..],
        };

        let initial_text = self.measure_text(
            absolute_ctx,
            first_line_text,
            text_description,
            offset_ctx.available_width,
            wrap_mode,
        );

        if remaining_text.is_empty() {
            return (initial_text, None);
        }

        let rest_text = self.measure_text(
            absolute_ctx,
            remaining_text,
            text_description,
            available_width,
            wrap_mode,
        );

        (initial_text, Some(rest_text))
    }

    /// Measures the rendered size of the given text with specified styles and constraints.
    fn measure_text(
        &mut self,
        absolute_ctx: &AbsoluteContext,
        text: &str,
        text_description: &TextDescription,
        available_width: f32,
        wrap_mode: Wrap,
    ) -> Text {
        let line_height_px = text_description
            .line_height
            .to_px(absolute_ctx, text_description.font_size_px);

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

        if text.trim().is_empty() && !preserve_whitespace {
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

    fn resolve_font_weight(font_weight: &FontWeight) -> Weight {
        match font_weight {
            FontWeight::Thin => Weight::THIN,
            FontWeight::ExtraLight => Weight::EXTRA_LIGHT,
            FontWeight::Light => Weight::LIGHT,
            FontWeight::Normal => Weight::NORMAL,
            FontWeight::Medium => Weight::MEDIUM,
            FontWeight::SemiBold => Weight::SEMIBOLD,
            FontWeight::Bold => Weight::BOLD,
            FontWeight::ExtraBold => Weight::EXTRA_BOLD,
            FontWeight::Black => Weight::BLACK,
        }
    }
}

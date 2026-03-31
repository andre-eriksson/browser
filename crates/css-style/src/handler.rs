use css_cssom::{ComponentValue, ComponentValueStream, CssTokenKind, HashType};
use css_values::{
    CSSParsable,
    background::{Attachment, BgClip, RepeatStyle, Size, VisualBox, WidthHeightSize},
    border::{BorderStyle, BorderWidth},
    color::{Color, base::ColorBase},
    combination::LengthPercentage,
    error::CssValueError,
    global::Global,
    image::Image,
    numeric::Percentage,
    position::BgPosition,
    quantity::{Length, LengthUnit},
    text::{FontWeight, WritingMode},
};
use tracing::trace;

use crate::{
    AbsoluteContext, RelativeContext, RelativeType,
    properties::{
        CSSProperty, PixelRepr,
        background::{
            BackgroundAttachment, BackgroundClip, BackgroundImage, BackgroundOrigin, BackgroundPosition,
            BackgroundPositionX, BackgroundPositionY, BackgroundRepeat, BackgroundSize,
        },
        offset::Offset,
    },
    specified::SpecifiedStyle,
};

/// Context for updating a CSS property, containing necessary information and utilities for the update process.
pub(crate) struct PropertyUpdateContext<'css> {
    pub absolute_ctx: &'css AbsoluteContext<'css>,
    pub specified_style: &'css mut SpecifiedStyle,
    pub relative_ctx: &'css RelativeContext,
    pub errors: Vec<PropertyError>,
}

/// Represents an error that occurred during the property update process, including the property name, the value that caused the error, and a descriptive error message.
#[derive(Debug)]
pub(crate) struct PropertyError {
    pub property: String,
    pub value: String,
    pub error: CssValueError,
}

impl<'css> PropertyUpdateContext<'css> {
    pub fn new(
        absolute_ctx: &'css AbsoluteContext,
        specified_style: &'css mut SpecifiedStyle,
        relative_ctx: &'css RelativeContext,
    ) -> Self {
        Self {
            absolute_ctx,
            specified_style,
            relative_ctx,
            errors: Vec::new(),
        }
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    fn record_error(&mut self, property: &str, value: String, error: CssValueError) {
        self.errors.push(PropertyError {
            property: property.to_string(),
            value,
            error,
        });
    }

    fn record_error_from_stream(&mut self, property: &str, stream: &ComponentValueStream, error: CssValueError) {
        let value = stream
            .values()
            .iter()
            .map(|cv| cv.to_string())
            .collect::<String>();

        self.record_error(property, value, error);
    }

    /// Resolves the current writing mode from the specified style, falling back to the parent's
    /// writing mode or the initial value (`HorizontalTb`) if the property is set to a global value.
    fn resolve_writing_mode(&self) -> WritingMode {
        match self.specified_style.writing_mode {
            CSSProperty::Value(val) => val,
            CSSProperty::Global(_) => self
                .specified_style
                .writing_mode
                .compute(self.relative_ctx.parent.writing_mode),
        }
    }

    pub fn log_errors(&self) {
        if !self.errors.is_empty() {
            trace!("Property update errors:");

            for err in &self.errors {
                trace!("  {}: '{}' - {}", err.property, err.value, err.error);
            }
        }
    }
}

/// A macro to generate simple property handler functions that update a specific field in the specified style based on the provided component values.
/// The macro takes the function name, the field to update, and the property name for error reporting.
macro_rules! simple_property_handler {
    ($fn_name:ident, $field:ident, $prop_name:expr) => {
        pub fn $fn_name(ctx: &mut PropertyUpdateContext, stream: &mut ComponentValueStream) {
            if let Err(e) = CSSProperty::update_property(&mut ctx.specified_style.$field, stream) {
                ctx.record_error_from_stream($prop_name, stream, e);
            }
        }
    };
}

/// A macro to generate shorthand property handlers for 4-side offset properties (e.g. `margin`, `padding`).
/// Parses the value once as either a `Global` or an `Offset`, then assigns to all four physical side fields.
macro_rules! offset_shorthand_handler {
    ($fn_name:ident, $prop_name:expr, $top:ident, $right:ident, $bottom:ident, $left:ident) => {
        pub(crate) fn $fn_name(ctx: &mut PropertyUpdateContext, stream: &mut ComponentValueStream) {
            let checkpoint = stream.checkpoint();

            if let Ok(global) = Global::parse(stream) {
                ctx.specified_style.$top = CSSProperty::Global(global);
                ctx.specified_style.$right = CSSProperty::Global(global);
                ctx.specified_style.$bottom = CSSProperty::Global(global);
                ctx.specified_style.$left = CSSProperty::Global(global);
            } else {
                stream.restore(checkpoint);
                if let Ok(offset) = Offset::parse(stream) {
                    ctx.specified_style.$top = offset.top.into();
                    ctx.specified_style.$right = offset.right.into();
                    ctx.specified_style.$bottom = offset.bottom.into();
                    ctx.specified_style.$left = offset.left.into();
                } else {
                    stream.restore(checkpoint);
                    ctx.record_error_from_stream(
                        $prop_name,
                        stream,
                        CssValueError::InvalidValue(format!("Invalid value for {} property", $prop_name)),
                    );
                }
            }
        }
    };
}

/// A macro to generate writing-mode-aware logical block pair handlers (e.g. `margin-block`, `padding-block`).
/// Parses the value once as either a `Global` or an `Offset`, then assigns to the two physical fields
/// that correspond to the block axis based on the resolved writing mode.
///
/// Arguments: `(fn_name, prop_name, htb_start, htb_end, vrl_start, vrl_end, vlr_start, vlr_end)`
/// where each `*_start`/`*_end` is a field on `SpecifiedStyle`.
macro_rules! logical_pair_handler {
    ($fn_name:ident, $prop_name:expr,
     $htb_start:ident, $htb_end:ident,
     $vrl_start:ident, $vrl_end:ident,
     $vlr_start:ident, $vlr_end:ident) => {
        pub(crate) fn $fn_name(ctx: &mut PropertyUpdateContext, stream: &mut ComponentValueStream) {
            let checkpoint = stream.checkpoint();
            let global = Global::parse(stream).ok();

            let offset = if global.is_none() {
                stream.restore(checkpoint);
                Offset::parse(stream).ok()
            } else {
                None
            };

            let (start, end) = match ctx.resolve_writing_mode() {
                WritingMode::HorizontalTb => (&mut ctx.specified_style.$htb_start, &mut ctx.specified_style.$htb_end),
                WritingMode::VerticalRl => (&mut ctx.specified_style.$vrl_start, &mut ctx.specified_style.$vrl_end),
                WritingMode::VerticalLr => (&mut ctx.specified_style.$vlr_start, &mut ctx.specified_style.$vlr_end),
                _ => {
                    stream.restore(checkpoint);
                    ctx.record_error_from_stream(
                        $prop_name,
                        stream,
                        CssValueError::InvalidValue("Unsupported writing mode".into()),
                    );
                    return;
                }
            };

            if let Some(global) = global {
                *start = CSSProperty::Global(global);
                *end = CSSProperty::Global(global);
            } else if let Some(offset) = offset {
                *start = offset.top.into();
                *end = offset.bottom.into();
            }
        }
    };
}

/// A macro to generate writing-mode-aware logical edge handlers (e.g. `margin-block-start`, `padding-block-end`).
/// Parses the value once, then assigns to the single physical field that corresponds to the given
/// logical edge based on the resolved writing mode.
///
/// Arguments: `(fn_name, prop_name, htb_field, vrl_field, vlr_field)`
/// where each `*_field` is a field on `SpecifiedStyle`.
macro_rules! logical_edge_handler {
    ($fn_name:ident, $prop_name:expr, $htb:ident, $vrl:ident, $vlr:ident) => {
        pub(crate) fn $fn_name(ctx: &mut PropertyUpdateContext, stream: &mut ComponentValueStream) {
            let field = match ctx.resolve_writing_mode() {
                WritingMode::HorizontalTb => &mut ctx.specified_style.$htb,
                WritingMode::VerticalRl => &mut ctx.specified_style.$vrl,
                WritingMode::VerticalLr => &mut ctx.specified_style.$vlr,
                _ => {
                    ctx.record_error_from_stream(
                        $prop_name,
                        stream,
                        CssValueError::InvalidValue("Unsupported writing mode".into()),
                    );
                    return;
                }
            };

            if let Err(e) = CSSProperty::update_property(field, stream) {
                ctx.record_error_from_stream($prop_name, stream, e);
            }
        }
    };
}

offset_shorthand_handler!(handle_margin, "margin", margin_top, margin_right, margin_bottom, margin_left);
offset_shorthand_handler!(handle_padding, "padding", padding_top, padding_right, padding_bottom, padding_left);
simple_property_handler!(handle_background_attachment, background_attachment, "background-attachment");
simple_property_handler!(handle_background_blend_mode, background_blend_mode, "background-blend-mode");
simple_property_handler!(handle_background_clip, background_clip, "background-clip");
simple_property_handler!(handle_background_color, background_color, "background-color");
simple_property_handler!(handle_background_image, background_image, "background-image");
simple_property_handler!(handle_background_origin, background_origin, "background-origin");
simple_property_handler!(handle_background_position_x, background_position_x, "background-position-x");
simple_property_handler!(handle_background_position_y, background_position_y, "background-position-y");
simple_property_handler!(handle_background_repeat, background_repeat, "background-repeat");
simple_property_handler!(handle_background_size, background_size, "background-size");
simple_property_handler!(handle_border_bottom_color, border_bottom_color, "border-bottom-color");
simple_property_handler!(handle_border_bottom_style, border_bottom_style, "border-bottom-style");
simple_property_handler!(handle_border_bottom_width, border_bottom_width, "border-bottom-width");
simple_property_handler!(handle_border_left_color, border_left_color, "border-left-color");
simple_property_handler!(handle_border_left_style, border_left_style, "border-left-style");
simple_property_handler!(handle_border_left_width, border_left_width, "border-left-width");
simple_property_handler!(handle_border_right_color, border_right_color, "border-right-color");
simple_property_handler!(handle_border_right_style, border_right_style, "border-right-style");
simple_property_handler!(handle_border_right_width, border_right_width, "border-right-width");
simple_property_handler!(handle_border_top_color, border_top_color, "border-top-color");
simple_property_handler!(handle_border_top_style, border_top_style, "border-top-style");
simple_property_handler!(handle_border_top_width, border_top_width, "border-top-width");
simple_property_handler!(handle_bottom, bottom, "bottom");
simple_property_handler!(handle_clear, clear, "clear");
simple_property_handler!(handle_color, color, "color");
simple_property_handler!(handle_cursor, cursor, "cursor");
simple_property_handler!(handle_display, display, "display");
simple_property_handler!(handle_float, float, "float");
simple_property_handler!(handle_font_family, font_family, "font-family");
simple_property_handler!(handle_height, height, "height");
simple_property_handler!(handle_left, left, "left");
simple_property_handler!(handle_line_height, line_height, "line-height");
simple_property_handler!(handle_margin_bottom, margin_bottom, "margin-bottom");
simple_property_handler!(handle_margin_left, margin_left, "margin-left");
simple_property_handler!(handle_margin_right, margin_right, "margin-right");
simple_property_handler!(handle_margin_top, margin_top, "margin-top");
simple_property_handler!(handle_max_height, max_height, "max-height");
simple_property_handler!(handle_max_width, max_width, "max-width");
simple_property_handler!(handle_padding_bottom, padding_bottom, "padding-bottom");
simple_property_handler!(handle_padding_left, padding_left, "padding-left");
simple_property_handler!(handle_padding_right, padding_right, "padding-right");
simple_property_handler!(handle_padding_top, padding_top, "padding-top");
simple_property_handler!(handle_position, position, "position");
simple_property_handler!(handle_right, right, "right");
simple_property_handler!(handle_text_align, text_align, "text-align");
simple_property_handler!(handle_top, top, "top");
simple_property_handler!(handle_whitespace, whitespace, "white-space");
simple_property_handler!(handle_width, width, "width");
simple_property_handler!(handle_writing_mode, writing_mode, "writing-mode");
logical_pair_handler!(
    handle_margin_block,
    "margin-block",
    margin_top,
    margin_bottom,
    margin_right,
    margin_left,
    margin_left,
    margin_right
);
logical_pair_handler!(
    handle_padding_block,
    "padding-block",
    padding_top,
    padding_bottom,
    padding_right,
    padding_left,
    padding_left,
    padding_right
);
logical_pair_handler!(
    handle_margin_inline,
    "margin-inline",
    margin_left,
    margin_right,
    margin_top,
    margin_bottom,
    margin_top,
    margin_bottom
);
logical_pair_handler!(
    handle_padding_inline,
    "padding-inline",
    padding_left,
    padding_right,
    padding_top,
    padding_bottom,
    padding_top,
    padding_bottom
);
logical_edge_handler!(handle_margin_block_start, "margin-block-start", margin_top, margin_right, margin_left);
logical_edge_handler!(handle_margin_block_end, "margin-block-end", margin_bottom, margin_left, margin_right);
logical_edge_handler!(handle_padding_block_start, "padding-block-start", padding_top, padding_right, padding_left);
logical_edge_handler!(handle_padding_block_end, "padding-block-end", padding_bottom, padding_left, padding_right);
logical_edge_handler!(handle_margin_inline_start, "margin-inline-start", margin_left, margin_top, margin_top);
logical_edge_handler!(handle_margin_inline_end, "margin-inline-end", margin_right, margin_bottom, margin_bottom);
logical_edge_handler!(handle_padding_inline_start, "padding-inline-start", padding_left, padding_top, padding_top);
logical_edge_handler!(handle_padding_inline_end, "padding-inline-end", padding_right, padding_bottom, padding_bottom);

pub(crate) fn handle_background_position(ctx: &mut PropertyUpdateContext, stream: &mut ComponentValueStream) {
    let checkpoint = stream.checkpoint();

    if let Ok(global) = Global::parse(stream) {
        ctx.specified_style.background_position_x = CSSProperty::Global(global);
        ctx.specified_style.background_position_y = CSSProperty::Global(global);
        return;
    }

    stream.restore(checkpoint);

    let writing_mode = ctx.resolve_writing_mode();

    let mut x_pos = Vec::new();
    let mut y_pos = Vec::new();

    stream.skip_whitespace();
    match BackgroundPosition::parse(stream) {
        Ok(bg_position) => {
            for position in bg_position.0 {
                BackgroundPosition::resolve_bg_position_layer(position, writing_mode, &mut x_pos, &mut y_pos);
            }
        }

        Err(e) => {
            ctx.record_error_from_stream(
                "background-position",
                stream,
                CssValueError::InvalidValue(format!("Invalid value for background-position: {}", e)),
            );
        }
    }

    if !x_pos.is_empty() {
        ctx.specified_style.background_position_x = CSSProperty::Value(BackgroundPositionX(x_pos));
    }

    if !y_pos.is_empty() {
        ctx.specified_style.background_position_y = CSSProperty::Value(BackgroundPositionY(y_pos));
    }
}

/// Handles the `background` shorthand property.
///
/// CSS grammar:
/// background =
/// <bg-layer>#? , <final-bg-layer>
///
/// <bg-layer> =
///   <bg-image>                      ||
///   <bg-position> [ / <bg-size> ]?  ||
///   <repeat-style>                  ||
///   <attachment>                    ||
///   <bg-clip>                       ||
///   <visual-box>
///
/// <final-bg-layer> =
///   <bg-image>                      ||
///   <bg-position> [ / <bg-size> ]?  ||
///   <repeat-style>                  ||
///   <attachment>                    ||
///   <bg-clip>                       ||
///   <visual-box>                    ||
///   <'background-color'>
pub(crate) fn handle_background(ctx: &mut PropertyUpdateContext, stream: &mut ComponentValueStream) {
    let checkpoint = stream.checkpoint();

    if let Ok(global) = Global::parse(stream) {
        ctx.specified_style.background_attachment = CSSProperty::Global(global);
        ctx.specified_style.background_clip = CSSProperty::Global(global);
        ctx.specified_style.background_color = CSSProperty::Global(global);
        ctx.specified_style.background_image = CSSProperty::Global(global);
        ctx.specified_style.background_origin = CSSProperty::Global(global);
        ctx.specified_style.background_repeat = CSSProperty::Global(global);
        ctx.specified_style.background_position_x = CSSProperty::Global(global);
        ctx.specified_style.background_position_y = CSSProperty::Global(global);
        ctx.specified_style.background_size = CSSProperty::Global(global);
        return;
    }

    stream.restore(checkpoint);

    let cp = stream.checkpoint();
    stream.skip_whitespace();
    if let Some(ComponentValue::Token(t)) = stream.next_non_whitespace()
        && matches!(&t.kind, CssTokenKind::Ident(s) if s.eq_ignore_ascii_case("none"))
    {
        stream.skip_whitespace();
        if stream.peek().is_none() {
            ctx.specified_style.background_attachment = CSSProperty::Global(Global::Initial);
            ctx.specified_style.background_clip = CSSProperty::Global(Global::Initial);
            ctx.specified_style.background_color = CSSProperty::Global(Global::Initial);
            ctx.specified_style.background_image = CSSProperty::Global(Global::Initial);
            ctx.specified_style.background_origin = CSSProperty::Global(Global::Initial);
            ctx.specified_style.background_repeat = CSSProperty::Global(Global::Initial);
            ctx.specified_style.background_position_x = CSSProperty::Global(Global::Initial);
            ctx.specified_style.background_position_y = CSSProperty::Global(Global::Initial);
            ctx.specified_style.background_size = CSSProperty::Global(Global::Initial);
            return;
        }
    }
    stream.restore(cp);

    let writing_mode = ctx.resolve_writing_mode();

    let mut images = Vec::new();
    let mut attachments = Vec::new();
    let mut repeats = Vec::new();
    let mut origins = Vec::new();
    let mut clips = Vec::new();
    let mut sizes = Vec::new();
    let mut x_positions = Vec::new();
    let mut y_positions = Vec::new();
    let mut final_color = Color::Base(ColorBase::Transparent);

    /// Try to parse a single `Size` value (1–2 tokens: cover | contain | auto | <length-percentage> ){1,2}
    /// directly from the stream. Returns `None` and restores on failure.
    fn try_parse_single_size(stream: &mut ComponentValueStream) -> Option<Size> {
        let cp = stream.checkpoint();

        fn parse_width_height_token(kind: &CssTokenKind) -> Option<WidthHeightSize> {
            match kind {
                CssTokenKind::Ident(s) if s.eq_ignore_ascii_case("auto") => Some(WidthHeightSize::Auto),
                CssTokenKind::Dimension { value, unit } => {
                    let len_unit = unit.parse::<LengthUnit>().ok()?;
                    Some(WidthHeightSize::Length(LengthPercentage::Length(Length::new(
                        value.to_f64() as f32,
                        len_unit,
                    ))))
                }
                CssTokenKind::Percentage(pct) => {
                    Some(WidthHeightSize::Length(LengthPercentage::Percentage(Percentage::new(pct.to_f64() as f32))))
                }
                CssTokenKind::Number(n) if n.to_f64() == 0.0 => {
                    Some(WidthHeightSize::Length(LengthPercentage::Length(Length::new(0.0, LengthUnit::Px))))
                }
                _ => None,
            }
        }

        if let Some(ComponentValue::Token(t)) = stream.next_non_whitespace() {
            if let CssTokenKind::Ident(s) = &t.kind {
                if s.eq_ignore_ascii_case("cover") {
                    return Some(Size::Cover);
                }
                if s.eq_ignore_ascii_case("contain") {
                    return Some(Size::Contain);
                }
            }
            if let Some(w) = parse_width_height_token(&t.kind) {
                let cp2 = stream.checkpoint();
                if let Some(ComponentValue::Token(t2)) = stream.next_non_whitespace()
                    && let Some(h) = parse_width_height_token(&t2.kind)
                {
                    return Some(Size::WidthHeight(w, Some(h)));
                }
                stream.restore(cp2);
                return Some(Size::WidthHeight(w, None));
            }
        }

        stream.restore(cp);
        None
    }

    let mut done = false;

    while !done {
        let mut layer_image: Option<Image> = None;
        let mut layer_attachment: Option<Attachment> = None;
        let mut layer_repeat_h: Option<RepeatStyle> = None;
        let mut layer_repeat_v: Option<RepeatStyle> = None;
        let mut layer_origin: Option<VisualBox> = None;
        let mut layer_clip: Option<VisualBox> = None;
        let mut layer_size: Option<Size> = None;
        let mut layer_position: Option<BgPosition> = None;
        let mut layer_color: Option<Color> = None;

        loop {
            stream.skip_whitespace();
            if stream.peek().is_none() {
                done = true;
                break;
            }

            if let Some(ComponentValue::Token(t)) = stream.peek()
                && matches!(t.kind, CssTokenKind::Comma)
            {
                stream.next_cv();
                break;
            }

            if layer_position.is_none() {
                let cp = stream.checkpoint();
                if let Ok(pos) = BgPosition::parse(stream) {
                    layer_position = Some(pos);

                    let cp_slash = stream.checkpoint();
                    stream.skip_whitespace();
                    if let Some(ComponentValue::Token(t)) = stream.peek()
                        && matches!(t.kind, CssTokenKind::Delim('/'))
                    {
                        stream.next_cv();
                        if let Some(sz) = try_parse_single_size(stream) {
                            layer_size = Some(sz);
                        } else {
                            stream.restore(cp_slash);
                        }
                    }
                    continue;
                }
                stream.restore(cp);
            }

            enum BgToken {
                Ident(String),
                Url(String),
                Hash(String),
                Function(css_cssom::Function),
                Other,
            }

            let bg_token = match stream.next_non_whitespace() {
                Some(ComponentValue::Token(t)) => match &t.kind {
                    CssTokenKind::Ident(s) => BgToken::Ident(s.clone()),
                    CssTokenKind::Url(u) => BgToken::Url(u.clone()),
                    CssTokenKind::Hash { value, .. } => BgToken::Hash(value.clone()),
                    _ => BgToken::Other,
                },
                Some(ComponentValue::Function(f)) => BgToken::Function(f.clone()),
                Some(_) => BgToken::Other,
                None => {
                    done = true;
                    break;
                }
            };

            match bg_token {
                BgToken::Function(func) => {
                    if layer_image.is_none()
                        && let Ok(img) = Image::try_from(&func)
                    {
                        layer_image = Some(img);
                        continue;
                    }
                    if layer_color.is_none() {
                        let one = [ComponentValue::Function(func)];
                        if let Ok(c) = Color::parse(&mut one.as_slice().into()) {
                            layer_color = Some(c);
                            continue;
                        }
                    }
                }
                BgToken::Ident(ident) => {
                    if layer_image.is_none() && ident.eq_ignore_ascii_case("none") {
                        continue;
                    }

                    if layer_attachment.is_none() {
                        if ident.eq_ignore_ascii_case("scroll") {
                            layer_attachment = Some(Attachment::Scroll);
                            continue;
                        } else if ident.eq_ignore_ascii_case("fixed") {
                            layer_attachment = Some(Attachment::Fixed);
                            continue;
                        } else if ident.eq_ignore_ascii_case("local") {
                            layer_attachment = Some(Attachment::Local);
                            continue;
                        }
                    }

                    if layer_repeat_h.is_none() {
                        if ident.eq_ignore_ascii_case("repeat-x") {
                            layer_repeat_h = Some(RepeatStyle::Repeat);
                            layer_repeat_v = Some(RepeatStyle::NoRepeat);
                            continue;
                        } else if ident.eq_ignore_ascii_case("repeat-y") {
                            layer_repeat_h = Some(RepeatStyle::NoRepeat);
                            layer_repeat_v = Some(RepeatStyle::Repeat);
                            continue;
                        } else if ident.eq_ignore_ascii_case("repeat") {
                            layer_repeat_h = Some(RepeatStyle::Repeat);
                            continue;
                        } else if ident.eq_ignore_ascii_case("space") {
                            layer_repeat_h = Some(RepeatStyle::Space);
                            continue;
                        } else if ident.eq_ignore_ascii_case("round") {
                            layer_repeat_h = Some(RepeatStyle::Round);
                            continue;
                        } else if ident.eq_ignore_ascii_case("no-repeat") {
                            layer_repeat_h = Some(RepeatStyle::NoRepeat);
                            continue;
                        }
                    } else if layer_repeat_v.is_none()
                        && let Ok(rs) = ident.parse::<RepeatStyle>()
                    {
                        layer_repeat_v = Some(rs);
                        continue;
                    }

                    if ident.eq_ignore_ascii_case("content-box") {
                        layer_origin = Some(VisualBox::Content);
                        layer_clip = Some(VisualBox::Content);
                        continue;
                    } else if ident.eq_ignore_ascii_case("padding-box") {
                        if layer_origin.is_none() {
                            layer_origin = Some(VisualBox::Padding);
                        } else if layer_clip.is_none() {
                            layer_clip = Some(VisualBox::Padding);
                        }
                        continue;
                    } else if ident.eq_ignore_ascii_case("border-box") {
                        if layer_origin.is_none() {
                            layer_origin = Some(VisualBox::Border);
                        } else if layer_clip.is_none() {
                            layer_clip = Some(VisualBox::Border);
                        }
                        continue;
                    }

                    if layer_color.is_none() {
                        let token = css_cssom::CssToken {
                            kind: CssTokenKind::Ident(ident),
                            position: None,
                        };
                        let one = [ComponentValue::Token(token)];
                        if let Ok(c) = Color::parse(&mut one.as_slice().into()) {
                            layer_color = Some(c);
                            continue;
                        }
                    }
                }
                BgToken::Url(url) => {
                    if layer_image.is_none() {
                        layer_image = Some(Image::Url(url));
                        continue;
                    }
                }
                BgToken::Hash(value) => {
                    if layer_color.is_none() {
                        let token = css_cssom::CssToken {
                            kind: CssTokenKind::Hash {
                                value,
                                type_flag: HashType::Id,
                            },
                            position: None,
                        };
                        let one = [ComponentValue::Token(token)];
                        if let Ok(c) = Color::parse(&mut one.as_slice().into()) {
                            layer_color = Some(c);
                            continue;
                        }
                    }
                }
                BgToken::Other => {}
            }
        }

        images.push(layer_image.unwrap_or(Image::None));
        attachments.push(layer_attachment.unwrap_or(Attachment::Scroll));

        let repeat_h = layer_repeat_h.unwrap_or(RepeatStyle::Repeat);
        let repeat_v = layer_repeat_v.unwrap_or(repeat_h);
        repeats.push((repeat_h, repeat_v));

        origins.push(layer_origin.unwrap_or(VisualBox::Padding));
        clips.push(BgClip::Visual(layer_clip.unwrap_or(VisualBox::Border)));

        if let Some(sz) = layer_size {
            sizes.push(sz);
        }

        if let Some(pos) = layer_position {
            BackgroundPosition::resolve_bg_position_layer(pos, writing_mode, &mut x_positions, &mut y_positions);
        }

        if done {
            final_color = layer_color.unwrap_or(Color::Base(ColorBase::Transparent));
        }
    }

    ctx.specified_style.background_color = CSSProperty::Value(final_color);
    ctx.specified_style.background_image = CSSProperty::Value(BackgroundImage(images));

    if attachments.is_empty() {
        ctx.specified_style.background_attachment = CSSProperty::Value(BackgroundAttachment::default());
    } else {
        ctx.specified_style.background_attachment = CSSProperty::Value(BackgroundAttachment(attachments));
    }

    if repeats.is_empty() {
        ctx.specified_style.background_repeat = CSSProperty::Value(BackgroundRepeat::default());
    } else {
        ctx.specified_style.background_repeat = CSSProperty::Value(BackgroundRepeat(repeats));
    }

    if origins.is_empty() {
        ctx.specified_style.background_origin = CSSProperty::Value(BackgroundOrigin::default());
    } else {
        ctx.specified_style.background_origin = CSSProperty::Value(BackgroundOrigin(origins));
    }

    if clips.is_empty() {
        ctx.specified_style.background_clip = CSSProperty::Value(BackgroundClip::default());
    } else {
        ctx.specified_style.background_clip = CSSProperty::Value(BackgroundClip(clips));
    }

    if sizes.is_empty() {
        ctx.specified_style.background_size = CSSProperty::Value(BackgroundSize::default());
    } else {
        ctx.specified_style.background_size = CSSProperty::Value(BackgroundSize(sizes));
    }

    if !x_positions.is_empty() {
        ctx.specified_style.background_position_x = CSSProperty::Value(BackgroundPositionX(x_positions));
    }

    if !y_positions.is_empty() {
        ctx.specified_style.background_position_y = CSSProperty::Value(BackgroundPositionY(y_positions));
    }
}

/// Handles the `border` shorthand property by parsing the provided component values and updating the corresponding border properties (style, width, color) in the specified style.
pub(crate) fn handle_border(ctx: &mut PropertyUpdateContext, stream: &mut ComponentValueStream) {
    fn reset_border_color(ctx: &mut PropertyUpdateContext) {
        ctx.specified_style.border_top_color = CSSProperty::Global(Global::Initial);
        ctx.specified_style.border_right_color = CSSProperty::Global(Global::Initial);
        ctx.specified_style.border_bottom_color = CSSProperty::Global(Global::Initial);
        ctx.specified_style.border_left_color = CSSProperty::Global(Global::Initial);
    }

    fn reset_border_style(ctx: &mut PropertyUpdateContext) {
        ctx.specified_style.border_top_style = CSSProperty::Global(Global::Initial);
        ctx.specified_style.border_right_style = CSSProperty::Global(Global::Initial);
        ctx.specified_style.border_bottom_style = CSSProperty::Global(Global::Initial);
        ctx.specified_style.border_left_style = CSSProperty::Global(Global::Initial);
    }

    fn reset_border_width(ctx: &mut PropertyUpdateContext) {
        ctx.specified_style.border_top_width = CSSProperty::Global(Global::Initial);
        ctx.specified_style.border_right_width = CSSProperty::Global(Global::Initial);
        ctx.specified_style.border_bottom_width = CSSProperty::Global(Global::Initial);
        ctx.specified_style.border_left_width = CSSProperty::Global(Global::Initial);
    }

    let checkpoint = stream.checkpoint();

    if let Ok(global) = Global::parse(stream) {
        stream.skip_whitespace();
        if stream.peek().is_none() {
            ctx.specified_style.border_top_style = CSSProperty::Global(global);
            ctx.specified_style.border_right_style = CSSProperty::Global(global);
            ctx.specified_style.border_bottom_style = CSSProperty::Global(global);
            ctx.specified_style.border_left_style = CSSProperty::Global(global);
            ctx.specified_style.border_top_width = CSSProperty::Global(global);
            ctx.specified_style.border_right_width = CSSProperty::Global(global);
            ctx.specified_style.border_bottom_width = CSSProperty::Global(global);
            ctx.specified_style.border_left_width = CSSProperty::Global(global);
            ctx.specified_style.border_top_color = CSSProperty::Global(global);
            ctx.specified_style.border_right_color = CSSProperty::Global(global);
            ctx.specified_style.border_bottom_color = CSSProperty::Global(global);
            ctx.specified_style.border_left_color = CSSProperty::Global(global);
            return;
        }
    }

    stream.restore(checkpoint);

    let mut style = None;
    let mut width = None;
    let mut color = None;
    let mut parsed_any = false;

    while stream.peek().is_some() {
        stream.skip_whitespace();
        if stream.peek().is_none() {
            break;
        }

        if width.is_none() {
            let cp = stream.checkpoint();
            if let Ok(w) = BorderWidth::parse(stream) {
                width = Some(w);
                parsed_any = true;
                continue;
            }
            stream.restore(cp);
        }

        if style.is_none()
            && let Some(ComponentValue::Token(token)) = stream.peek()
            && let CssTokenKind::Ident(ident) = &token.kind
            && let Ok(s) = ident.parse::<BorderStyle>()
        {
            style = Some(s);
            parsed_any = true;
            stream.next_cv();
            continue;
        }

        if color.is_none()
            && let Some(cv) = stream.peek().cloned()
        {
            let mut one = ComponentValueStream::from(std::slice::from_ref(&cv));
            if let Ok(c) = Color::parse(&mut one) {
                color = Some(c);
                parsed_any = true;
                stream.next_cv();
                continue;
            }
        }

        stream.restore(checkpoint);
        ctx.record_error_from_stream("border", stream, CssValueError::InvalidValue("Border property".to_string()));
        return;
    }

    if !parsed_any {
        stream.restore(checkpoint);
        ctx.record_error_from_stream("border", stream, CssValueError::InvalidValue("Border property".to_string()));
        return;
    }

    match color {
        Some(c) => {
            ctx.specified_style.border_top_color = CSSProperty::Value(c.clone());
            ctx.specified_style.border_right_color = CSSProperty::Value(c.clone());
            ctx.specified_style.border_bottom_color = CSSProperty::Value(c.clone());
            ctx.specified_style.border_left_color = CSSProperty::Value(c);
        }
        None => reset_border_color(ctx),
    }

    match style {
        Some(s) => {
            ctx.specified_style.border_top_style = CSSProperty::Value(s);
            ctx.specified_style.border_right_style = CSSProperty::Value(s);
            ctx.specified_style.border_bottom_style = CSSProperty::Value(s);
            ctx.specified_style.border_left_style = CSSProperty::Value(s);
        }
        None => reset_border_style(ctx),
    }

    match width {
        Some(w) => {
            ctx.specified_style.border_top_width = CSSProperty::Value(w.clone());
            ctx.specified_style.border_right_width = CSSProperty::Value(w.clone());
            ctx.specified_style.border_bottom_width = CSSProperty::Value(w.clone());
            ctx.specified_style.border_left_width = CSSProperty::Value(w);
        }
        None => reset_border_width(ctx),
    }
}

/// Handles the `border-color` shorthand property by parsing the provided component values and updating the corresponding border color properties in the specified style.
/// The function supports both global values and individual color values for each side of the border.
pub(crate) fn handle_border_color(ctx: &mut PropertyUpdateContext, stream: &mut ComponentValueStream) {
    stream.skip_whitespace();
    let checkpoint = stream.checkpoint();

    if let Ok(global) = Global::parse(stream) {
        ctx.specified_style.border_top_color = CSSProperty::Global(global);
        ctx.specified_style.border_right_color = CSSProperty::Global(global);
        ctx.specified_style.border_bottom_color = CSSProperty::Global(global);
        ctx.specified_style.border_left_color = CSSProperty::Global(global);
        return;
    }

    stream.restore(checkpoint);
    let mut colors = Vec::new();

    while let Some(cv) = stream.next_non_whitespace() {
        if let Ok(c) = Color::try_from(cv) {
            colors.push(c);
        }
    }

    match colors.len() {
        1 => {
            ctx.specified_style.border_top_color = CSSProperty::Value(colors[0].clone());
            ctx.specified_style.border_right_color = CSSProperty::Value(colors[0].clone());
            ctx.specified_style.border_bottom_color = CSSProperty::Value(colors[0].clone());
            ctx.specified_style.border_left_color = CSSProperty::Value(colors[0].clone());
        }
        2 => {
            ctx.specified_style.border_top_color = CSSProperty::Value(colors[0].clone());
            ctx.specified_style.border_right_color = CSSProperty::Value(colors[1].clone());
            ctx.specified_style.border_bottom_color = CSSProperty::Value(colors[0].clone());
            ctx.specified_style.border_left_color = CSSProperty::Value(colors[1].clone());
        }
        3 => {
            ctx.specified_style.border_top_color = CSSProperty::Value(colors[0].clone());
            ctx.specified_style.border_right_color = CSSProperty::Value(colors[1].clone());
            ctx.specified_style.border_bottom_color = CSSProperty::Value(colors[2].clone());
            ctx.specified_style.border_left_color = CSSProperty::Value(colors[1].clone());
        }
        4 => {
            ctx.specified_style.border_top_color = CSSProperty::Value(colors[0].clone());
            ctx.specified_style.border_right_color = CSSProperty::Value(colors[1].clone());
            ctx.specified_style.border_bottom_color = CSSProperty::Value(colors[2].clone());
            ctx.specified_style.border_left_color = CSSProperty::Value(colors[3].clone());
        }
        _ => {
            ctx.record_error_from_stream(
                "border-color",
                stream,
                CssValueError::InvalidValue("Invalid number of color values".to_string()),
            );
        }
    }
}

/// Handles the `border-style` shorthand property by parsing the provided component values and updating the corresponding border style properties in the specified style.
/// The function supports both global values and individual style values for each side of the border.
pub(crate) fn handle_border_style(ctx: &mut PropertyUpdateContext, stream: &mut ComponentValueStream) {
    stream.skip_whitespace();
    let checkpoint = stream.checkpoint();

    if let Ok(global) = Global::parse(stream) {
        ctx.specified_style.border_top_style = CSSProperty::Global(global);
        ctx.specified_style.border_right_style = CSSProperty::Global(global);
        ctx.specified_style.border_bottom_style = CSSProperty::Global(global);
        ctx.specified_style.border_left_style = CSSProperty::Global(global);
        return;
    }

    stream.restore(checkpoint);
    let mut styles = Vec::new();

    while let Some(cv) = stream.next_non_whitespace() {
        if let ComponentValue::Token(token) = cv
            && let CssTokenKind::Ident(ident) = &token.kind
            && let Ok(s) = ident.parse::<BorderStyle>()
        {
            styles.push(s);
        }
    }

    match styles.len() {
        1 => {
            ctx.specified_style.border_top_style = CSSProperty::Value(styles[0]);
            ctx.specified_style.border_right_style = CSSProperty::Value(styles[0]);
            ctx.specified_style.border_bottom_style = CSSProperty::Value(styles[0]);
            ctx.specified_style.border_left_style = CSSProperty::Value(styles[0]);
        }
        2 => {
            ctx.specified_style.border_top_style = CSSProperty::Value(styles[0]);
            ctx.specified_style.border_right_style = CSSProperty::Value(styles[1]);
            ctx.specified_style.border_bottom_style = CSSProperty::Value(styles[0]);
            ctx.specified_style.border_left_style = CSSProperty::Value(styles[1]);
        }
        3 => {
            ctx.specified_style.border_top_style = CSSProperty::Value(styles[0]);
            ctx.specified_style.border_right_style = CSSProperty::Value(styles[1]);
            ctx.specified_style.border_bottom_style = CSSProperty::Value(styles[2]);
            ctx.specified_style.border_left_style = CSSProperty::Value(styles[1]);
        }
        4 => {
            ctx.specified_style.border_top_style = CSSProperty::Value(styles[0]);
            ctx.specified_style.border_right_style = CSSProperty::Value(styles[1]);
            ctx.specified_style.border_bottom_style = CSSProperty::Value(styles[2]);
            ctx.specified_style.border_left_style = CSSProperty::Value(styles[3]);
        }
        _ => {
            ctx.record_error_from_stream(
                "border-style",
                stream,
                CssValueError::InvalidValue("Invalid number of style values".to_string()),
            );
        }
    }
}

/// Handles the `border-width` shorthand property by parsing the provided component values and updating the corresponding border width properties in the specified style.
/// The function supports both global values and individual width values for each side of the border.
pub(crate) fn handle_border_width(ctx: &mut PropertyUpdateContext, stream: &mut ComponentValueStream) {
    stream.skip_whitespace();
    let checkpoint = stream.checkpoint();

    if let Ok(global) = Global::parse(stream) {
        ctx.specified_style.border_top_width = CSSProperty::Global(global);
        ctx.specified_style.border_right_width = CSSProperty::Global(global);
        ctx.specified_style.border_bottom_width = CSSProperty::Global(global);
        ctx.specified_style.border_left_width = CSSProperty::Global(global);
        return;
    }

    stream.restore(checkpoint);
    let mut widths = Vec::new();

    while let Some(cv) = stream.next_non_whitespace() {
        if let Ok(w) = BorderWidth::try_from(cv) {
            widths.push(w);
        }
    }

    match widths.len() {
        1 => {
            ctx.specified_style.border_top_width = CSSProperty::Value(widths[0].clone());
            ctx.specified_style.border_right_width = CSSProperty::Value(widths[0].clone());
            ctx.specified_style.border_bottom_width = CSSProperty::Value(widths[0].clone());
            ctx.specified_style.border_left_width = CSSProperty::Value(widths[0].clone());
        }
        2 => {
            ctx.specified_style.border_top_width = CSSProperty::Value(widths[0].clone());
            ctx.specified_style.border_right_width = CSSProperty::Value(widths[1].clone());
            ctx.specified_style.border_bottom_width = CSSProperty::Value(widths[0].clone());
            ctx.specified_style.border_left_width = CSSProperty::Value(widths[1].clone());
        }
        3 => {
            ctx.specified_style.border_top_width = CSSProperty::Value(widths[0].clone());
            ctx.specified_style.border_right_width = CSSProperty::Value(widths[1].clone());
            ctx.specified_style.border_bottom_width = CSSProperty::Value(widths[2].clone());
            ctx.specified_style.border_left_width = CSSProperty::Value(widths[1].clone());
        }
        4 => {
            ctx.specified_style.border_top_width = CSSProperty::Value(widths[0].clone());
            ctx.specified_style.border_right_width = CSSProperty::Value(widths[1].clone());
            ctx.specified_style.border_bottom_width = CSSProperty::Value(widths[2].clone());
            ctx.specified_style.border_left_width = CSSProperty::Value(widths[3].clone());
        }
        _ => {
            ctx.record_error_from_stream(
                "border-width",
                stream,
                CssValueError::InvalidValue("Invalid number of width values".to_string()),
            );
        }
    }
}

/// Handles the `font-size` property by updating the specified style's font size based on the provided component values. The function first attempts to update the font size
/// using the `CSSProperty::update_property` method.
pub(crate) fn handle_font_size(ctx: &mut PropertyUpdateContext, stream: &mut ComponentValueStream) {
    CSSProperty::update_property(&mut ctx.specified_style.font_size, stream).unwrap_or(());

    if let Ok(font_size) = CSSProperty::resolve(&ctx.specified_style.font_size) {
        ctx.specified_style.computed_font_size_px =
            font_size.to_px(Some(RelativeType::FontSize), Some(ctx.relative_ctx), ctx.absolute_ctx);
    }
}

/// Handles the `font-weight` property by parsing the provided component values and updating the specified style's font weight accordingly. The function checks for the presence
/// of `lighter` and `bolder` keywords, which adjust the font weight relative to the parent's font weight.
pub(crate) fn handle_font_weight(ctx: &mut PropertyUpdateContext, stream: &mut ComponentValueStream) {
    let checkpoint = stream.checkpoint();

    while let Some(cv) = stream.next_cv() {
        match cv {
            ComponentValue::Token(token) => match &token.kind {
                CssTokenKind::Ident(ident) => {
                    if ident.eq_ignore_ascii_case("lighter") {
                        let lighter = ctx.relative_ctx.parent.font_weight - 100;
                        ctx.specified_style.font_weight = CSSProperty::Value(FontWeight::from(lighter));
                        return;
                    } else if ident.eq_ignore_ascii_case("bolder") {
                        let bolder = ctx.relative_ctx.parent.font_weight + 100;
                        ctx.specified_style.font_weight = CSSProperty::Value(FontWeight::from(bolder));
                        return;
                    }
                }
                _ => continue,
            },
            _ => continue,
        }
    }

    stream.restore(checkpoint);

    if let Err(e) = CSSProperty::update_property(&mut ctx.specified_style.font_weight, stream) {
        ctx.record_error_from_stream("font-weight", stream, e);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use css_cssom::CSSStyleSheet;
    use css_values::position::{PositionX, PositionY};

    #[test]
    fn test_border_extra_tokens() {
        let abs = AbsoluteContext::default();
        let rel = RelativeContext::default();
        let mut specified = SpecifiedStyle {
            border_top_style: CSSProperty::Value(BorderStyle::Solid),
            border_right_style: CSSProperty::Value(BorderStyle::Solid),
            border_bottom_style: CSSProperty::Value(BorderStyle::Solid),
            border_left_style: CSSProperty::Value(BorderStyle::Solid),
            ..Default::default()
        };

        let before = specified.clone();

        let decls = CSSStyleSheet::from_inline("border: red 1px solid somethingelse;");
        let values = decls[0].original_values.clone();
        let mut stream = ComponentValueStream::from(&values);
        let mut ctx = PropertyUpdateContext::new(&abs, &mut specified, &rel);

        handle_border(&mut ctx, &mut stream);

        assert_eq!(ctx.errors.len(), 1);
        assert_eq!(specified, before);
    }

    #[test]
    fn test_border_any_order() {
        let abs = AbsoluteContext::default();
        let rel = RelativeContext::default();
        let mut specified = SpecifiedStyle::default();

        let decls = CSSStyleSheet::from_inline("border: red 1px solid;");
        let values = decls[0].original_values.clone();
        let mut stream = ComponentValueStream::from(&values);
        let mut ctx = PropertyUpdateContext::new(&abs, &mut specified, &rel);

        handle_border(&mut ctx, &mut stream);

        assert!(ctx.errors.is_empty());

        assert_eq!(specified.border_top_style, CSSProperty::Value(BorderStyle::Solid));
        assert_eq!(specified.border_top_width, CSSProperty::Value(BorderWidth::px(1.0)));
    }

    #[test]
    fn test_border_some_missing() {
        let abs = AbsoluteContext::default();
        let rel = RelativeContext::default();
        let mut specified = SpecifiedStyle::default();

        let decls = CSSStyleSheet::from_inline("border: solid;");
        let values = decls[0].original_values.clone();
        let mut stream = ComponentValueStream::from(&values);
        let mut ctx = PropertyUpdateContext::new(&abs, &mut specified, &rel);

        handle_border(&mut ctx, &mut stream);

        assert!(ctx.errors.is_empty());

        assert_eq!(specified.border_top_style, CSSProperty::Value(BorderStyle::Solid));
        assert_eq!(specified.border_top_width, CSSProperty::Global(Global::Initial));
        assert_eq!(specified.border_top_color, CSSProperty::Global(Global::Initial));
    }

    #[test]
    fn test_border_global_keyword() {
        let abs = AbsoluteContext::default();
        let rel = RelativeContext::default();
        let mut specified = SpecifiedStyle::default();

        let before = specified.clone();

        let decls = CSSStyleSheet::from_inline("border: inherit solid;");
        let values = decls[0].original_values.clone();
        let mut stream = ComponentValueStream::from(&values);
        let mut ctx = PropertyUpdateContext::new(&abs, &mut specified, &rel);

        handle_border(&mut ctx, &mut stream);

        assert_eq!(ctx.errors.len(), 1);
        assert_eq!(specified, before);
    }

    #[test]
    fn test_background_multiple_layers_with_size_and_color() {
        let abs = AbsoluteContext::default();
        let rel = RelativeContext::default();
        let mut specified = SpecifiedStyle::default();

        let decls = CSSStyleSheet::from_inline(
            "background: linear-gradient(red, blue) left top / 10px 20px no-repeat fixed padding-box border-box, none right bottom / contain repeat-y scroll #00ff00;",
        );
        let values = decls[0].original_values.clone();
        let mut stream = ComponentValueStream::from(&values);
        let mut ctx = PropertyUpdateContext::new(&abs, &mut specified, &rel);

        handle_background(&mut ctx, &mut stream);

        assert!(ctx.errors.is_empty());

        match &specified.background_image {
            CSSProperty::Value(BackgroundImage(images)) => {
                assert_eq!(images.len(), 2);
                assert!(matches!(images[0], Image::Gradient(_)));
                assert!(matches!(images[1], Image::None));
            }
            _ => panic!("expected background-image value"),
        }

        match &specified.background_attachment {
            CSSProperty::Value(BackgroundAttachment(a)) => {
                assert_eq!(a.len(), 2);
                assert_eq!(a[0], Attachment::Fixed);
                assert_eq!(a[1], Attachment::Scroll);
            }
            _ => panic!("expected background-attachment value"),
        }

        match &specified.background_repeat {
            CSSProperty::Value(BackgroundRepeat(r)) => {
                assert_eq!(r.len(), 2);
                assert_eq!(r[0], (RepeatStyle::NoRepeat, RepeatStyle::NoRepeat));
                assert_eq!(r[1], (RepeatStyle::NoRepeat, RepeatStyle::Repeat));
            }
            _ => panic!("expected background-repeat value"),
        }

        match &specified.background_origin {
            CSSProperty::Value(BackgroundOrigin(o)) => {
                assert_eq!(o.len(), 2);
                assert_eq!(o[0], VisualBox::Padding);
                assert_eq!(o[1], VisualBox::Padding);
            }
            _ => panic!("expected background-origin value"),
        }

        match &specified.background_clip {
            CSSProperty::Value(BackgroundClip(c)) => {
                assert_eq!(c.len(), 2);
                assert_eq!(c[0], BgClip::Visual(VisualBox::Border));
                assert_eq!(c[1], BgClip::Visual(VisualBox::Border));
            }
            _ => panic!("expected background-clip value"),
        }

        match &specified.background_size {
            CSSProperty::Value(BackgroundSize(s)) => {
                assert_eq!(s.len(), 2);
                assert_eq!(
                    s[0],
                    Size::WidthHeight(
                        WidthHeightSize::Length(LengthPercentage::Length(Length::new(10.0, LengthUnit::Px))),
                        Some(WidthHeightSize::Length(LengthPercentage::Length(Length::new(20.0, LengthUnit::Px))))
                    )
                );
                assert_eq!(s[1], Size::Contain);
            }
            _ => panic!("expected background-size value"),
        }

        match &specified.background_color {
            CSSProperty::Value(c) => assert!(matches!(c, Color::Base(ColorBase::Hex(_)))),
            _ => panic!("expected background-color value"),
        }

        match &specified.background_position_x {
            CSSProperty::Value(BackgroundPositionX(xs)) => assert_eq!(xs.len(), 2),
            _ => panic!("expected background-position-x value"),
        }

        match &specified.background_position_y {
            CSSProperty::Value(BackgroundPositionY(ys)) => assert_eq!(ys.len(), 2),
            _ => panic!("expected background-position-y value"),
        }
    }

    #[test]
    fn test_background_repeat_two_values() {
        let abs = AbsoluteContext::default();
        let rel = RelativeContext::default();
        let mut specified = SpecifiedStyle::default();

        let decls = CSSStyleSheet::from_inline("background: none left top / cover repeat no-repeat;");
        let values = decls[0].original_values.clone();
        let mut stream = ComponentValueStream::from(&values);
        let mut ctx = PropertyUpdateContext::new(&abs, &mut specified, &rel);

        handle_background(&mut ctx, &mut stream);

        assert!(ctx.errors.is_empty());

        match &specified.background_repeat {
            CSSProperty::Value(BackgroundRepeat(r)) => {
                assert_eq!(r.len(), 1);
                assert_eq!(r[0], (RepeatStyle::Repeat, RepeatStyle::NoRepeat));
            }
            _ => panic!("expected background-repeat value"),
        }

        match &specified.background_size {
            CSSProperty::Value(BackgroundSize(s)) => {
                assert_eq!(s.len(), 1);
                assert_eq!(s[0], Size::Cover);
            }
            _ => panic!("expected background-size value"),
        }
    }

    #[test]
    fn test_background_origin_clip_chain_padding_border() {
        let abs = AbsoluteContext::default();
        let rel = RelativeContext::default();
        let mut specified = SpecifiedStyle::default();

        let decls = CSSStyleSheet::from_inline("background: none padding-box border-box;");
        let values = decls[0].original_values.clone();
        let mut stream = ComponentValueStream::from(&values);
        let mut ctx = PropertyUpdateContext::new(&abs, &mut specified, &rel);

        handle_background(&mut ctx, &mut stream);

        assert!(ctx.errors.is_empty());

        match &specified.background_origin {
            CSSProperty::Value(BackgroundOrigin(o)) => {
                assert_eq!(o.len(), 1);
                assert_eq!(o[0], VisualBox::Padding);
            }
            _ => panic!("expected background-origin value"),
        }

        match &specified.background_clip {
            CSSProperty::Value(BackgroundClip(c)) => {
                assert_eq!(c.len(), 1);
                assert_eq!(c[0], BgClip::Visual(VisualBox::Border));
            }
            _ => panic!("expected background-clip value"),
        }
    }

    #[test]
    fn test_background_zero_zero() {
        let abs = AbsoluteContext::default();
        let rel = RelativeContext::default();
        let mut specified = SpecifiedStyle::default();

        let decls = CSSStyleSheet::from_inline("background: 0 0;");
        let values = decls[0].original_values.clone();
        let mut stream = ComponentValueStream::from(&values);
        let mut ctx = PropertyUpdateContext::new(&abs, &mut specified, &rel);

        handle_background(&mut ctx, &mut stream);

        assert!(ctx.errors.is_empty());
        match &specified.background_position_x {
            CSSProperty::Value(BackgroundPositionX(xs)) => {
                assert_eq!(xs.len(), 1);
                assert_eq!(
                    xs[0],
                    PositionX::Relative((None, Some(LengthPercentage::Length(Length::new(0.0, LengthUnit::Px)))))
                );
            }
            _ => panic!("expected background-position-x value"),
        }
        match &specified.background_position_y {
            CSSProperty::Value(BackgroundPositionY(ys)) => {
                assert_eq!(ys.len(), 1);
                assert_eq!(
                    ys[0],
                    PositionY::Relative((None, Some(LengthPercentage::Length(Length::new(0.0, LengthUnit::Px)))))
                );
            }
            _ => panic!("expected background-position-y value"),
        }
    }
}

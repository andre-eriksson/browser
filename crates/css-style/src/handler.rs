use css_cssom::{ComponentValue, CssTokenKind};

use crate::global::Global;
use crate::length::{Length, LengthUnit};
use crate::properties::text::WritingMode;
use crate::properties::{AbsoluteContext, CSSProperty};
use crate::specified::SpecifiedStyle;
use crate::{BorderStyle, BorderWidth, Color, FontWeight, Offset, RelativeContext};

/// Context for updating a CSS property, containing necessary information and utilities for the update process.
pub(crate) struct PropertyUpdateContext<'a> {
    pub absolute_ctx: &'a AbsoluteContext,
    pub specified_style: &'a mut SpecifiedStyle,
    pub relative_ctx: &'a RelativeContext,
    pub errors: Vec<PropertyError>,
}

/// Represents an error that occurred during the property update process, including the property name, the value that caused the error, and a descriptive error message.
#[derive(Debug)]
pub(crate) struct PropertyError {
    pub property: String,
    pub value: Vec<ComponentValue>,
    pub error: String,
}

impl<'a> PropertyUpdateContext<'a> {
    pub fn new(
        absolute_ctx: &'a AbsoluteContext,
        specified_style: &'a mut SpecifiedStyle,
        relative_ctx: &'a RelativeContext,
    ) -> Self {
        Self {
            absolute_ctx,
            specified_style,
            relative_ctx,
            errors: Vec::new(),
        }
    }

    fn record_error(&mut self, property: &str, value: Vec<ComponentValue>, error: String) {
        self.errors.push(PropertyError {
            property: property.to_string(),
            value,
            error,
        });
    }

    /// Resolves the current writing mode from the specified style, falling back to the parent's
    /// writing mode or the initial value (`HorizontalTb`) if the property is set to a global value.
    fn resolve_writing_mode(&self) -> WritingMode {
        match self.specified_style.writing_mode {
            CSSProperty::Value(val) => val,
            CSSProperty::Global(_) => self
                .specified_style
                .writing_mode
                .resolve_with_context_owned(self.relative_ctx.parent.writing_mode, WritingMode::HorizontalTb),
        }
    }

    pub fn log_errors(&self) {
        if !self.errors.is_empty() {
            eprintln!("Property update errors:");

            for err in &self.errors {
                let mut errors = String::with_capacity(32);
                for cv in &err.value {
                    errors.push_str(cv.to_css_string().as_str());
                }

                eprintln!("  {}: '{}' - {}", err.property, errors, err.error);
            }
        }
    }
}

/// A macro to generate simple property handler functions that update a specific field in the specified style based on the provided component values.
/// The macro takes the function name, the field to update, and the property name for error reporting.
macro_rules! simple_property_handler {
    ($fn_name:ident, $field:ident, $prop_name:expr) => {
        pub fn $fn_name(ctx: &mut PropertyUpdateContext, value: &[ComponentValue]) {
            if let Err(e) = CSSProperty::update_property(&mut ctx.specified_style.$field, value) {
                ctx.record_error($prop_name, value.to_vec(), e);
            }
        }
    };
}

/// A macro to generate shorthand property handlers for 4-side offset properties (e.g. `margin`, `padding`).
/// Parses the value once as either a `Global` or an `Offset`, then assigns to all four physical side fields.
macro_rules! offset_shorthand_handler {
    ($fn_name:ident, $prop_name:expr, $top:ident, $right:ident, $bottom:ident, $left:ident) => {
        pub(crate) fn $fn_name(ctx: &mut PropertyUpdateContext, value: &[ComponentValue]) {
            if let Ok(global) = Global::try_from(value) {
                ctx.specified_style.$top = CSSProperty::Global(global);
                ctx.specified_style.$right = CSSProperty::Global(global);
                ctx.specified_style.$bottom = CSSProperty::Global(global);
                ctx.specified_style.$left = CSSProperty::Global(global);
            } else if let Ok(offset) = Offset::try_from(value) {
                ctx.specified_style.$top = offset.top.into();
                ctx.specified_style.$right = offset.right.into();
                ctx.specified_style.$bottom = offset.bottom.into();
                ctx.specified_style.$left = offset.left.into();
            } else {
                ctx.record_error($prop_name, value.to_vec(), format!("Invalid value for {} property", $prop_name));
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
macro_rules! logical_block_handler {
    ($fn_name:ident, $prop_name:expr,
     $htb_start:ident, $htb_end:ident,
     $vrl_start:ident, $vrl_end:ident,
     $vlr_start:ident, $vlr_end:ident) => {
        pub(crate) fn $fn_name(ctx: &mut PropertyUpdateContext, value: &[ComponentValue]) {
            let global = Global::try_from(value).ok();
            let offset = if global.is_none() {
                Offset::try_from(value).ok()
            } else {
                None
            };

            let (start, end) = match ctx.resolve_writing_mode() {
                WritingMode::HorizontalTb => (&mut ctx.specified_style.$htb_start, &mut ctx.specified_style.$htb_end),
                WritingMode::VerticalRl => (&mut ctx.specified_style.$vrl_start, &mut ctx.specified_style.$vrl_end),
                WritingMode::VerticalLr => (&mut ctx.specified_style.$vlr_start, &mut ctx.specified_style.$vlr_end),
                _ => {
                    ctx.record_error($prop_name, value.to_vec(), String::from("Unsupported writing mode"));
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
        pub(crate) fn $fn_name(ctx: &mut PropertyUpdateContext, value: &[ComponentValue]) {
            let field = match ctx.resolve_writing_mode() {
                WritingMode::HorizontalTb => &mut ctx.specified_style.$htb,
                WritingMode::VerticalRl => &mut ctx.specified_style.$vrl,
                WritingMode::VerticalLr => &mut ctx.specified_style.$vlr,
                _ => {
                    ctx.record_error($prop_name, value.to_vec(), String::from("Unsupported writing mode"));
                    return;
                }
            };

            if let Err(e) = CSSProperty::update_property(field, value) {
                ctx.record_error($prop_name, value.to_vec(), e);
            }
        }
    };
}

simple_property_handler!(handle_background_color, background_color, "background-color");
simple_property_handler!(handle_border_top_color, border_top_color, "border-top-color");
simple_property_handler!(handle_border_right_color, border_right_color, "border-right-color");
simple_property_handler!(handle_border_bottom_color, border_bottom_color, "border-bottom-color");
simple_property_handler!(handle_border_left_color, border_left_color, "border-left-color");
simple_property_handler!(handle_border_top_style, border_top_style, "border-top-style");
simple_property_handler!(handle_border_right_style, border_right_style, "border-right-style");
simple_property_handler!(handle_border_bottom_style, border_bottom_style, "border-bottom-style");
simple_property_handler!(handle_border_left_style, border_left_style, "border-left-style");
simple_property_handler!(handle_border_top_width, border_top_width, "border-top-width");
simple_property_handler!(handle_border_right_width, border_right_width, "border-right-width");
simple_property_handler!(handle_border_bottom_width, border_bottom_width, "border-bottom-width");
simple_property_handler!(handle_border_left_width, border_left_width, "border-left-width");
simple_property_handler!(handle_color, color, "color");
simple_property_handler!(handle_display, display, "display");
simple_property_handler!(handle_font_family, font_family, "font-family");
simple_property_handler!(handle_height, height, "height");
simple_property_handler!(handle_max_height, max_height, "max-height");
simple_property_handler!(handle_line_height, line_height, "line-height");
simple_property_handler!(handle_margin_top, margin_top, "margin-top");
simple_property_handler!(handle_margin_bottom, margin_bottom, "margin-bottom");
simple_property_handler!(handle_margin_left, margin_left, "margin-left");
simple_property_handler!(handle_margin_right, margin_right, "margin-right");
simple_property_handler!(handle_padding_top, padding_top, "padding-top");
simple_property_handler!(handle_padding_bottom, padding_bottom, "padding-bottom");
simple_property_handler!(handle_padding_left, padding_left, "padding-left");
simple_property_handler!(handle_padding_right, padding_right, "padding-right");
simple_property_handler!(handle_position, position, "position");
simple_property_handler!(handle_text_align, text_align, "text-align");
simple_property_handler!(handle_whitespace, whitespace, "white-space");
simple_property_handler!(handle_width, width, "width");
simple_property_handler!(handle_max_width, max_width, "max-width");
simple_property_handler!(handle_writing_mode, writing_mode, "writing-mode");
offset_shorthand_handler!(handle_margin, "margin", margin_top, margin_right, margin_bottom, margin_left);
offset_shorthand_handler!(handle_padding, "padding", padding_top, padding_right, padding_bottom, padding_left);
logical_block_handler!(
    handle_margin_block,
    "margin-block",
    margin_top,
    margin_bottom,
    margin_right,
    margin_left,
    margin_left,
    margin_right
);
logical_block_handler!(
    handle_padding_block,
    "padding-block",
    padding_top,
    padding_bottom,
    padding_right,
    padding_left,
    padding_left,
    padding_right
);
logical_edge_handler!(handle_margin_block_start, "margin-block-start", margin_top, margin_right, margin_left);
logical_edge_handler!(handle_margin_block_end, "margin-block-end", margin_bottom, margin_left, margin_right);
logical_edge_handler!(handle_padding_block_start, "padding-block-start", padding_top, padding_right, padding_left);
logical_edge_handler!(handle_padding_block_end, "padding-block-end", padding_bottom, padding_left, padding_right);

/// Handles the `border` shorthand property by parsing the provided component values and updating the corresponding border properties (style, width, color) in the specified style.
pub(crate) fn handle_border(ctx: &mut PropertyUpdateContext, value: &[ComponentValue]) {
    let mut style = None;
    let mut width = None;
    let mut color = None;

    for (i, cv) in value.iter().enumerate() {
        match cv {
            ComponentValue::Token(token) => match &token.kind {
                CssTokenKind::Ident(ident) => {
                    if ident.eq_ignore_ascii_case("none") {
                        break;
                    } else if let Ok(w) = BorderWidth::try_from(&value[i..])
                        && width.is_none()
                    {
                        width = Some(w);
                    } else if let Ok(s) = ident.parse::<BorderStyle>() {
                        style = Some(s);
                    } else if let Ok(c) = Color::try_from(&value[i..]) {
                        color = Some(c);
                    }
                }
                CssTokenKind::Number(num) => {
                    if width.is_some() || num.to_f64() != 0.0 {
                        continue;
                    }

                    width = Some(BorderWidth::Length(Length::zero()));
                }
                CssTokenKind::Dimension { value, unit } => {
                    if width.is_some() {
                        continue;
                    }

                    if let Ok(len_unit) = unit.parse::<LengthUnit>() {
                        width = Some(BorderWidth::Length(Length::new(value.to_f64() as f32, len_unit)));
                    }
                }
                _ => continue,
            },
            _ => continue,
        }
    }

    match style {
        Some(s) => {
            ctx.specified_style.border_top_style = CSSProperty::Value(s);
            ctx.specified_style.border_right_style = CSSProperty::Value(s);
            ctx.specified_style.border_bottom_style = CSSProperty::Value(s);
            ctx.specified_style.border_left_style = CSSProperty::Value(s);
        }
        None => {
            ctx.specified_style.border_top_style = CSSProperty::Value(BorderStyle::default());
            ctx.specified_style.border_right_style = CSSProperty::Value(BorderStyle::default());
            ctx.specified_style.border_bottom_style = CSSProperty::Value(BorderStyle::default());
            ctx.specified_style.border_left_style = CSSProperty::Value(BorderStyle::default());
        }
    }

    match width {
        Some(w) => {
            ctx.specified_style.border_top_width = CSSProperty::Value(w.clone());
            ctx.specified_style.border_right_width = CSSProperty::Value(w.clone());
            ctx.specified_style.border_bottom_width = CSSProperty::Value(w.clone());
            ctx.specified_style.border_left_width = CSSProperty::Value(w);
        }
        None => {
            ctx.specified_style.border_top_width = CSSProperty::Value(BorderWidth::default());
            ctx.specified_style.border_right_width = CSSProperty::Value(BorderWidth::default());
            ctx.specified_style.border_bottom_width = CSSProperty::Value(BorderWidth::default());
            ctx.specified_style.border_left_width = CSSProperty::Value(BorderWidth::default());
        }
    }

    match color {
        Some(c) => {
            ctx.specified_style.border_top_color = CSSProperty::Value(c);
            ctx.specified_style.border_right_color = CSSProperty::Value(c);
            ctx.specified_style.border_bottom_color = CSSProperty::Value(c);
            ctx.specified_style.border_left_color = CSSProperty::Value(c);
        }
        None => {
            ctx.specified_style.border_top_color = CSSProperty::Value(Color::from(ctx.relative_ctx.parent.color));
            ctx.specified_style.border_right_color = CSSProperty::Value(Color::from(ctx.relative_ctx.parent.color));
            ctx.specified_style.border_bottom_color = CSSProperty::Value(Color::from(ctx.relative_ctx.parent.color));
            ctx.specified_style.border_left_color = CSSProperty::Value(Color::from(ctx.relative_ctx.parent.color));
        }
    }
}

/// Handles the `font-size` property by updating the specified style's font size based on the provided component values. The function first attempts to update the font size
/// using the `CSSProperty::update_property` method.
pub(crate) fn handle_font_size(ctx: &mut PropertyUpdateContext, value: &[ComponentValue]) {
    CSSProperty::update_property(&mut ctx.specified_style.font_size, value).unwrap_or(());

    if let Ok(font_size) = CSSProperty::resolve(&ctx.specified_style.font_size) {
        ctx.specified_style.computed_font_size_px =
            font_size.to_px(ctx.absolute_ctx, ctx.relative_ctx.parent.font_size);
    }
}

/// Handles the `font-weight` property by parsing the provided component values and updating the specified style's font weight accordingly. The function checks for the presence
/// of `lighter` and `bolder` keywords, which adjust the font weight relative to the parent's font weight.
pub(crate) fn handle_font_weight(ctx: &mut PropertyUpdateContext, value: &[ComponentValue]) {
    for cv in value {
        match cv {
            ComponentValue::Token(token) => match &token.kind {
                CssTokenKind::Ident(ident) => {
                    if ident.eq_ignore_ascii_case("lighter") {
                        let lighter = ctx.relative_ctx.parent.font_weight - 100;
                        ctx.specified_style.font_weight =
                            CSSProperty::Value(FontWeight::try_from(lighter).unwrap_or(FontWeight::Thin));
                        return;
                    } else if ident.eq_ignore_ascii_case("bolder") {
                        let bolder = ctx.relative_ctx.parent.font_weight + 100;
                        ctx.specified_style.font_weight =
                            CSSProperty::Value(FontWeight::try_from(bolder).unwrap_or(FontWeight::Black));
                        return;
                    }
                }
                _ => continue,
            },
            _ => continue,
        }
    }

    if let Err(e) = CSSProperty::update_property(&mut ctx.specified_style.font_weight, value) {
        ctx.record_error("font-weight", value.to_vec(), e);
    }
}

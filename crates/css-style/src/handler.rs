use css_cssom::{ComponentValue, CssToken, CssTokenKind, Function, Property, SimpleBlock};

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

/// Resolves CSS variables in a given value by replacing any `var()` functions with their corresponding values from the provided variables list.
/// This function recursively resolves nested `var()` functions and handles fallback values if a variable is not found.
pub(crate) fn resolve_css_variables(
    variables: &[(Property, Vec<ComponentValue>)],
    value: &[ComponentValue],
) -> Vec<ComponentValue> {
    let mut output: Vec<ComponentValue> = Vec::new();

    for (i, cv) in value.iter().enumerate() {
        match cv {
            ComponentValue::Function(func) if func.name.eq_ignore_ascii_case("var") => {
                let resolved = resolve_var_function(variables, func);

                if !resolved.is_empty() {
                    let needs_leading_whitespace = !output.is_empty()
                        && !output.last().unwrap().is_whitespace()
                        && !resolved.first().unwrap().is_whitespace()
                        && i > 0
                        && value[i - 1].is_whitespace();

                    let needs_trailing_whitespace = !resolved.is_empty()
                        && !resolved.last().unwrap().is_whitespace()
                        && i + 1 < value.len()
                        && !&value[i + 1].is_whitespace();

                    if needs_leading_whitespace {
                        output.push(ComponentValue::Token(CssToken {
                            kind: CssTokenKind::Whitespace,
                            position: None,
                        }));
                    }

                    output.extend(resolved);

                    if needs_trailing_whitespace {
                        output.push(ComponentValue::Token(CssToken {
                            kind: CssTokenKind::Whitespace,
                            position: None,
                        }));
                    }
                }
            }
            ComponentValue::Function(func) => {
                let resolved_inner = resolve_css_variables(variables, &func.value);
                output.push(ComponentValue::Function(Function {
                    name: func.name.clone(),
                    value: resolved_inner,
                }));
            }
            ComponentValue::SimpleBlock(block) => {
                let resolved_inner = resolve_css_variables(variables, &block.value);
                output.push(ComponentValue::SimpleBlock(SimpleBlock {
                    associated_token: block.associated_token,
                    value: resolved_inner,
                }));
            }
            _ => {
                output.push(cv.clone());
            }
        }
    }

    output
}

/// Resolves a `var()` function by extracting the variable name and fallback values, then attempting to find the variable in the provided list of variables.
/// If the variable is found, its value is returned. If not, the fallback values are resolved and returned. If there are no fallback values,
/// the original `var()` function is returned as a component value.
fn resolve_var_function(
    variables: &[(Property, Vec<ComponentValue>)],
    func: &Function,
) -> Vec<ComponentValue> {
    let mut var_name = String::new();
    let mut fallback_values = Vec::new();
    let mut found_comma = false;

    for cv in func.value.iter() {
        match cv {
            ComponentValue::Token(token) if matches!(token.kind, CssTokenKind::Comma) => {
                found_comma = true;
            }
            ComponentValue::Token(token) if token.kind == CssTokenKind::Whitespace => {
                if !found_comma {
                    continue;
                }
                fallback_values.push(cv.clone());
            }
            _ => {
                if !found_comma {
                    var_name.push_str(&cv.to_css_string());
                } else {
                    fallback_values.push(cv.clone());
                }
            }
        }
    }

    let var_name = var_name.trim();

    if let Some(resolved) = try_resolve_variable(variables, var_name) {
        return resolved;
    }

    if !fallback_values.is_empty() {
        return resolve_css_variables(variables, &fallback_values);
    }

    vec![ComponentValue::Function(func.clone())]
}

/// Attempts to resolve a CSS variable by searching for its name in the provided list of variables. If the variable is found, its value is resolved and returned.
/// If the variable is not found or if its value is empty, `None` is returned.
fn try_resolve_variable(
    variables: &[(Property, Vec<ComponentValue>)],
    var_name: &str,
) -> Option<Vec<ComponentValue>> {
    for (name, vals) in variables {
        if name.to_string() != var_name {
            continue;
        }

        if vals.is_empty() {
            return None;
        }

        let resolved = resolve_css_variables(variables, vals);

        if resolved.is_empty() {
            return None;
        }

        return Some(resolved);
    }

    None
}

simple_property_handler!(
    handle_background_color,
    background_color,
    "background-color"
);
simple_property_handler!(
    handle_border_top_color,
    border_top_color,
    "border-top-color"
);
simple_property_handler!(
    handle_border_right_color,
    border_right_color,
    "border-right-color"
);
simple_property_handler!(
    handle_border_bottom_color,
    border_bottom_color,
    "border-bottom-color"
);
simple_property_handler!(
    handle_border_left_color,
    border_left_color,
    "border-left-color"
);
simple_property_handler!(
    handle_border_top_style,
    border_top_style,
    "border-top-style"
);
simple_property_handler!(
    handle_border_right_style,
    border_right_style,
    "border-right-style"
);
simple_property_handler!(
    handle_border_bottom_style,
    border_bottom_style,
    "border-bottom-style"
);
simple_property_handler!(
    handle_border_left_style,
    border_left_style,
    "border-left-style"
);
simple_property_handler!(
    handle_border_top_width,
    border_top_width,
    "border-top-width"
);
simple_property_handler!(
    handle_border_right_width,
    border_right_width,
    "border-right-width"
);
simple_property_handler!(
    handle_border_bottom_width,
    border_bottom_width,
    "border-bottom-width"
);
simple_property_handler!(
    handle_border_left_width,
    border_left_width,
    "border-left-width"
);
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

/// Handles the `margin` shorthand property by parsing the provided component values and updating the corresponding margin properties in the specified style.
pub(crate) fn handle_margin(ctx: &mut PropertyUpdateContext, value: &[ComponentValue]) {
    let global = Global::try_from(value).ok();
    let offset = Offset::try_from(value).ok();

    if let Some(global) = global {
        ctx.specified_style.margin_top = CSSProperty::Global(global);
        ctx.specified_style.margin_right = CSSProperty::Global(global);
        ctx.specified_style.margin_bottom = CSSProperty::Global(global);
        ctx.specified_style.margin_left = CSSProperty::Global(global);
    } else if let Some(offset) = offset {
        ctx.specified_style.margin_top = offset.top.into();
        ctx.specified_style.margin_right = offset.right.into();
        ctx.specified_style.margin_bottom = offset.bottom.into();
        ctx.specified_style.margin_left = offset.left.into();
    } else {
        ctx.record_error(
            "margin",
            value.to_vec(),
            String::from("Invalid value for margin property"),
        );
    }
}

/// Handles the `padding` shorthand property by parsing the provided component values and updating the corresponding padding properties in the specified style.
pub(crate) fn handle_padding(ctx: &mut PropertyUpdateContext, value: &[ComponentValue]) {
    let global = Global::try_from(value).ok();
    let offset = Offset::try_from(value).ok();

    if let Some(global) = global {
        ctx.specified_style.padding_top = CSSProperty::Global(global);
        ctx.specified_style.padding_right = CSSProperty::Global(global);
        ctx.specified_style.padding_bottom = CSSProperty::Global(global);
        ctx.specified_style.padding_left = CSSProperty::Global(global);
    } else if let Some(offset) = offset {
        ctx.specified_style.padding_top = offset.top.into();
        ctx.specified_style.padding_right = offset.right.into();
        ctx.specified_style.padding_bottom = offset.bottom.into();
        ctx.specified_style.padding_left = offset.left.into();
    } else {
        ctx.record_error(
            "padding",
            value.to_vec(),
            String::from("Invalid value for padding property"),
        );
    }
}

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
                        width = Some(BorderWidth::Length(Length::new(
                            value.to_f64() as f32,
                            len_unit,
                        )));
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
            ctx.specified_style.border_top_color =
                CSSProperty::Value(Color::from(ctx.relative_ctx.parent.color));
            ctx.specified_style.border_right_color =
                CSSProperty::Value(Color::from(ctx.relative_ctx.parent.color));
            ctx.specified_style.border_bottom_color =
                CSSProperty::Value(Color::from(ctx.relative_ctx.parent.color));
            ctx.specified_style.border_left_color =
                CSSProperty::Value(Color::from(ctx.relative_ctx.parent.color));
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
                        ctx.specified_style.font_weight = CSSProperty::Value(
                            FontWeight::try_from(lighter).unwrap_or(FontWeight::Thin),
                        );
                        return;
                    } else if ident.eq_ignore_ascii_case("bolder") {
                        let bolder = ctx.relative_ctx.parent.font_weight + 100;
                        ctx.specified_style.font_weight = CSSProperty::Value(
                            FontWeight::try_from(bolder).unwrap_or(FontWeight::Black),
                        );
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

/// Handles the `margin-block`, `margin-block-start`, and `margin-block-end` properties by parsing the provided component values and updating the corresponding margin properties
/// in the specified style based on the current writing mode.
pub(crate) fn handle_margin_block(ctx: &mut PropertyUpdateContext, value: &[ComponentValue]) {
    let global = Global::try_from(value).ok();
    let offset = Offset::try_from(value).ok();

    let writing_mode = match ctx.specified_style.writing_mode {
        CSSProperty::Global(_) => ctx.specified_style.writing_mode.resolve_with_context_owned(
            ctx.relative_ctx.parent.writing_mode,
            WritingMode::HorizontalTb,
        ),
        CSSProperty::Value(val) => val,
    };

    match writing_mode {
        WritingMode::HorizontalTb => {
            if let Some(global) = global {
                ctx.specified_style.margin_top = CSSProperty::Global(global);
                ctx.specified_style.margin_bottom = CSSProperty::Global(global);
            } else if let Some(offset) = offset {
                ctx.specified_style.margin_top = offset.top.into();
                ctx.specified_style.margin_bottom = offset.bottom.into();
            }
        }
        WritingMode::VerticalRl => {
            if let Some(global) = global {
                ctx.specified_style.margin_right = CSSProperty::Global(global);
                ctx.specified_style.margin_left = CSSProperty::Global(global);
            } else if let Some(offset) = offset {
                ctx.specified_style.margin_right = offset.top.into();
                ctx.specified_style.margin_left = offset.bottom.into();
            }
        }
        WritingMode::VerticalLr => {
            if let Some(global) = global {
                ctx.specified_style.margin_left = CSSProperty::Global(global);
                ctx.specified_style.margin_right = CSSProperty::Global(global);
            } else if let Some(offset) = offset {
                ctx.specified_style.margin_left = offset.top.into();
                ctx.specified_style.margin_right = offset.bottom.into();
            }
        }
        _ => {
            ctx.record_error(
                "margin-block",
                value.to_vec(),
                String::from("Unsupported writing mode"),
            );
        }
    }
}

/// Handles the `padding-block`, `padding-block-start`, and `padding-block-end` properties by parsing the provided component values and updating the corresponding padding properties
/// in the specified style based on the current writing mode.
pub(crate) fn handle_margin_block_start(ctx: &mut PropertyUpdateContext, value: &[ComponentValue]) {
    let global = Global::try_from(value).ok();
    let offset = Offset::try_from(value).ok();

    let writing_mode = match ctx.specified_style.writing_mode {
        CSSProperty::Global(_) => ctx.specified_style.writing_mode.resolve_with_context_owned(
            ctx.relative_ctx.parent.writing_mode,
            WritingMode::HorizontalTb,
        ),
        CSSProperty::Value(val) => val,
    };

    match writing_mode {
        WritingMode::HorizontalTb => {
            if let Some(global) = global {
                ctx.specified_style.margin_top = CSSProperty::Global(global);
            } else if let Some(offset) = offset {
                ctx.specified_style.margin_top = offset.top.into();
            }
        }
        WritingMode::VerticalRl => {
            if let Some(global) = global {
                ctx.specified_style.margin_right = CSSProperty::Global(global);
            } else if let Some(offset) = offset {
                ctx.specified_style.margin_right = offset.top.into();
            }
        }
        WritingMode::VerticalLr => {
            if let Some(global) = global {
                ctx.specified_style.margin_left = CSSProperty::Global(global);
            } else if let Some(offset) = offset {
                ctx.specified_style.margin_left = offset.top.into();
            }
        }
        _ => {
            ctx.record_error(
                "margin-block-start",
                value.to_vec(),
                String::from("Unsupported writing mode"),
            );
        }
    }
}

/// Handles the `margin-block-end` property by parsing the provided component values and updating the corresponding margin properties in the specified style based on the current writing mode.
/// an error is recorded in the context indicating that the writing mode is unsupported.
pub(crate) fn handle_margin_block_end(ctx: &mut PropertyUpdateContext, value: &[ComponentValue]) {
    let global = Global::try_from(value).ok();
    let offset = Offset::try_from(value).ok();

    let writing_mode = match ctx.specified_style.writing_mode {
        CSSProperty::Global(_) => ctx.specified_style.writing_mode.resolve_with_context_owned(
            ctx.relative_ctx.parent.writing_mode,
            WritingMode::HorizontalTb,
        ),
        CSSProperty::Value(val) => val,
    };

    match writing_mode {
        WritingMode::HorizontalTb => {
            if let Some(global) = global {
                ctx.specified_style.margin_bottom = CSSProperty::Global(global);
            } else if let Some(offset) = offset {
                ctx.specified_style.margin_bottom = offset.top.into();
            }
        }
        WritingMode::VerticalRl => {
            if let Some(global) = global {
                ctx.specified_style.margin_left = CSSProperty::Global(global);
            } else if let Some(offset) = offset {
                ctx.specified_style.margin_left = offset.top.into();
            }
        }
        WritingMode::VerticalLr => {
            if let Some(global) = global {
                ctx.specified_style.margin_right = CSSProperty::Global(global);
            } else if let Some(offset) = offset {
                ctx.specified_style.margin_right = offset.top.into();
            }
        }
        _ => {
            ctx.record_error(
                "margin-block-end",
                value.to_vec(),
                String::from("Unsupported writing mode"),
            );
        }
    }
}

/// Handles the `padding-block-start` property by parsing the provided component values and updating the corresponding padding properties in the specified style based on the current writing mode.
/// recorded in the context indicating that the writing mode is unsupported.
pub(crate) fn handle_padding_block(ctx: &mut PropertyUpdateContext, value: &[ComponentValue]) {
    let global = Global::try_from(value).ok();
    let offset = Offset::try_from(value).ok();

    let writing_mode = match ctx.specified_style.writing_mode {
        CSSProperty::Global(_) => ctx.specified_style.writing_mode.resolve_with_context_owned(
            ctx.relative_ctx.parent.writing_mode,
            WritingMode::HorizontalTb,
        ),
        CSSProperty::Value(val) => val,
    };

    match writing_mode {
        WritingMode::HorizontalTb => {
            if let Some(global) = global {
                ctx.specified_style.padding_top = CSSProperty::Global(global);
                ctx.specified_style.padding_bottom = CSSProperty::Global(global);
            } else if let Some(offset) = offset {
                ctx.specified_style.padding_top = offset.top.into();
                ctx.specified_style.padding_bottom = offset.bottom.into();
            }
        }
        WritingMode::VerticalRl => {
            if let Some(global) = global {
                ctx.specified_style.padding_right = CSSProperty::Global(global);
                ctx.specified_style.padding_left = CSSProperty::Global(global);
            } else if let Some(offset) = offset {
                ctx.specified_style.padding_right = offset.top.into();
                ctx.specified_style.padding_left = offset.bottom.into();
            }
        }
        WritingMode::VerticalLr => {
            if let Some(global) = global {
                ctx.specified_style.padding_left = CSSProperty::Global(global);
                ctx.specified_style.padding_right = CSSProperty::Global(global);
            } else if let Some(offset) = offset {
                ctx.specified_style.padding_left = offset.top.into();
                ctx.specified_style.padding_right = offset.bottom.into();
            }
        }
        _ => {
            ctx.record_error(
                "padding-block",
                value.to_vec(),
                String::from("Unsupported writing mode"),
            );
        }
    }
}

/// Handles the `padding-block-start` property by parsing the provided component values and updating the corresponding padding property in the specified style based on the current writing mode.
pub(crate) fn handle_padding_block_start(
    ctx: &mut PropertyUpdateContext,
    value: &[ComponentValue],
) {
    let global = Global::try_from(value).ok();
    let offset = Offset::try_from(value).ok();

    let writing_mode = match ctx.specified_style.writing_mode {
        CSSProperty::Global(_) => ctx.specified_style.writing_mode.resolve_with_context_owned(
            ctx.relative_ctx.parent.writing_mode,
            WritingMode::HorizontalTb,
        ),
        CSSProperty::Value(val) => val,
    };

    match writing_mode {
        WritingMode::HorizontalTb => {
            if let Some(global) = global {
                ctx.specified_style.padding_top = CSSProperty::Global(global);
            } else if let Some(offset) = offset {
                ctx.specified_style.padding_top = offset.top.into();
            }
        }
        WritingMode::VerticalRl => {
            if let Some(global) = global {
                ctx.specified_style.padding_right = CSSProperty::Global(global);
            } else if let Some(offset) = offset {
                ctx.specified_style.padding_right = offset.top.into();
            }
        }
        WritingMode::VerticalLr => {
            if let Some(global) = global {
                ctx.specified_style.padding_left = CSSProperty::Global(global);
            } else if let Some(offset) = offset {
                ctx.specified_style.padding_left = offset.top.into();
            }
        }
        _ => {
            ctx.record_error(
                "padding-block-start",
                value.to_vec(),
                String::from("Unsupported writing mode"),
            );
        }
    }
}

/// Handles the `margin-block-end` property by parsing the provided component values and updating the corresponding margin properties in the specified style based on the current writing mode.
pub(crate) fn handle_padding_block_end(ctx: &mut PropertyUpdateContext, value: &[ComponentValue]) {
    let global = Global::try_from(value).ok();
    let offset = Offset::try_from(value).ok();

    let writing_mode = match ctx.specified_style.writing_mode {
        CSSProperty::Global(_) => ctx.specified_style.writing_mode.resolve_with_context_owned(
            ctx.relative_ctx.parent.writing_mode,
            WritingMode::HorizontalTb,
        ),
        CSSProperty::Value(val) => val,
    };

    match writing_mode {
        WritingMode::HorizontalTb => {
            if let Some(global) = global {
                ctx.specified_style.padding_bottom = CSSProperty::Global(global);
            } else if let Some(offset) = offset {
                ctx.specified_style.padding_bottom = offset.top.into();
            }
        }
        WritingMode::VerticalRl => {
            if let Some(global) = global {
                ctx.specified_style.padding_left = CSSProperty::Global(global);
            } else if let Some(offset) = offset {
                ctx.specified_style.padding_left = offset.top.into();
            }
        }
        WritingMode::VerticalLr => {
            if let Some(global) = global {
                ctx.specified_style.padding_right = CSSProperty::Global(global);
            } else if let Some(offset) = offset {
                ctx.specified_style.padding_right = offset.top.into();
            }
        }
        _ => {
            ctx.record_error(
                "padding-block-end",
                value.to_vec(),
                String::from("Unsupported writing mode"),
            );
        }
    }
}

use css_cssom::{ComponentValue, CssTokenKind, Property};

use crate::calculate::CalcExpression;
use crate::properties::text::WritingMode;
use crate::properties::{AbsoluteContext, CSSProperty};
use crate::specified::SpecifiedStyle;
use crate::{BorderStyleValue, BorderWidthValue, Color, OffsetValue, RelativeContext};

pub struct PropertyUpdateContext<'a> {
    pub absolute_ctx: &'a AbsoluteContext,
    pub specified_style: &'a mut SpecifiedStyle,
    pub relative_ctx: &'a RelativeContext,
    pub errors: Vec<PropertyError>,
}

#[derive(Debug)]
pub struct PropertyError {
    pub property: String,
    pub value: String,
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

    fn record_error(&mut self, property: &str, value: &str, error: String) {
        self.errors.push(PropertyError {
            property: property.to_string(),
            value: value.to_string(),
            error,
        });
    }

    pub fn log_errors(&self) {
        if !self.errors.is_empty() {
            eprintln!("Property update errors:");
            for err in &self.errors {
                eprintln!("  {}: '{}' - {}", err.property, err.value, err.error);
            }
        }
    }
}

macro_rules! simple_property_handler {
    ($fn_name:ident, $field:ident, $prop_name:expr) => {
        pub fn $fn_name(ctx: &mut PropertyUpdateContext, value: &str) {
            if let Err(e) = CSSProperty::update_property(&mut ctx.specified_style.$field, value) {
                ctx.record_error($prop_name, value, e);
            }
        }
    };
}

fn split_top_level_whitespace(input: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth = 0usize;
    let mut token_start: Option<usize> = None;

    for (idx, ch) in input.char_indices() {
        if ch.is_whitespace() && depth == 0 {
            if let Some(start) = token_start.take() {
                parts.push(&input[start..idx]);
            }
            continue;
        }

        if token_start.is_none() {
            token_start = Some(idx);
        }

        match ch {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            _ => {}
        }
    }

    if let Some(start) = token_start {
        parts.push(&input[start..]);
    }

    parts
}

fn split_var_segments(value: &str) -> Vec<&str> {
    let mut segments = Vec::new();
    let mut start = 0;
    let mut i = 0;
    let bytes = value.as_bytes();

    while i < bytes.len() {
        if value[i..].starts_with("var(") {
            if i > start {
                segments.push(&value[start..i]);
            }

            let var_start = i;
            let mut depth = 0;
            while i < bytes.len() {
                match bytes[i] {
                    b'(' => depth += 1,
                    b')' => {
                        depth -= 1;
                        if depth == 0 {
                            i += 1;
                            break;
                        }
                    }
                    _ => {}
                }
                i += 1;
            }

            segments.push(&value[var_start..i]);
            start = i;
        } else {
            i += 1;
        }
    }

    if start < value.len() {
        segments.push(&value[start..]);
    }

    segments
}

pub fn resolve_css_variables(
    variables: &[(Property, Vec<ComponentValue>)],
    value: String,
) -> String {
    let segments = split_var_segments(&value);

    if segments.len() == 1 && !segments[0].starts_with("var(") {
        return value;
    }

    let mut output = String::new();
    let mut prev_was_var = false;

    for segment in segments {
        let is_var = segment.starts_with("var(");
        let resolved = if is_var {
            resolve_single_var(variables, segment)
        } else {
            segment.to_string()
        };

        if prev_was_var && is_var {
            output.push(' ');
        }

        output.push_str(&resolved);
        prev_was_var = is_var;
    }

    output
}

fn resolve_single_var(variables: &[(Property, Vec<ComponentValue>)], value: &str) -> String {
    let Some(inner) = value.strip_prefix("var(").and_then(|s| s.strip_suffix(')')) else {
        return value.to_string();
    };

    let (var_name, fallback) = match inner.find(',') {
        Some(pos) => (inner[..pos].trim(), Some(inner[pos + 1..].trim())),
        None => (inner.trim(), None),
    };

    if let Some(resolved) = try_resolve(variables, var_name) {
        return resolved;
    }

    if let Some(fallback_value) = fallback {
        if fallback_value.starts_with("var(") {
            return resolve_single_var(variables, fallback_value);
        }
        return fallback_value.to_string();
    }

    value.to_string()
}

fn try_resolve(variables: &[(Property, Vec<ComponentValue>)], var_name: &str) -> Option<String> {
    for (name, vals) in variables {
        if name.to_string() != var_name {
            continue;
        }

        let resolved = vals
            .iter()
            .filter_map(|cv| match cv {
                ComponentValue::Token(token) => {
                    if token.kind == CssTokenKind::Whitespace {
                        Some(" ".to_string())
                    } else {
                        Some(cv.to_css_string())
                    }
                }
                ComponentValue::Function(func) if func.name.eq_ignore_ascii_case("var") => {
                    let inner = func
                        .value
                        .iter()
                        .map(|cv| cv.to_css_string())
                        .collect::<String>();
                    let reconstructed = format!("var({})", inner);
                    Some(resolve_single_var(variables, &reconstructed))
                }
                ComponentValue::Function(_) => Some(cv.to_css_string()),
                _ => None,
            })
            .collect::<String>();

        if resolved.is_empty() {
            return None;
        }

        return Some(resolved.trim().to_string());
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
simple_property_handler!(handle_font_weight, font_weight, "font-weight");
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

pub fn handle_margin(ctx: &mut PropertyUpdateContext, value: &str) {
    let parts: Vec<&str> = value.split_whitespace().collect();
    match parts.len() {
        1 => {
            if let Ok(all) = parts[0].parse::<CSSProperty<OffsetValue>>() {
                CSSProperty::update_multiple(
                    &mut [
                        &mut ctx.specified_style.margin_top,
                        &mut ctx.specified_style.margin_right,
                        &mut ctx.specified_style.margin_bottom,
                        &mut ctx.specified_style.margin_left,
                    ],
                    all,
                );
            } else {
                ctx.record_error(
                    "margin",
                    value,
                    format!("Invalid OffsetValue: {}", parts[0]),
                );
            }
        }
        2 => {
            if let (Ok(vertical), Ok(horizontal)) = (
                parts[0].parse::<OffsetValue>(),
                parts[1].parse::<OffsetValue>(),
            ) {
                CSSProperty::update_multiple(
                    &mut [
                        &mut ctx.specified_style.margin_top,
                        &mut ctx.specified_style.margin_bottom,
                    ],
                    vertical.into(),
                );
                CSSProperty::update_multiple(
                    &mut [
                        &mut ctx.specified_style.margin_right,
                        &mut ctx.specified_style.margin_left,
                    ],
                    horizontal.into(),
                );
            } else {
                ctx.record_error(
                    "margin",
                    value,
                    format!("Invalid OffsetValues: {} {}", parts[0], parts[1]),
                );
            }
        }
        3 => {
            if let (Ok(top), Ok(horizontal), Ok(bottom)) = (
                parts[0].parse::<OffsetValue>(),
                parts[1].parse::<OffsetValue>(),
                parts[2].parse::<OffsetValue>(),
            ) {
                CSSProperty::update(&mut ctx.specified_style.margin_top, top.into());
                CSSProperty::update(&mut ctx.specified_style.margin_bottom, bottom.into());

                CSSProperty::update_multiple(
                    &mut [
                        &mut ctx.specified_style.margin_right,
                        &mut ctx.specified_style.margin_left,
                    ],
                    horizontal.into(),
                );
            } else {
                ctx.record_error(
                    "margin",
                    value,
                    format!(
                        "Invalid OffsetValues: {} {} {}",
                        parts[0], parts[1], parts[2]
                    ),
                );
            }
        }
        4 => {
            if let (Ok(top), Ok(right), Ok(bottom), Ok(left)) = (
                parts[0].parse::<OffsetValue>(),
                parts[1].parse::<OffsetValue>(),
                parts[2].parse::<OffsetValue>(),
                parts[3].parse::<OffsetValue>(),
            ) {
                {
                    CSSProperty::update(&mut ctx.specified_style.margin_top, top.into());
                    CSSProperty::update(&mut ctx.specified_style.margin_right, right.into());
                    CSSProperty::update(&mut ctx.specified_style.margin_bottom, bottom.into());
                    CSSProperty::update(&mut ctx.specified_style.margin_left, left.into());
                }
            } else {
                ctx.record_error(
                    "margin",
                    value,
                    format!(
                        "Invalid OffsetValues: {} {} {} {}",
                        parts[0], parts[1], parts[2], parts[3]
                    ),
                );
            }
        }
        _ => {
            ctx.record_error(
                "margin",
                value,
                format!("Invalid number of values: {}", parts.len()),
            );
        }
    }
}

pub fn handle_padding(ctx: &mut PropertyUpdateContext, value: &str) {
    let parts: Vec<&str> = value.split_whitespace().collect();
    match parts.len() {
        1 => {
            if let Ok(all) = parts[0].parse::<OffsetValue>() {
                CSSProperty::update_multiple(
                    &mut [
                        &mut ctx.specified_style.padding_top,
                        &mut ctx.specified_style.padding_right,
                        &mut ctx.specified_style.padding_bottom,
                        &mut ctx.specified_style.padding_left,
                    ],
                    all.into(),
                );
            } else {
                ctx.record_error(
                    "padding",
                    value,
                    format!("Invalid OffsetValue: {}", parts[0]),
                );
            }
        }
        2 => {
            if let (Ok(vertical), Ok(horizontal)) = (
                parts[0].parse::<OffsetValue>(),
                parts[1].parse::<OffsetValue>(),
            ) {
                CSSProperty::update_multiple(
                    &mut [
                        &mut ctx.specified_style.padding_top,
                        &mut ctx.specified_style.padding_bottom,
                    ],
                    vertical.into(),
                );
                CSSProperty::update_multiple(
                    &mut [
                        &mut ctx.specified_style.padding_right,
                        &mut ctx.specified_style.padding_left,
                    ],
                    horizontal.into(),
                );
            } else {
                ctx.record_error(
                    "padding",
                    value,
                    format!("Invalid OffsetValues: {} {}", parts[0], parts[1]),
                );
            }
        }
        3 => {
            if let (Ok(top), Ok(horizontal), Ok(bottom)) = (
                parts[0].parse::<OffsetValue>(),
                parts[1].parse::<OffsetValue>(),
                parts[2].parse::<OffsetValue>(),
            ) {
                CSSProperty::update(&mut ctx.specified_style.padding_top, top.into());
                CSSProperty::update(&mut ctx.specified_style.padding_bottom, bottom.into());
                CSSProperty::update_multiple(
                    &mut [
                        &mut ctx.specified_style.padding_right,
                        &mut ctx.specified_style.padding_left,
                    ],
                    horizontal.into(),
                );
            } else {
                ctx.record_error(
                    "padding",
                    value,
                    format!(
                        "Invalid OffsetValues: {} {} {}",
                        parts[0], parts[1], parts[2]
                    ),
                );
            }
        }
        4 => {
            if let (Ok(top), Ok(right), Ok(bottom), Ok(left)) = (
                parts[0].parse::<OffsetValue>(),
                parts[1].parse::<OffsetValue>(),
                parts[2].parse::<OffsetValue>(),
                parts[3].parse::<OffsetValue>(),
            ) {
                {
                    CSSProperty::update(&mut ctx.specified_style.padding_top, top.into());
                    CSSProperty::update(&mut ctx.specified_style.padding_right, right.into());
                    CSSProperty::update(&mut ctx.specified_style.padding_bottom, bottom.into());
                    CSSProperty::update(&mut ctx.specified_style.padding_left, left.into());
                }
            } else {
                ctx.record_error(
                    "padding",
                    value,
                    format!(
                        "Invalid OffsetValues: {} {} {} {}",
                        parts[0], parts[1], parts[2], parts[3]
                    ),
                );
            }
        }
        _ => {
            ctx.record_error(
                "padding",
                value,
                format!("Invalid number of values: {}", parts.len()),
            );
        }
    }
}

pub fn handle_border(ctx: &mut PropertyUpdateContext, value: &str) {
    if value.eq_ignore_ascii_case("none") {
        CSSProperty::update_multiple(
            &mut [
                &mut ctx.specified_style.border_top_style,
                &mut ctx.specified_style.border_right_style,
                &mut ctx.specified_style.border_bottom_style,
                &mut ctx.specified_style.border_left_style,
            ],
            BorderStyleValue::None.into(),
        );
        return;
    }

    let parts = split_top_level_whitespace(value);

    for part in parts {
        if let Ok(width) = part.parse::<BorderWidthValue>() {
            CSSProperty::update_multiple(
                &mut [
                    &mut ctx.specified_style.border_top_width,
                    &mut ctx.specified_style.border_right_width,
                    &mut ctx.specified_style.border_bottom_width,
                    &mut ctx.specified_style.border_left_width,
                ],
                width.into(),
            );
        }

        if let Ok(style) = part.parse::<BorderStyleValue>() {
            CSSProperty::update_multiple(
                &mut [
                    &mut ctx.specified_style.border_top_style,
                    &mut ctx.specified_style.border_right_style,
                    &mut ctx.specified_style.border_bottom_style,
                    &mut ctx.specified_style.border_left_style,
                ],
                style.into(),
            );
        }

        if let Ok(color) = part.parse::<Color>() {
            CSSProperty::update_multiple(
                &mut [
                    &mut ctx.specified_style.border_top_color,
                    &mut ctx.specified_style.border_right_color,
                    &mut ctx.specified_style.border_bottom_color,
                    &mut ctx.specified_style.border_left_color,
                ],
                color.into(),
            );
        }
    }
}

pub fn handle_font_size(ctx: &mut PropertyUpdateContext, value: &str) {
    CSSProperty::update_property(&mut ctx.specified_style.font_size, value).unwrap_or(());

    if let Ok(font_size) = CSSProperty::resolve(&ctx.specified_style.font_size) {
        let parent_px = ctx.relative_ctx.parent_font_size;
        ctx.specified_style.computed_font_size_px = font_size.to_px(ctx.absolute_ctx, parent_px);
    }
}

pub fn handle_margin_block(ctx: &mut PropertyUpdateContext, value: &str) {
    if value.starts_with("calc") {
        let calc = if let Ok(calc) = CalcExpression::parse(value) {
            calc
        } else {
            ctx.record_error(
                "margin-block",
                value,
                format!("Invalid calc expression: {}", value),
            );
            return;
        };

        let val = calc.to_px(None, ctx.relative_ctx, ctx.absolute_ctx);
        CSSProperty::update_multiple(
            &mut [
                &mut ctx.specified_style.margin_top,
                &mut ctx.specified_style.margin_right,
                &mut ctx.specified_style.margin_bottom,
                &mut ctx.specified_style.margin_left,
            ],
            OffsetValue::px(val).into(),
        );
        return;
    }

    let parts: Vec<&str> = value.split_whitespace().collect();
    match parts.len() {
        1 => {
            handle_margin_block_start(ctx, parts[0]);
            handle_margin_block_end(ctx, parts[0]);
        }
        2 => {
            handle_margin_block_start(ctx, parts[0]);
            handle_margin_block_end(ctx, parts[1]);
        }
        _ => {
            ctx.record_error(
                "margin-block",
                value,
                format!("Invalid number of values: {}", parts.len()),
            );
        }
    }
}

pub fn handle_margin_block_start(ctx: &mut PropertyUpdateContext, value: &str) {
    match ctx.specified_style.writing_mode {
        CSSProperty::Global(global) => {
            ctx.record_error(
                "margin-block-start",
                value,
                format!("Unsupported global value: {:?}", global),
            );
        }
        CSSProperty::Value(val) => match val {
            WritingMode::HorizontalTb => {
                if let Err(e) =
                    CSSProperty::update_property(&mut ctx.specified_style.margin_top, value)
                {
                    ctx.record_error("margin-block-start", value, e);
                }
            }
            WritingMode::VerticalRl => {
                if let Err(e) =
                    CSSProperty::update_property(&mut ctx.specified_style.margin_right, value)
                {
                    ctx.record_error("margin-block-start", value, e);
                }
            }
            WritingMode::VerticalLr => {
                if let Err(e) =
                    CSSProperty::update_property(&mut ctx.specified_style.margin_left, value)
                {
                    ctx.record_error("margin-block-start", value, e);
                }
            }
            _ => {
                ctx.record_error(
                    "margin-block-start",
                    value,
                    String::from("Unsupported writing mode"),
                );
            }
        },
    }
}

pub fn handle_margin_block_end(ctx: &mut PropertyUpdateContext, value: &str) {
    match ctx.specified_style.writing_mode {
        CSSProperty::Global(global) => {
            ctx.record_error(
                "margin-block-end",
                value,
                format!("Unsupported global value: {:?}", global),
            );
        }
        CSSProperty::Value(val) => match val {
            WritingMode::HorizontalTb => {
                if let Err(e) =
                    CSSProperty::update_property(&mut ctx.specified_style.margin_bottom, value)
                {
                    ctx.record_error("margin-block-end", value, e);
                }
            }
            WritingMode::VerticalRl => {
                if let Err(e) =
                    CSSProperty::update_property(&mut ctx.specified_style.margin_left, value)
                {
                    ctx.record_error("margin-block-end", value, e);
                }
            }
            WritingMode::VerticalLr => {
                if let Err(e) =
                    CSSProperty::update_property(&mut ctx.specified_style.margin_right, value)
                {
                    ctx.record_error("margin-block-end", value, e);
                }
            }
            _ => {
                ctx.record_error(
                    "margin-block-end",
                    value,
                    String::from("Unsupported writing mode"),
                );
            }
        },
    }
}

pub fn handle_padding_block(ctx: &mut PropertyUpdateContext, value: &str) {
    if value.starts_with("calc") {
        let calc = if let Ok(calc) = CalcExpression::parse(value) {
            calc
        } else {
            ctx.record_error(
                "padding-block",
                value,
                format!("Invalid calc expression: {}", value),
            );
            return;
        };

        let val = calc.to_px(None, ctx.relative_ctx, ctx.absolute_ctx);
        CSSProperty::update_multiple(
            &mut [
                &mut ctx.specified_style.padding_top,
                &mut ctx.specified_style.padding_right,
                &mut ctx.specified_style.padding_bottom,
                &mut ctx.specified_style.padding_left,
            ],
            OffsetValue::px(val).into(),
        );
        return;
    }

    let parts = value.split_whitespace().collect::<Vec<&str>>();

    match parts.len() {
        1 => {
            handle_padding_block_start(ctx, parts[0]);
            handle_padding_block_end(ctx, parts[0]);
        }
        2 => {
            handle_padding_block_start(ctx, parts[0]);
            handle_padding_block_end(ctx, parts[1]);
        }
        _ => {
            ctx.record_error(
                "padding-block",
                value,
                format!("Invalid number of values: {}", parts.len()),
            );
        }
    }
}

pub fn handle_padding_block_start(ctx: &mut PropertyUpdateContext, value: &str) {
    match ctx.specified_style.writing_mode {
        CSSProperty::Global(global) => {
            ctx.record_error(
                "padding-block-start",
                value,
                format!("Unsupported global value: {:?}", global),
            );
        }
        CSSProperty::Value(val) => match val {
            WritingMode::HorizontalTb => {
                if let Err(e) =
                    CSSProperty::update_property(&mut ctx.specified_style.padding_top, value)
                {
                    ctx.record_error("padding-block-start", value, e);
                }
            }
            WritingMode::VerticalRl => {
                if let Err(e) =
                    CSSProperty::update_property(&mut ctx.specified_style.padding_right, value)
                {
                    ctx.record_error("padding-block-start", value, e);
                }
            }
            WritingMode::VerticalLr => {
                if let Err(e) =
                    CSSProperty::update_property(&mut ctx.specified_style.padding_left, value)
                {
                    ctx.record_error("padding-block-start", value, e);
                }
            }
            _ => {
                ctx.record_error(
                    "padding-block-start",
                    value,
                    String::from("Unsupported writing mode"),
                );
            }
        },
    }
}

pub fn handle_padding_block_end(ctx: &mut PropertyUpdateContext, value: &str) {
    match ctx.specified_style.writing_mode {
        CSSProperty::Global(global) => {
            ctx.record_error(
                "padding-block-end",
                value,
                format!("Unsupported global value: {:?}", global),
            );
        }
        CSSProperty::Value(val) => match val {
            WritingMode::HorizontalTb => {
                if let Err(e) =
                    CSSProperty::update_property(&mut ctx.specified_style.padding_bottom, value)
                {
                    ctx.record_error("padding-block-end", value, e);
                }
            }
            WritingMode::VerticalRl => {
                if let Err(e) =
                    CSSProperty::update_property(&mut ctx.specified_style.padding_left, value)
                {
                    ctx.record_error("padding-block-end", value, e);
                }
            }
            WritingMode::VerticalLr => {
                if let Err(e) =
                    CSSProperty::update_property(&mut ctx.specified_style.padding_right, value)
                {
                    ctx.record_error("padding-block-end", value, e);
                }
            }
            _ => {
                ctx.record_error(
                    "padding-block-end",
                    value,
                    String::from("Unsupported writing mode"),
                );
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use css_cssom::{CssToken, Function};

    use super::*;
    use crate::{BorderStyleValue, BorderWidthValue};

    #[test]
    fn test_split_top_level_whitespace_preserves_functions() {
        let parts = split_top_level_whitespace("calc(100% - 2px) solid rgb(255 0 0 / 0.5)");
        assert_eq!(
            parts,
            vec!["calc(100% - 2px)", "solid", "rgb(255 0 0 / 0.5)"]
        );
    }

    #[test]
    fn test_handle_border_with_calc_and_function_color() {
        let absolute_ctx = AbsoluteContext::default();
        let relative_ctx = RelativeContext {
            parent_font_size: 16.0,
            ..Default::default()
        };
        let mut style = SpecifiedStyle::default();
        let mut ctx = PropertyUpdateContext::new(&absolute_ctx, &mut style, &relative_ctx);

        handle_border(&mut ctx, "calc(100% - 2px) solid rgb(255 0 0 / 0.5)");

        assert!(ctx.errors.is_empty());
        assert_eq!(
            CSSProperty::resolve(&ctx.specified_style.border_top_style),
            Ok(&BorderStyleValue::Solid)
        );
        assert!(matches!(
            CSSProperty::resolve(&ctx.specified_style.border_top_width),
            Ok(BorderWidthValue::Calc(_))
        ));
    }

    #[test]
    fn test_simple_variable_parsing() {
        let variables = vec![
            (
                Property::Custom("--main-color".to_string()),
                vec![ComponentValue::Token(CssToken {
                    kind: CssTokenKind::Ident("blue".to_string()),
                    position: None,
                })],
            ),
            (
                Property::Custom("--fallback-color".to_string()),
                vec![ComponentValue::Token(CssToken {
                    kind: CssTokenKind::Ident("red".to_string()),
                    position: None,
                })],
            ),
        ];

        let value = "var(--main-color, var(--fallback-color, green))".to_string();
        let resolved = resolve_css_variables(&variables, value);
        assert_eq!(resolved, "blue");
    }

    #[test]
    fn test_variable_with_fallback() {
        let variables = vec![(
            Property::Custom("--bg-color".to_string()),
            vec![ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("red".to_string()),
                position: None,
            })],
        )];

        let value = "var(--undefined-color, var(--bg-color, blue))".to_string();
        let resolved = resolve_css_variables(&variables, value);
        assert_eq!(resolved, "red");
    }

    #[test]
    fn test_nested_variable_resolution() {
        let variables = vec![
            (
                Property::Custom("--color-a".to_string()),
                vec![ComponentValue::Function(Function {
                    name: "var".to_string(),
                    value: vec![ComponentValue::Token(CssToken {
                        kind: CssTokenKind::Ident("--color-b".to_string()),
                        position: None,
                    })],
                })],
            ),
            (
                Property::Custom("--color-b".to_string()),
                vec![ComponentValue::Token(CssToken {
                    kind: CssTokenKind::Ident("green".to_string()),
                    position: None,
                })],
            ),
        ];

        let value = "var(--color-a, blue)".to_string();
        let resolved = resolve_css_variables(&variables, value);
        assert_eq!(resolved, "green");
    }

    #[test]
    fn test_chained_variables() {
        let variables = vec![
            (
                Property::Custom("--padding-vertical".to_string()),
                vec![ComponentValue::Token(CssToken {
                    kind: CssTokenKind::Ident("20px".to_string()),
                    position: None,
                })],
            ),
            (
                Property::Custom("--padding-horizontal".to_string()),
                vec![ComponentValue::Token(CssToken {
                    kind: CssTokenKind::Ident("50px".to_string()),
                    position: None,
                })],
            ),
        ];

        let value = "var(--padding-vertical) var(--padding-horizontal)".to_string();
        let resolved = resolve_css_variables(&variables, value);
        assert_eq!(resolved, "20px 50px");

        let value = "var(--padding-vertical)var(--padding-horizontal)".to_string();
        let resolved = resolve_css_variables(&variables, value);
        assert_eq!(resolved, "20px 50px");
    }

    #[test]
    fn test_variable_with_no_fallback() {
        let variables = vec![];

        let value = "var(--undefined-color)".to_string();
        let resolved = resolve_css_variables(&variables, value);
        assert_eq!(resolved, "var(--undefined-color)");
    }

    #[test]
    fn test_mixed_variable_and_non_variable_segments() {
        let variables = vec![(
            Property::Custom("--main-color".to_string()),
            vec![ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("blue".to_string()),
                position: None,
            })],
        )];

        let value = "1px solid var(--main-color)".to_string();
        let resolved = resolve_css_variables(&variables, value);
        assert_eq!(resolved, "1px solid blue");
    }

    #[test]
    fn test_mixed_variable_suffix() {
        let variables = vec![
            (
                Property::Custom("--padding-vertical".to_string()),
                vec![ComponentValue::Token(CssToken {
                    kind: CssTokenKind::Ident("2".to_string()),
                    position: None,
                })],
            ),
            (
                Property::Custom("--padding-horizontal".to_string()),
                vec![ComponentValue::Token(CssToken {
                    kind: CssTokenKind::Ident("5".to_string()),
                    position: None,
                })],
            ),
        ];

        let value = "var(--padding-vertical)rem var(--padding-horizontal)vh".to_string();
        let resolved = resolve_css_variables(&variables, value);
        assert_eq!(resolved, "2rem 5vh");
    }
}

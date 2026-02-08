use std::str::FromStr;

use css_cssom::Property;

use crate::primitives::font::AbsoluteSize;
use crate::properties::text::WritingMode;
use crate::properties::{AbsoluteContext, CSSProperty};
use crate::{BorderStyleValue, BorderWidthValue, Color, ComputedStyle, OffsetValue};

pub struct PropertyUpdateContext<'a> {
    pub absolute_ctx: &'a AbsoluteContext,
    pub computed_style: &'a mut ComputedStyle,
    pub parent_style: Option<&'a ComputedStyle>,
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
        computed_style: &'a mut ComputedStyle,
        parent_style: Option<&'a ComputedStyle>,
    ) -> Self {
        Self {
            absolute_ctx,
            computed_style,
            parent_style,
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
            if let Err(e) = CSSProperty::update_property(&mut ctx.computed_style.$field, value) {
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

pub fn resolve_css_variable(
    variables: &[(Property, String)],
    value: String,
    variable_fallback: String,
) -> String {
    let mut result = value;
    let max_iterations = 10;

    for _ in 0..max_iterations {
        let Some(var_start) = result.find("var(") else {
            break;
        };

        let content_start = var_start + 4;
        let mut depth = 1;
        let mut end_pos = None;
        for (i, ch) in result[content_start..].char_indices() {
            match ch {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        end_pos = Some(content_start + i);
                        break;
                    }
                }
                _ => {}
            }
        }

        let Some(closing_paren) = end_pos else {
            break;
        };

        let inner = &result[content_start..closing_paren];

        let mut comma_pos = None;
        let mut depth = 0;
        for (i, ch) in inner.char_indices() {
            match ch {
                '(' => depth += 1,
                ')' => depth -= 1,
                ',' if depth == 0 => {
                    comma_pos = Some(i);
                    break;
                }
                _ => {}
            }
        }

        let var_name = match comma_pos {
            Some(pos) => inner[..pos].trim(),
            None => inner.trim(),
        };
        let prop = if let Ok(p) = Property::from_str(var_name) {
            p
        } else {
            break;
        };

        let fallback = comma_pos.map(|pos| inner[pos + 1..].trim().to_string());

        let resolved = if let Some((_, val)) = variables.iter().find(|(name, _)| *name == prop) {
            val.clone()
        } else {
            fallback.unwrap_or_else(|| variable_fallback.clone())
        };

        result = format!(
            "{}{}{}",
            &result[..var_start],
            resolved,
            &result[closing_paren + 1..]
        );
    }

    result
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
            if let Ok(all) = parts[0].parse::<OffsetValue>() {
                CSSProperty::update_multiple(
                    &mut [
                        &mut ctx.computed_style.margin_top,
                        &mut ctx.computed_style.margin_right,
                        &mut ctx.computed_style.margin_bottom,
                        &mut ctx.computed_style.margin_left,
                    ],
                    all.into(),
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
                        &mut ctx.computed_style.margin_top,
                        &mut ctx.computed_style.margin_bottom,
                    ],
                    vertical.into(),
                );
                CSSProperty::update_multiple(
                    &mut [
                        &mut ctx.computed_style.margin_right,
                        &mut ctx.computed_style.margin_left,
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
                CSSProperty::update(&mut ctx.computed_style.margin_top, top.into());
                CSSProperty::update(&mut ctx.computed_style.margin_bottom, bottom.into());

                CSSProperty::update_multiple(
                    &mut [
                        &mut ctx.computed_style.margin_right,
                        &mut ctx.computed_style.margin_left,
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
                    CSSProperty::update(&mut ctx.computed_style.margin_top, top.into());
                    CSSProperty::update(&mut ctx.computed_style.margin_right, right.into());
                    CSSProperty::update(&mut ctx.computed_style.margin_bottom, bottom.into());
                    CSSProperty::update(&mut ctx.computed_style.margin_left, left.into());
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
                        &mut ctx.computed_style.padding_top,
                        &mut ctx.computed_style.padding_right,
                        &mut ctx.computed_style.padding_bottom,
                        &mut ctx.computed_style.padding_left,
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
                        &mut ctx.computed_style.padding_top,
                        &mut ctx.computed_style.padding_bottom,
                    ],
                    vertical.into(),
                );
                CSSProperty::update_multiple(
                    &mut [
                        &mut ctx.computed_style.padding_right,
                        &mut ctx.computed_style.padding_left,
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
                CSSProperty::update(&mut ctx.computed_style.padding_top, top.into());
                CSSProperty::update(&mut ctx.computed_style.padding_bottom, bottom.into());
                CSSProperty::update_multiple(
                    &mut [
                        &mut ctx.computed_style.padding_right,
                        &mut ctx.computed_style.padding_left,
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
                    CSSProperty::update(&mut ctx.computed_style.padding_top, top.into());
                    CSSProperty::update(&mut ctx.computed_style.padding_right, right.into());
                    CSSProperty::update(&mut ctx.computed_style.padding_bottom, bottom.into());
                    CSSProperty::update(&mut ctx.computed_style.padding_left, left.into());
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
                &mut ctx.computed_style.border_top_style,
                &mut ctx.computed_style.border_right_style,
                &mut ctx.computed_style.border_bottom_style,
                &mut ctx.computed_style.border_left_style,
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
                    &mut ctx.computed_style.border_top_width,
                    &mut ctx.computed_style.border_right_width,
                    &mut ctx.computed_style.border_bottom_width,
                    &mut ctx.computed_style.border_left_width,
                ],
                width.into(),
            );
        }

        if let Ok(style) = part.parse::<BorderStyleValue>() {
            CSSProperty::update_multiple(
                &mut [
                    &mut ctx.computed_style.border_top_style,
                    &mut ctx.computed_style.border_right_style,
                    &mut ctx.computed_style.border_bottom_style,
                    &mut ctx.computed_style.border_left_style,
                ],
                style.into(),
            );
        }

        if let Ok(color) = part.parse::<Color>() {
            CSSProperty::update_multiple(
                &mut [
                    &mut ctx.computed_style.border_top_color,
                    &mut ctx.computed_style.border_right_color,
                    &mut ctx.computed_style.border_bottom_color,
                    &mut ctx.computed_style.border_left_color,
                ],
                color.into(),
            );
        }
    }
}

pub fn handle_font_size(ctx: &mut PropertyUpdateContext, value: &str) {
    CSSProperty::update_property(&mut ctx.computed_style.font_size, value).unwrap_or(());

    if let Ok(font_size) = CSSProperty::resolve(&ctx.computed_style.font_size) {
        let parent_px = ctx
            .parent_style
            .map(|p| p.computed_font_size_px)
            .unwrap_or(AbsoluteSize::Medium.to_px());
        ctx.computed_style.computed_font_size_px = font_size.to_px(ctx.absolute_ctx, parent_px);
    }
}

pub fn handle_margin_block(ctx: &mut PropertyUpdateContext, value: &str) {
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
    match ctx.computed_style.writing_mode {
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
                    CSSProperty::update_property(&mut ctx.computed_style.margin_top, value)
                {
                    ctx.record_error("margin-block-start", value, e);
                }
            }
            WritingMode::VerticalRl => {
                if let Err(e) =
                    CSSProperty::update_property(&mut ctx.computed_style.margin_right, value)
                {
                    ctx.record_error("margin-block-start", value, e);
                }
            }
            WritingMode::VerticalLr => {
                if let Err(e) =
                    CSSProperty::update_property(&mut ctx.computed_style.margin_left, value)
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
    match ctx.computed_style.writing_mode {
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
                    CSSProperty::update_property(&mut ctx.computed_style.margin_bottom, value)
                {
                    ctx.record_error("margin-block-end", value, e);
                }
            }
            WritingMode::VerticalRl => {
                if let Err(e) =
                    CSSProperty::update_property(&mut ctx.computed_style.margin_left, value)
                {
                    ctx.record_error("margin-block-end", value, e);
                }
            }
            WritingMode::VerticalLr => {
                if let Err(e) =
                    CSSProperty::update_property(&mut ctx.computed_style.margin_right, value)
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
    match ctx.computed_style.writing_mode {
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
                    CSSProperty::update_property(&mut ctx.computed_style.padding_top, value)
                {
                    ctx.record_error("padding-block-start", value, e);
                }
            }
            WritingMode::VerticalRl => {
                if let Err(e) =
                    CSSProperty::update_property(&mut ctx.computed_style.padding_right, value)
                {
                    ctx.record_error("padding-block-start", value, e);
                }
            }
            WritingMode::VerticalLr => {
                if let Err(e) =
                    CSSProperty::update_property(&mut ctx.computed_style.padding_left, value)
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
    match ctx.computed_style.writing_mode {
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
                    CSSProperty::update_property(&mut ctx.computed_style.padding_bottom, value)
                {
                    ctx.record_error("padding-block-end", value, e);
                }
            }
            WritingMode::VerticalRl => {
                if let Err(e) =
                    CSSProperty::update_property(&mut ctx.computed_style.padding_left, value)
                {
                    ctx.record_error("padding-block-end", value, e);
                }
            }
            WritingMode::VerticalLr => {
                if let Err(e) =
                    CSSProperty::update_property(&mut ctx.computed_style.padding_right, value)
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
        let mut style = ComputedStyle::default();
        let mut ctx = PropertyUpdateContext::new(&absolute_ctx, &mut style, None);

        handle_border(&mut ctx, "calc(100% - 2px) solid rgb(255 0 0 / 0.5)");

        assert!(ctx.errors.is_empty());
        assert_eq!(
            CSSProperty::resolve(&ctx.computed_style.border_top_style),
            Ok(&BorderStyleValue::Solid)
        );
        assert!(matches!(
            CSSProperty::resolve(&ctx.computed_style.border_top_width),
            Ok(BorderWidthValue::Calc(_))
        ));
    }
}

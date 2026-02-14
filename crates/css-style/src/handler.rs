use css_cssom::{ComponentValue, CssToken, CssTokenKind, Function, Property};

use crate::length::{Length, LengthUnit};
use crate::properties::text::WritingMode;
use crate::properties::{AbsoluteContext, CSSProperty};
use crate::specified::SpecifiedStyle;
use crate::{BorderStyle, BorderWidth, Color, Offset, RelativeContext};

pub struct PropertyUpdateContext<'a> {
    pub absolute_ctx: &'a AbsoluteContext,
    pub specified_style: &'a mut SpecifiedStyle,
    pub relative_ctx: &'a RelativeContext,
    pub errors: Vec<PropertyError>,
}

#[derive(Debug)]
pub struct PropertyError {
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

macro_rules! simple_property_handler {
    ($fn_name:ident, $field:ident, $prop_name:expr) => {
        pub fn $fn_name(ctx: &mut PropertyUpdateContext, value: &[ComponentValue]) {
            if let Err(e) = CSSProperty::update_property(&mut ctx.specified_style.$field, value) {
                ctx.record_error($prop_name, value.to_vec(), e);
            }
        }
    };
}

pub fn resolve_css_variables(
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
            _ => {
                output.push(cv.clone());
            }
        }
    }

    output
}

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

pub fn handle_margin(ctx: &mut PropertyUpdateContext, value: &[ComponentValue]) {
    let offset = match Offset::try_from(value) {
        Ok(offset) => offset,
        Err(e) => {
            ctx.record_error("margin", value.to_vec(), e);
            return;
        }
    };
    ctx.specified_style.margin_top = offset.top.into();
    ctx.specified_style.margin_right = offset.right.into();
    ctx.specified_style.margin_bottom = offset.bottom.into();
    ctx.specified_style.margin_left = offset.left.into();
}

pub fn handle_padding(ctx: &mut PropertyUpdateContext, value: &[ComponentValue]) {
    let offset = match Offset::try_from(value) {
        Ok(offset) => offset,
        Err(e) => {
            ctx.record_error("padding", value.to_vec(), e);
            return;
        }
    };

    ctx.specified_style.padding_top = offset.top.into();
    ctx.specified_style.padding_right = offset.right.into();
    ctx.specified_style.padding_bottom = offset.bottom.into();
    ctx.specified_style.padding_left = offset.left.into();
}

pub fn handle_border(ctx: &mut PropertyUpdateContext, value: &[ComponentValue]) {
    let mut style = None;
    let mut width = None;
    let mut color = None;

    for cv in value {
        match cv {
            ComponentValue::Token(token) => match &token.kind {
                CssTokenKind::Ident(ident) => {
                    if ident.eq_ignore_ascii_case("none") {
                        break;
                    } else if let Ok(w) = BorderWidth::try_from(value)
                        && width.is_none()
                    {
                        width = Some(w);
                    } else if let Ok(s) = ident.parse::<BorderStyle>() {
                        style = Some(s);
                    } else if let Ok(c) = Color::try_from(value) {
                        color = Some(c);
                    }
                }
                CssTokenKind::Number(num) => {
                    if width.is_some() || num.value != 0.0 {
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
                            value.value as f32,
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

pub fn handle_font_size(ctx: &mut PropertyUpdateContext, value: &[ComponentValue]) {
    CSSProperty::update_property(&mut ctx.specified_style.font_size, value).unwrap_or(());

    if let Ok(font_size) = CSSProperty::resolve(&ctx.specified_style.font_size) {
        ctx.specified_style.computed_font_size_px =
            font_size.to_px(ctx.absolute_ctx, ctx.relative_ctx.parent.font_size);
    }
}

pub fn handle_margin_block(ctx: &mut PropertyUpdateContext, value: &[ComponentValue]) {
    let offset = match Offset::try_from(value) {
        Ok(offset) => offset,
        Err(e) => {
            ctx.record_error("padding", value.to_vec(), e);
            return;
        }
    };

    match ctx.specified_style.writing_mode {
        CSSProperty::Global(global) => {
            ctx.record_error(
                "margin-block",
                value.to_vec(),
                format!("Unsupported global value: {:?}", global),
            );
        }
        CSSProperty::Value(val) => match val {
            WritingMode::HorizontalTb => {
                ctx.specified_style.margin_top = offset.top.into();
                ctx.specified_style.margin_bottom = offset.bottom.into();
            }
            WritingMode::VerticalRl => {
                ctx.specified_style.margin_right = offset.top.into();
                ctx.specified_style.margin_left = offset.bottom.into();
            }
            WritingMode::VerticalLr => {
                ctx.specified_style.margin_left = offset.top.into();
                ctx.specified_style.margin_right = offset.bottom.into();
            }
            _ => {
                ctx.record_error(
                    "margin-block",
                    value.to_vec(),
                    String::from("Unsupported writing mode"),
                );
            }
        },
    }
}

pub fn handle_margin_block_start(ctx: &mut PropertyUpdateContext, value: &[ComponentValue]) {
    let offset = match Offset::try_from(value) {
        Ok(offset) => offset,
        Err(e) => {
            ctx.record_error("margin-block-start", value.to_vec(), e);
            return;
        }
    };

    match ctx.specified_style.writing_mode {
        CSSProperty::Global(global) => {
            ctx.record_error(
                "margin-block-start",
                value.to_vec(),
                format!("Unsupported global value: {:?}", global),
            );
        }
        CSSProperty::Value(val) => match val {
            WritingMode::HorizontalTb => {
                ctx.specified_style.margin_top = offset.top.into();
            }
            WritingMode::VerticalRl => {
                ctx.specified_style.margin_right = offset.top.into();
            }
            WritingMode::VerticalLr => {
                ctx.specified_style.margin_left = offset.top.into();
            }
            _ => {
                ctx.record_error(
                    "margin-block-start",
                    value.to_vec(),
                    String::from("Unsupported writing mode"),
                );
            }
        },
    }
}

pub fn handle_margin_block_end(ctx: &mut PropertyUpdateContext, value: &[ComponentValue]) {
    let offset = match Offset::try_from(value) {
        Ok(offset) => offset,
        Err(e) => {
            ctx.record_error("margin-block-end", value.to_vec(), e);
            return;
        }
    };

    match ctx.specified_style.writing_mode {
        CSSProperty::Global(global) => {
            ctx.record_error(
                "margin-block-end",
                value.to_vec(),
                format!("Unsupported global value: {:?}", global),
            );
        }
        CSSProperty::Value(val) => match val {
            WritingMode::HorizontalTb => {
                ctx.specified_style.margin_bottom = offset.top.into();
            }
            WritingMode::VerticalRl => {
                ctx.specified_style.margin_left = offset.top.into();
            }
            WritingMode::VerticalLr => {
                ctx.specified_style.margin_right = offset.top.into();
            }
            _ => {
                ctx.record_error(
                    "margin-block-end",
                    value.to_vec(),
                    String::from("Unsupported writing mode"),
                );
            }
        },
    }
}

pub fn handle_padding_block(ctx: &mut PropertyUpdateContext, value: &[ComponentValue]) {
    let offset = match Offset::try_from(value) {
        Ok(offset) => offset,
        Err(e) => {
            ctx.record_error("padding-block", value.to_vec(), e);
            return;
        }
    };

    match ctx.specified_style.writing_mode {
        CSSProperty::Global(global) => {
            ctx.record_error(
                "padding-block",
                value.to_vec(),
                format!("Unsupported global value: {:?}", global),
            );
        }
        CSSProperty::Value(val) => match val {
            WritingMode::HorizontalTb => {
                ctx.specified_style.padding_top = offset.top.into();
                ctx.specified_style.padding_bottom = offset.bottom.into();
            }
            WritingMode::VerticalRl => {
                ctx.specified_style.padding_right = offset.top.into();
                ctx.specified_style.padding_left = offset.bottom.into();
            }
            WritingMode::VerticalLr => {
                ctx.specified_style.padding_left = offset.top.into();
                ctx.specified_style.padding_right = offset.bottom.into();
            }
            _ => {
                ctx.record_error(
                    "padding-block",
                    value.to_vec(),
                    String::from("Unsupported writing mode"),
                );
            }
        },
    }
}

pub fn handle_padding_block_start(ctx: &mut PropertyUpdateContext, value: &[ComponentValue]) {
    let offset = match Offset::try_from(value) {
        Ok(offset) => offset,
        Err(e) => {
            ctx.record_error("padding-block-start", value.to_vec(), e);
            return;
        }
    };

    match ctx.specified_style.writing_mode {
        CSSProperty::Global(global) => {
            ctx.record_error(
                "padding-block-start",
                value.to_vec(),
                format!("Unsupported global value: {:?}", global),
            );
        }
        CSSProperty::Value(val) => match val {
            WritingMode::HorizontalTb => {
                ctx.specified_style.padding_top = offset.top.into();
            }
            WritingMode::VerticalRl => {
                ctx.specified_style.padding_right = offset.top.into();
            }
            WritingMode::VerticalLr => {
                ctx.specified_style.padding_left = offset.top.into();
            }
            _ => {
                ctx.record_error(
                    "padding-block-start",
                    value.to_vec(),
                    String::from("Unsupported writing mode"),
                );
            }
        },
    }
}

pub fn handle_padding_block_end(ctx: &mut PropertyUpdateContext, value: &[ComponentValue]) {
    let offset = match Offset::try_from(value) {
        Ok(offset) => offset,
        Err(e) => {
            ctx.record_error("padding-block-end", value.to_vec(), e);
            return;
        }
    };

    match ctx.specified_style.writing_mode {
        CSSProperty::Global(global) => {
            ctx.record_error(
                "padding-block-end",
                value.to_vec(),
                format!("Unsupported global value: {:?}", global),
            );
        }
        CSSProperty::Value(val) => match val {
            WritingMode::HorizontalTb => {
                ctx.specified_style.padding_bottom = offset.top.into();
            }
            WritingMode::VerticalRl => {
                ctx.specified_style.padding_left = offset.top.into();
            }
            WritingMode::VerticalLr => {
                ctx.specified_style.padding_right = offset.top.into();
            }
            _ => {
                ctx.record_error(
                    "padding-block-end",
                    value.to_vec(),
                    String::from("Unsupported writing mode"),
                );
            }
        },
    }
}

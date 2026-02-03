use crate::length::Length;
use crate::primitives::font::AbsoluteSize;
use crate::properties::Property;
use crate::properties::text::WritingMode;
use crate::{
    BorderColor, BorderStyle, BorderStyleValue, BorderWidth, BorderWidthValue, Color, ComputedStyle,
};

pub struct PropertyUpdateContext<'a> {
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
        computed_style: &'a mut ComputedStyle,
        parent_style: Option<&'a ComputedStyle>,
    ) -> Self {
        Self {
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
            if let Err(e) = Property::update_property(&mut ctx.computed_style.$field, value) {
                ctx.record_error($prop_name, value, e);
            }
        }
    };
}

macro_rules! simple_property_field_handler {
    ($fn_name:ident, $field:ident, $subfield:ident, $prop_name:expr) => {
        pub fn $fn_name(ctx: &mut PropertyUpdateContext, value: &str) {
            if let Err(e) = Property::update_property_field(
                &mut ctx.computed_style.$field,
                |f| &mut f.$subfield,
                value,
            ) {
                ctx.record_error($prop_name, value, e);
            }
        }
    };
}

pub fn resolve_css_variable(variables: &Vec<(String, String)>, value: String) -> String {
    if let Some(stripped) = value.strip_prefix("var(").and_then(|s| s.strip_suffix(')')) {
        let var_name = stripped.trim();
        for (name, val) in variables {
            if name == var_name {
                return val.clone();
            }
        }
    }
    value
}

simple_property_handler!(
    handle_background_color,
    background_color,
    "background-color"
);
simple_property_handler!(handle_border_color, border_color, "border-color");
simple_property_handler!(handle_border_width, border_width, "border-width");
simple_property_handler!(handle_border_style, border_style, "border-style");
simple_property_handler!(handle_color, color, "color");
simple_property_handler!(handle_display, display, "display");
simple_property_handler!(handle_font_family, font_family, "font-family");
simple_property_handler!(handle_font_weight, font_weight, "font-weight");
simple_property_handler!(handle_height, height, "height");
simple_property_handler!(handle_max_height, max_height, "max-height");
simple_property_handler!(handle_line_height, line_height, "line-height");
simple_property_handler!(handle_margin, margin, "margin");
simple_property_field_handler!(handle_margin_top, margin, top, "margin-top");
simple_property_field_handler!(handle_margin_bottom, margin, bottom, "margin-bottom");
simple_property_field_handler!(handle_margin_left, margin, left, "margin-left");
simple_property_field_handler!(handle_margin_right, margin, right, "margin-right");
simple_property_handler!(handle_padding, padding, "padding");
simple_property_field_handler!(handle_padding_top, padding, top, "padding-top");
simple_property_field_handler!(handle_padding_bottom, padding, bottom, "padding-bottom");
simple_property_field_handler!(handle_padding_left, padding, left, "padding-left");
simple_property_field_handler!(handle_padding_right, padding, right, "padding-right");
simple_property_handler!(handle_position, position, "position");
simple_property_handler!(handle_text_align, text_align, "text-align");
simple_property_handler!(handle_whitespace, whitespace, "white-space");
simple_property_handler!(handle_width, width, "width");
simple_property_handler!(handle_max_width, max_width, "max-width");
simple_property_handler!(handle_writing_mode, writing_mode, "writing-mode");

pub fn handle_border(ctx: &mut PropertyUpdateContext, value: &str) {
    let parts = value.split_whitespace().collect::<Vec<&str>>();

    for part in parts {
        if let Ok(length) = part.parse::<Length>() {
            Property::update(
                &mut ctx.computed_style.border_width,
                Property::wrap_value(BorderWidth::all(BorderWidthValue::Length(length))),
            )
        }

        if let Ok(style) = part.parse::<BorderStyleValue>() {
            Property::update(
                &mut ctx.computed_style.border_style,
                Property::wrap_value(BorderStyle::all(style)),
            )
        }

        if let Ok(color) = part.parse::<Color>() {
            Property::update(
                &mut ctx.computed_style.border_color,
                Property::wrap_value(BorderColor::all(color)),
            )
        }
    }
}

pub fn handle_font_size(ctx: &mut PropertyUpdateContext, value: &str) {
    if let Ok(font_size) = Property::resolve(&ctx.computed_style.font_size) {
        let parent_px = ctx
            .parent_style
            .map(|p| p.computed_font_size_px)
            .unwrap_or(AbsoluteSize::Medium.to_px());
        ctx.computed_style.computed_font_size_px = font_size.to_px(parent_px);
    }

    Property::update_property(&mut ctx.computed_style.font_size, value).unwrap_or(())
}

pub fn handle_margin_block(ctx: &mut PropertyUpdateContext, value: &str) {
    match ctx.computed_style.writing_mode {
        Property::Global(global) => {
            ctx.record_error(
                "margin-block",
                value,
                format!("Unsupported global value: {:?}", global),
            );
        }
        Property::Value(val) => match val {
            WritingMode::HorizontalTb => {
                let res_top = Property::update_property_field(
                    &mut ctx.computed_style.margin,
                    |m| &mut m.top,
                    value,
                );
                let res_bottom = Property::update_property_field(
                    &mut ctx.computed_style.margin,
                    |m| &mut m.bottom,
                    value,
                );
                if let Err(e) = res_top {
                    ctx.record_error("margin-block", value, e);
                }
                if let Err(e) = res_bottom {
                    ctx.record_error("margin-block", value, e);
                }
            }
            WritingMode::VerticalRl | WritingMode::VerticalLr => {
                let res_left = Property::update_property_field(
                    &mut ctx.computed_style.margin,
                    |m| &mut m.left,
                    value,
                );
                let res_right = Property::update_property_field(
                    &mut ctx.computed_style.margin,
                    |m| &mut m.right,
                    value,
                );
                if let Err(e) = res_left {
                    ctx.record_error("margin-block", value, e);
                }
                if let Err(e) = res_right {
                    ctx.record_error("margin-block", value, e);
                }
            }
            _ => {
                ctx.record_error(
                    "margin-block",
                    value,
                    String::from("Unsupported writing mode"),
                );
            }
        },
    }
}

pub fn handle_margin_block_start(ctx: &mut PropertyUpdateContext, value: &str) {
    match ctx.computed_style.writing_mode {
        Property::Global(global) => {
            ctx.record_error(
                "margin-block-start",
                value,
                format!("Unsupported global value: {:?}", global),
            );
        }
        Property::Value(val) => match val {
            WritingMode::HorizontalTb => {
                if let Err(e) = Property::update_property_field(
                    &mut ctx.computed_style.margin,
                    |m| &mut m.top,
                    value,
                ) {
                    ctx.record_error("margin-block-start", value, e);
                }
            }
            WritingMode::VerticalRl => {
                if let Err(e) = Property::update_property_field(
                    &mut ctx.computed_style.margin,
                    |m| &mut m.right,
                    value,
                ) {
                    ctx.record_error("margin-block-start", value, e);
                }
            }
            WritingMode::VerticalLr => {
                if let Err(e) = Property::update_property_field(
                    &mut ctx.computed_style.margin,
                    |m| &mut m.left,
                    value,
                ) {
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
        Property::Global(global) => {
            ctx.record_error(
                "margin-block-end",
                value,
                format!("Unsupported global value: {:?}", global),
            );
        }
        Property::Value(val) => match val {
            WritingMode::HorizontalTb => {
                if let Err(e) = Property::update_property_field(
                    &mut ctx.computed_style.margin,
                    |m| &mut m.bottom,
                    value,
                ) {
                    ctx.record_error("margin-block-end", value, e);
                }
            }
            WritingMode::VerticalRl => {
                if let Err(e) = Property::update_property_field(
                    &mut ctx.computed_style.margin,
                    |m| &mut m.left,
                    value,
                ) {
                    ctx.record_error("margin-block-end", value, e);
                }
            }
            WritingMode::VerticalLr => {
                if let Err(e) = Property::update_property_field(
                    &mut ctx.computed_style.margin,
                    |m| &mut m.right,
                    value,
                ) {
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
    match ctx.computed_style.writing_mode {
        Property::Global(global) => {
            ctx.record_error(
                "padding-block",
                value,
                format!("Unsupported global value: {:?}", global),
            );
        }
        Property::Value(val) => match val {
            WritingMode::HorizontalTb => {
                let res_top = Property::update_property_field(
                    &mut ctx.computed_style.padding,
                    |m| &mut m.top,
                    value,
                );
                let res_bottom = Property::update_property_field(
                    &mut ctx.computed_style.padding,
                    |m| &mut m.bottom,
                    value,
                );
                if let Err(e) = res_top {
                    ctx.record_error("padding-block", value, e);
                }
                if let Err(e) = res_bottom {
                    ctx.record_error("padding-block", value, e);
                }
            }
            WritingMode::VerticalRl | WritingMode::VerticalLr => {
                let res_left = Property::update_property_field(
                    &mut ctx.computed_style.padding,
                    |m| &mut m.left,
                    value,
                );
                let res_right = Property::update_property_field(
                    &mut ctx.computed_style.padding,
                    |m| &mut m.right,
                    value,
                );
                if let Err(e) = res_left {
                    ctx.record_error("padding-block", value, e);
                }
                if let Err(e) = res_right {
                    ctx.record_error("padding-block", value, e);
                }
            }
            _ => {
                ctx.record_error(
                    "padding-block",
                    value,
                    String::from("Unsupported writing mode"),
                );
            }
        },
    }
}

pub fn handle_padding_block_start(ctx: &mut PropertyUpdateContext, value: &str) {
    match ctx.computed_style.writing_mode {
        Property::Global(global) => {
            ctx.record_error(
                "padding-block-start",
                value,
                format!("Unsupported global value: {:?}", global),
            );
        }
        Property::Value(val) => match val {
            WritingMode::HorizontalTb => {
                if let Err(e) = Property::update_property_field(
                    &mut ctx.computed_style.padding,
                    |m| &mut m.top,
                    value,
                ) {
                    ctx.record_error("padding-block-start", value, e);
                }
            }
            WritingMode::VerticalRl => {
                if let Err(e) = Property::update_property_field(
                    &mut ctx.computed_style.padding,
                    |m| &mut m.right,
                    value,
                ) {
                    ctx.record_error("padding-block-start", value, e);
                }
            }
            WritingMode::VerticalLr => {
                if let Err(e) = Property::update_property_field(
                    &mut ctx.computed_style.padding,
                    |m| &mut m.left,
                    value,
                ) {
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
        Property::Global(global) => {
            ctx.record_error(
                "padding-block-end",
                value,
                format!("Unsupported global value: {:?}", global),
            );
        }
        Property::Value(val) => match val {
            WritingMode::HorizontalTb => {
                if let Err(e) = Property::update_property_field(
                    &mut ctx.computed_style.padding,
                    |m| &mut m.bottom,
                    value,
                ) {
                    ctx.record_error("padding-block-end", value, e);
                }
            }
            WritingMode::VerticalRl => {
                if let Err(e) = Property::update_property_field(
                    &mut ctx.computed_style.padding,
                    |m| &mut m.left,
                    value,
                ) {
                    ctx.record_error("padding-block-end", value, e);
                }
            }
            WritingMode::VerticalLr => {
                if let Err(e) = Property::update_property_field(
                    &mut ctx.computed_style.padding,
                    |m| &mut m.right,
                    value,
                ) {
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

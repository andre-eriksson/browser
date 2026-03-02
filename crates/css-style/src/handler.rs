use css_cssom::{ComponentValue, CssTokenKind};

use crate::background::{Attachment, BgClip, VisualBox};
use crate::global::Global;
use crate::image::Image;
use crate::length::{Length, LengthUnit};
use crate::position::{
    BackgroundPosition, BlockAxis, HorizontalOrXSide, HorizontalSide, InlineAxis, PositionFour, PositionOne,
    PositionThree, PositionTwo, PositionX, PositionY, RelativeAxis, RelativeHorizontalSide, RelativeVerticalSide, Side,
    VerticalOrYSide, VerticalSide, XAxis, XAxisOrLengthPercentage, XSide, YAxis, YAxisOrLengthPercentage, YSide,
};
use crate::properties::background::{
    BackgroundAttachment, BackgroundClip, BackgroundOrigin, BackgroundRepeat, BackgroundSize, RepeatStyle,
};
use crate::properties::text::WritingMode;
use crate::properties::{AbsoluteContext, CSSProperty};
use crate::specified::SpecifiedStyle;
use crate::{BorderStyle, BorderWidth, Color, FontWeight, Offset, RelativeContext};

/// Context for updating a CSS property, containing necessary information and utilities for the update process.
pub(crate) struct PropertyUpdateContext<'a> {
    pub absolute_ctx: &'a AbsoluteContext<'a>,
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
macro_rules! logical_pair_handler {
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

simple_property_handler!(handle_background_attachment, background_attachment, "background-attachment");
simple_property_handler!(handle_background_blend_mode, background_blend_mode, "background-blend-mode");
simple_property_handler!(handle_background_clip, background_clip, "background-clip");
simple_property_handler!(handle_background_color, background_color, "background-color");
simple_property_handler!(handle_background_origin, background_origin, "background-origin");
simple_property_handler!(handle_background_position_x, background_position_x, "background-position-x");
simple_property_handler!(handle_background_position_y, background_position_y, "background-position-y");
simple_property_handler!(handle_background_repeat, background_repeat, "background-repeat");
simple_property_handler!(handle_background_size, background_size, "background-size");
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

pub(crate) fn handle_background_position(ctx: &mut PropertyUpdateContext, value: &[ComponentValue]) {
    if let Ok(global) = Global::try_from(value) {
        ctx.specified_style.background_position_x = CSSProperty::Global(global);
        ctx.specified_style.background_position_y = CSSProperty::Global(global);
        return;
    }

    let writing_mode = ctx.resolve_writing_mode();

    fn resolve_horizontal_side(horizontal: HorizontalSide, writing_mode: WritingMode) -> HorizontalOrXSide {
        match writing_mode {
            WritingMode::HorizontalTb | WritingMode::SidewaysLr | WritingMode::SidewaysRl => {
                HorizontalOrXSide::Horizontal(horizontal)
            }
            WritingMode::VerticalRl | WritingMode::VerticalLr => match horizontal {
                HorizontalSide::Left => HorizontalOrXSide::XSide(XSide::XStart),
                HorizontalSide::Right => HorizontalOrXSide::XSide(XSide::XEnd),
            },
        }
    }

    fn resolve_vertical_side(vertical: VerticalSide, writing_mode: WritingMode) -> VerticalOrYSide {
        match writing_mode {
            WritingMode::HorizontalTb => match vertical {
                VerticalSide::Top => VerticalOrYSide::YSide(YSide::YStart),
                VerticalSide::Bottom => VerticalOrYSide::YSide(YSide::YEnd),
            },
            WritingMode::VerticalRl | WritingMode::VerticalLr => VerticalOrYSide::Vertical(vertical),
            WritingMode::SidewaysLr | WritingMode::SidewaysRl => match vertical {
                VerticalSide::Top => VerticalOrYSide::YSide(YSide::YStart),
                VerticalSide::Bottom => VerticalOrYSide::YSide(YSide::YEnd),
            },
        }
    }

    fn resolve_horizontal_x_side(side: Side, writing_mode: WritingMode) -> HorizontalOrXSide {
        match writing_mode {
            WritingMode::HorizontalTb | WritingMode::SidewaysLr | WritingMode::SidewaysRl => match side {
                Side::Start => HorizontalOrXSide::Horizontal(HorizontalSide::Left),
                Side::End => HorizontalOrXSide::Horizontal(HorizontalSide::Right),
            },
            WritingMode::VerticalRl | WritingMode::VerticalLr => match side {
                Side::Start => HorizontalOrXSide::XSide(XSide::XStart),
                Side::End => HorizontalOrXSide::XSide(XSide::XEnd),
            },
        }
    }

    fn resolve_vertical_y_side(side: Side, writing_mode: WritingMode) -> VerticalOrYSide {
        match writing_mode {
            WritingMode::HorizontalTb => match side {
                Side::Start => VerticalOrYSide::YSide(YSide::YStart),
                Side::End => VerticalOrYSide::YSide(YSide::YEnd),
            },
            WritingMode::VerticalRl | WritingMode::VerticalLr => match side {
                Side::Start => VerticalOrYSide::Vertical(VerticalSide::Top),
                Side::End => VerticalOrYSide::Vertical(VerticalSide::Bottom),
            },
            WritingMode::SidewaysLr | WritingMode::SidewaysRl => match side {
                Side::Start => VerticalOrYSide::YSide(YSide::YStart),
                Side::End => VerticalOrYSide::YSide(YSide::YEnd),
            },
        }
    }

    fn resolve_inline_axis(inline: InlineAxis, writing_mode: WritingMode) -> HorizontalOrXSide {
        match writing_mode {
            WritingMode::HorizontalTb => match inline {
                InlineAxis::InlineStart => HorizontalOrXSide::XSide(XSide::XStart),
                InlineAxis::InlineEnd => HorizontalOrXSide::XSide(XSide::XEnd),
            },
            WritingMode::SidewaysLr => match inline {
                InlineAxis::InlineStart => HorizontalOrXSide::Horizontal(HorizontalSide::Left),
                InlineAxis::InlineEnd => HorizontalOrXSide::Horizontal(HorizontalSide::Right),
            },
            WritingMode::SidewaysRl => match inline {
                InlineAxis::InlineStart => HorizontalOrXSide::Horizontal(HorizontalSide::Right),
                InlineAxis::InlineEnd => HorizontalOrXSide::Horizontal(HorizontalSide::Left),
            },
            WritingMode::VerticalRl | WritingMode::VerticalLr => match inline {
                InlineAxis::InlineStart => HorizontalOrXSide::XSide(XSide::XStart),
                InlineAxis::InlineEnd => HorizontalOrXSide::XSide(XSide::XEnd),
            },
        }
    }

    fn resolve_block_axis(block: BlockAxis, writing_mode: WritingMode) -> VerticalOrYSide {
        match writing_mode {
            WritingMode::HorizontalTb => match block {
                BlockAxis::BlockStart => VerticalOrYSide::YSide(YSide::YStart),
                BlockAxis::BlockEnd => VerticalOrYSide::YSide(YSide::YEnd),
            },
            WritingMode::VerticalRl => match block {
                BlockAxis::BlockStart => VerticalOrYSide::Vertical(VerticalSide::Top),
                BlockAxis::BlockEnd => VerticalOrYSide::Vertical(VerticalSide::Bottom),
            },
            WritingMode::VerticalLr => match block {
                BlockAxis::BlockStart => VerticalOrYSide::Vertical(VerticalSide::Bottom),
                BlockAxis::BlockEnd => VerticalOrYSide::Vertical(VerticalSide::Top),
            },
            WritingMode::SidewaysLr | WritingMode::SidewaysRl => match block {
                BlockAxis::BlockStart => VerticalOrYSide::YSide(YSide::YStart),
                BlockAxis::BlockEnd => VerticalOrYSide::YSide(YSide::YEnd),
            },
        }
    }

    fn resolve_x_side(x_side: XSide) -> PositionX {
        match x_side {
            XSide::XStart => PositionX::Relative((Some(HorizontalOrXSide::XSide(XSide::XStart)), None)),
            XSide::XEnd => PositionX::Relative((Some(HorizontalOrXSide::XSide(XSide::XEnd)), None)),
        }
    }

    fn resolve_y_side(y_side: YSide) -> PositionY {
        match y_side {
            YSide::YStart => PositionY::Relative((Some(VerticalOrYSide::YSide(YSide::YStart)), None)),
            YSide::YEnd => PositionY::Relative((Some(VerticalOrYSide::YSide(YSide::YEnd)), None)),
        }
    }

    fn resolve_x_axis(x_axis: XAxis) -> PositionX {
        match x_axis {
            XAxis::Center(center) => PositionX::Center(center, None),
            XAxis::Horizontal(horizontal) => {
                PositionX::Relative((Some(HorizontalOrXSide::Horizontal(horizontal)), None))
            }
            XAxis::XSide(xside) => resolve_x_side(xside),
        }
    }

    fn resolve_y_axis(y_axis: YAxis) -> PositionY {
        match y_axis {
            YAxis::Center(center) => PositionY::Center(center, None),
            YAxis::Vertical(vertical) => PositionY::Relative((Some(VerticalOrYSide::Vertical(vertical)), None)),
            YAxis::YSide(yside) => resolve_y_side(yside),
        }
    }

    let mut x_pos = Vec::new();
    let mut y_pos = Vec::new();

    let values = value
        .split(|cv| matches!(cv, ComponentValue::Token(t) if matches!(t.kind, CssTokenKind::Comma)))
        .collect::<Vec<_>>();

    for cv in values {
        if let Ok(bg_position) = BackgroundPosition::try_from(cv) {
            match bg_position {
                BackgroundPosition::One(one) => match one {
                    PositionOne::LengthPercentage(lp) => {
                        x_pos.push(PositionX::Relative((None, Some(lp))));
                        y_pos.push(PositionY::Relative((None, Some(lp))));
                    }
                    PositionOne::Horizontal(horizontal) => match horizontal {
                        HorizontalOrXSide::XSide(xside) => x_pos.push(resolve_x_side(xside)),
                        HorizontalOrXSide::Horizontal(h) => {
                            x_pos.push(PositionX::Relative((Some(HorizontalOrXSide::Horizontal(h)), None)));
                        }
                    },
                    PositionOne::Vertical(vertical) => match vertical {
                        VerticalOrYSide::YSide(yside) => y_pos.push(resolve_y_side(yside)),
                        VerticalOrYSide::Vertical(v) => {
                            y_pos.push(PositionY::Relative((Some(VerticalOrYSide::Vertical(v)), None)));
                        }
                    },
                    PositionOne::Center(center) => {
                        x_pos.push(PositionX::Center(center, None));
                        y_pos.push(PositionY::Center(center, None));
                    }
                    PositionOne::BlockAxis(block) => {
                        let resolved = resolve_block_axis(block, writing_mode);
                        y_pos.push(PositionY::Relative((Some(resolved), None)));
                    }
                    PositionOne::InlineAxis(inline) => {
                        let resolved = resolve_inline_axis(inline, writing_mode);
                        x_pos.push(PositionX::Relative((Some(resolved), None)));
                    }
                },
                BackgroundPosition::Two(two) => match two {
                    PositionTwo::Axis(x, y) => {
                        x_pos.push(resolve_x_axis(x));
                        y_pos.push(resolve_y_axis(y));
                    }
                    PositionTwo::Relative(x_rel, y_rel) => {
                        match x_rel {
                            RelativeAxis::Center(center) => x_pos.push(PositionX::Center(center, None)),
                            RelativeAxis::Side(side) => x_pos
                                .push(PositionX::Relative((Some(resolve_horizontal_x_side(side, writing_mode)), None))),
                        }

                        match y_rel {
                            RelativeAxis::Center(center) => y_pos.push(PositionY::Center(center, None)),
                            RelativeAxis::Side(side) => y_pos
                                .push(PositionY::Relative((Some(resolve_vertical_y_side(side, writing_mode)), None))),
                        }
                    }
                    PositionTwo::AxisOrPercentage(x_pct, y_pct) => {
                        match x_pct {
                            XAxisOrLengthPercentage::XAxis(x_axis) => x_pos.push(resolve_x_axis(x_axis)),
                            XAxisOrLengthPercentage::LengthPercentage(lp) => {
                                x_pos.push(PositionX::Relative((None, Some(lp))));
                            }
                        }

                        match y_pct {
                            YAxisOrLengthPercentage::YAxis(y_axis) => y_pos.push(resolve_y_axis(y_axis)),
                            YAxisOrLengthPercentage::LengthPercentage(lp) => {
                                y_pos.push(PositionY::Relative((None, Some(lp))));
                            }
                        }
                    }
                    PositionTwo::BlockInline(block, inline) => {
                        let resolved_block = resolve_block_axis(block, writing_mode);
                        let resolved_inline = resolve_inline_axis(inline, writing_mode);
                        y_pos.push(PositionY::Relative((Some(resolved_block), None)));
                        x_pos.push(PositionX::Relative((Some(resolved_inline), None)));
                    }
                },

                BackgroundPosition::Three(three) => {
                    match three {
                        PositionThree::RelativeHorizontal((horizontal, len_pct), rel_vertical_side) => {
                            let resolved_horizontal = resolve_horizontal_side(horizontal, writing_mode);
                            x_pos.push(PositionX::Relative((Some(resolved_horizontal), Some(len_pct))));

                            match rel_vertical_side {
                                RelativeVerticalSide::Center(center) => y_pos.push(PositionY::Center(center, None)),
                                RelativeVerticalSide::Vertical(vertical_side) => y_pos
                                    .push(PositionY::Relative((Some(VerticalOrYSide::Vertical(vertical_side)), None))),
                            }
                        }
                        PositionThree::RelativeVertical(rel_horionztal_side, (vertical, len_pct)) => {
                            let resolved_vertical = resolve_vertical_side(vertical, writing_mode);
                            y_pos.push(PositionY::Relative((Some(resolved_vertical), Some(len_pct))));

                            match rel_horionztal_side {
                                RelativeHorizontalSide::Center(center) => x_pos.push(PositionX::Center(center, None)),
                                RelativeHorizontalSide::Horizontal(horizontal_side) => x_pos.push(PositionX::Relative(
                                    (Some(HorizontalOrXSide::Horizontal(horizontal_side)), None),
                                )),
                            }
                        }
                    }
                }

                BackgroundPosition::Four(four) => match four {
                    PositionFour::BlockInline((block, x_len_pct), (inline, y_len_pct)) => {
                        let resolved_block = resolve_block_axis(block, writing_mode);
                        let resolved_inline = resolve_inline_axis(inline, writing_mode);
                        y_pos.push(PositionY::Relative((Some(resolved_block), Some(y_len_pct))));
                        x_pos.push(PositionX::Relative((Some(resolved_inline), Some(x_len_pct))));
                    }
                    PositionFour::StartEnd((x_side, x_len_pct), (y_side, y_len_pct)) => {
                        let resolved_x_side = resolve_horizontal_x_side(x_side, writing_mode);
                        let resolved_y_side = resolve_vertical_y_side(y_side, writing_mode);
                        x_pos.push(PositionX::Relative((Some(resolved_x_side), Some(x_len_pct))));
                        y_pos.push(PositionY::Relative((Some(resolved_y_side), Some(y_len_pct))));
                    }
                    PositionFour::XYPercentage((horizontal_side, x_len_pct), (vertical_side, y_len_pct)) => {
                        match horizontal_side {
                            HorizontalOrXSide::Horizontal(h) => {
                                x_pos.push(PositionX::Relative((
                                    Some(HorizontalOrXSide::Horizontal(h)),
                                    Some(x_len_pct),
                                )));
                            }
                            HorizontalOrXSide::XSide(xside) => {
                                x_pos.push(PositionX::Relative((
                                    Some(HorizontalOrXSide::XSide(xside)),
                                    Some(x_len_pct),
                                )));
                            }
                        }

                        match vertical_side {
                            VerticalOrYSide::Vertical(v) => {
                                y_pos.push(PositionY::Relative((Some(VerticalOrYSide::Vertical(v)), Some(y_len_pct))));
                            }
                            VerticalOrYSide::YSide(yside) => {
                                y_pos.push(PositionY::Relative((Some(VerticalOrYSide::YSide(yside)), Some(y_len_pct))));
                            }
                        }
                    }
                },
            }
        } else {
            ctx.record_error(
                "background-position",
                value.to_vec(),
                "Invalid value for background-position".to_string(),
            );
        }
    }
}

/// Handles the `background` shorthand property.
///
/// CSS grammar (simplified, single layer):
/// <final-bg-layer> =
///   <'background-color'> || <bg-image> || <bg-position> [ / <bg-size> ]? ||
///   <repeat-style> || <attachment> || <visual-box>{1,2}
pub(crate) fn handle_background(ctx: &mut PropertyUpdateContext, value: &[ComponentValue]) {
    if let Ok(global) = Global::try_from(value) {
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

    if value.iter().any(|cv| {
        matches!(cv, ComponentValue::Token(t) if matches!(&t.kind, CssTokenKind::Ident(s) if s.eq_ignore_ascii_case("none")))
    }) && value
        .iter()
        .filter(|cv| !matches!(cv, ComponentValue::Token(t) if matches!(t.kind, CssTokenKind::Whitespace)))
        .count()
        == 1
    {
        ctx.specified_style.background_image = CSSProperty::Global(Global::Initial);
        ctx.specified_style.background_color = CSSProperty::Value(Color::Transparent);
        return;
    }

    let mut color: Option<Color> = None;
    let mut image: Option<Image> = None;
    let mut image_none = false;
    let mut attachment: Option<Attachment> = None;
    let mut repeat_h: Option<RepeatStyle> = None;
    let mut repeat_v: Option<RepeatStyle> = None;
    let mut boxes: Vec<VisualBox> = Vec::new();
    let mut position_cvs: Vec<ComponentValue> = Vec::new();
    let mut size: Option<BackgroundSize> = None;

    let mut collecting_position = false;
    let mut collecting_size = false;
    let mut size_cvs: Vec<ComponentValue> = Vec::new();

    fn is_position_keyword(ident: &str) -> bool {
        ident.eq_ignore_ascii_case("left")
            || ident.eq_ignore_ascii_case("right")
            || ident.eq_ignore_ascii_case("top")
            || ident.eq_ignore_ascii_case("bottom")
            || ident.eq_ignore_ascii_case("center")
    }

    fn is_size_token(cv: &ComponentValue) -> bool {
        match cv {
            ComponentValue::Token(token) => match &token.kind {
                CssTokenKind::Ident(ident) => {
                    ident.eq_ignore_ascii_case("auto")
                        || ident.eq_ignore_ascii_case("cover")
                        || ident.eq_ignore_ascii_case("contain")
                }
                CssTokenKind::Dimension { .. } => true,
                CssTokenKind::Percentage(_) => true,
                CssTokenKind::Number(n) if n.to_f64() == 0.0 => true,
                _ => false,
            },
            _ => false,
        }
    }

    fn finalize_size(size_cvs: &mut Vec<ComponentValue>, size: &mut Option<BackgroundSize>) {
        if !size_cvs.is_empty() {
            if let Ok(bg_size) = BackgroundSize::try_from(size_cvs.as_slice()) {
                *size = Some(bg_size);
            }
            size_cvs.clear();
        }
    }

    let mut i = 0;
    while i < value.len() {
        let cv = &value[i];

        if collecting_size {
            if matches!(cv, ComponentValue::Token(token) if matches!(token.kind, CssTokenKind::Whitespace)) {
                i += 1;
                continue;
            }

            if is_size_token(cv) {
                size_cvs.push(cv.clone());
                i += 1;
                continue;
            }

            finalize_size(&mut size_cvs, &mut size);
            collecting_size = false;
            continue;
        }

        if collecting_position {
            match cv {
                ComponentValue::Token(token) => match &token.kind {
                    CssTokenKind::Whitespace => {
                        position_cvs.push(cv.clone());
                        i += 1;
                        continue;
                    }
                    CssTokenKind::Delim('/') => {
                        collecting_position = false;
                        collecting_size = true;
                        size_cvs.clear();
                        i += 1;
                        continue;
                    }
                    CssTokenKind::Ident(ident) if is_position_keyword(ident) => {
                        position_cvs.push(cv.clone());
                        i += 1;
                        continue;
                    }
                    CssTokenKind::Dimension { .. } | CssTokenKind::Percentage(_) => {
                        position_cvs.push(cv.clone());
                        i += 1;
                        continue;
                    }
                    CssTokenKind::Number(n) if n.to_f64() == 0.0 => {
                        position_cvs.push(cv.clone());
                        i += 1;
                        continue;
                    }
                    _ => {
                        collecting_position = false;
                        continue;
                    }
                },
                _ => {
                    collecting_position = false;
                    continue;
                }
            }
        }

        match cv {
            ComponentValue::Function(func) => {
                if image.is_none()
                    && let Ok(img) = Image::try_from(func)
                {
                    image = Some(img);
                    image_none = false;
                    i += 1;
                    continue;
                }
                if color.is_none()
                    && let Ok(c) = Color::try_from(&value[i..=i])
                {
                    color = Some(c);
                    i += 1;
                    continue;
                }
            }
            ComponentValue::Token(token) => match &token.kind {
                CssTokenKind::Whitespace => {
                    i += 1;
                    continue;
                }
                CssTokenKind::Ident(ident) => {
                    if image.is_none() && !image_none && ident.eq_ignore_ascii_case("none") {
                        image_none = true;
                        i += 1;
                        continue;
                    }

                    if attachment.is_none() {
                        if ident.eq_ignore_ascii_case("scroll") {
                            attachment = Some(Attachment::Scroll);
                            i += 1;
                            continue;
                        } else if ident.eq_ignore_ascii_case("fixed") {
                            attachment = Some(Attachment::Fixed);
                            i += 1;
                            continue;
                        } else if ident.eq_ignore_ascii_case("local") {
                            attachment = Some(Attachment::Local);
                            i += 1;
                            continue;
                        }
                    }

                    if repeat_h.is_none() {
                        if ident.eq_ignore_ascii_case("repeat-x") {
                            repeat_h = Some(RepeatStyle::Repeat);
                            repeat_v = Some(RepeatStyle::NoRepeat);
                            i += 1;
                            continue;
                        } else if ident.eq_ignore_ascii_case("repeat-y") {
                            repeat_h = Some(RepeatStyle::NoRepeat);
                            repeat_v = Some(RepeatStyle::Repeat);
                            i += 1;
                            continue;
                        } else if ident.eq_ignore_ascii_case("repeat") {
                            repeat_h = Some(RepeatStyle::Repeat);
                            i += 1;
                            continue;
                        } else if ident.eq_ignore_ascii_case("space") {
                            repeat_h = Some(RepeatStyle::Space);
                            i += 1;
                            continue;
                        } else if ident.eq_ignore_ascii_case("round") {
                            repeat_h = Some(RepeatStyle::Round);
                            i += 1;
                            continue;
                        } else if ident.eq_ignore_ascii_case("no-repeat") {
                            repeat_h = Some(RepeatStyle::NoRepeat);
                            i += 1;
                            continue;
                        }
                    } else if repeat_v.is_none()
                        && let Ok(rs) = ident.parse::<RepeatStyle>()
                    {
                        repeat_v = Some(rs);
                        i += 1;
                        continue;
                    }

                    if ident.eq_ignore_ascii_case("content-box") {
                        boxes.push(VisualBox::Content);
                        i += 1;
                        continue;
                    } else if ident.eq_ignore_ascii_case("padding-box") {
                        boxes.push(VisualBox::Padding);
                        i += 1;
                        continue;
                    } else if ident.eq_ignore_ascii_case("border-box") {
                        boxes.push(VisualBox::Border);
                        i += 1;
                        continue;
                    }

                    if is_position_keyword(ident) && position_cvs.is_empty() {
                        collecting_position = true;
                        position_cvs.push(cv.clone());
                        i += 1;
                        continue;
                    }

                    if color.is_none()
                        && let Ok(c) = Color::try_from(&value[i..=i])
                    {
                        color = Some(c);
                        i += 1;
                        continue;
                    }
                }
                CssTokenKind::Hash { .. } => {
                    if color.is_none()
                        && let Ok(c) = Color::try_from(&value[i..=i])
                    {
                        color = Some(c);
                        i += 1;
                        continue;
                    }
                }
                CssTokenKind::Dimension { .. } | CssTokenKind::Percentage(_) => {
                    if position_cvs.is_empty() {
                        collecting_position = true;
                        position_cvs.push(cv.clone());
                        i += 1;
                        continue;
                    }
                }
                CssTokenKind::Number(n) if n.to_f64() == 0.0 => {
                    if position_cvs.is_empty() {
                        collecting_position = true;
                        position_cvs.push(cv.clone());
                        i += 1;
                        continue;
                    }
                }
                _ => {}
            },
            _ => {}
        }

        i += 1;
    }

    if collecting_size {
        finalize_size(&mut size_cvs, &mut size);
    }

    ctx.specified_style.background_color = CSSProperty::Value(color.unwrap_or(Color::Transparent));

    if image.is_some() {
        let _ = CSSProperty::update_property(&mut ctx.specified_style.background_image, value);
    } else if image_none {
        ctx.specified_style.background_image =
            CSSProperty::Value(crate::properties::background::BackgroundImage::default());
    }

    if let Some(att) = attachment {
        ctx.specified_style.background_attachment = CSSProperty::Value(BackgroundAttachment(vec![att]));
    }

    if let Some(rh) = repeat_h {
        let rv = repeat_v.unwrap_or(rh);
        ctx.specified_style.background_repeat = CSSProperty::Value(BackgroundRepeat(vec![(rh, rv)]));
    }

    match boxes.len() {
        1 => {
            ctx.specified_style.background_origin = CSSProperty::Value(BackgroundOrigin(vec![boxes[0]]));
            ctx.specified_style.background_clip = CSSProperty::Value(BackgroundClip(vec![BgClip::Visual(boxes[0])]));
        }
        n if n >= 2 => {
            ctx.specified_style.background_origin = CSSProperty::Value(BackgroundOrigin(vec![boxes[0]]));
            ctx.specified_style.background_clip = CSSProperty::Value(BackgroundClip(vec![BgClip::Visual(boxes[1])]));
        }
        _ => {}
    }

    if !position_cvs.is_empty() {
        handle_background_position(ctx, &position_cvs);
    }

    if let Some(sz) = size {
        ctx.specified_style.background_size = CSSProperty::Value(sz);
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
            ctx.specified_style.border_top_color = CSSProperty::Value(c.clone());
            ctx.specified_style.border_right_color = CSSProperty::Value(c.clone());
            ctx.specified_style.border_bottom_color = CSSProperty::Value(c.clone());
            ctx.specified_style.border_left_color = CSSProperty::Value(c.clone());
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

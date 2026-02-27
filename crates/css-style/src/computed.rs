use std::{f32, sync::Arc};

use css_cssom::{ComponentValue, Property};
use html_dom::{DocumentRoot, NodeId};

use crate::{
    BorderStyle, BorderWidth, FontSize, FontWeight, OffsetValue, RelativeContext, RelativeType,
    cascade::{GeneratedRule, RuleIndex},
    color::named::NamedColor,
    computed::{
        color::Color4f,
        dimension::{ComputedDimension, ComputedMaxDimension},
    },
    length::Length,
    primitives::{
        display::{InsideDisplay, OutsideDisplay},
        font::GenericName,
    },
    properties::{
        AbsoluteContext, CSSProperty,
        color::Color,
        dimension::{Dimension, MaxDimension},
        display::Display,
        font::{FontFamily, FontFamilyName},
        position::Position,
        text::{LineHeight, TextAlign, Whitespace, WritingMode},
    },
    specified::SpecifiedStyle,
};

pub mod color;
pub mod dimension;

/// The ComputedStyle struct represents the computed style for a DOM node after applying all CSS rules, resolving inheritance,
/// and applying the cascade. It contains all the properties that affect the layout and rendering of the node,
/// with all values resolved to their final forms (e.g., colors as RGBA, lengths in pixels, etc.).
#[derive(Debug, Clone, PartialEq)]
pub struct ComputedStyle {
    pub background_color: Color4f,
    pub border_top_color: Color4f,
    pub border_right_color: Color4f,
    pub border_bottom_color: Color4f,
    pub border_left_color: Color4f,
    pub border_top_style: BorderStyle,
    pub border_right_style: BorderStyle,
    pub border_bottom_style: BorderStyle,
    pub border_left_style: BorderStyle,
    pub border_top_width: f32,
    pub border_right_width: f32,
    pub border_bottom_width: f32,
    pub border_left_width: f32,
    pub color: Color4f,
    pub display: Display,
    pub font_family: Arc<FontFamily>,
    pub font_size: f32,
    pub font_weight: u16,
    pub height: ComputedDimension,
    pub intrinsic_height: f32,
    pub max_height: ComputedMaxDimension,
    pub max_intrinsic_height: f32,
    pub line_height: f32,
    pub margin_top: f32,
    pub margin_right: f32,
    pub margin_bottom: f32,
    pub margin_left: f32,
    pub padding_top: f32,
    pub padding_right: f32,
    pub padding_bottom: f32,
    pub padding_left: f32,
    pub margin_top_auto: bool,
    pub margin_right_auto: bool,
    pub margin_bottom_auto: bool,
    pub margin_left_auto: bool,
    pub padding_top_auto: bool,
    pub padding_right_auto: bool,
    pub padding_bottom_auto: bool,
    pub padding_left_auto: bool,
    pub position: Position,
    pub text_align: TextAlign,
    pub whitespace: Whitespace,
    pub width: ComputedDimension,
    pub intrinsic_width: f32,
    pub max_width: ComputedMaxDimension,
    pub max_intrinsic_width: f32,
    pub writing_mode: WritingMode,
    pub variables: Arc<Vec<(Property, Vec<ComponentValue>)>>,
}

impl ComputedStyle {
    /// Computes the ComputedStyle for a given node in the DOM.
    pub fn from_node(
        absolute_ctx: &AbsoluteContext,
        relative_ctx: &mut RelativeContext,
        node_id: &NodeId,
        dom: &DocumentRoot,
        rules: &[GeneratedRule],
        rule_index: &RuleIndex,
        parent_style: Option<&ComputedStyle>,
    ) -> Self {
        let specified_style =
            SpecifiedStyle::from_node(absolute_ctx, relative_ctx, node_id, dom, rules, rule_index, parent_style);

        let margin_top = specified_style.margin_top.resolve_with_context_owned(
            OffsetValue::Length(Length::px(relative_ctx.parent.margin_top)),
            OffsetValue::zero(),
        );
        let margin_right = specified_style.margin_right.resolve_with_context_owned(
            OffsetValue::Length(Length::px(relative_ctx.parent.margin_right)),
            OffsetValue::zero(),
        );
        let margin_bottom = specified_style.margin_bottom.resolve_with_context_owned(
            OffsetValue::Length(Length::px(relative_ctx.parent.margin_bottom)),
            OffsetValue::zero(),
        );
        let margin_left = specified_style.margin_left.resolve_with_context_owned(
            OffsetValue::Length(Length::px(relative_ctx.parent.margin_left)),
            OffsetValue::zero(),
        );
        let padding_top = specified_style.padding_top.resolve_with_context_owned(
            OffsetValue::Length(Length::px(relative_ctx.parent.padding_top)),
            OffsetValue::zero(),
        );
        let padding_right = specified_style.padding_right.resolve_with_context_owned(
            OffsetValue::Length(Length::px(relative_ctx.parent.padding_right)),
            OffsetValue::zero(),
        );
        let padding_bottom = specified_style.padding_bottom.resolve_with_context_owned(
            OffsetValue::Length(Length::px(relative_ctx.parent.padding_bottom)),
            OffsetValue::zero(),
        );
        let padding_left = specified_style.padding_left.resolve_with_context_owned(
            OffsetValue::Length(Length::px(relative_ctx.parent.padding_left)),
            OffsetValue::zero(),
        );
        let max_height = specified_style.max_height.resolve_with_context_owned(
            MaxDimension::Length(Length::px(relative_ctx.parent.max_intrinsic_height)),
            MaxDimension::None,
        );
        let max_width = specified_style.max_width.resolve_with_context_owned(
            MaxDimension::Length(Length::px(relative_ctx.parent.max_intrinsic_width)),
            MaxDimension::None,
        );

        Self {
            background_color: Color4f::from_css_color_property(
                &specified_style.background_color,
                &CSSProperty::Value(Color::Transparent),
                &Color::Transparent,
                parent_style.map(|s| Color::from(s.background_color)),
                absolute_ctx,
            ),
            border_top_color: Color4f::from_css_color_property(
                &specified_style.border_top_color,
                &specified_style.color,
                &Color::Current,
                parent_style.map(|s| Color::from(s.border_top_color)),
                absolute_ctx,
            ),
            border_right_color: Color4f::from_css_color_property(
                &specified_style.border_right_color,
                &specified_style.color,
                &Color::Current,
                parent_style.map(|s| Color::from(s.border_right_color)),
                absolute_ctx,
            ),
            border_bottom_color: Color4f::from_css_color_property(
                &specified_style.border_bottom_color,
                &specified_style.color,
                &Color::Current,
                parent_style.map(|s| Color::from(s.border_bottom_color)),
                absolute_ctx,
            ),
            border_left_color: Color4f::from_css_color_property(
                &specified_style.border_left_color,
                &specified_style.color,
                &Color::Current,
                parent_style.map(|s| Color::from(s.border_left_color)),
                absolute_ctx,
            ),
            border_top_style: specified_style
                .border_top_style
                .resolve_with_context_owned(relative_ctx.parent.border_top_style, BorderStyle::None),
            border_right_style: specified_style
                .border_right_style
                .resolve_with_context_owned(relative_ctx.parent.border_right_style, BorderStyle::None),
            border_bottom_style: specified_style
                .border_bottom_style
                .resolve_with_context_owned(relative_ctx.parent.border_bottom_style, BorderStyle::None),
            border_left_style: specified_style
                .border_left_style
                .resolve_with_context_owned(relative_ctx.parent.border_left_style, BorderStyle::None),
            border_top_width: specified_style
                .border_top_width
                .resolve_with_context_owned(BorderWidth::px(relative_ctx.parent.border_top_width), BorderWidth::zero())
                .to_px(relative_ctx, absolute_ctx),
            border_right_width: specified_style
                .border_right_width
                .resolve_with_context_owned(
                    BorderWidth::px(relative_ctx.parent.border_right_width),
                    BorderWidth::zero(),
                )
                .to_px(relative_ctx, absolute_ctx),
            border_bottom_width: specified_style
                .border_bottom_width
                .resolve_with_context_owned(
                    BorderWidth::px(relative_ctx.parent.border_bottom_width),
                    BorderWidth::zero(),
                )
                .to_px(relative_ctx, absolute_ctx),
            border_left_width: specified_style
                .border_left_width
                .resolve_with_context_owned(BorderWidth::px(relative_ctx.parent.border_left_width), BorderWidth::zero())
                .to_px(relative_ctx, absolute_ctx),
            color: Color4f::from_css_color_property(
                &specified_style.color,
                &CSSProperty::Value(Color::Named(NamedColor::Black)),
                &Color::Named(NamedColor::Black),
                parent_style.map(|s| Color::from(s.color)),
                absolute_ctx,
            ),
            display: specified_style
                .display
                .resolve_with_context_owned(relative_ctx.parent.display, Display::default()),
            font_family: Arc::new(specified_style.font_family.resolve_with_context_owned(
                (*relative_ctx.parent.font_family).clone(),
                FontFamily::new(&[FontFamilyName::Generic(GenericName::Serif)]),
            )),
            font_size: specified_style
                .font_size
                .resolve_with_context_owned(FontSize::px(relative_ctx.parent.font_size), FontSize::px(16.0))
                .to_px(absolute_ctx, relative_ctx.parent.font_size),
            font_weight: specified_style.font_weight.resolve_with_context_owned(
                FontWeight::try_from(relative_ctx.parent.font_weight).unwrap_or(FontWeight::Normal),
                FontWeight::Normal,
            ) as u16,
            intrinsic_height: match &CSSProperty::resolve(&specified_style.height) {
                Ok(Dimension::Length(l)) => l.to_px(relative_ctx, absolute_ctx),
                Ok(Dimension::Percentage(p)) => relative_ctx.parent.intrinsic_height * p.as_fraction(),
                Ok(Dimension::Calc(calc)) => calc.to_px(Some(RelativeType::ParentHeight), relative_ctx, absolute_ctx),
                _ => 0.0,
            },
            height: ComputedDimension::from(
                specified_style
                    .height
                    .resolve_with_context_owned(relative_ctx.parent.height.into(), Dimension::Auto),
            ),
            max_intrinsic_height: max_height.to_px(RelativeType::ParentHeight, relative_ctx, absolute_ctx),
            max_height: ComputedMaxDimension::from(max_height),
            line_height: specified_style
                .line_height
                .resolve_with_context_owned(LineHeight::px(relative_ctx.parent.line_height), LineHeight::Normal)
                .to_px(absolute_ctx, relative_ctx.parent.font_size),
            margin_top: margin_top.to_px(Some(RelativeType::ParentWidth), relative_ctx, absolute_ctx),
            margin_right: margin_right.to_px(Some(RelativeType::ParentWidth), relative_ctx, absolute_ctx),
            margin_bottom: margin_bottom.to_px(Some(RelativeType::ParentWidth), relative_ctx, absolute_ctx),
            margin_left: margin_left.to_px(Some(RelativeType::ParentWidth), relative_ctx, absolute_ctx),
            margin_top_auto: margin_top.is_auto(),
            margin_right_auto: margin_right.is_auto(),
            margin_bottom_auto: margin_bottom.is_auto(),
            margin_left_auto: margin_left.is_auto(),
            padding_top: padding_top.to_px(Some(RelativeType::ParentWidth), relative_ctx, absolute_ctx),
            padding_right: padding_right.to_px(Some(RelativeType::ParentWidth), relative_ctx, absolute_ctx),
            padding_bottom: padding_bottom.to_px(Some(RelativeType::ParentWidth), relative_ctx, absolute_ctx),
            padding_left: padding_left.to_px(Some(RelativeType::ParentWidth), relative_ctx, absolute_ctx),
            padding_top_auto: padding_top.is_auto(),
            padding_right_auto: padding_right.is_auto(),
            padding_bottom_auto: padding_bottom.is_auto(),
            padding_left_auto: padding_left.is_auto(),
            position: specified_style
                .position
                .resolve_with_context_owned(relative_ctx.parent.position, Position::Static),
            text_align: specified_style
                .text_align
                .resolve_with_context_owned(relative_ctx.parent.text_align, TextAlign::Start),
            whitespace: specified_style
                .whitespace
                .resolve_with_context_owned(relative_ctx.parent.whitespace, Whitespace::Normal),
            intrinsic_width: match &CSSProperty::resolve(&specified_style.width) {
                Ok(Dimension::Length(l)) => l.to_px(relative_ctx, absolute_ctx),
                Ok(Dimension::Percentage(p)) => relative_ctx.parent.intrinsic_width * p.as_fraction(),
                Ok(Dimension::Calc(calc)) => calc.to_px(Some(RelativeType::ParentWidth), relative_ctx, absolute_ctx),
                _ => 0.0,
            },
            width: ComputedDimension::from(
                specified_style
                    .width
                    .resolve_with_context_owned(relative_ctx.parent.width.into(), Dimension::Auto),
            ),
            max_intrinsic_width: max_width.to_px(RelativeType::ParentWidth, relative_ctx, absolute_ctx),
            max_width: ComputedMaxDimension::from(max_width),
            writing_mode: specified_style
                .writing_mode
                .resolve_with_context_owned(relative_ctx.parent.writing_mode, WritingMode::HorizontalTb),
            variables: specified_style.variables.clone(),
        }
    }

    /// Returns a subset of the ComputedStyle containing only inherited properties.
    pub fn inherited_subset(&self) -> Self {
        ComputedStyle {
            color: self.color,
            font_family: Arc::clone(&self.font_family),
            font_size: self.font_size,
            line_height: self.line_height,
            text_align: self.text_align,
            font_weight: self.font_weight,
            whitespace: self.whitespace,
            writing_mode: self.writing_mode,
            ..ComputedStyle::default()
        }
    }
}

impl Default for ComputedStyle {
    fn default() -> Self {
        Self {
            background_color: Color4f::new(0.0, 0.0, 0.0, 0.0),
            border_top_color: Color4f::new(0.0, 0.0, 0.0, 1.0),
            border_right_color: Color4f::new(0.0, 0.0, 0.0, 1.0),
            border_bottom_color: Color4f::new(0.0, 0.0, 0.0, 1.0),
            border_left_color: Color4f::new(0.0, 0.0, 0.0, 1.0),
            border_top_style: BorderStyle::None,
            border_right_style: BorderStyle::None,
            border_bottom_style: BorderStyle::None,
            border_left_style: BorderStyle::None,
            border_top_width: 0.0,
            border_right_width: 0.0,
            border_bottom_width: 0.0,
            border_left_width: 0.0,
            color: Color4f::new(0.0, 0.0, 0.0, 1.0),
            display: Display::new(Some(OutsideDisplay::Inline), Some(InsideDisplay::Flow), None, None, None),
            font_family: Arc::new(FontFamily::new(&[FontFamilyName::Generic(GenericName::Serif)])),
            font_size: 16.0,
            font_weight: 500,
            height: ComputedDimension::Auto,
            intrinsic_height: 0.0,
            max_height: ComputedMaxDimension::None,
            max_intrinsic_height: f32::INFINITY,
            line_height: 16.0 * 1.2,
            margin_top: 0.0,
            margin_right: 0.0,
            margin_bottom: 0.0,
            margin_left: 0.0,
            padding_top: 0.0,
            padding_right: 0.0,
            padding_bottom: 0.0,
            padding_left: 0.0,
            margin_top_auto: false,
            margin_right_auto: false,
            margin_bottom_auto: false,
            margin_left_auto: false,
            padding_top_auto: false,
            padding_right_auto: false,
            padding_bottom_auto: false,
            padding_left_auto: false,
            position: Position::Static,
            text_align: TextAlign::Start,
            whitespace: Whitespace::Normal,
            width: ComputedDimension::Auto,
            intrinsic_width: 0.0,
            max_width: ComputedMaxDimension::None,
            max_intrinsic_width: f32::INFINITY,
            writing_mode: WritingMode::HorizontalTb,
            variables: Arc::new(vec![]),
        }
    }
}

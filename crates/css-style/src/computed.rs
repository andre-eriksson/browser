use css_cssom::{ComponentValue, Property};
use html_dom::{DocumentRoot, NodeId};

use crate::{
    BorderStyle, BorderWidth, FontSize, FontWeight, OffsetValue, RelativeContext,
    cascade::GeneratedRule,
    color::named::NamedColor,
    computed::color::Color4f,
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
    pub font_family: FontFamily,
    pub font_size: f32,
    pub font_weight: u16,
    pub height: Dimension,
    pub intrinsic_height: f32,
    pub max_height: MaxDimension,
    pub line_height: LineHeight,
    pub margin_top: OffsetValue,
    pub margin_right: OffsetValue,
    pub margin_bottom: OffsetValue,
    pub margin_left: OffsetValue,
    pub padding_top: OffsetValue,
    pub padding_right: OffsetValue,
    pub padding_bottom: OffsetValue,
    pub padding_left: OffsetValue,
    pub position: Position,
    pub text_align: TextAlign,
    pub whitespace: Whitespace,
    pub width: Dimension,
    pub intrinsic_width: f32,
    pub max_width: MaxDimension,
    pub writing_mode: WritingMode,
    pub variables: Vec<(Property, Vec<ComponentValue>)>,
}

impl ComputedStyle {
    /// Computes the ComputedStyle for a given node in the DOM.
    pub fn from_node(
        absolute_ctx: &AbsoluteContext,
        relative_ctx: &mut RelativeContext,
        node_id: &NodeId,
        dom: &DocumentRoot,
        rules: &[GeneratedRule],
        parent_style: Option<&ComputedStyle>,
    ) -> Self {
        let specified_style = SpecifiedStyle::from_node(
            absolute_ctx,
            relative_ctx,
            node_id,
            dom,
            rules,
            parent_style,
        );

        Self {
            background_color: Color4f::from_css_color_property(
                &specified_style.background_color,
                &CSSProperty::Value(Color::Transparent),
                &Color::Transparent,
                parent_style.map(|s| Color::from(s.background_color)),
            ),
            border_top_color: Color4f::from_css_color_property(
                &specified_style.border_top_color,
                &specified_style.color,
                &Color::Current,
                parent_style.map(|s| Color::from(s.border_top_color)),
            ),
            border_right_color: Color4f::from_css_color_property(
                &specified_style.border_right_color,
                &specified_style.color,
                &Color::Current,
                parent_style.map(|s| Color::from(s.border_right_color)),
            ),
            border_bottom_color: Color4f::from_css_color_property(
                &specified_style.border_bottom_color,
                &specified_style.color,
                &Color::Current,
                parent_style.map(|s| Color::from(s.border_bottom_color)),
            ),
            border_left_color: Color4f::from_css_color_property(
                &specified_style.border_left_color,
                &specified_style.color,
                &Color::Current,
                parent_style.map(|s| Color::from(s.border_left_color)),
            ),
            border_top_style: specified_style.border_top_style.resolve_with_context_owned(
                relative_ctx.parent.border_top_style,
                BorderStyle::None,
            ),
            border_right_style: specified_style
                .border_right_style
                .resolve_with_context_owned(
                    relative_ctx.parent.border_right_style,
                    BorderStyle::None,
                ),
            border_bottom_style: specified_style
                .border_bottom_style
                .resolve_with_context_owned(
                    relative_ctx.parent.border_bottom_style,
                    BorderStyle::None,
                ),
            border_left_style: specified_style
                .border_left_style
                .resolve_with_context_owned(
                    relative_ctx.parent.border_left_style,
                    BorderStyle::None,
                ),
            border_top_width: specified_style
                .border_top_width
                .resolve_with_context_owned(
                    BorderWidth::px(relative_ctx.parent.border_top_width),
                    BorderWidth::zero(),
                )
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
                .resolve_with_context_owned(
                    BorderWidth::px(relative_ctx.parent.border_left_width),
                    BorderWidth::zero(),
                )
                .to_px(relative_ctx, absolute_ctx),
            color: Color4f::from_css_color_property(
                &specified_style.color,
                &CSSProperty::Value(Color::Named(NamedColor::Black)),
                &Color::Named(NamedColor::Black),
                parent_style.map(|s| Color::from(s.color)),
            ),
            display: specified_style
                .display
                .resolve_with_context_owned(relative_ctx.parent.display, Display::default()),
            font_family: specified_style.font_family.resolve_with_context_owned(
                relative_ctx.parent.font_family.clone(),
                FontFamily::new(&[FontFamilyName::Generic(GenericName::Serif)]),
            ),
            font_size: specified_style
                .font_size
                .resolve_with_context_owned(
                    FontSize::px(relative_ctx.parent.font_size),
                    FontSize::px(16.0),
                )
                .to_px(absolute_ctx, relative_ctx.parent.font_size),
            font_weight: specified_style.font_weight.resolve_with_context_owned(
                FontWeight::try_from(relative_ctx.parent.font_weight).unwrap_or(FontWeight::Normal),
                FontWeight::Normal,
            ) as u16,
            intrinsic_height: match &CSSProperty::resolve(&specified_style.height) {
                Ok(Dimension::Length(l)) => l.to_px(relative_ctx, absolute_ctx),
                _ => 0.0,
            },
            height: specified_style
                .height
                .resolve_with_context_owned(relative_ctx.parent.height.clone(), Dimension::Auto),
            max_height: specified_style.max_height.resolve_with_context_owned(
                relative_ctx.parent.max_height.clone(),
                MaxDimension::None,
            ),
            line_height: specified_style.line_height.resolve_with_context_owned(
                relative_ctx.parent.line_height.clone(),
                LineHeight::Normal,
            ),
            margin_top: specified_style.margin_top.resolve_with_context_owned(
                relative_ctx.parent.margin_top.clone(),
                OffsetValue::zero(),
            ),
            margin_right: specified_style.margin_right.resolve_with_context_owned(
                relative_ctx.parent.margin_right.clone(),
                OffsetValue::zero(),
            ),
            margin_bottom: specified_style.margin_bottom.resolve_with_context_owned(
                relative_ctx.parent.margin_bottom.clone(),
                OffsetValue::zero(),
            ),
            margin_left: specified_style.margin_left.resolve_with_context_owned(
                relative_ctx.parent.margin_left.clone(),
                OffsetValue::zero(),
            ),
            padding_top: specified_style.padding_top.resolve_with_context_owned(
                relative_ctx.parent.padding_top.clone(),
                OffsetValue::zero(),
            ),
            padding_right: specified_style.padding_right.resolve_with_context_owned(
                relative_ctx.parent.padding_right.clone(),
                OffsetValue::zero(),
            ),
            padding_bottom: specified_style.padding_bottom.resolve_with_context_owned(
                relative_ctx.parent.padding_bottom.clone(),
                OffsetValue::zero(),
            ),
            padding_left: specified_style.padding_left.resolve_with_context_owned(
                relative_ctx.parent.padding_left.clone(),
                OffsetValue::zero(),
            ),
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
                _ => 0.0,
            },
            width: specified_style
                .width
                .resolve_with_context_owned(relative_ctx.parent.width.clone(), Dimension::Auto),
            max_width: specified_style.max_width.resolve_with_context_owned(
                relative_ctx.parent.max_width.clone(),
                MaxDimension::None,
            ),
            writing_mode: specified_style.writing_mode.resolve_with_context_owned(
                relative_ctx.parent.writing_mode,
                WritingMode::HorizontalTb,
            ),
            variables: specified_style.variables.clone(),
        }
    }

    /// Returns a subset of the ComputedStyle containing only inherited properties.
    pub fn inherited_subset(&self) -> Self {
        ComputedStyle {
            color: self.color,
            font_family: self.font_family.clone(),
            font_size: self.font_size,
            line_height: self.line_height.clone(),
            text_align: self.text_align,
            font_weight: self.font_weight,
            whitespace: self.whitespace,
            writing_mode: self.writing_mode,
            variables: self.variables.clone(),
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
            display: Display::new(
                Some(OutsideDisplay::Inline),
                Some(InsideDisplay::Flow),
                None,
                None,
                None,
            ),
            font_family: FontFamily::new(&[FontFamilyName::Generic(GenericName::Serif)]),
            font_size: 16.0,
            font_weight: 500,
            height: Dimension::Auto,
            intrinsic_height: 0.0,
            max_height: MaxDimension::None,
            line_height: LineHeight::Normal,
            margin_top: OffsetValue::zero(),
            margin_right: OffsetValue::zero(),
            margin_bottom: OffsetValue::zero(),
            margin_left: OffsetValue::zero(),
            padding_top: OffsetValue::zero(),
            padding_right: OffsetValue::zero(),
            padding_bottom: OffsetValue::zero(),
            padding_left: OffsetValue::zero(),
            position: Position::Static,
            text_align: TextAlign::Start,
            whitespace: Whitespace::Normal,
            width: Dimension::Auto,
            intrinsic_width: 0.0,
            max_width: MaxDimension::None,
            writing_mode: WritingMode::HorizontalTb,
            variables: Vec::new(),
        }
    }
}

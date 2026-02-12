use css_cssom::{ComponentValue, Property};
use html_dom::{DocumentRoot, NodeId};

use crate::{
    BorderStyleValue, CSSProperty, OffsetValue, RelativeContext, RelativeType,
    cascade::GeneratedRule,
    color::named::NamedColor,
    computed::color::Color4f,
    primitives::{
        display::{InsideDisplay, OutsideDisplay},
        font::GenericName,
    },
    properties::{
        AbsoluteContext,
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

#[derive(Debug, Clone, PartialEq)]
pub struct ComputedStyle {
    pub background_color: Color4f,
    pub border_top_color: Color4f,
    pub border_right_color: Color4f,
    pub border_bottom_color: Color4f,
    pub border_left_color: Color4f,
    pub border_top_style: BorderStyleValue,
    pub border_right_style: BorderStyleValue,
    pub border_bottom_style: BorderStyleValue,
    pub border_left_style: BorderStyleValue,
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
    pub max_width: MaxDimension,
    pub writing_mode: WritingMode,
    pub variables: Vec<(Property, Vec<ComponentValue>)>,
}

impl ComputedStyle {
    /// Computes the ComputedStyle for a given node in the DOM.
    ///
    /// # Arguments
    /// * `node_id` - The NodeId of the DOM node to compute the style for.
    /// * `dom` - The DocumentRoot representing the DOM tree.
    /// * `rules` - A slice of GeneratedRule representing the CSS rules to apply.
    /// * `parent_style` - An optional reference to the ComputedStyle of the parent node for inheritance.
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

        Self::update_relative_context(absolute_ctx, relative_ctx, &specified_style);

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
            border_top_style: specified_style
                .border_top_style
                .as_value_owned()
                .unwrap_or_default(),
            border_right_style: specified_style
                .border_right_style
                .as_value_owned()
                .unwrap_or_default(),
            border_bottom_style: specified_style
                .border_bottom_style
                .as_value_owned()
                .unwrap_or_default(),
            border_left_style: specified_style
                .border_left_style
                .as_value_owned()
                .unwrap_or_default(),
            border_top_width: specified_style
                .border_top_width
                .as_value_owned()
                .unwrap_or_default()
                .to_px(relative_ctx, absolute_ctx),
            border_right_width: specified_style
                .border_right_width
                .as_value_owned()
                .unwrap_or_default()
                .to_px(relative_ctx, absolute_ctx),
            border_bottom_width: specified_style
                .border_bottom_width
                .as_value_owned()
                .unwrap_or_default()
                .to_px(relative_ctx, absolute_ctx),
            border_left_width: specified_style
                .border_left_width
                .as_value_owned()
                .unwrap_or_default()
                .to_px(relative_ctx, absolute_ctx),
            color: Color4f::from_css_color_property(
                &specified_style.color,
                &CSSProperty::Value(Color::Named(NamedColor::Black)),
                &Color::Named(NamedColor::Black),
                parent_style.map(|s| Color::from(s.color)),
            ),
            display: specified_style.display.as_value_owned().unwrap_or_default(),
            font_family: specified_style
                .font_family
                .as_value_owned()
                .unwrap_or_else(|| FontFamily::new(&[FontFamilyName::Generic(GenericName::Serif)])),
            font_size: specified_style
                .font_size
                .as_value_owned()
                .unwrap_or_default()
                .to_px(absolute_ctx, relative_ctx.parent_font_size),
            font_weight: specified_style
                .font_weight
                .as_value_owned()
                .unwrap_or_default() as u16,
            height: specified_style.height.as_value_owned().unwrap_or_default(),
            max_height: specified_style
                .max_height
                .as_value_owned()
                .unwrap_or_default(),
            line_height: specified_style
                .line_height
                .as_value_owned()
                .unwrap_or_default(),
            margin_top: specified_style
                .margin_top
                .as_value_owned()
                .unwrap_or_default(),
            margin_right: specified_style
                .margin_right
                .as_value_owned()
                .unwrap_or_default(),
            margin_bottom: specified_style
                .margin_bottom
                .as_value_owned()
                .unwrap_or_default(),
            margin_left: specified_style
                .margin_left
                .as_value_owned()
                .unwrap_or_default(),
            padding_top: specified_style
                .padding_top
                .as_value_owned()
                .unwrap_or_default(),
            padding_right: specified_style
                .padding_right
                .as_value_owned()
                .unwrap_or_default(),
            padding_bottom: specified_style
                .padding_bottom
                .as_value_owned()
                .unwrap_or_default(),
            padding_left: specified_style
                .padding_left
                .as_value_owned()
                .unwrap_or_default(),
            position: specified_style
                .position
                .as_value_owned()
                .unwrap_or_default(),
            text_align: specified_style
                .text_align
                .as_value_owned()
                .unwrap_or_default(),
            whitespace: specified_style
                .whitespace
                .as_value_owned()
                .unwrap_or_default(),
            width: specified_style.width.as_value_owned().unwrap_or_default(),
            max_width: specified_style
                .max_width
                .as_value_owned()
                .unwrap_or_default(),
            writing_mode: specified_style
                .writing_mode
                .as_value_owned()
                .unwrap_or_default(),
            variables: specified_style.variables.clone(),
        }
    }

    fn update_relative_context(
        absolute_ctx: &AbsoluteContext,
        relative_ctx: &mut RelativeContext,
        style: &SpecifiedStyle,
    ) {
        if let Some(font_size) = style.font_size.as_value_ref() {
            relative_ctx.parent_font_size =
                font_size.to_px(absolute_ctx, relative_ctx.parent_font_size);
        }

        if let Some(height) = style.height.as_value_ref() {
            relative_ctx.parent_height =
                height.to_px(RelativeType::ParentHeight, relative_ctx, absolute_ctx);
        }

        if let Some(width) = style.width.as_value_ref() {
            relative_ctx.parent_width =
                width.to_px(RelativeType::ParentWidth, relative_ctx, absolute_ctx);
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
            border_top_style: BorderStyleValue::None,
            border_right_style: BorderStyleValue::None,
            border_bottom_style: BorderStyleValue::None,
            border_left_style: BorderStyleValue::None,
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
            max_width: MaxDimension::None,
            writing_mode: WritingMode::HorizontalTb,
            variables: Vec::new(),
        }
    }
}

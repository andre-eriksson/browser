use std::sync::Arc;

use css_cssom::{ComponentValue, Property};
use css_values::{
    border::{BorderStyle, BorderWidth},
    color::{Color, base::ColorBase, named::NamedColor},
    cursor::Cursor,
    dimension::{MaxSize, OffsetValue},
    display::{Clear, Float},
    text::{FontSize, LineHeight, TextAlign, Whitespace, WritingMode},
};
use html_dom::{DocumentRoot, NodeId};

use crate::{
    AbsoluteContext, Color4f, ComputedMaxDimension, ComputedSize, Display, FontFamily, Position, RelativeContext,
    RelativeType,
    cascade::RuleIndex,
    clone_compute, compute, compute_parent_px, compute_px,
    computed::{image::ComputedBackgroundImage, position::ComputedBackgroundSize},
    into_compute,
    properties::{
        CSSProperty, PixelRepr,
        background::{
            BackgroundAttachment, BackgroundBlendMode, BackgroundClip, BackgroundOrigin, BackgroundPositionX,
            BackgroundPositionY, BackgroundRepeat,
        },
    },
    rules::GeneratedRule,
    specified::SpecifiedStyle,
    tree::PropertyRegistry,
};

pub mod color;
pub mod dimension;
mod handler;
pub mod image;
pub mod position;

/// The final style resolution for a DOM node.
///
/// Represents the style for a DOM node after applying all CSS rules, resolving inheritance,
/// and applying the cascade. It contains all the properties that affect the layout and rendering of the node,
/// with all values resolved to their final forms (e.g., colors as RGBA, lengths in pixels, etc.).
#[derive(Debug, Clone, PartialEq)]
pub struct ComputedStyle {
    pub background_attachment: BackgroundAttachment,
    pub background_blend_mode: BackgroundBlendMode,
    pub background_clip: BackgroundClip,
    pub background_color: Color4f,
    pub background_image: ComputedBackgroundImage,
    pub background_origin: BackgroundOrigin,
    pub background_position_x: BackgroundPositionX,
    pub background_position_y: BackgroundPositionY,
    pub background_repeat: BackgroundRepeat,
    pub background_size: ComputedBackgroundSize,
    pub border_bottom_color: Color4f,
    pub border_bottom_style: BorderStyle,
    pub border_bottom_width: f64,
    pub border_left_color: Color4f,
    pub border_left_style: BorderStyle,
    pub border_left_width: f64,
    pub border_right_color: Color4f,
    pub border_right_style: BorderStyle,
    pub border_right_width: f64,
    pub border_top_color: Color4f,
    pub border_top_style: BorderStyle,
    pub border_top_width: f64,
    pub bottom: f64,
    pub bottom_auto: bool,
    pub clear: Clear,
    pub color: Color4f,
    pub cursor: Cursor,
    pub display: Display,
    pub float: Float,
    pub font_family: Arc<FontFamily>,
    pub font_size: f64,
    pub font_weight: u16,
    pub height: ComputedSize,
    pub intrinsic_height: f64,
    pub intrinsic_width: f64,
    pub left: f64,
    pub left_auto: bool,
    pub line_height: f64,
    pub margin_bottom: f64,
    pub margin_bottom_auto: bool,
    pub margin_left: f64,
    pub margin_left_auto: bool,
    pub margin_right: f64,
    pub margin_right_auto: bool,
    pub margin_top: f64,
    pub margin_top_auto: bool,
    pub max_height: ComputedMaxDimension,
    pub max_intrinsic_height: f64,
    pub max_intrinsic_width: f64,
    pub max_width: ComputedMaxDimension,
    pub padding_bottom: f64,
    pub padding_bottom_auto: bool,
    pub padding_left: f64,
    pub padding_left_auto: bool,
    pub padding_right: f64,
    pub padding_right_auto: bool,
    pub padding_top: f64,
    pub padding_top_auto: bool,
    pub position: Position,
    pub right: f64,
    pub right_auto: bool,
    pub text_align: TextAlign,
    pub top: f64,
    pub top_auto: bool,
    pub whitespace: Whitespace,
    pub width: ComputedSize,
    pub writing_mode: WritingMode,

    pub variables: Arc<Vec<(Property, Vec<ComponentValue>)>>,
}

impl ComputedStyle {
    /// Computes the `ComputedStyle` for a given node in the DOM.
    pub fn from_node(
        absolute_ctx: &AbsoluteContext,
        relative_ctx: &mut RelativeContext,
        node_id: NodeId,
        dom: &DocumentRoot,
        rules: &[GeneratedRule],
        rule_index: &RuleIndex,
        property_registry: &mut PropertyRegistry,
    ) -> Self {
        let parent = &relative_ctx.parent;

        let specified_style =
            SpecifiedStyle::from_node(absolute_ctx, relative_ctx, node_id, dom, rules, rule_index, property_registry);

        let margin_top = compute_px!(specified_style, parent, margin_top, OffsetValue);
        let margin_right = compute_px!(specified_style, parent, margin_right, OffsetValue);
        let margin_bottom = compute_px!(specified_style, parent, margin_bottom, OffsetValue);
        let margin_left = compute_px!(specified_style, parent, margin_left, OffsetValue);
        let padding_top = compute_px!(specified_style, parent, padding_top, OffsetValue);
        let padding_right = compute_px!(specified_style, parent, padding_right, OffsetValue);
        let padding_bottom = compute_px!(specified_style, parent, padding_bottom, OffsetValue);
        let padding_left = compute_px!(specified_style, parent, padding_left, OffsetValue);
        let bottom = compute_px!(specified_style, parent, bottom, OffsetValue);
        let left = compute_px!(specified_style, parent, left, OffsetValue);
        let right = compute_px!(specified_style, parent, right, OffsetValue);
        let top = compute_px!(specified_style, parent, top, OffsetValue);
        let height = into_compute!(specified_style, parent, height);
        let max_height = compute_parent_px!(specified_style, parent, max_height, max_intrinsic_height, MaxSize);
        let width = into_compute!(specified_style, parent, width);
        let max_width = compute_parent_px!(specified_style, parent, max_width, max_intrinsic_width, MaxSize);
        let font_size = specified_style
            .font_size
            .compute(FontSize::px(parent.font_size))
            .to_px(Some(RelativeType::FontSize), Some(relative_ctx), absolute_ctx);
        let float = compute!(specified_style, parent, float);

        relative_ctx.font_size = font_size;

        Self {
            background_attachment: clone_compute!(specified_style, parent, background_attachment),
            background_blend_mode: clone_compute!(specified_style, parent, background_blend_mode),
            background_clip: clone_compute!(specified_style, parent, background_clip),
            background_color: Color4f::from_css_color_property(
                &specified_style.background_color,
                &specified_style.color,
                &Color::Base(ColorBase::Transparent),
                &relative_ctx.parent.background_color.into(),
                relative_ctx,
                absolute_ctx,
            ),
            background_origin: clone_compute!(specified_style, parent, background_origin),
            background_image: ComputedBackgroundImage::resolve(
                specified_style
                    .background_image
                    .compute(parent.background_image.clone().into())
                    .0,
                absolute_ctx,
            )
            .unwrap_or_default(),
            background_position_x: clone_compute!(specified_style, parent, background_position_x),
            background_position_y: clone_compute!(specified_style, parent, background_position_y),
            background_repeat: clone_compute!(specified_style, parent, background_repeat),
            background_size: ComputedBackgroundSize::resolve(
                specified_style
                    .background_size
                    .compute(parent.background_size.clone().into()),
                Some(RelativeType::BackgroundArea),
                relative_ctx,
                absolute_ctx,
            ),
            border_top_color: Color4f::from_css_color_property(
                &specified_style.border_top_color,
                &specified_style.color,
                &Color::Current,
                &relative_ctx.parent.border_top_color.into(),
                relative_ctx,
                absolute_ctx,
            ),
            border_right_color: Color4f::from_css_color_property(
                &specified_style.border_right_color,
                &specified_style.color,
                &Color::Current,
                &relative_ctx.parent.border_right_color.into(),
                relative_ctx,
                absolute_ctx,
            ),
            border_bottom_color: Color4f::from_css_color_property(
                &specified_style.border_bottom_color,
                &specified_style.color,
                &Color::Current,
                &relative_ctx.parent.border_bottom_color.into(),
                relative_ctx,
                absolute_ctx,
            ),
            border_left_color: Color4f::from_css_color_property(
                &specified_style.border_left_color,
                &specified_style.color,
                &Color::Current,
                &relative_ctx.parent.border_left_color.into(),
                relative_ctx,
                absolute_ctx,
            ),
            border_top_style: compute!(specified_style, parent, border_top_style),
            border_right_style: compute!(specified_style, parent, border_right_style),
            border_bottom_style: compute!(specified_style, parent, border_bottom_style),
            border_left_style: compute!(specified_style, parent, border_left_style),
            border_top_width: compute_px!(specified_style, parent, border_top_width, BorderWidth).to_px(
                None,
                Some(relative_ctx),
                absolute_ctx,
            ),
            border_right_width: compute_px!(specified_style, parent, border_right_width, BorderWidth).to_px(
                None,
                Some(relative_ctx),
                absolute_ctx,
            ),
            border_bottom_width: compute_px!(specified_style, parent, border_bottom_width, BorderWidth).to_px(
                None,
                Some(relative_ctx),
                absolute_ctx,
            ),
            border_left_width: compute_px!(specified_style, parent, border_left_width, BorderWidth).to_px(
                None,
                Some(relative_ctx),
                absolute_ctx,
            ),
            bottom: bottom.to_px(Some(RelativeType::ParentHeight), Some(relative_ctx), absolute_ctx),
            bottom_auto: bottom.is_auto(),
            clear: compute!(specified_style, parent, clear),
            color: Color4f::from_css_color_property(
                &specified_style.color,
                &CSSProperty::Value(Color::BLACK),
                &Color::Base(ColorBase::Named(NamedColor::Black)),
                &relative_ctx.parent.color.into(),
                relative_ctx,
                absolute_ctx,
            ),
            cursor: compute!(specified_style, parent, cursor),
            display: compute!(specified_style, parent, display).adjust_float(float),
            float,
            font_family: Arc::new(
                specified_style
                    .font_family
                    .compute((*parent.font_family).clone()),
            ),
            font_size,
            font_weight: specified_style
                .font_weight
                .compute(parent.font_weight.into()) as u16,
            intrinsic_height: height.to_px(Some(RelativeType::ParentHeight), Some(relative_ctx), absolute_ctx),
            height: height.into(),
            left: left.to_px(Some(RelativeType::ParentWidth), Some(relative_ctx), absolute_ctx),
            left_auto: left.is_auto(),
            max_intrinsic_height: max_height.to_px(Some(RelativeType::ParentHeight), Some(relative_ctx), absolute_ctx),
            max_height: max_height.into(),
            line_height: compute_px!(specified_style, parent, line_height, LineHeight).to_px(
                None,
                Some(relative_ctx),
                absolute_ctx,
            ),
            margin_top: margin_top.to_px(Some(RelativeType::ParentWidth), Some(relative_ctx), absolute_ctx),
            margin_right: margin_right.to_px(Some(RelativeType::ParentWidth), Some(relative_ctx), absolute_ctx),
            margin_bottom: margin_bottom.to_px(Some(RelativeType::ParentWidth), Some(relative_ctx), absolute_ctx),
            margin_left: margin_left.to_px(Some(RelativeType::ParentWidth), Some(relative_ctx), absolute_ctx),
            margin_top_auto: margin_top.is_auto(),
            margin_right_auto: margin_right.is_auto(),
            margin_bottom_auto: margin_bottom.is_auto(),
            margin_left_auto: margin_left.is_auto(),
            padding_top: padding_top.to_px(Some(RelativeType::ParentWidth), Some(relative_ctx), absolute_ctx),
            padding_right: padding_right.to_px(Some(RelativeType::ParentWidth), Some(relative_ctx), absolute_ctx),
            padding_bottom: padding_bottom.to_px(Some(RelativeType::ParentWidth), Some(relative_ctx), absolute_ctx),
            padding_left: padding_left.to_px(Some(RelativeType::ParentWidth), Some(relative_ctx), absolute_ctx),
            padding_top_auto: padding_top.is_auto(),
            padding_right_auto: padding_right.is_auto(),
            padding_bottom_auto: padding_bottom.is_auto(),
            padding_left_auto: padding_left.is_auto(),
            position: compute!(specified_style, parent, position),
            right: right.to_px(Some(RelativeType::ParentWidth), Some(relative_ctx), absolute_ctx),
            right_auto: right.is_auto(),
            text_align: compute!(specified_style, parent, text_align),
            top: top.to_px(Some(RelativeType::ParentHeight), Some(relative_ctx), absolute_ctx),
            top_auto: top.is_auto(),
            whitespace: compute!(specified_style, parent, whitespace),
            intrinsic_width: width.to_px(Some(RelativeType::ParentWidth), Some(relative_ctx), absolute_ctx),
            width: width.into(),
            max_intrinsic_width: max_width.to_px(Some(RelativeType::ParentWidth), Some(relative_ctx), absolute_ctx),
            max_width: max_width.into(),
            writing_mode: compute!(specified_style, parent, writing_mode),
            variables: Arc::clone(&specified_style.variables),
        }
    }

    /// Returns a subset of the `ComputedStyle` containing only inherited properties.
    #[must_use]
    pub fn inherited_subset(&self) -> Self {
        Self {
            color: self.color,
            cursor: self.cursor,
            font_family: Arc::clone(&self.font_family),
            font_size: self.font_size,
            line_height: self.line_height,
            text_align: self.text_align,
            font_weight: self.font_weight,
            whitespace: self.whitespace,
            writing_mode: self.writing_mode,
            ..Self::default()
        }
    }
}

impl Default for ComputedStyle {
    fn default() -> Self {
        Self {
            background_attachment: BackgroundAttachment::default(),
            background_blend_mode: BackgroundBlendMode::default(),
            background_clip: BackgroundClip::default(),
            background_color: Color4f::TRANSPARENT,
            background_image: ComputedBackgroundImage::none(),
            background_origin: BackgroundOrigin::default(),
            background_position_x: BackgroundPositionX::default(),
            background_position_y: BackgroundPositionY::default(),
            background_repeat: BackgroundRepeat::default(),
            background_size: ComputedBackgroundSize::default(),
            border_bottom_color: Color4f::BLACK,
            border_bottom_style: BorderStyle::None,
            border_bottom_width: 0.0,
            border_left_color: Color4f::BLACK,
            border_left_style: BorderStyle::None,
            border_left_width: 0.0,
            border_right_color: Color4f::BLACK,
            border_right_style: BorderStyle::None,
            border_right_width: 0.0,
            border_top_color: Color4f::BLACK,
            border_top_style: BorderStyle::None,
            border_top_width: 0.0,
            bottom: 0.0,
            bottom_auto: true,
            clear: Clear::default(),
            color: Color4f::BLACK,
            cursor: Cursor::default(),
            display: Display::default(),
            float: Float::default(),
            font_family: Arc::new(FontFamily::default()),
            font_size: 16.0,
            font_weight: 500,
            height: ComputedSize::Auto,
            intrinsic_height: 0.0,
            intrinsic_width: 0.0,
            left: 0.0,
            left_auto: true,
            line_height: 1.5 * 16.0,
            margin_bottom: 0.0,
            margin_bottom_auto: false,
            margin_left: 0.0,
            margin_left_auto: false,
            margin_right: 0.0,
            margin_right_auto: false,
            margin_top: 0.0,
            margin_top_auto: false,
            max_height: ComputedMaxDimension::None,
            max_intrinsic_height: f64::INFINITY,
            max_intrinsic_width: f64::INFINITY,
            max_width: ComputedMaxDimension::None,
            padding_bottom: 0.0,
            padding_bottom_auto: false,
            padding_left: 0.0,
            padding_left_auto: false,
            padding_right: 0.0,
            padding_right_auto: false,
            padding_top: 0.0,
            padding_top_auto: false,
            position: Position::Static,
            right: 0.0,
            right_auto: true,
            text_align: TextAlign::Start,
            top: 0.0,
            top_auto: true,
            whitespace: Whitespace::Normal,
            width: ComputedSize::Auto,
            writing_mode: WritingMode::HorizontalTb,

            variables: Arc::new(vec![]),
        }
    }
}

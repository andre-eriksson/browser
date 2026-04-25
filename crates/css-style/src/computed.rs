use std::sync::Arc;

use browser_config::BrowserConfig;
use css_cssom::{ComponentValue, Property};
use css_values::{
    border::{BorderStyle, BorderWidth},
    color::{Color, base::ColorBase, named::NamedColor},
    cursor::Cursor,
    display::{Clear, Float},
    flex::{FlexDirection, FlexWrap},
    numeric::NumberOrCalc,
    text::{FontSize, LineHeight, TextAlign, Whitespace, WritingMode},
};
use html_dom::{DocumentRoot, NodeId};

use crate::{
    AbsoluteContext, Color4f, ComputedMaxSize, ComputedSize, Display, FontFamily, Position, RelativeContext,
    RelativeType, clone_compute, compute, compute_px,
    computed::{
        image::ComputedBackgroundImage,
        offset::{ComputedMargin, ComputedOffset},
        position::ComputedBackgroundSize,
    },
    into_compute,
    properties::{
        CSSProperty, PixelRepr,
        background::{
            BackgroundAttachment, BackgroundBlendMode, BackgroundClip, BackgroundOrigin, BackgroundPositionX,
            BackgroundPositionY, BackgroundRepeat,
        },
    },
    rules::Rules,
    specified::SpecifiedStyle,
    tree::PropertyRegistry,
};

pub mod color;
pub mod dimension;
pub mod flex;
mod handler;
pub mod image;
pub mod offset;
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
    pub bottom: ComputedMargin,
    pub clear: Clear,
    pub color: Color4f,
    pub cursor: Cursor,
    pub display: Display,
    pub flex_basis: ComputedFlexBasis,
    pub flex_direction: FlexDirection,
    pub flex_grow: f64,
    pub flex_shrink: f64,
    pub flex_wrap: FlexWrap,
    pub float: Float,
    pub font_family: Arc<FontFamily>,
    pub font_size: f64,
    pub font_weight: u16,
    pub height: ComputedSize,
    pub left: ComputedMargin,
    pub line_height: f64,
    pub margin_bottom: ComputedMargin,
    pub margin_left: ComputedMargin,
    pub margin_right: ComputedMargin,
    pub margin_top: ComputedMargin,
    pub max_height: ComputedMaxSize,
    pub max_width: ComputedMaxSize,
    pub padding_bottom: ComputedOffset,
    pub padding_left: ComputedOffset,
    pub padding_right: ComputedOffset,
    pub padding_top: ComputedOffset,
    pub position: Position,
    pub right: ComputedMargin,
    pub text_align: TextAlign,
    pub top: ComputedMargin,
    pub whitespace: Whitespace,
    pub width: ComputedSize,
    pub writing_mode: WritingMode,

    pub variables: Arc<Vec<(Property, Vec<ComponentValue>)>>,
}

impl ComputedStyle {
    /// Computes the `ComputedStyle` for a given node in the DOM.
    pub fn from_node(
        config: &BrowserConfig,
        absolute_ctx: &AbsoluteContext,
        relative_ctx: &mut RelativeContext,
        node_id: NodeId,
        dom: &DocumentRoot,
        rules: &Rules,
        property_registry: &mut PropertyRegistry,
    ) -> Self {
        let parent = &relative_ctx.parent;

        let specified_style =
            SpecifiedStyle::from_node(absolute_ctx, relative_ctx, node_id, dom, rules, property_registry);

        let margin_top = into_compute!(specified_style, parent, margin_top);
        let margin_right = into_compute!(specified_style, parent, margin_right);
        let margin_bottom = into_compute!(specified_style, parent, margin_bottom);
        let margin_left = into_compute!(specified_style, parent, margin_left);
        let padding_top = into_compute!(specified_style, parent, padding_top);
        let padding_right = into_compute!(specified_style, parent, padding_right);
        let padding_bottom = into_compute!(specified_style, parent, padding_bottom);
        let padding_left = into_compute!(specified_style, parent, padding_left);
        let top = into_compute!(specified_style, parent, top);
        let right = into_compute!(specified_style, parent, right);
        let bottom = into_compute!(specified_style, parent, bottom);
        let left = into_compute!(specified_style, parent, left);
        let height = into_compute!(specified_style, parent, height);
        let max_height = into_compute!(specified_style, parent, max_height);
        let width = into_compute!(specified_style, parent, width);
        let max_width = into_compute!(specified_style, parent, max_width);
        let font_size = specified_style
            .font_size
            .compute(FontSize::px(parent.font_size))
            .to_px(Some(RelativeType::FontSize), Some(relative_ctx), absolute_ctx)
            .unwrap_or(parent.font_size);
        let float = compute!(specified_style, parent, float);

        relative_ctx.font_size = font_size;

        let mut computed = Self {
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
            border_top_width: compute_px!(specified_style, parent, border_top_width, BorderWidth)
                .to_px(None, Some(relative_ctx), absolute_ctx)
                .unwrap_or(0.0),
            border_right_width: compute_px!(specified_style, parent, border_right_width, BorderWidth)
                .to_px(None, Some(relative_ctx), absolute_ctx)
                .unwrap_or(0.0),
            border_bottom_width: compute_px!(specified_style, parent, border_bottom_width, BorderWidth)
                .to_px(None, Some(relative_ctx), absolute_ctx)
                .unwrap_or(0.0),
            border_left_width: compute_px!(specified_style, parent, border_left_width, BorderWidth)
                .to_px(None, Some(relative_ctx), absolute_ctx)
                .unwrap_or(0.0),
            bottom: ComputedMargin::resolve(bottom, Some(RelativeType::ParentHeight), relative_ctx, absolute_ctx)
                .unwrap_or_default(),
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
            flex_basis: into_compute!(specified_style, parent, flex_basis).into(),
            flex_direction: compute!(specified_style, parent, flex_direction),
            flex_grow: specified_style
                .flex_grow
                .resolve_with_context(&parent.flex_grow.into(), &0.0f64.into())
                .0
                .clone()
                .into(),
            flex_shrink: specified_style
                .flex_shrink
                .resolve_with_context(&parent.flex_shrink.into(), &1.0f64.into())
                .0
                .clone()
                .into(),
            flex_wrap: compute!(specified_style, parent, flex_wrap),
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
            height: ComputedSize::resolve(height, RelativeType::ParentHeight, relative_ctx, absolute_ctx)
                .unwrap_or_default(),
            left: ComputedMargin::resolve(left, Some(RelativeType::ParentWidth), relative_ctx, absolute_ctx)
                .unwrap_or(ComputedMargin::Auto),
            max_height: ComputedMaxSize::resolve(max_height, RelativeType::ParentHeight, relative_ctx, absolute_ctx)
                .unwrap_or_default(),
            line_height: compute_px!(specified_style, parent, line_height, LineHeight).to_px_unchecked(
                None,
                Some(relative_ctx),
                absolute_ctx,
            ),
            margin_top: ComputedMargin::resolve(
                margin_top,
                Some(RelativeType::ParentWidth),
                relative_ctx,
                absolute_ctx,
            )
            .unwrap_or_default(),
            margin_right: ComputedMargin::resolve(
                margin_right,
                Some(RelativeType::ParentWidth),
                relative_ctx,
                absolute_ctx,
            )
            .unwrap_or_default(),
            margin_bottom: ComputedMargin::resolve(
                margin_bottom,
                Some(RelativeType::ParentWidth),
                relative_ctx,
                absolute_ctx,
            )
            .unwrap_or_default(),
            margin_left: ComputedMargin::resolve(
                margin_left,
                Some(RelativeType::ParentWidth),
                relative_ctx,
                absolute_ctx,
            )
            .unwrap_or_default(),
            padding_top: ComputedOffset::resolve(
                padding_top,
                Some(RelativeType::ParentWidth),
                relative_ctx,
                absolute_ctx,
            )
            .unwrap_or_default(),
            padding_right: ComputedOffset::resolve(
                padding_right,
                Some(RelativeType::ParentWidth),
                relative_ctx,
                absolute_ctx,
            )
            .unwrap_or_default(),
            padding_bottom: ComputedOffset::resolve(
                padding_bottom,
                Some(RelativeType::ParentWidth),
                relative_ctx,
                absolute_ctx,
            )
            .unwrap_or_default(),
            padding_left: ComputedOffset::resolve(
                padding_left,
                Some(RelativeType::ParentWidth),
                relative_ctx,
                absolute_ctx,
            )
            .unwrap_or_default(),
            position: compute!(specified_style, parent, position),
            right: ComputedMargin::resolve(right, Some(RelativeType::ParentWidth), relative_ctx, absolute_ctx)
                .unwrap_or(ComputedMargin::Auto),
            text_align: compute!(specified_style, parent, text_align),
            top: ComputedMargin::resolve(top, Some(RelativeType::ParentHeight), relative_ctx, absolute_ctx)
                .unwrap_or(ComputedMargin::Auto),
            whitespace: compute!(specified_style, parent, whitespace),
            width: ComputedSize::resolve(width, RelativeType::ParentWidth, relative_ctx, absolute_ctx)
                .unwrap_or_default(),
            max_width: ComputedMaxSize::resolve(max_width, RelativeType::ParentWidth, relative_ctx, absolute_ctx)
                .unwrap_or_default(),
            writing_mode: compute!(specified_style, parent, writing_mode),
            variables: Arc::clone(&specified_style.variables),
        };

        if config.args().preferences.force_dark {
            computed.apply_dark_heuristic();
        }

        computed
    }

    pub fn apply_dark_heuristic(&mut self) {
        if self.background_color.is_dark() {
            self.background_color = self.background_color.invert_dark_mode();
        }

        if self.color.is_dark() {
            self.color = self.color.invert_dark_mode();
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
            bottom: 0.0.into(),
            clear: Clear::default(),
            color: Color4f::BLACK,
            cursor: Cursor::default(),
            display: Display::default(),
            flex_basis: ComputedFlexBasis::default(),
            flex_direction: FlexDirection::default(),
            flex_grow: 0.0,
            flex_shrink: 1.0,
            flex_wrap: FlexWrap::default(),
            float: Float::default(),
            font_family: Arc::new(FontFamily::default()),
            font_size: 16.0,
            font_weight: 500,
            height: ComputedSize::Auto,
            left: 0.0.into(),
            line_height: 1.5 * 16.0,
            margin_bottom: 0.0.into(),
            margin_left: 0.0.into(),
            margin_right: 0.0.into(),
            margin_top: 0.0.into(),
            max_height: ComputedMaxSize::None,
            max_width: ComputedMaxSize::None,
            padding_bottom: 0.0.into(),
            padding_left: 0.0.into(),
            padding_right: 0.0.into(),
            padding_top: 0.0.into(),
            position: Position::Static,
            right: 0.0.into(),
            text_align: TextAlign::Start,
            top: 0.0.into(),
            whitespace: Whitespace::Normal,
            width: ComputedSize::Auto,
            writing_mode: WritingMode::HorizontalTb,

            variables: Arc::new(vec![]),
        }
    }
}

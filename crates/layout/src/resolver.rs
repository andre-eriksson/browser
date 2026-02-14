use css_style::{
    AbsoluteContext, ComputedStyle, Dimension, MaxDimension, OffsetValue, RelativeContext,
    RelativeType, StyledNode,
};

use crate::SideOffset;

pub struct PropertyResolver;

impl PropertyResolver {
    pub(crate) fn resolve_box_model(
        absolute_ctx: &AbsoluteContext,
        style: &ComputedStyle,
        containing_width: f32,
        font_size_px: f32,
    ) -> (SideOffset, SideOffset, SideOffset) {
        let margins = Self::resolve_margin(absolute_ctx, style, containing_width, font_size_px);
        let padding = Self::resolve_padding(absolute_ctx, style, containing_width, font_size_px);
        let borders = Self::resolve_border(style);

        (margins, padding, borders)
    }

    /// Resolve margin values to pixels
    pub(crate) fn resolve_node_margins(
        absolute_ctx: &AbsoluteContext,
        styled_node: &StyledNode,
        containing_width: f32,
        font_size_px: f32,
    ) -> SideOffset {
        Self::resolve_margin(
            absolute_ctx,
            &styled_node.style,
            containing_width,
            font_size_px,
        )
    }

    /// Resolve a single margin value to pixels
    pub(crate) fn resolve_margin_value(
        absolute_ctx: &AbsoluteContext,
        value: &OffsetValue,
        containing_width: f32,
        font_size_px: f32,
    ) -> f32 {
        let rel_ctx = RelativeContext {
            parent: ComputedStyle {
                font_size: font_size_px,
                width: Dimension::px(containing_width),
                ..Default::default()
            }
            .into(),
        };

        value.to_px(Some(RelativeType::ParentWidth), &rel_ctx, absolute_ctx)
    }

    fn resolve_margin(
        absolute_ctx: &AbsoluteContext,
        style: &ComputedStyle,
        containing_width: f32,
        font_size_px: f32,
    ) -> SideOffset {
        SideOffset {
            top: Self::resolve_margin_value(
                absolute_ctx,
                &style.margin_top,
                containing_width,
                font_size_px,
            ),
            right: Self::resolve_margin_value(
                absolute_ctx,
                &style.margin_right,
                containing_width,
                font_size_px,
            ),
            bottom: Self::resolve_margin_value(
                absolute_ctx,
                &style.margin_bottom,
                containing_width,
                font_size_px,
            ),
            left: Self::resolve_margin_value(
                absolute_ctx,
                &style.margin_left,
                containing_width,
                font_size_px,
            ),
        }
    }

    fn resolve_padding(
        absolute_ctx: &AbsoluteContext,
        style: &ComputedStyle,
        containing_width: f32,
        font_size_px: f32,
    ) -> SideOffset {
        SideOffset {
            top: Self::resolve_padding_value(
                absolute_ctx,
                &style.padding_top,
                containing_width,
                font_size_px,
            ),
            right: Self::resolve_padding_value(
                absolute_ctx,
                &style.padding_right,
                containing_width,
                font_size_px,
            ),
            bottom: Self::resolve_padding_value(
                absolute_ctx,
                &style.padding_bottom,
                containing_width,
                font_size_px,
            ),
            left: Self::resolve_padding_value(
                absolute_ctx,
                &style.padding_left,
                containing_width,
                font_size_px,
            ),
        }
    }

    fn resolve_border(style: &ComputedStyle) -> SideOffset {
        SideOffset {
            top: style.border_top_width,
            right: style.border_right_width,
            bottom: style.border_bottom_width,
            left: style.border_left_width,
        }
    }

    pub(crate) fn resolve_padding_value(
        absolute_ctx: &AbsoluteContext,
        value: &OffsetValue,
        containing_width: f32,
        font_size_px: f32,
    ) -> f32 {
        let rel_ctx = RelativeContext {
            parent: ComputedStyle {
                font_size: font_size_px,
                width: Dimension::px(containing_width),
                ..Default::default()
            }
            .into(),
        };

        value.to_px(Some(RelativeType::ParentWidth), &rel_ctx, absolute_ctx)
    }

    /// Calculate content width (top-down from containing block)
    pub(crate) fn calculate_width(
        absolute_ctx: &AbsoluteContext,
        styled_node: &StyledNode,
        width: f32,
    ) -> f32 {
        let font_size = styled_node.style.font_size;
        let rel_ctx = RelativeContext {
            parent: ComputedStyle {
                font_size,
                width: Dimension::px(width),
                ..Default::default()
            }
            .into(),
        };

        let max_width = match &styled_node.style.max_width {
            MaxDimension::None => f32::INFINITY,
            MaxDimension::Length(len) => len.to_px(&rel_ctx, absolute_ctx),
            MaxDimension::Percentage(pct) => pct.as_fraction() * width,
            MaxDimension::Calc(calc) => {
                calc.to_px(Some(RelativeType::ParentWidth), &rel_ctx, absolute_ctx)
            }
            MaxDimension::MaxContent
            | MaxDimension::MinContent
            | MaxDimension::FitContent(_)
            | MaxDimension::Stretch => {
                f32::INFINITY // TODO: implement intrinsic sizing
            }
        };

        let left_margin = Self::resolve_margin_value(
            absolute_ctx,
            &styled_node.style.margin_left,
            width,
            font_size,
        );
        let right_margin = Self::resolve_margin_value(
            absolute_ctx,
            &styled_node.style.margin_right,
            width,
            font_size,
        );
        let available_width = f32::min(width - (left_margin + right_margin), max_width);

        match &styled_node.style.width {
            Dimension::Auto => available_width.max(0.0),
            Dimension::Length(len) => len.to_px(&rel_ctx, absolute_ctx).max(0.0),
            Dimension::Percentage(pct) => pct.as_fraction() * width,
            Dimension::Calc(calc) => calc
                .to_px(Some(RelativeType::ParentWidth), &rel_ctx, absolute_ctx)
                .max(0.0),
            Dimension::MaxContent
            | Dimension::MinContent
            | Dimension::FitContent(_)
            | Dimension::Stretch => {
                // TODO: implement intrinsic sizing
                available_width.max(0.0)
            }
        }
    }

    pub(crate) fn calculate_height(
        absolute_ctx: &AbsoluteContext,
        styled_node: &StyledNode,
        height: f32,
        children_height: f32,
    ) -> f32 {
        let rel_ctx = RelativeContext {
            parent: ComputedStyle {
                font_size: styled_node.style.font_size,
                height: Dimension::px(height),
                ..Default::default()
            }
            .into(),
        };

        match &styled_node.style.height {
            Dimension::Auto => children_height,
            Dimension::Length(len) => len.to_px(&rel_ctx, absolute_ctx),
            Dimension::Percentage(pct) => pct.as_fraction() * height,
            Dimension::Calc(calc) => {
                calc.to_px(Some(RelativeType::ParentHeight), &rel_ctx, absolute_ctx)
            }
            Dimension::MaxContent
            | Dimension::MinContent
            | Dimension::FitContent(_)
            | Dimension::Stretch => {
                // TODO: implement intrinsic sizing
                children_height
            }
        }
    }
}

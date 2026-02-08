use css_style::{
    AbsoluteContext, BorderWidthValue, CSSProperty, ComputedStyle, Dimension, MaxDimension,
    OffsetValue, RelativeContext, RelativeType, StyledNode,
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
        let borders = Self::resolve_border(absolute_ctx, style, font_size_px);

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
            parent_width: containing_width,
            font_size: font_size_px,
            ..Default::default()
        };

        value.to_px(RelativeType::ParentWidth, &rel_ctx, absolute_ctx)
    }

    fn resolve_margin(
        absolute_ctx: &AbsoluteContext,
        style: &ComputedStyle,
        containing_width: f32,
        font_size_px: f32,
    ) -> SideOffset {
        let mut margins = SideOffset::default();
        if let Ok(top) = CSSProperty::resolve(&style.margin_top) {
            margins.top =
                Self::resolve_margin_value(absolute_ctx, top, containing_width, font_size_px);
        }
        if let Ok(right) = CSSProperty::resolve(&style.margin_right) {
            margins.right =
                Self::resolve_margin_value(absolute_ctx, right, containing_width, font_size_px);
        }
        if let Ok(bottom) = CSSProperty::resolve(&style.margin_bottom) {
            margins.bottom =
                Self::resolve_margin_value(absolute_ctx, bottom, containing_width, font_size_px);
        }
        if let Ok(left) = CSSProperty::resolve(&style.margin_left) {
            margins.left =
                Self::resolve_margin_value(absolute_ctx, left, containing_width, font_size_px);
        }
        margins
    }

    fn resolve_padding(
        absolute_ctx: &AbsoluteContext,
        style: &ComputedStyle,
        containing_width: f32,
        font_size_px: f32,
    ) -> SideOffset {
        let mut padding = SideOffset::default();
        if let Ok(top) = CSSProperty::resolve(&style.padding_top) {
            padding.top =
                Self::resolve_padding_value(absolute_ctx, top, containing_width, font_size_px);
        }
        if let Ok(right) = CSSProperty::resolve(&style.padding_right) {
            padding.right =
                Self::resolve_padding_value(absolute_ctx, right, containing_width, font_size_px);
        }
        if let Ok(bottom) = CSSProperty::resolve(&style.padding_bottom) {
            padding.bottom =
                Self::resolve_padding_value(absolute_ctx, bottom, containing_width, font_size_px);
        }
        if let Ok(left) = CSSProperty::resolve(&style.padding_left) {
            padding.left =
                Self::resolve_padding_value(absolute_ctx, left, containing_width, font_size_px);
        }
        padding
    }

    fn resolve_border(
        absolute_ctx: &AbsoluteContext,
        style: &ComputedStyle,
        font_size_px: f32,
    ) -> SideOffset {
        let mut borders = SideOffset::default();
        if let Ok(top) = CSSProperty::resolve(&style.border_top_width) {
            borders.top = Self::resolve_border_value(absolute_ctx, top, font_size_px);
        }
        if let Ok(right) = CSSProperty::resolve(&style.border_right_width) {
            borders.right = Self::resolve_border_value(absolute_ctx, right, font_size_px);
        }
        if let Ok(bottom) = CSSProperty::resolve(&style.border_bottom_width) {
            borders.bottom = Self::resolve_border_value(absolute_ctx, bottom, font_size_px);
        }
        if let Ok(left) = CSSProperty::resolve(&style.border_left_width) {
            borders.left = Self::resolve_border_value(absolute_ctx, left, font_size_px);
        }
        borders
    }

    pub(crate) fn resolve_padding_value(
        absolute_ctx: &AbsoluteContext,
        value: &OffsetValue,
        containing_width: f32,
        font_size_px: f32,
    ) -> f32 {
        let rel_ctx = RelativeContext {
            parent_width: containing_width,
            font_size: font_size_px,
            ..Default::default()
        };

        value.to_px(RelativeType::ParentWidth, &rel_ctx, absolute_ctx)
    }

    pub(crate) fn resolve_border_value(
        absolute_ctx: &AbsoluteContext,
        value: &BorderWidthValue,
        font_size_px: f32,
    ) -> f32 {
        let rel_ctx = RelativeContext {
            font_size: font_size_px,
            ..Default::default()
        };

        value.to_px(RelativeType::FontSize, &rel_ctx, absolute_ctx)
    }

    /// Calculate content width (top-down from containing block)
    pub(crate) fn calculate_width(
        absolute_ctx: &AbsoluteContext,
        styled_node: &StyledNode,
        width: f32,
    ) -> f32 {
        let font_size = styled_node.style.computed_font_size_px;

        let max_width =
            if let Ok(max_width_prop) = CSSProperty::resolve(&styled_node.style.max_width) {
                match &max_width_prop {
                    MaxDimension::None => f32::INFINITY,
                    MaxDimension::Length(len) => {
                        let rel_ctx = RelativeContext {
                            parent_width: width,
                            font_size,
                            ..Default::default()
                        };

                        len.to_px(&rel_ctx, absolute_ctx)
                    }
                    MaxDimension::Percentage(pct) => pct.as_fraction() * width,
                    MaxDimension::Calc(calc) => {
                        let rel_ctx = RelativeContext {
                            parent_width: width,
                            font_size,
                            ..Default::default()
                        };
                        calc.to_px(RelativeType::ParentWidth, &rel_ctx, absolute_ctx)
                    }
                    MaxDimension::MaxContent
                    | MaxDimension::MinContent
                    | MaxDimension::FitContent(_)
                    | MaxDimension::Stretch => {
                        f32::INFINITY // TODO: implement intrinsic sizing
                    }
                }
            } else {
                f32::INFINITY
            };

        let font_size = styled_node.style.computed_font_size_px;

        if let (Ok(margin_left), Ok(margin_right), Ok(node_width)) = (
            CSSProperty::resolve(&styled_node.style.margin_left),
            CSSProperty::resolve(&styled_node.style.margin_right),
            CSSProperty::resolve(&styled_node.style.width),
        ) {
            let left_margin =
                Self::resolve_margin_value(absolute_ctx, margin_left, width, font_size);
            let right_margin =
                Self::resolve_margin_value(absolute_ctx, margin_right, width, font_size);
            let available_width = f32::min(width - (left_margin + right_margin), max_width);

            match &node_width {
                Dimension::Auto => available_width.max(0.0),
                Dimension::Length(len) => {
                    let rel_ctx = RelativeContext {
                        parent_width: width,
                        font_size,
                        ..Default::default()
                    };

                    len.to_px(&rel_ctx, absolute_ctx).max(0.0)
                }
                Dimension::Percentage(pct) => pct.as_fraction() * width,
                Dimension::Calc(calc) => {
                    let rel_ctx = RelativeContext {
                        parent_width: width,
                        font_size,
                        ..Default::default()
                    };
                    calc.to_px(RelativeType::ParentWidth, &rel_ctx, absolute_ctx)
                        .max(0.0)
                }
                Dimension::MaxContent
                | Dimension::MinContent
                | Dimension::FitContent(_)
                | Dimension::Stretch => {
                    // TODO: implement intrinsic sizing
                    available_width.max(0.0)
                }
            }
        } else {
            max_width
        }
    }

    pub(crate) fn calculate_height(
        absolute_ctx: &AbsoluteContext,
        styled_node: &StyledNode,
        height: f32,
        children_height: f32,
    ) -> f32 {
        if let Ok(node_height) = CSSProperty::resolve(&styled_node.style.height) {
            let rel_ctx = RelativeContext {
                parent_height: height,
                font_size: styled_node.style.computed_font_size_px,
                ..Default::default()
            };

            match &node_height {
                Dimension::Auto => children_height,
                Dimension::Length(len) => len.to_px(&rel_ctx, absolute_ctx),
                Dimension::Percentage(pct) => pct.as_fraction() * height,
                Dimension::Calc(calc) => {
                    calc.to_px(RelativeType::ParentHeight, &rel_ctx, absolute_ctx)
                }
                Dimension::MaxContent
                | Dimension::MinContent
                | Dimension::FitContent(_)
                | Dimension::Stretch => {
                    // TODO: implement intrinsic sizing
                    children_height
                }
            }
        } else {
            children_height
        }
    }
}

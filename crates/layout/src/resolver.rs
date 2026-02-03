use css_style::{
    BorderWidthValue, ComputedStyle, Dimension, MaxDimension, OffsetValue, Property, StyledNode,
};

use crate::SideOffset;

pub struct PropertyResolver;

impl PropertyResolver {
    pub(crate) fn resolve_box_model(
        style: &ComputedStyle,
        containing_width: f32,
        font_size_px: f32,
    ) -> (SideOffset, SideOffset, SideOffset) {
        let margins = Self::resolve_margin(style, containing_width, font_size_px);
        let padding = Self::resolve_padding(style, containing_width, font_size_px);
        let borders = Self::resolve_border(style, font_size_px);

        (margins, padding, borders)
    }

    /// Resolve margin values to pixels
    pub(crate) fn resolve_node_margins(
        styled_node: &StyledNode,
        containing_width: f32,
        font_size_px: f32,
    ) -> SideOffset {
        Self::resolve_margin(&styled_node.style, containing_width, font_size_px)
    }

    /// Resolve a single margin value to pixels
    pub(crate) fn resolve_margin_value(
        value: &OffsetValue,
        containing_width: f32,
        font_size_px: f32,
    ) -> f32 {
        match value {
            OffsetValue::Length(len) => len.to_px(0.0, font_size_px),
            OffsetValue::Percentage(pct) => pct.to_px(containing_width),
            OffsetValue::Auto => 0.0,
        }
    }

    fn resolve_margin(
        style: &ComputedStyle,
        containing_width: f32,
        font_size_px: f32,
    ) -> SideOffset {
        let mut margins = SideOffset::default();
        if let Ok(top) = Property::resolve(&style.margin_top) {
            margins.top = Self::resolve_margin_value(top, containing_width, font_size_px);
        }
        if let Ok(right) = Property::resolve(&style.margin_right) {
            margins.right = Self::resolve_margin_value(right, containing_width, font_size_px);
        }
        if let Ok(bottom) = Property::resolve(&style.margin_bottom) {
            margins.bottom = Self::resolve_margin_value(bottom, containing_width, font_size_px);
        }
        if let Ok(left) = Property::resolve(&style.margin_left) {
            margins.left = Self::resolve_margin_value(left, containing_width, font_size_px);
        }
        margins
    }

    fn resolve_padding(
        style: &ComputedStyle,
        containing_width: f32,
        font_size_px: f32,
    ) -> SideOffset {
        let mut padding = SideOffset::default();
        if let Ok(top) = Property::resolve(&style.padding_top) {
            padding.top = Self::resolve_padding_value(top, containing_width, font_size_px);
        }
        if let Ok(right) = Property::resolve(&style.padding_right) {
            padding.right = Self::resolve_padding_value(right, containing_width, font_size_px);
        }
        if let Ok(bottom) = Property::resolve(&style.padding_bottom) {
            padding.bottom = Self::resolve_padding_value(bottom, containing_width, font_size_px);
        }
        if let Ok(left) = Property::resolve(&style.padding_left) {
            padding.left = Self::resolve_padding_value(left, containing_width, font_size_px);
        }
        padding
    }

    fn resolve_border(style: &ComputedStyle, font_size_px: f32) -> SideOffset {
        let mut borders = SideOffset::default();
        if let Ok(top) = Property::resolve(&style.border_top_width) {
            borders.top = Self::resolve_border_value(top, font_size_px);
        }
        if let Ok(right) = Property::resolve(&style.border_right_width) {
            borders.right = Self::resolve_border_value(right, font_size_px);
        }
        if let Ok(bottom) = Property::resolve(&style.border_bottom_width) {
            borders.bottom = Self::resolve_border_value(bottom, font_size_px);
        }
        if let Ok(left) = Property::resolve(&style.border_left_width) {
            borders.left = Self::resolve_border_value(left, font_size_px);
        }
        borders
    }

    pub(crate) fn resolve_padding_value(
        value: &OffsetValue,
        containing_width: f32,
        font_size_px: f32,
    ) -> f32 {
        match value {
            OffsetValue::Length(len) => len.to_px(0.0, font_size_px),
            OffsetValue::Percentage(pct) => pct.to_px(containing_width),
            OffsetValue::Auto => 0.0,
        }
    }

    pub(crate) fn resolve_border_value(value: &BorderWidthValue, font_size_px: f32) -> f32 {
        match value {
            BorderWidthValue::Length(len) => len.to_px(0.0, font_size_px),
            BorderWidthValue::Thin => 1.0,
            BorderWidthValue::Medium => 3.0,
            BorderWidthValue::Thick => 5.0,
        }
    }

    /// Calculate content width (top-down from containing block)
    pub(crate) fn calculate_width(styled_node: &StyledNode, width: f32) -> f32 {
        let font_size = styled_node.style.computed_font_size_px;

        let max_width = if let Ok(max_width_prop) = Property::resolve(&styled_node.style.max_width)
        {
            match &max_width_prop {
                MaxDimension::None => f32::INFINITY,
                MaxDimension::Length(len) => len.to_px(width, font_size),
                MaxDimension::Percentage(pct) => pct.to_px(width),
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
            Property::resolve(&styled_node.style.margin_left),
            Property::resolve(&styled_node.style.margin_right),
            Property::resolve(&styled_node.style.width),
        ) {
            let left_margin = Self::resolve_margin_value(margin_left, width, font_size);
            let right_margin = Self::resolve_margin_value(margin_right, width, font_size);
            let available_width = f32::min(width - (left_margin + right_margin), max_width);

            match &node_width {
                Dimension::Auto => available_width.max(0.0),
                Dimension::Length(len) => len.to_px(available_width, font_size),
                Dimension::Percentage(pct) => pct.to_px(width).max(0.0),
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
        styled_node: &StyledNode,
        height: f32,
        children_height: f32,
    ) -> f32 {
        if let Ok(node_height) = Property::resolve(&styled_node.style.height) {
            match &node_height {
                Dimension::Auto => children_height,
                Dimension::Length(len) => len.value(),
                Dimension::Percentage(pct) => pct.to_px(height),
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

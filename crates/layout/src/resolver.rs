use css_style::{
    StyledNode,
    types::{
        height::Height,
        margin::{Margin, MarginValue},
        padding::{Padding, PaddingValue},
        width::Width,
    },
};

use crate::SideOffset;

pub struct PropertyResolver;

impl PropertyResolver {
    /// Resolve margin values to pixels
    pub(crate) fn resolve_node_margins(
        styled_node: &StyledNode,
        containing_width: f32,
        font_size_px: f32,
    ) -> SideOffset {
        let margin = &styled_node.style.margin;
        SideOffset {
            top: Self::resolve_margin_value(&margin.top, containing_width, font_size_px),
            right: Self::resolve_margin_value(&margin.right, containing_width, font_size_px),
            bottom: Self::resolve_margin_value(&margin.bottom, containing_width, font_size_px),
            left: Self::resolve_margin_value(&margin.left, containing_width, font_size_px),
        }
    }

    pub(crate) fn resolve_margins(
        margin: &Margin,
        containing_width: f32,
        font_size_px: f32,
    ) -> SideOffset {
        SideOffset {
            top: Self::resolve_margin_value(&margin.top, containing_width, font_size_px),
            right: Self::resolve_margin_value(&margin.right, containing_width, font_size_px),
            bottom: Self::resolve_margin_value(&margin.bottom, containing_width, font_size_px),
            left: Self::resolve_margin_value(&margin.left, containing_width, font_size_px),
        }
    }

    /// Resolve a single margin value to pixels
    pub(crate) fn resolve_margin_value(
        value: &MarginValue,
        containing_width: f32,
        font_size_px: f32,
    ) -> f32 {
        match value {
            MarginValue::Length(len) => len.to_px(font_size_px),
            MarginValue::Percentage(pct) => pct * containing_width / 100.0,
            MarginValue::Auto => 0.0,
            MarginValue::Global(_) => 0.0,
        }
    }

    /// Resolve padding values to pixels
    pub(crate) fn resolve_node_padding(
        styled_node: &StyledNode,
        containing_width: f32,
        font_size_px: f32,
    ) -> SideOffset {
        let padding = &styled_node.style.padding;
        SideOffset {
            top: Self::resolve_padding_value(&padding.top, containing_width, font_size_px),
            right: Self::resolve_padding_value(&padding.right, containing_width, font_size_px),
            bottom: Self::resolve_padding_value(&padding.bottom, containing_width, font_size_px),
            left: Self::resolve_padding_value(&padding.left, containing_width, font_size_px),
        }
    }

    pub(crate) fn resolve_padding(
        padding: &Padding,
        containing_width: f32,
        font_size_px: f32,
    ) -> SideOffset {
        SideOffset {
            top: Self::resolve_padding_value(&padding.top, containing_width, font_size_px),
            right: Self::resolve_padding_value(&padding.right, containing_width, font_size_px),
            bottom: Self::resolve_padding_value(&padding.bottom, containing_width, font_size_px),
            left: Self::resolve_padding_value(&padding.left, containing_width, font_size_px),
        }
    }

    pub(crate) fn resolve_padding_value(
        value: &PaddingValue,
        containing_width: f32,
        font_size_px: f32,
    ) -> f32 {
        match value {
            PaddingValue::Length(len) => len.to_px(font_size_px),
            PaddingValue::Percentage(pct) => pct * containing_width / 100.0,
            PaddingValue::Auto => 0.0,
            PaddingValue::Global(_) => 0.0,
        }
    }

    /// Calculate content width (top-down from containing block)
    pub(crate) fn calculate_width(
        styled_node: &StyledNode,
        width: f32,
        margin: &SideOffset,
        padding: &SideOffset,
    ) -> f32 {
        let available_width = width - margin.horizontal() - padding.horizontal();

        match &styled_node.style.width {
            Width::Auto => available_width.max(0.0),
            Width::Length(len) => len.to_px(available_width),
            Width::Percentage(pct) => (pct * width / 100.0).max(0.0),
            Width::Global(_) => available_width.max(0.0),
            Width::MaxContent | Width::MinContent | Width::FitContent(_) | Width::Stretch => {
                // TODO: implement intrinsic sizing
                available_width.max(0.0)
            }
        }
    }

    pub(crate) fn calculate_height(
        styled_node: &StyledNode,
        height: f32,
        children_height: f32,
    ) -> f32 {
        match &styled_node.style.height {
            Height::Auto => children_height,
            Height::Length(len) => len.value,
            Height::Percentage(pct) => pct * height / 100.0,
            Height::Global(_) => children_height,
            Height::MaxContent | Height::MinContent | Height::FitContent(_) | Height::Stretch => {
                // TODO: implement intrinsic sizing
                children_height
            }
        }
    }
}

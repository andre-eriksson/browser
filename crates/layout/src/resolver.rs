use css_style::{
    BorderWidthValue, Dimension, MaxDimension, Offset, OffsetValue, Property, StyledNode,
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
        if let Ok(margin) = Property::resolve(&styled_node.style.margin) {
            SideOffset {
                top: Self::resolve_margin_value(&margin.top, containing_width, font_size_px),
                right: Self::resolve_margin_value(&margin.right, containing_width, font_size_px),
                bottom: Self::resolve_margin_value(&margin.bottom, containing_width, font_size_px),
                left: Self::resolve_margin_value(&margin.left, containing_width, font_size_px),
            }
        } else {
            SideOffset::default()
        }
    }

    pub(crate) fn resolve_margins(
        margin: &Property<Offset>,
        containing_width: f32,
        font_size_px: f32,
    ) -> SideOffset {
        if let Ok(margin) = Property::resolve(margin) {
            SideOffset {
                top: Self::resolve_margin_value(&margin.top, containing_width, font_size_px),
                right: Self::resolve_margin_value(&margin.right, containing_width, font_size_px),
                bottom: Self::resolve_margin_value(&margin.bottom, containing_width, font_size_px),
                left: Self::resolve_margin_value(&margin.left, containing_width, font_size_px),
            }
        } else {
            SideOffset::default()
        }
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

    /// Resolve padding values to pixels
    pub(crate) fn resolve_node_padding(
        styled_node: &StyledNode,
        containing_width: f32,
        font_size_px: f32,
    ) -> SideOffset {
        if let Ok(padding) = Property::resolve(&styled_node.style.padding) {
            SideOffset {
                top: Self::resolve_padding_value(&padding.top, containing_width, font_size_px),
                right: Self::resolve_padding_value(&padding.right, containing_width, font_size_px),
                bottom: Self::resolve_padding_value(
                    &padding.bottom,
                    containing_width,
                    font_size_px,
                ),
                left: Self::resolve_padding_value(&padding.left, containing_width, font_size_px),
            }
        } else {
            SideOffset::default()
        }
    }

    pub(crate) fn resolve_padding(
        padding: &Property<Offset>,
        containing_width: f32,
        font_size_px: f32,
    ) -> SideOffset {
        if let Ok(padding) = Property::resolve(padding) {
            SideOffset {
                top: Self::resolve_padding_value(&padding.top, containing_width, font_size_px),
                right: Self::resolve_padding_value(&padding.right, containing_width, font_size_px),
                bottom: Self::resolve_padding_value(
                    &padding.bottom,
                    containing_width,
                    font_size_px,
                ),
                left: Self::resolve_padding_value(&padding.left, containing_width, font_size_px),
            }
        } else {
            SideOffset::default()
        }
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

    pub(crate) fn resolve_node_borders(styled_node: &StyledNode, font_size_px: f32) -> SideOffset {
        if let Ok(border) = Property::resolve(&styled_node.style.border_width) {
            SideOffset {
                top: Self::resolve_border_value(&border.top(), font_size_px),
                right: Self::resolve_border_value(&border.right(), font_size_px),
                bottom: Self::resolve_border_value(&border.bottom(), font_size_px),
                left: Self::resolve_border_value(&border.left(), font_size_px),
            }
        } else {
            SideOffset::default()
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

        if let (Ok(margin), Ok(node_width)) = (
            Property::resolve(&styled_node.style.margin),
            Property::resolve(&styled_node.style.width),
        ) {
            let left_margin = Self::resolve_margin_value(&margin.left, width, font_size);
            let right_margin = Self::resolve_margin_value(&margin.right, width, font_size);
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

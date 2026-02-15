use css_style::{ComputedDimension, ComputedMaxDimension, ComputedStyle, StyledNode};

use crate::SideOffset;

pub struct PropertyResolver;

impl PropertyResolver {
    pub(crate) fn resolve_box_model(style: &ComputedStyle) -> (SideOffset, SideOffset, SideOffset) {
        let margins = Self::resolve_margin(style);
        let padding = Self::resolve_padding(style);
        let borders = Self::resolve_border(style);

        (margins, padding, borders)
    }

    /// Resolve margin values to pixels
    pub fn resolve_margin(style: &ComputedStyle) -> SideOffset {
        SideOffset {
            top: style.margin_top,
            right: style.margin_right,
            bottom: style.margin_bottom,
            left: style.margin_left,
        }
    }

    pub(crate) fn resolve_padding(style: &ComputedStyle) -> SideOffset {
        SideOffset {
            top: style.padding_top,
            right: style.padding_right,
            bottom: style.padding_bottom,
            left: style.padding_left,
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

    /// Calculate content width (top-down from containing block)
    pub(crate) fn calculate_width(styled_node: &StyledNode, width: f32) -> f32 {
        let max_width = match &styled_node.style.max_width {
            ComputedMaxDimension::None => f32::INFINITY,
            ComputedMaxDimension::Fixed => styled_node.style.max_intrinsic_width,
            ComputedMaxDimension::MaxContent
            | ComputedMaxDimension::MinContent
            | ComputedMaxDimension::FitContent(_)
            | ComputedMaxDimension::Stretch => styled_node.style.max_intrinsic_width,
        };

        let available_width = f32::min(
            width - (styled_node.style.margin_left + styled_node.style.margin_right),
            max_width,
        );

        match &styled_node.style.width {
            ComputedDimension::Auto => available_width.max(0.0),
            ComputedDimension::Fixed => styled_node.style.intrinsic_width,
            ComputedDimension::MaxContent
            | ComputedDimension::MinContent
            | ComputedDimension::FitContent(_)
            | ComputedDimension::Stretch => styled_node.style.intrinsic_width,
        }
    }

    pub(crate) fn calculate_height(styled_node: &StyledNode, children_height: f32) -> f32 {
        match &styled_node.style.height {
            ComputedDimension::Auto => children_height.max(styled_node.style.intrinsic_height),
            ComputedDimension::Fixed => styled_node.style.intrinsic_height,
            ComputedDimension::MaxContent
            | ComputedDimension::MinContent
            | ComputedDimension::FitContent(_)
            | ComputedDimension::Stretch => children_height.max(styled_node.style.intrinsic_height),
        }
    }
}

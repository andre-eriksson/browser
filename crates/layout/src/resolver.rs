use css_style::{ComputedDimension, ComputedMaxDimension, ComputedStyle, Position, StyledNode};
use css_values::display::{Float, InsideDisplay};

use crate::SideOffset;

pub struct PropertyResolver;

impl PropertyResolver {
    pub(crate) const fn resolve_box_model(style: &ComputedStyle) -> (SideOffset, SideOffset, SideOffset) {
        let margins = Self::resolve_margin(style);
        let padding = Self::resolve_padding(style);
        let borders = Self::resolve_border(style);

        (margins, padding, borders)
    }

    #[allow(dead_code, reason = "TODO: Support all positions")]
    pub const fn establishes_bfc(style: &ComputedStyle) -> bool {
        !matches!(style.float, Float::None)
            || !matches!(style.position, Position::Static | Position::Relative)
            || matches!(style.display.inside(), Some(InsideDisplay::FlowRoot))
        //TODO: || style.overflow != Overflow::Visible
    }

    pub fn has_top_fence(style: &ComputedStyle) -> bool {
        style.padding_top > 0.0 || style.border_top_width > 0.0
    }

    pub fn has_bottom_fence(style: &ComputedStyle) -> bool {
        style.padding_bottom > 0.0 || style.border_bottom_width > 0.0
    }

    /// Resolve margin values to pixels
    pub const fn resolve_margin(style: &ComputedStyle) -> SideOffset {
        SideOffset {
            top: style.margin_top,
            right: style.margin_right,
            bottom: style.margin_bottom,
            left: style.margin_left,
        }
    }

    pub(crate) const fn resolve_padding(style: &ComputedStyle) -> SideOffset {
        SideOffset {
            top: style.padding_top,
            right: style.padding_right,
            bottom: style.padding_bottom,
            left: style.padding_left,
        }
    }

    const fn resolve_border(style: &ComputedStyle) -> SideOffset {
        SideOffset {
            top: style.border_top_width,
            right: style.border_right_width,
            bottom: style.border_bottom_width,
            left: style.border_left_width,
        }
    }

    /// Calculate content width (top-down from containing block)
    pub(crate) fn calculate_width(styled_node: &StyledNode, width: f64) -> f64 {
        let max_width = match &styled_node.style.max_width {
            ComputedMaxDimension::None => f64::INFINITY,
            ComputedMaxDimension::Fixed => styled_node.style.max_intrinsic_width,
            ComputedMaxDimension::Percentage(f) => (width * f).max(0.0),
            ComputedMaxDimension::MaxContent
            | ComputedMaxDimension::MinContent
            | ComputedMaxDimension::FitContent(_)
            | ComputedMaxDimension::Stretch => styled_node.style.max_intrinsic_width,
        };

        let available_width = f64::min(
            width - (styled_node.style.margin_left + styled_node.style.margin_right),
            if max_width == 0.0 && styled_node.style.width == ComputedDimension::Auto {
                f64::INFINITY
            } else {
                max_width
            },
        );

        let width = match &styled_node.style.width {
            ComputedDimension::Auto => available_width.max(0.0),
            ComputedDimension::Fixed => styled_node.style.intrinsic_width,
            ComputedDimension::Percentage(f) => (width * f).max(0.0),
            ComputedDimension::MaxContent
            | ComputedDimension::MinContent
            | ComputedDimension::FitContent(_)
            | ComputedDimension::Stretch => styled_node.style.intrinsic_width,
        };

        width.min(available_width)
    }

    pub(crate) fn calculate_height(styled_node: &StyledNode, children_height: f64, containing_height: f64) -> f64 {
        let height = match &styled_node.style.height {
            ComputedDimension::Auto => children_height.max(styled_node.style.intrinsic_height),
            ComputedDimension::Fixed => styled_node.style.intrinsic_height,
            ComputedDimension::Percentage(f) => (containing_height * f).max(0.0),
            ComputedDimension::MaxContent
            | ComputedDimension::MinContent
            | ComputedDimension::FitContent(_)
            | ComputedDimension::Stretch => children_height.max(styled_node.style.intrinsic_height),
        };

        if styled_node.style.max_height == ComputedMaxDimension::None {
            height
        } else {
            let max_height = match &styled_node.style.max_height {
                ComputedMaxDimension::Fixed => styled_node.style.max_intrinsic_height,
                ComputedMaxDimension::Percentage(f) => (containing_height * f).max(0.0),
                _ => f64::INFINITY,
            };

            let available_height = f64::min(
                containing_height - (styled_node.style.margin_top + styled_node.style.margin_bottom),
                if max_height == 0.0 && styled_node.style.height == ComputedDimension::Auto {
                    f64::INFINITY
                } else {
                    max_height
                },
            );

            height.min(available_height)
        }
    }
}

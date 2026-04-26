use css_style::{ComputedMaxSize, ComputedSize, ComputedStyle, StyledNode};

use crate::{
    Margin,
    primitives::{MarginValue, SideOffset},
};

pub struct PropertyResolver;

impl PropertyResolver {
    pub(crate) fn resolve_box_model(style: &ComputedStyle, containing_width: f64) -> (Margin, SideOffset, SideOffset) {
        let margins = Self::resolve_margin(style, containing_width);
        let padding = Self::resolve_padding(style, containing_width);
        let borders = Self::resolve_border(style);

        (margins, padding, borders)
    }

    // TODO: Support all positions
    // pub const fn establishes_bfc(style: &ComputedStyle) -> bool {
    //     !matches!(style.float, Float::None)
    //         || !matches!(style.position, Position::Static | Position::Relative)
    //         || matches!(style.display.inside(), Some(InsideDisplay::FlowRoot))
    //     //TODO: || style.overflow != Overflow::Visible
    // }

    pub fn has_top_fence(style: &ComputedStyle, containing_width: f64) -> bool {
        style.padding_top.to_px(containing_width) > 0.0 || style.border_top_width > 0.0
    }

    pub fn has_bottom_fence(style: &ComputedStyle, containing_width: f64) -> bool {
        style.padding_bottom.to_px(containing_width) > 0.0 || style.border_bottom_width > 0.0
    }

    /// Resolve margin values to pixels
    pub fn resolve_margin(style: &ComputedStyle, containing_width: f64) -> Margin {
        let top = MarginValue::resolve(style.margin_top, containing_width);
        let right = MarginValue::resolve(style.margin_right, containing_width);
        let bottom = MarginValue::resolve(style.margin_bottom, containing_width);
        let left = MarginValue::resolve(style.margin_left, containing_width);

        Margin {
            top,
            right,
            bottom,
            left,
        }
    }

    pub fn resolve_padding(style: &ComputedStyle, containing_width: f64) -> SideOffset {
        let top = style.padding_top.to_px(containing_width);
        let right = style.padding_right.to_px(containing_width);
        let bottom = style.padding_bottom.to_px(containing_width);
        let left = style.padding_left.to_px(containing_width);

        SideOffset {
            top,
            right,
            bottom,
            left,
        }
    }

    pub fn resolve_border(style: &ComputedStyle) -> SideOffset {
        SideOffset {
            top: style.border_top_width,
            right: style.border_right_width,
            bottom: style.border_bottom_width,
            left: style.border_left_width,
        }
    }

    /// Calculate content width (top-down from containing block)
    pub(crate) fn calculate_width(styled_node: &StyledNode, containing_width: f64) -> f64 {
        let max_width = match &styled_node.style.max_width {
            ComputedMaxSize::None => f64::INFINITY,
            ComputedMaxSize::Px(px) => *px,
            ComputedMaxSize::Percentage(f) => (containing_width * f).max(0.0),
            ComputedMaxSize::MaxContent
            | ComputedMaxSize::MinContent
            | ComputedMaxSize::FitContent
            | ComputedMaxSize::Stretch => containing_width, // TODO: Fix
        };

        let left = MarginValue::resolve(styled_node.style.margin_left, containing_width);
        let right = MarginValue::resolve(styled_node.style.margin_right, containing_width);

        let available_width = f64::min(
            match (left, right) {
                (MarginValue::Auto, MarginValue::Auto) => containing_width,
                (MarginValue::Auto, MarginValue::Px(px)) => containing_width - px,
                (MarginValue::Px(px), MarginValue::Auto) => containing_width - px,
                (MarginValue::Px(left_px), MarginValue::Px(right_px)) => containing_width - left_px - right_px,
            },
            if max_width == 0.0 && styled_node.style.width == ComputedSize::Auto {
                f64::INFINITY
            } else {
                max_width
            },
        );

        let width = match &styled_node.style.width {
            ComputedSize::Auto => available_width.max(0.0),
            ComputedSize::Px(px) => *px,
            ComputedSize::Percentage(f) => (containing_width * f).max(0.0),
            ComputedSize::MaxContent | ComputedSize::MinContent | ComputedSize::FitContent | ComputedSize::Stretch => {
                available_width // TODO: Fix
            }
        };

        width.min(available_width)
    }

    pub(crate) fn calculate_height(styled_node: &StyledNode, children_height: f64, containing_height: f64) -> f64 {
        let height = match &styled_node.style.height {
            ComputedSize::Auto => children_height,
            ComputedSize::Px(px) => *px,
            ComputedSize::Percentage(f) => (containing_height * f).max(0.0),
            ComputedSize::MaxContent | ComputedSize::MinContent | ComputedSize::FitContent | ComputedSize::Stretch => {
                children_height // TODO: Fix
            }
        };

        if styled_node.style.max_height == ComputedMaxSize::None {
            height
        } else {
            let max_height = match &styled_node.style.max_height {
                ComputedMaxSize::Px(px) => *px,
                ComputedMaxSize::Percentage(f) => (containing_height * f).max(0.0),
                _ => f64::INFINITY,
            };

            let top = MarginValue::resolve(styled_node.style.margin_top, containing_height);
            let bottom = MarginValue::resolve(styled_node.style.margin_bottom, containing_height);

            let available_height = f64::min(
                match (top, bottom) {
                    (MarginValue::Auto, MarginValue::Auto) => containing_height,
                    (MarginValue::Auto, MarginValue::Px(px)) => containing_height - px,
                    (MarginValue::Px(px), MarginValue::Auto) => containing_height - px,
                    (MarginValue::Px(top_px), MarginValue::Px(bottom_px)) => containing_height - top_px - bottom_px,
                },
                if max_height == 0.0 && styled_node.style.height == ComputedSize::Auto {
                    f64::INFINITY
                } else {
                    max_height
                },
            );

            height.min(available_height)
        }
    }
}

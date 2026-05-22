use css_display::BoxNode;
use css_style::Display;
use css_values::display::{InsideDisplay, OutsideDisplay};

pub mod block;
//pub mod inline;

/// Layout mode determines how children are positioned
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum LayoutMode {
    #[default]
    Block,
    Inline,
    Flex, // TODO: implement
    Grid, // TODO: implement
}

impl LayoutMode {
    pub fn new(box_node: &BoxNode) -> Self {
        let style = &*box_node.style;

        debug_assert!(!style.display.is_none(), "Should've been pruned by the BoxTree.");

        if style.position.is_out_of_flow() {
            return Self::Block;
        }

        if let Display::Normal { outside, inside } = style.display {
            if let Some(val) = inside {
                match val {
                    InsideDisplay::Flex => return Self::Flex,
                    InsideDisplay::Grid => return Self::Grid,
                    _ => {}
                }
            }

            if let Some(val) = outside {
                match val {
                    OutsideDisplay::Block => return Self::Block,
                    OutsideDisplay::Inline => return Self::Inline,
                }
            }
        }

        Self::Block
    }
}

#[cfg(test)]
mod tests {
    use css_style::ComputedStyle;
    use html_dom::NodeId;

    use super::*;

    #[test]
    fn test_layout_mode_block() {
        let style = ComputedStyle {
            display: Display::from(OutsideDisplay::Block),
            ..Default::default()
        };

        let box_node = BoxNode::new(&NodeId(0), &style, vec![]);

        assert_eq!(LayoutMode::new(&box_node), LayoutMode::Block);
    }

    #[test]
    fn test_layout_mode_inline() {
        let style = ComputedStyle {
            display: Display::from(OutsideDisplay::Inline),
            ..Default::default()
        };

        let box_node = BoxNode::new(&NodeId(0), &style, vec![]);

        assert_eq!(LayoutMode::new(&box_node), LayoutMode::Inline);
    }

    #[test]
    fn test_layout_mode_flex() {
        let style = ComputedStyle {
            display: Display::from(InsideDisplay::Flex),
            ..Default::default()
        };

        let box_node = BoxNode::new(&NodeId(0), &style, vec![]);

        assert_eq!(LayoutMode::new(&box_node), LayoutMode::Flex);
    }

    #[test]
    fn test_layout_mode_grid() {
        let style = ComputedStyle {
            display: Display::from(InsideDisplay::Grid),
            ..Default::default()
        };

        let box_node = BoxNode::new(&NodeId(0), &style, vec![]);

        assert_eq!(LayoutMode::new(&box_node), LayoutMode::Grid);
    }
}

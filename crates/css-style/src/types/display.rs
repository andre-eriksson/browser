//! <https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/display>

use crate::types::{Parseable, global::Global};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum OutsideDisplay {
    Block,
    Inline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum InsideDisplay {
    Flow,
    FlowRoot,
    Table,
    Flex,
    Grid,
    Ruby,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum InternalDisplay {
    TableRowGroup,
    TableHeaderGroup,
    TableFooterGroup,
    TableRow,
    TableCell,
    TableColumnGroup,
    TableColumn,
    TableCaption,
    RubyBase,
    RubyText,
    RubyBaseContainer,
    RubyTextContainer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum BoxDisplay {
    Contents,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Display {
    pub outside: Option<OutsideDisplay>,
    pub inside: Option<InsideDisplay>,
    pub internal: Option<InternalDisplay>,
    pub box_display: Option<BoxDisplay>,
    pub global: Option<Global>,
}

impl Parseable for Display {
    fn parse(value: &str) -> Option<Self> {
        let parts = value.split_whitespace().collect::<Vec<&str>>();

        if parts.len() == 2 {
            let outside = match parts[0] {
                "block" => Some(OutsideDisplay::Block),
                "inline" => Some(OutsideDisplay::Inline),
                _ => None,
            };

            let inside = match parts[1] {
                "flow" => Some(InsideDisplay::Flow),
                "flow-root" => Some(InsideDisplay::FlowRoot),
                "table" => Some(InsideDisplay::Table),
                "flex" => Some(InsideDisplay::Flex),
                "grid" => Some(InsideDisplay::Grid),
                "ruby" => Some(InsideDisplay::Ruby),
                _ => None,
            };

            if outside.is_some() && inside.is_some() {
                return Some(Display {
                    outside,
                    inside,
                    internal: None,
                    box_display: None,
                    global: None,
                });
            }
        } else if parts.len() == 1 {
            if let Some(global_value) = Global::parse(parts[0]) {
                return Some(Display {
                    outside: None,
                    inside: None,
                    internal: None,
                    box_display: None,
                    global: Some(global_value),
                });
            }

            match parts[0] {
                "inline" => {
                    return Some(Display {
                        outside: Some(OutsideDisplay::Inline),
                        inside: Some(InsideDisplay::Flow),
                        internal: None,
                        box_display: None,
                        global: None,
                    });
                }
                "inline-block" => {
                    return Some(Display {
                        outside: Some(OutsideDisplay::Inline),
                        inside: Some(InsideDisplay::FlowRoot),
                        internal: None,
                        box_display: None,
                        global: None,
                    });
                }
                "inline-table" => {
                    return Some(Display {
                        outside: Some(OutsideDisplay::Inline),
                        inside: Some(InsideDisplay::Table),
                        internal: None,
                        box_display: None,
                        global: None,
                    });
                }
                "inline-flex" => {
                    return Some(Display {
                        outside: Some(OutsideDisplay::Inline),
                        inside: Some(InsideDisplay::Flex),
                        internal: None,
                        box_display: None,
                        global: None,
                    });
                }
                "inline-grid" => {
                    return Some(Display {
                        outside: Some(OutsideDisplay::Inline),
                        inside: Some(InsideDisplay::Grid),
                        internal: None,
                        box_display: None,
                        global: None,
                    });
                }
                "block" => {
                    return Some(Display {
                        outside: Some(OutsideDisplay::Block),
                        inside: Some(InsideDisplay::Flow),
                        internal: None,
                        box_display: None,
                        global: None,
                    });
                }
                "flow" => {
                    return Some(Display {
                        outside: Some(OutsideDisplay::Block),
                        inside: Some(InsideDisplay::Flow),
                        internal: None,
                        box_display: None,
                        global: None,
                    });
                }
                "flow-root" => {
                    return Some(Display {
                        outside: Some(OutsideDisplay::Block),
                        inside: Some(InsideDisplay::FlowRoot),
                        internal: None,
                        box_display: None,
                        global: None,
                    });
                }
                "table" => {
                    return Some(Display {
                        outside: Some(OutsideDisplay::Block),
                        inside: Some(InsideDisplay::Table),
                        internal: None,
                        box_display: None,
                        global: None,
                    });
                }
                "flex" => {
                    return Some(Display {
                        outside: Some(OutsideDisplay::Block),
                        inside: Some(InsideDisplay::Flex),
                        internal: None,
                        box_display: None,
                        global: None,
                    });
                }
                "grid" => {
                    return Some(Display {
                        outside: Some(OutsideDisplay::Block),
                        inside: Some(InsideDisplay::Grid),
                        internal: None,
                        box_display: None,
                        global: None,
                    });
                }
                "ruby" => {
                    return Some(Display {
                        outside: Some(OutsideDisplay::Inline),
                        inside: Some(InsideDisplay::Ruby),
                        internal: None,
                        box_display: None,
                        global: None,
                    });
                }
                "contents" => {
                    return Some(Display {
                        outside: None,
                        inside: None,
                        internal: None,
                        box_display: Some(BoxDisplay::Contents),
                        global: None,
                    });
                }
                "none" => {
                    return Some(Display {
                        outside: None,
                        inside: None,
                        internal: None,
                        box_display: Some(BoxDisplay::None),
                        global: None,
                    });
                }
                _ => {}
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_parse() {
        let display = Display::parse("inline flex").unwrap();
        assert_eq!(display.outside, Some(OutsideDisplay::Inline));
        assert_eq!(display.inside, Some(InsideDisplay::Flex));
        assert_eq!(display.box_display, None);

        let display = Display::parse("block").unwrap();
        assert_eq!(display.outside, Some(OutsideDisplay::Block));
        assert_eq!(display.inside, Some(InsideDisplay::Flow));
        assert_eq!(display.box_display, None);

        let display = Display::parse("none").unwrap();
        assert_eq!(display.box_display, Some(BoxDisplay::None));
        assert_eq!(display.outside, None);
        assert_eq!(display.inside, None);
    }
}

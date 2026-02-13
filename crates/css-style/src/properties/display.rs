use std::str::FromStr;

use css_cssom::{ComponentValue, CssTokenKind};

use crate::{
    display::ListItemDisplay,
    primitives::display::{BoxDisplay, InsideDisplay, InternalDisplay, OutsideDisplay},
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Display {
    outside: Option<OutsideDisplay>,
    inside: Option<InsideDisplay>,
    list_item: Option<ListItemDisplay>,
    internal: Option<InternalDisplay>,
    box_display: Option<BoxDisplay>,
}

impl Display {
    pub fn new(
        outside: Option<OutsideDisplay>,
        inside: Option<InsideDisplay>,
        list_item: Option<ListItemDisplay>,
        internal: Option<InternalDisplay>,
        box_display: Option<BoxDisplay>,
    ) -> Self {
        Self {
            outside,
            inside,
            list_item,
            internal,
            box_display,
        }
    }

    pub fn outside(&self) -> Option<OutsideDisplay> {
        self.outside
    }

    pub fn inside(&self) -> Option<InsideDisplay> {
        self.inside
    }

    pub fn list_item(&self) -> Option<ListItemDisplay> {
        self.list_item
    }

    pub fn internal(&self) -> Option<InternalDisplay> {
        self.internal
    }

    pub fn box_display(&self) -> Option<BoxDisplay> {
        self.box_display
    }
}

impl From<OutsideDisplay> for Display {
    fn from(outside: OutsideDisplay) -> Self {
        Display {
            outside: Some(outside),
            inside: None,
            list_item: None,
            internal: None,
            box_display: None,
        }
    }
}

impl From<InsideDisplay> for Display {
    fn from(inside: InsideDisplay) -> Self {
        Display {
            outside: None,
            inside: Some(inside),
            list_item: None,
            internal: None,
            box_display: None,
        }
    }
}

impl From<ListItemDisplay> for Display {
    fn from(list_item: ListItemDisplay) -> Self {
        Display {
            outside: None,
            inside: Some(InsideDisplay::Flow),
            list_item: Some(list_item),
            internal: None,
            box_display: None,
        }
    }
}

impl From<InternalDisplay> for Display {
    fn from(internal: InternalDisplay) -> Self {
        Display {
            outside: None,
            inside: None,
            list_item: None,
            internal: Some(internal),
            box_display: None,
        }
    }
}

impl From<BoxDisplay> for Display {
    fn from(box_display: BoxDisplay) -> Self {
        Display {
            outside: None,
            inside: None,
            list_item: None,
            internal: None,
            box_display: Some(box_display),
        }
    }
}

impl TryFrom<&[ComponentValue]> for Display {
    type Error = String;

    fn try_from(value: &[ComponentValue]) -> Result<Self, Self::Error> {
        let parts: Vec<&str> = value
            .iter()
            .filter_map(|cv| match cv {
                ComponentValue::Token(token) => Some(token),
                _ => None,
            })
            .filter(|token| !matches!(token.kind, CssTokenKind::Whitespace))
            .map(|token| match &token.kind {
                CssTokenKind::Ident(ident) => Ok(ident.as_str()),
                other => Err(format!("Unexpected token in display value: {:?}", other)),
            })
            .collect::<Result<_, _>>()?;

        match parts.as_slice() {
            ["inline"] => Ok(Display {
                outside: Some(OutsideDisplay::Inline),
                inside: Some(InsideDisplay::Flow),
                ..Default::default()
            }),
            ["inline-block"] => Ok(Display {
                outside: Some(OutsideDisplay::Inline),
                inside: Some(InsideDisplay::FlowRoot),
                ..Default::default()
            }),
            ["inline-table"] => Ok(Display {
                outside: Some(OutsideDisplay::Inline),
                inside: Some(InsideDisplay::Table),
                ..Default::default()
            }),
            ["inline-flex"] => Ok(Display {
                outside: Some(OutsideDisplay::Inline),
                inside: Some(InsideDisplay::Flex),
                ..Default::default()
            }),
            ["inline-grid"] => Ok(Display {
                outside: Some(OutsideDisplay::Inline),
                inside: Some(InsideDisplay::Grid),
                ..Default::default()
            }),
            ["block"] => Ok(Display {
                outside: Some(OutsideDisplay::Block),
                inside: Some(InsideDisplay::Flow),
                ..Default::default()
            }),
            ["flow"] => Ok(Display::from(InsideDisplay::Flow)),
            ["flow-root"] => Ok(Display::from(InsideDisplay::FlowRoot)),
            ["table"] => Ok(Display::from(InsideDisplay::Table)),
            ["flex"] => Ok(Display::from(InsideDisplay::Flex)),
            ["grid"] => Ok(Display::from(InsideDisplay::Grid)),
            ["ruby"] => Ok(Display::from(InsideDisplay::Ruby)),
            ["list-item"] => Ok(Display::from(ListItemDisplay::ListItem)),
            ["table-row-group"] => Ok(Display::from(InternalDisplay::TableRowGroup)),
            ["table-header-group"] => Ok(Display::from(InternalDisplay::TableHeaderGroup)),
            ["table-footer-group"] => Ok(Display::from(InternalDisplay::TableFooterGroup)),
            ["table-row"] => Ok(Display::from(InternalDisplay::TableRow)),
            ["table-cell"] => Ok(Display::from(InternalDisplay::TableCell)),
            ["table-column-group"] => Ok(Display::from(InternalDisplay::TableColumnGroup)),
            ["table-column"] => Ok(Display::from(InternalDisplay::TableColumn)),
            ["table-caption"] => Ok(Display::from(InternalDisplay::TableCaption)),
            ["ruby-base"] => Ok(Display::from(InternalDisplay::RubyBase)),
            ["ruby-text"] => Ok(Display::from(InternalDisplay::RubyText)),
            ["ruby-base-container"] => Ok(Display::from(InternalDisplay::RubyBaseContainer)),
            ["ruby-text-container"] => Ok(Display::from(InternalDisplay::RubyTextContainer)),
            ["contents"] => Ok(Display::from(BoxDisplay::Contents)),
            ["none"] => Ok(Display::from(BoxDisplay::None)),
            [outside, list_item_or_inside] => {
                let outside = outside.parse()?;

                if let Ok(list_item) = list_item_or_inside.parse::<ListItemDisplay>() {
                    return Ok(Display {
                        outside: Some(outside),
                        inside: Some(InsideDisplay::Flow),
                        list_item: Some(list_item),
                        ..Default::default()
                    });
                }

                let inside = list_item_or_inside.parse()?;

                Ok(Display {
                    outside: Some(outside),
                    inside: Some(inside),
                    ..Default::default()
                })
            }
            [outside, inside, list_item] => {
                let outside = outside.parse()?;
                let inside = inside.parse()?;
                let list_item = list_item.parse()?;

                Ok(Display {
                    outside: Some(outside),
                    inside: Some(inside),
                    list_item: Some(list_item),
                    ..Default::default()
                })
            }
            _ => Err(format!("Invalid display value: {:?}", parts)),
        }
    }
}

impl FromStr for Display {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s.split_whitespace().collect::<Vec<&str>>();

        match parts.len() {
            1 => match parts[0] {
                "inline" => Ok(Display {
                    outside: Some(OutsideDisplay::Inline),
                    inside: Some(InsideDisplay::Flow),
                    ..Default::default()
                }),
                "inline-block" => Ok(Display {
                    outside: Some(OutsideDisplay::Inline),
                    inside: Some(InsideDisplay::FlowRoot),
                    ..Default::default()
                }),
                "inline-table" => Ok(Display {
                    outside: Some(OutsideDisplay::Inline),
                    inside: Some(InsideDisplay::Table),
                    ..Default::default()
                }),
                "inline-flex" => Ok(Display {
                    outside: Some(OutsideDisplay::Inline),
                    inside: Some(InsideDisplay::Flex),
                    ..Default::default()
                }),
                "inline-grid" => Ok(Display {
                    outside: Some(OutsideDisplay::Inline),
                    inside: Some(InsideDisplay::Grid),
                    ..Default::default()
                }),
                "block" => Ok(Display {
                    outside: Some(OutsideDisplay::Block),
                    inside: Some(InsideDisplay::Flow),
                    ..Default::default()
                }),
                "flow" => Ok(Display::from(InsideDisplay::Flow)),
                "flow-root" => Ok(Display::from(InsideDisplay::FlowRoot)),
                "table" => Ok(Display::from(InsideDisplay::Table)),
                "flex" => Ok(Display::from(InsideDisplay::Flex)),
                "grid" => Ok(Display::from(InsideDisplay::Grid)),
                "ruby" => Ok(Display::from(InsideDisplay::Ruby)),
                "list-item" => Ok(Display::from(ListItemDisplay::ListItem)),
                "table-row-group" => Ok(Display::from(InternalDisplay::TableRowGroup)),
                "table-header-group" => Ok(Display::from(InternalDisplay::TableHeaderGroup)),
                "table-footer-group" => Ok(Display::from(InternalDisplay::TableFooterGroup)),
                "table-row" => Ok(Display::from(InternalDisplay::TableRow)),
                "table-cell" => Ok(Display::from(InternalDisplay::TableCell)),
                "table-column-group" => Ok(Display::from(InternalDisplay::TableColumnGroup)),
                "table-column" => Ok(Display::from(InternalDisplay::TableColumn)),
                "table-caption" => Ok(Display::from(InternalDisplay::TableCaption)),
                "ruby-base" => Ok(Display::from(InternalDisplay::RubyBase)),
                "ruby-text" => Ok(Display::from(InternalDisplay::RubyText)),
                "ruby-base-container" => Ok(Display::from(InternalDisplay::RubyBaseContainer)),
                "ruby-text-container" => Ok(Display::from(InternalDisplay::RubyTextContainer)),
                "contents" => Ok(Display::from(BoxDisplay::Contents)),
                "none" => Ok(Display::from(BoxDisplay::None)),
                _ => Err(format!("Invalid display value: {}", s)),
            },
            2 => {
                let outside = parts[0].parse()?;

                if let Ok(list_item) = parts[1].parse::<ListItemDisplay>() {
                    return Ok(Display {
                        outside: Some(outside),
                        inside: Some(InsideDisplay::Flow),
                        list_item: Some(list_item),
                        ..Default::default()
                    });
                }

                let inside = parts[1].parse()?;

                Ok(Display {
                    outside: Some(outside),
                    inside: Some(inside),
                    ..Default::default()
                })
            }
            3 => {
                let outside = parts[0].parse()?;
                let inside = parts[1].parse()?;
                let list_item = parts[2].parse()?;

                Ok(Display {
                    outside: Some(outside),
                    inside: Some(inside),
                    list_item: Some(list_item),
                    ..Default::default()
                })
            }
            _ => Err(format!("Invalid display value: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_parse() {
        let display = "inline flex".parse::<Display>().unwrap();
        assert_eq!(display.outside, Some(OutsideDisplay::Inline));
        assert_eq!(display.inside, Some(InsideDisplay::Flex));
        assert_eq!(display.box_display, None);

        let display = "block".parse::<Display>().unwrap();
        assert_eq!(display.outside, Some(OutsideDisplay::Block));
        assert_eq!(display.inside, Some(InsideDisplay::Flow));
        assert_eq!(display.box_display, None);

        let display = "none".parse::<Display>().unwrap();
        assert_eq!(display.box_display, Some(BoxDisplay::None));
        assert_eq!(display.outside, None);
        assert_eq!(display.inside, None);
    }
}

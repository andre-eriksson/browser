//! This module defines the `Display` struct, which represents the computed value of the CSS `display` property.
//! The `Display` struct is designed to capture the various components of the `display` property, including outside,
//! inside, list-item, internal, and box display types. This structured representation allows for easier handling of
//! the `display` property in the layout engine.

use css_cssom::{ComponentValue, CssTokenKind};

use crate::{
    display::ListItemDisplay,
    primitives::display::{BoxDisplay, InsideDisplay, InternalDisplay, OutsideDisplay},
};

/// Represents the computed value of the CSS `display` property, which can be a combination of outside, inside, list-item, internal, and box display types.
/// This struct allows for a more structured representation of the `display` property, making it easier to work with in the layout engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Display {
    outside: Option<OutsideDisplay>,
    inside: Option<InsideDisplay>,
    list_item: Option<ListItemDisplay>,
    internal: Option<InternalDisplay>,
    box_display: Option<BoxDisplay>,
}

impl Display {
    pub(crate) fn new(
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

    /// Returns the outside display type, if set.
    pub fn outside(&self) -> Option<OutsideDisplay> {
        self.outside
    }

    /// Returns the inside display type, if set.
    pub fn inside(&self) -> Option<InsideDisplay> {
        self.inside
    }

    /// Returns the list-item display type, if set.
    pub fn list_item(&self) -> Option<ListItemDisplay> {
        self.list_item
    }

    /// Returns the internal display type, if set.
    pub fn internal(&self) -> Option<InternalDisplay> {
        self.internal
    }

    /// Returns the box display type, if set.
    pub fn box_display(&self) -> Option<BoxDisplay> {
        self.box_display
    }
}

impl Default for Display {
    /// The CSS initial value of `display` is `inline` (i.e., `outside: Inline, inside: Flow`).
    fn default() -> Self {
        Display {
            outside: Some(OutsideDisplay::Inline),
            inside: Some(InsideDisplay::Flow),
            list_item: None,
            internal: None,
            box_display: None,
        }
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

#[cfg(test)]
mod tests {
    use css_cssom::CssToken;

    use super::*;

    #[test]
    fn test_display_parse() {
        let input = vec![
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("inline".into()),
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Whitespace,
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("flow-root".into()),
                position: None,
            }),
        ];

        let display = Display::try_from(input.as_slice()).expect("Failed to parse display value");
        assert_eq!(
            display,
            Display {
                outside: Some(OutsideDisplay::Inline),
                inside: Some(InsideDisplay::FlowRoot),
                list_item: None,
                internal: None,
                box_display: None,
            }
        );
    }
}

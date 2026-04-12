//! This module defines the `Display` struct, which represents the computed value of the CSS `display` property.
//! The `Display` struct is designed to capture the various components of the `display` property, including outside,
//! inside, list-item, internal, and box display types. This structured representation allows for easier handling of
//! the `display` property in the layout engine.

use css_cssom::{ComponentValue, ComponentValueStream, CssTokenKind};
use css_values::{
    CSSParsable,
    display::{BoxDisplay, Float, InsideDisplay, InternalDisplay, ListItemDisplay, OutsideDisplay},
    error::CssValueError,
};

/// Represents the computed value of the CSS `display` property, which can be a combination of outside, inside, list-item, internal, and box display types.
///
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
    /// Returns the outside display type, if set.
    pub const fn outside(&self) -> Option<OutsideDisplay> {
        self.outside
    }

    /// Returns the inside display type, if set.
    pub const fn inside(&self) -> Option<InsideDisplay> {
        self.inside
    }

    /// Returns the list-item display type, if set.
    pub const fn list_item(&self) -> Option<ListItemDisplay> {
        self.list_item
    }

    /// Returns the internal display type, if set.
    pub const fn internal(&self) -> Option<InternalDisplay> {
        self.internal
    }

    /// Returns the box display type, if set.
    pub const fn box_display(&self) -> Option<BoxDisplay> {
        self.box_display
    }

    pub fn adjust_float(self, float: Float) -> Self {
        if matches!(float, Float::None) {
            self
        } else if matches!(
            self.internal,
            Some(
                InternalDisplay::TableRowGroup
                    | InternalDisplay::TableHeaderGroup
                    | InternalDisplay::TableFooterGroup
                    | InternalDisplay::TableCell
                    | InternalDisplay::TableColumnGroup
                    | InternalDisplay::TableColumn
                    | InternalDisplay::TableCaption
            )
        ) {
            Self::from(InsideDisplay::Table)
        } else if matches!(self.outside, Some(OutsideDisplay::Inline)) {
            match self.inside {
                Some(InsideDisplay::FlowRoot | InsideDisplay::Flow) => Self {
                    outside: Some(OutsideDisplay::Block),
                    inside: Some(InsideDisplay::Flow),
                    ..Default::default()
                },
                Some(InsideDisplay::Table) => Self {
                    inside: Some(InsideDisplay::Table),
                    ..Default::default()
                },
                Some(InsideDisplay::Flex) => Self {
                    inside: Some(InsideDisplay::Flex),
                    ..Default::default()
                },
                Some(InsideDisplay::Grid) => Self {
                    inside: Some(InsideDisplay::Grid),
                    ..Default::default()
                },
                _ => self,
            }
        } else {
            self
        }
    }
}

impl Default for Display {
    /// The CSS initial value of `display` is `inline` (i.e., `outside: Inline, inside: Flow`).
    fn default() -> Self {
        Self {
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
        Self {
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
        Self {
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
        Self {
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
        Self {
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
        Self {
            outside: None,
            inside: None,
            list_item: None,
            internal: None,
            box_display: Some(box_display),
        }
    }
}

impl CSSParsable for Display {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, CssValueError> {
        let mut parts: Vec<String> = Vec::with_capacity(3);

        while let Some(cv) = stream.next_cv() {
            match cv {
                ComponentValue::Token(token) => match &token.kind {
                    CssTokenKind::Ident(ident) => parts.push(ident.to_ascii_lowercase()),
                    CssTokenKind::Whitespace => continue,
                    _ => return Err(CssValueError::InvalidToken(token.kind.clone())),
                },
                _ => return Err(CssValueError::InvalidComponentValue(cv.clone())),
            }
        }

        if parts.is_empty() || parts.len() > 3 {
            return Err(CssValueError::InvalidValue(format!(
                "Invalid number of components for display property: {}",
                parts.len()
            )));
        }

        let parts: Vec<&str> = parts.iter().map(|s| s.as_str()).collect();
        match parts.as_slice() {
            ["inline"] => Ok(Self {
                outside: Some(OutsideDisplay::Inline),
                inside: Some(InsideDisplay::Flow),
                ..Default::default()
            }),
            ["inline-block"] => Ok(Self {
                outside: Some(OutsideDisplay::Inline),
                inside: Some(InsideDisplay::FlowRoot),
                ..Default::default()
            }),
            ["inline-table"] => Ok(Self {
                outside: Some(OutsideDisplay::Inline),
                inside: Some(InsideDisplay::Table),
                ..Default::default()
            }),
            ["inline-flex"] => Ok(Self {
                outside: Some(OutsideDisplay::Inline),
                inside: Some(InsideDisplay::Flex),
                ..Default::default()
            }),
            ["inline-grid"] => Ok(Self {
                outside: Some(OutsideDisplay::Inline),
                inside: Some(InsideDisplay::Grid),
                ..Default::default()
            }),
            ["block"] => Ok(Self {
                outside: Some(OutsideDisplay::Block),
                inside: Some(InsideDisplay::Flow),
                ..Default::default()
            }),
            ["flow"] => Ok(Self::from(InsideDisplay::Flow)),
            ["flow-root"] => Ok(Self::from(InsideDisplay::FlowRoot)),
            ["table"] => Ok(Self::from(InsideDisplay::Table)),
            ["flex"] => Ok(Self::from(InsideDisplay::Flex)),
            ["grid"] => Ok(Self::from(InsideDisplay::Grid)),
            ["ruby"] => Ok(Self::from(InsideDisplay::Ruby)),
            ["list-item"] => Ok(Self::from(ListItemDisplay::ListItem)),
            ["table-row-group"] => Ok(Self::from(InternalDisplay::TableRowGroup)),
            ["table-header-group"] => Ok(Self::from(InternalDisplay::TableHeaderGroup)),
            ["table-footer-group"] => Ok(Self::from(InternalDisplay::TableFooterGroup)),
            ["table-row"] => Ok(Self::from(InternalDisplay::TableRow)),
            ["table-cell"] => Ok(Self::from(InternalDisplay::TableCell)),
            ["table-column-group"] => Ok(Self::from(InternalDisplay::TableColumnGroup)),
            ["table-column"] => Ok(Self::from(InternalDisplay::TableColumn)),
            ["table-caption"] => Ok(Self::from(InternalDisplay::TableCaption)),
            ["ruby-base"] => Ok(Self::from(InternalDisplay::RubyBase)),
            ["ruby-text"] => Ok(Self::from(InternalDisplay::RubyText)),
            ["ruby-base-container"] => Ok(Self::from(InternalDisplay::RubyBaseContainer)),
            ["ruby-text-container"] => Ok(Self::from(InternalDisplay::RubyTextContainer)),
            ["contents"] => Ok(Self::from(BoxDisplay::Contents)),
            ["none"] => Ok(Self::from(BoxDisplay::None)),
            [outside, list_item_or_inside] => {
                let outside = outside
                    .parse()
                    .map_err(|_| CssValueError::InvalidValue(format!("Invalid outside display value: {}", outside)))?;

                if let Ok(list_item) = list_item_or_inside.parse::<ListItemDisplay>() {
                    return Ok(Self {
                        outside: Some(outside),
                        inside: Some(InsideDisplay::Flow),
                        list_item: Some(list_item),
                        ..Default::default()
                    });
                }

                let inside = list_item_or_inside.parse().map_err(|_| {
                    CssValueError::InvalidValue(format!("Invalid inside display value: {}", list_item_or_inside))
                })?;

                Ok(Self {
                    outside: Some(outside),
                    inside: Some(inside),
                    ..Default::default()
                })
            }
            [outside, inside, list_item] => {
                let outside = outside
                    .parse()
                    .map_err(|_| CssValueError::InvalidValue(format!("Invalid outside display value: {}", outside)))?;
                let inside = inside
                    .parse()
                    .map_err(|_| CssValueError::InvalidValue(format!("Invalid inside display value: {}", inside)))?;
                let list_item = list_item.parse().map_err(|_| {
                    CssValueError::InvalidValue(format!("Invalid list-item display value: {}", list_item))
                })?;

                Ok(Self {
                    outside: Some(outside),
                    inside: Some(inside),
                    list_item: Some(list_item),
                    ..Default::default()
                })
            }
            _ => Err(CssValueError::InvalidValue(format!("Invalid combination of display values: {:?}", parts))),
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

        let display = Display::parse(&mut input.as_slice().into()).expect("Failed to parse display value");
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

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
pub enum Display {
    /// [ <display-outside> || <display-inside> ]
    Normal {
        outside: Option<OutsideDisplay>,
        inside: Option<InsideDisplay>,
    },

    /// <display-listitem> =
    ///   <display-outside>?     &&
    ///   [ flow | flow-root ]?  &&
    ///   list-item
    ListItem {
        outside: Option<OutsideDisplay>,
        flow_root: bool,
    },

    /// <display-internal>
    Internal(InternalDisplay),

    /// <display-box>
    Box(BoxDisplay),
}

impl Display {
    /// Checks if the display value is `display: none`.
    pub const fn is_none(&self) -> bool {
        matches!(self, Self::Box(BoxDisplay::None))
    }

    #[must_use]
    pub fn adjust_float(self, float: Float) -> Self {
        if matches!(float, Float::None) {
            self
        } else if let Self::Internal(internal) = self
            && matches!(
                internal,
                InternalDisplay::TableRowGroup
                    | InternalDisplay::TableHeaderGroup
                    | InternalDisplay::TableFooterGroup
                    | InternalDisplay::TableCell
                    | InternalDisplay::TableColumnGroup
                    | InternalDisplay::TableColumn
                    | InternalDisplay::TableCaption
            )
        {
            Self::from(InsideDisplay::Table)
        } else if let Self::Normal { outside, inside } = self
            && matches!(outside, Some(OutsideDisplay::Inline))
        {
            match inside {
                Some(InsideDisplay::FlowRoot | InsideDisplay::Flow) => Self::Normal {
                    outside: Some(OutsideDisplay::Block),
                    inside: Some(InsideDisplay::Flow),
                },
                Some(InsideDisplay::Table) => Self::Normal {
                    outside: Some(OutsideDisplay::Block),
                    inside: Some(InsideDisplay::Table),
                },
                Some(InsideDisplay::Flex) => Self::Normal {
                    outside: Some(OutsideDisplay::Block),
                    inside: Some(InsideDisplay::Flex),
                },
                Some(InsideDisplay::Grid) => Self::Normal {
                    outside: Some(OutsideDisplay::Block),
                    inside: Some(InsideDisplay::Grid),
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
        Self::Normal {
            outside: Some(OutsideDisplay::Inline),
            inside: Some(InsideDisplay::Flow),
        }
    }
}

impl From<OutsideDisplay> for Display {
    fn from(outside: OutsideDisplay) -> Self {
        Self::Normal {
            outside: Some(outside),
            inside: Some(InsideDisplay::Flow),
        }
    }
}

impl From<InsideDisplay> for Display {
    fn from(inside: InsideDisplay) -> Self {
        Self::Normal {
            outside: None,
            inside: Some(inside),
        }
    }
}

impl From<InternalDisplay> for Display {
    fn from(internal: InternalDisplay) -> Self {
        Self::Internal(internal)
    }
}

impl From<BoxDisplay> for Display {
    fn from(box_display: BoxDisplay) -> Self {
        Self::Box(box_display)
    }
}

impl CSSParsable for Display {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, CssValueError> {
        let mut parts: Vec<String> = Vec::with_capacity(3);

        while let Some(cv) = stream.next_cv() {
            match cv {
                ComponentValue::Token(token) => match &token.kind {
                    CssTokenKind::Ident(ident) => parts.push(ident.to_ascii_lowercase()),
                    CssTokenKind::Whitespace => {}
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

        let parts: Vec<&str> = parts.iter().map(std::string::String::as_str).collect();
        match parts.as_slice() {
            ["inline"] => Ok(Self::Normal {
                outside: Some(OutsideDisplay::Inline),
                inside: Some(InsideDisplay::Flow),
            }),
            ["inline-block"] => Ok(Self::Normal {
                outside: Some(OutsideDisplay::Inline),
                inside: Some(InsideDisplay::FlowRoot),
            }),
            ["inline-table"] => Ok(Self::Normal {
                outside: Some(OutsideDisplay::Inline),
                inside: Some(InsideDisplay::Table),
            }),
            ["inline-flex"] => Ok(Self::Normal {
                outside: Some(OutsideDisplay::Inline),
                inside: Some(InsideDisplay::Flex),
            }),
            ["inline-grid"] => Ok(Self::Normal {
                outside: Some(OutsideDisplay::Inline),
                inside: Some(InsideDisplay::Grid),
            }),
            ["block"] => Ok(Self::Normal {
                outside: Some(OutsideDisplay::Block),
                inside: Some(InsideDisplay::Flow),
            }),
            ["flow"] => Ok(Self::from(InsideDisplay::Flow)),
            ["flow-root"] => Ok(Self::from(InsideDisplay::FlowRoot)),
            ["table"] => Ok(Self::from(InsideDisplay::Table)),
            ["flex"] => Ok(Self::from(InsideDisplay::Flex)),
            ["grid"] => Ok(Self::from(InsideDisplay::Grid)),
            ["ruby"] => Ok(Self::from(InsideDisplay::Ruby)),
            ["list-item"] => Ok(Self::ListItem {
                outside: None,
                flow_root: false,
            }),
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
            [outside_or_flow, list_item_or_inside] => {
                if let Ok(outside) = outside_or_flow.parse() {
                    if list_item_or_inside.parse::<ListItemDisplay>().is_ok() {
                        return Ok(Self::ListItem {
                            outside: Some(outside),
                            flow_root: false,
                        });
                    }

                    let inside = list_item_or_inside.parse().map_err(|_| {
                        CssValueError::InvalidValue(format!("Invalid inside display value: {list_item_or_inside}"))
                    })?;

                    Ok(Self::Normal {
                        outside: Some(outside),
                        inside: Some(inside),
                    })
                } else if outside_or_flow.eq_ignore_ascii_case("flow")
                    || outside_or_flow.eq_ignore_ascii_case("flow-root")
                {
                    if list_item_or_inside.parse::<ListItemDisplay>().is_err() {
                        return Err(CssValueError::InvalidValue(format!(
                            "Invalid list-item display value: {list_item_or_inside}"
                        )));
                    }

                    let is_flow_root = outside_or_flow.eq_ignore_ascii_case("flow-root");

                    Ok(Self::ListItem {
                        outside: None,
                        flow_root: is_flow_root,
                    })
                } else {
                    Err(CssValueError::InvalidValue(format!(
                        "Invalid display value: {outside_or_flow} {list_item_or_inside}"
                    )))
                }
            }
            [outside, flow, list_item] => {
                let outside = outside
                    .parse()
                    .map_err(|_| CssValueError::InvalidValue(format!("Invalid outside display value: {outside}")))?;

                let is_flow_root = flow.eq_ignore_ascii_case("flow-root");
                if !flow.eq_ignore_ascii_case("flow") && !is_flow_root {
                    return Err(CssValueError::InvalidValue(format!("Invalid flow display value: {flow}")));
                }

                if list_item.parse::<ListItemDisplay>().is_err() {
                    return Err(CssValueError::InvalidValue(format!("Invalid list-item display value: {list_item}")));
                }

                Ok(Self::ListItem {
                    outside: Some(outside),
                    flow_root: is_flow_root,
                })
            }
            _ => Err(CssValueError::InvalidValue(format!("Invalid combination of display values: {parts:?}"))),
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
            Display::Normal {
                outside: Some(OutsideDisplay::Inline),
                inside: Some(InsideDisplay::FlowRoot),
            }
        );
    }
}

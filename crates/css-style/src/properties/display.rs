use std::str::FromStr;

use crate::primitives::display::{BoxDisplay, InsideDisplay, InternalDisplay, OutsideDisplay};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Display {
    outside: Option<OutsideDisplay>,
    inside: Option<InsideDisplay>,
    internal: Option<InternalDisplay>,
    box_display: Option<BoxDisplay>,
}

impl Display {
    pub fn new(
        outside: Option<OutsideDisplay>,
        inside: Option<InsideDisplay>,
        internal: Option<InternalDisplay>,
        box_display: Option<BoxDisplay>,
    ) -> Self {
        Self {
            outside,
            inside,
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
            internal: None,
            box_display: Some(box_display),
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
                    internal: None,
                    box_display: None,
                }),
                "inline-block" => Ok(Display {
                    outside: Some(OutsideDisplay::Inline),
                    inside: Some(InsideDisplay::FlowRoot),
                    internal: None,
                    box_display: None,
                }),
                "inline-table" => Ok(Display {
                    outside: Some(OutsideDisplay::Inline),
                    inside: Some(InsideDisplay::Table),
                    internal: None,
                    box_display: None,
                }),
                "inline-flex" => Ok(Display {
                    outside: Some(OutsideDisplay::Inline),
                    inside: Some(InsideDisplay::Flex),
                    internal: None,
                    box_display: None,
                }),
                "inline-grid" => Ok(Display {
                    outside: Some(OutsideDisplay::Inline),
                    inside: Some(InsideDisplay::Grid),
                    internal: None,
                    box_display: None,
                }),
                "block" => Ok(Display {
                    outside: Some(OutsideDisplay::Block),
                    inside: Some(InsideDisplay::Flow),
                    internal: None,
                    box_display: None,
                }),
                "flow" => Ok(Display {
                    outside: None,
                    inside: Some(InsideDisplay::Flow),
                    internal: None,
                    box_display: None,
                }),
                "flow-root" => Ok(Display {
                    outside: None,
                    inside: Some(InsideDisplay::FlowRoot),
                    internal: None,
                    box_display: None,
                }),
                "table" => Ok(Display {
                    outside: None,
                    inside: Some(InsideDisplay::Table),
                    internal: None,
                    box_display: None,
                }),
                "flex" => Ok(Display {
                    outside: None,
                    inside: Some(InsideDisplay::Flex),
                    internal: None,
                    box_display: None,
                }),
                "grid" => Ok(Display {
                    outside: None,
                    inside: Some(InsideDisplay::Grid),
                    internal: None,
                    box_display: None,
                }),
                "ruby" => Ok(Display {
                    outside: None,
                    inside: Some(InsideDisplay::Ruby),
                    internal: None,
                    box_display: None,
                }),
                "contents" => Ok(Display {
                    outside: None,
                    inside: None,
                    internal: None,
                    box_display: Some(BoxDisplay::Contents),
                }),
                "none" => Ok(Display {
                    outside: None,
                    inside: None,
                    internal: None,
                    box_display: Some(BoxDisplay::None),
                }),
                _ => Err(format!("Invalid display value: {}", s)),
            },
            2 => {
                let outside = parts[0]
                    .parse::<OutsideDisplay>()
                    .map_err(|_| format!("Invalid outside display value: {}", parts[0]))?;
                let inside = parts[1]
                    .parse::<InsideDisplay>()
                    .map_err(|_| format!("Invalid inside display value: {}", parts[1]))?;

                Ok(Display {
                    outside: Some(outside),
                    inside: Some(inside),
                    internal: None,
                    box_display: None,
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

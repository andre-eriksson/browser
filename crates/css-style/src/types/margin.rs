use crate::{
    types::{
        Parseable,
        global::Global,
        length::{Length, LengthUnit},
    },
    unit::Unit,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MarginValue {
    Percentage(f32),
    Length(Length),
    Global(Global),
    Auto,
}

impl MarginValue {
    pub fn px(value: f32) -> Self {
        Self::Length(Length::px(value))
    }

    pub fn to_px(&self, reference: f32) -> Option<f32> {
        match self {
            MarginValue::Length(length) => Some(length.to_px(reference, 0.0)),
            MarginValue::Percentage(percent) => Some(reference * (percent / 100.0)),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Margin {
    pub top: MarginValue,
    pub right: MarginValue,
    pub bottom: MarginValue,
    pub left: MarginValue,
}

impl Margin {
    pub fn new(
        top: MarginValue,
        right: MarginValue,
        bottom: MarginValue,
        left: MarginValue,
    ) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    pub fn zero() -> Self {
        Self::all(MarginValue::Length(Length {
            value: 0.0,
            unit: LengthUnit::Px,
        }))
    }

    pub fn block(value: MarginValue) -> Self {
        Self {
            top: value,
            right: MarginValue::Length(Length {
                value: 0.0,
                unit: LengthUnit::Px,
            }),
            bottom: value,
            left: MarginValue::Length(Length {
                value: 0.0,
                unit: LengthUnit::Px,
            }),
        }
    }

    pub fn block_two(top: MarginValue, bottom: MarginValue) -> Self {
        Self {
            top,
            right: MarginValue::Length(Length {
                value: 0.0,
                unit: LengthUnit::Px,
            }),
            bottom,
            left: MarginValue::Length(Length {
                value: 0.0,
                unit: LengthUnit::Px,
            }),
        }
    }

    /// Set all margins to the same value
    pub fn all(value: MarginValue) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    /// Set vertical and horizontal margins
    pub fn two(vertical: MarginValue, horizontal: MarginValue) -> Self {
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }

    /// Set top, horizontal, and bottom margins
    pub fn three(top: MarginValue, horizontal: MarginValue, bottom: MarginValue) -> Self {
        Self {
            top,
            right: horizontal,
            bottom,
            left: horizontal,
        }
    }
}

impl Parseable for MarginValue {
    fn parse(value: &str) -> Option<Self> {
        if value.eq_ignore_ascii_case("auto") {
            return Some(Self::Auto);
        }

        if let Some(global) = Global::parse(value) {
            return Some(Self::Global(global));
        }

        if let Some(length) = Length::parse(value) {
            return Some(Self::Length(length));
        }

        if let Some(percentage) = Unit::resolve_percentage(value) {
            return Some(Self::Percentage(percentage));
        }

        None
    }
}

impl Parseable for Margin {
    fn parse(value: &str) -> Option<Self> {
        let parts = value.split_whitespace().collect::<Vec<&str>>();

        match parts.len() {
            1 => {
                let value = MarginValue::parse(parts[0])?;
                Some(Self::all(value))
            }
            2 => {
                let vertical = MarginValue::parse(parts[0])?;
                let horizontal = MarginValue::parse(parts[1])?;
                Some(Self::two(vertical, horizontal))
            }
            3 => {
                let top = MarginValue::parse(parts[0])?;
                let horizontal = MarginValue::parse(parts[1])?;
                let bottom = MarginValue::parse(parts[2])?;
                Some(Self::three(top, horizontal, bottom))
            }
            4 => {
                let top = MarginValue::parse(parts[0])?;
                let right = MarginValue::parse(parts[1])?;
                let bottom = MarginValue::parse(parts[2])?;
                let left = MarginValue::parse(parts[3])?;
                Some(Self::new(top, right, bottom, left))
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_margin() {
        assert_eq!(
            Margin::parse("10px"),
            Some(Margin::all(MarginValue::Length(Length {
                value: 10.0,
                unit: LengthUnit::Px,
            })))
        );

        assert_eq!(
            Margin::parse("10px 20px"),
            Some(Margin::two(
                MarginValue::Length(Length {
                    value: 10.0,
                    unit: LengthUnit::Px,
                }),
                MarginValue::Length(Length {
                    value: 20.0,
                    unit: LengthUnit::Px,
                })
            ))
        );

        assert_eq!(
            Margin::parse("10px 20px 30px"),
            Some(Margin::three(
                MarginValue::Length(Length {
                    value: 10.0,
                    unit: LengthUnit::Px,
                }),
                MarginValue::Length(Length {
                    value: 20.0,
                    unit: LengthUnit::Px,
                }),
                MarginValue::Length(Length {
                    value: 30.0,
                    unit: LengthUnit::Px,
                })
            ))
        );

        assert_eq!(
            Margin::parse("10px 20px 30px 40px"),
            Some(Margin::new(
                MarginValue::Length(Length {
                    value: 10.0,
                    unit: LengthUnit::Px,
                }),
                MarginValue::Length(Length {
                    value: 20.0,
                    unit: LengthUnit::Px,
                }),
                MarginValue::Length(Length {
                    value: 30.0,
                    unit: LengthUnit::Px,
                }),
                MarginValue::Length(Length {
                    value: 40.0,
                    unit: LengthUnit::Px,
                })
            ))
        );

        assert_eq!(Margin::parse("auto"), Some(Margin::all(MarginValue::Auto)))
    }
}

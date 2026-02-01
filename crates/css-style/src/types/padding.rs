use crate::{
    types::{
        Parseable,
        global::Global,
        length::{Length, LengthUnit},
    },
    unit::Unit,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PaddingValue {
    Percentage(f32),
    Length(Length),
    Global(Global),
    Auto,
}

impl PaddingValue {
    pub fn px(value: f32) -> Self {
        Self::Length(Length {
            value,
            unit: LengthUnit::Px,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Padding {
    pub top: PaddingValue,
    pub right: PaddingValue,
    pub bottom: PaddingValue,
    pub left: PaddingValue,
}

impl Padding {
    pub fn new(
        top: PaddingValue,
        right: PaddingValue,
        bottom: PaddingValue,
        left: PaddingValue,
    ) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    pub fn zero() -> Self {
        Self::all(PaddingValue::Length(Length {
            value: 0.0,
            unit: LengthUnit::Px,
        }))
    }

    /// Set all paddings to the same value
    pub fn all(value: PaddingValue) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    /// Set vertical and horizontal paddings
    pub fn two(vertical: PaddingValue, horizontal: PaddingValue) -> Self {
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }

    /// Set top, horizontal, and bottom paddings
    pub fn three(top: PaddingValue, horizontal: PaddingValue, bottom: PaddingValue) -> Self {
        Self {
            top,
            right: horizontal,
            bottom,
            left: horizontal,
        }
    }
}

impl Parseable for PaddingValue {
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

impl Parseable for Padding {
    fn parse(value: &str) -> Option<Self> {
        let parts = value.split_whitespace().collect::<Vec<&str>>();

        match parts.len() {
            1 => {
                let value = PaddingValue::parse(parts[0])?;
                Some(Self::all(value))
            }
            2 => {
                let vertical = PaddingValue::parse(parts[0])?;
                let horizontal = PaddingValue::parse(parts[1])?;
                Some(Self::two(vertical, horizontal))
            }
            3 => {
                let top = PaddingValue::parse(parts[0])?;
                let horizontal = PaddingValue::parse(parts[1])?;
                let bottom = PaddingValue::parse(parts[2])?;
                Some(Self::three(top, horizontal, bottom))
            }
            4 => {
                let top = PaddingValue::parse(parts[0])?;
                let right = PaddingValue::parse(parts[1])?;
                let bottom = PaddingValue::parse(parts[2])?;
                let left = PaddingValue::parse(parts[3])?;
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
    fn test_parse_padding() {
        assert_eq!(
            Padding::parse("10px"),
            Some(Padding::all(PaddingValue::Length(Length {
                value: 10.0,
                unit: LengthUnit::Px,
            })))
        );

        assert_eq!(
            Padding::parse("10px 20px"),
            Some(Padding::two(
                PaddingValue::Length(Length {
                    value: 10.0,
                    unit: LengthUnit::Px,
                }),
                PaddingValue::Length(Length {
                    value: 20.0,
                    unit: LengthUnit::Px,
                })
            ))
        );

        assert_eq!(
            Padding::parse("10px 20px 30px"),
            Some(Padding::three(
                PaddingValue::Length(Length {
                    value: 10.0,
                    unit: LengthUnit::Px,
                }),
                PaddingValue::Length(Length {
                    value: 20.0,
                    unit: LengthUnit::Px,
                }),
                PaddingValue::Length(Length {
                    value: 30.0,
                    unit: LengthUnit::Px,
                })
            ))
        );

        assert_eq!(
            Padding::parse("10px 20px 30px 40px"),
            Some(Padding::new(
                PaddingValue::Length(Length {
                    value: 10.0,
                    unit: LengthUnit::Px,
                }),
                PaddingValue::Length(Length {
                    value: 20.0,
                    unit: LengthUnit::Px,
                }),
                PaddingValue::Length(Length {
                    value: 30.0,
                    unit: LengthUnit::Px,
                }),
                PaddingValue::Length(Length {
                    value: 40.0,
                    unit: LengthUnit::Px,
                })
            ))
        );

        assert_eq!(
            Padding::parse("auto"),
            Some(Padding::all(PaddingValue::Auto))
        );
    }
}

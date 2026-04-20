use css_cssom::{ComponentValue, ComponentValueStream, CssToken, CssTokenKind};

use crate::{
    CSSParsable,
    error::CssValueError,
    numeric::Percentage,
    quantity::{Angle, Frequency, Length, LengthUnit, Time},
};

/// Represents the <length-percentage> type
///
/// Can be either a Length or a Percentage value. This is used for CSS properties
/// that accept both length and percentage values, such as width, height, margin,
/// padding, etc.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LengthPercentage {
    Length(Length),
    Percentage(Percentage),
}

impl From<Length> for LengthPercentage {
    fn from(length: Length) -> Self {
        Self::Length(length)
    }
}

impl From<Percentage> for LengthPercentage {
    fn from(percentage: Percentage) -> Self {
        Self::Percentage(percentage)
    }
}

impl CSSParsable for LengthPercentage {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, CssValueError> {
        if let Some(cv) = stream.next_non_whitespace() {
            match cv {
                ComponentValue::Token(token) => match &token.kind {
                    CssTokenKind::Dimension { value, unit } => {
                        let unit = LengthUnit::try_from(unit.as_str())
                            .map_err(|_| CssValueError::InvalidUnit(unit.clone()))?;

                        Ok(Self::Length(Length::new(value.to_f64(), unit)))
                    }
                    CssTokenKind::Percentage(numeric) => Ok(Self::Percentage(Percentage::new(numeric.to_f64()))),
                    CssTokenKind::Number(numeric) => {
                        Ok(Self::Percentage(Percentage::from_fraction(numeric.to_f64() / 100.0)))
                    }
                    kind => Err(CssValueError::InvalidToken(kind.clone())),
                },
                cvs => Err(CssValueError::InvalidComponentValue(cvs.clone())),
            }
        } else {
            Err(CssValueError::UnexpectedEndOfInput)
        }
    }
}

/// Represents the <frequency-percentage> type
///
/// Can be either a Frequency or a Percentage value. This is used
/// for CSS properties that accept both frequency and percentage values,
/// such as audio properties.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FrequencyPercentage {
    Frequency(Frequency),
    Percentage(Percentage),
}

impl From<Frequency> for FrequencyPercentage {
    fn from(frequency: Frequency) -> Self {
        Self::Frequency(frequency)
    }
}

impl From<Percentage> for FrequencyPercentage {
    fn from(percentage: Percentage) -> Self {
        Self::Percentage(percentage)
    }
}

/// Represents the <angle-percentage> type
///
/// Can be either an Angle or a Percentage value. This is used for CSS properties
/// that accept both angle and percentage values, such as hue in HSL colors or
/// rotation in transforms.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AnglePercentage {
    Angle(Angle),
    Percentage(Percentage),
}

impl From<Angle> for AnglePercentage {
    fn from(angle: Angle) -> Self {
        Self::Angle(angle)
    }
}

impl From<Percentage> for AnglePercentage {
    fn from(percentage: Percentage) -> Self {
        Self::Percentage(percentage)
    }
}

/// Represents the <time-percentage> type
///
/// Can be either a Time or a Percentage value. This is used for CSS properties
/// that accept both time and percentage values, such as animation duration or delay.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimePercentage {
    Time(Time),
    Percentage(Percentage),
}

impl From<Time> for TimePercentage {
    fn from(time: Time) -> Self {
        Self::Time(time)
    }
}

impl From<Percentage> for TimePercentage {
    fn from(percentage: Percentage) -> Self {
        Self::Percentage(percentage)
    }
}

/// Represents a combination of an Angle value or the math "zero".
/// This is used for gradients.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AngleZero {
    Angle(Angle),
    Zero,
}

impl TryFrom<&CssToken> for AngleZero {
    type Error = CssValueError;

    fn try_from(value: &CssToken) -> Result<Self, Self::Error> {
        Angle::try_from(value).map_or_else(
            |_| Err(CssValueError::InvalidValue("Expected an angle, <zero> is unsupported.".into())),
            |angle| Ok(Self::Angle(angle)),
        )
    }
}

impl From<Angle> for AngleZero {
    fn from(angle: Angle) -> Self {
        Self::Angle(angle)
    }
}

/// Represents a combination of an Angle or Percentage value or the math "zero".
/// This is used for gradients.
#[derive(Debug, Clone, PartialEq)]
pub enum AnglePercentageZero {
    AnglePercentage(AnglePercentage),
    Zero,
}

impl From<AnglePercentage> for AnglePercentageZero {
    fn from(angle_percentage: AnglePercentage) -> Self {
        Self::AnglePercentage(angle_percentage)
    }
}

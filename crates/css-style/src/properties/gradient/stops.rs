use css_cssom::{ComponentValue, CssTokenKind};

use crate::{
    Color,
    gradient::AnglePercentageOrZero,
    percentage::{AnglePercentage, LengthPercentage},
    primitives::{angle::Angle, percentage::Percentage},
    properties::gradient::{meaningful_cvs, split_on_commas, strip_whitespace, try_parse_length_percentage},
};

#[derive(Debug, Clone, PartialEq)]
pub struct ColorStopLength(pub LengthPercentage, pub Option<LengthPercentage>);

#[derive(Debug, Clone, PartialEq)]
pub struct LinearColorStop {
    pub color: Color,
    pub length: Option<ColorStopLength>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LinearColorHint(pub LengthPercentage);

#[derive(Debug, Clone, PartialEq)]
pub struct ColorStopList {
    pub first: LinearColorStop,
    pub rest: Vec<(Option<LinearColorHint>, LinearColorStop)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ColorStopAngle(pub AnglePercentageOrZero, pub Option<AnglePercentageOrZero>);

#[derive(Debug, Clone, PartialEq)]
pub struct AngularColorStop {
    pub color: Color,
    pub angle: Option<ColorStopAngle>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AngularColorHint {
    AnglePercentage(AnglePercentage),
    Zero,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AngularColorStopList {
    pub first: AngularColorStop,
    pub rest: Vec<(Option<AngularColorHint>, AngularColorStop)>,
}

/// Try to parse a single `ComponentValue` as an `AnglePercentageOrZero`.
fn try_parse_angle_percentage_or_zero(cv: &ComponentValue) -> Result<AnglePercentageOrZero, String> {
    match cv {
        ComponentValue::Token(token) => match &token.kind {
            CssTokenKind::Dimension { .. } | CssTokenKind::Number(_) => {
                let angle = Angle::try_from(token)?;
                Ok(AnglePercentageOrZero::AnglePercentage(AnglePercentage::Angle(angle)))
            }
            CssTokenKind::Percentage(value) => {
                let pct = Percentage::new(value.to_f64() as f32);
                Ok(AnglePercentageOrZero::AnglePercentage(AnglePercentage::Percentage(pct)))
            }
            _ => Err(format!("Expected angle, percentage, or zero, got {:?}", token.kind)),
        },
        _ => Err("Expected a token for angle/percentage".to_string()),
    }
}

/// Determines how many leading `ComponentValue`s form a color in a
/// whitespace-stripped segment.
///
/// Colors in CSS are always a single `ComponentValue`:
/// - An ident token (named colors, `currentColor`, `transparent`)
/// - A hash token (hex colors like `#fff`)
/// - A function (`rgb(…)`, `hsl(…)`, etc.)
///
/// Returns `Some(1)` when the first CV looks like a color, `None` otherwise.
fn color_cv_count(segment: &[ComponentValue]) -> Option<usize> {
    match segment.first()? {
        ComponentValue::Function(_) => Some(1),
        ComponentValue::Token(token) => match &token.kind {
            CssTokenKind::Ident(_) => Some(1),
            CssTokenKind::Hash { .. } => Some(1),
            _ => None,
        },
        _ => None,
    }
}

/// Returns `true` if the (already whitespace-stripped) segment starts with
/// something that looks like a color rather than a bare length/percentage
/// hint.
fn segment_is_color_stop(segment: &[ComponentValue]) -> bool {
    color_cv_count(segment).is_some()
}

impl TryFrom<&[ComponentValue]> for LinearColorHint {
    type Error = String;

    fn try_from(value: &[ComponentValue]) -> Result<Self, Self::Error> {
        let stripped = strip_whitespace(value);
        if stripped.is_empty() {
            return Err("Empty segment for color hint".to_string());
        }

        let meaningful = meaningful_cvs(stripped);
        if meaningful.len() != 1 {
            return Err(format!("Color hint must be a single length or percentage, got {} tokens", meaningful.len()));
        }

        let lp = try_parse_length_percentage(meaningful[0])?;
        Ok(LinearColorHint(lp))
    }
}

impl TryFrom<&[ComponentValue]> for LinearColorStop {
    type Error = String;

    /// Parse a comma-separated segment into a `LinearColorStop`.
    ///
    /// Accepted forms:
    /// - `<color>`
    /// - `<color> <length-percentage>`
    /// - `<color> <length-percentage> <length-percentage>`
    fn try_from(value: &[ComponentValue]) -> Result<Self, Self::Error> {
        let stripped = strip_whitespace(value);
        if stripped.is_empty() {
            return Err("Empty segment for color stop".to_string());
        }

        let color_count = color_cv_count(stripped).ok_or_else(|| "Segment does not start with a color".to_string())?;
        let color = Color::try_from(&stripped[..color_count])?;

        let rest = &stripped[color_count..];
        let length_cvs = meaningful_cvs(rest);

        let length = match length_cvs.len() {
            0 => None,
            1 => {
                let lp = try_parse_length_percentage(length_cvs[0])?;
                Some(ColorStopLength(lp, None))
            }
            2 => {
                let lp1 = try_parse_length_percentage(length_cvs[0])?;
                let lp2 = try_parse_length_percentage(length_cvs[1])?;
                Some(ColorStopLength(lp1, Some(lp2)))
            }
            n => {
                return Err(format!("Too many length/percentage values in color stop (expected 0-2, got {})", n));
            }
        };

        Ok(LinearColorStop { color, length })
    }
}

impl TryFrom<&[ComponentValue]> for ColorStopList {
    type Error = String;

    /// Parse a `<color-stop-list>` from comma-separated `ComponentValue`s.
    ///
    /// ```text
    /// <color-stop-list> =
    ///   <linear-color-stop> , [ <linear-color-hint>? , <linear-color-stop> ]*
    /// ```
    fn try_from(value: &[ComponentValue]) -> Result<Self, Self::Error> {
        let segments = split_on_commas(value);

        if segments.is_empty() {
            return Err("Empty color stop list".to_string());
        }

        let mut iter = segments.iter();

        let first_segment = iter.next().unwrap();
        let first =
            LinearColorStop::try_from(first_segment.as_slice()).map_err(|e| format!("First color stop: {}", e))?;

        let mut rest: Vec<(Option<LinearColorHint>, LinearColorStop)> = Vec::new();
        let mut pending_hint: Option<LinearColorHint> = None;

        for segment in iter {
            let stripped = strip_whitespace(segment);

            if stripped.is_empty() {
                continue;
            }

            if segment_is_color_stop(stripped) {
                let stop = LinearColorStop::try_from(segment.as_slice()).map_err(|e| format!("Color stop: {}", e))?;
                rest.push((pending_hint.take(), stop));
            } else {
                if pending_hint.is_some() {
                    return Err("Two consecutive color hints without a color stop in between".to_string());
                }
                let hint = LinearColorHint::try_from(segment.as_slice()).map_err(|e| format!("Color hint: {}", e))?;
                pending_hint = Some(hint);
            }
        }

        if pending_hint.is_some() {
            return Err("Trailing color hint without a following color stop".to_string());
        }

        if rest.is_empty() {
            return Err("Color stop list must have at least 2 color stops".to_string());
        }

        Ok(ColorStopList { first, rest })
    }
}

impl TryFrom<&[ComponentValue]> for AngularColorHint {
    type Error = String;

    fn try_from(value: &[ComponentValue]) -> Result<Self, Self::Error> {
        let stripped = strip_whitespace(value);
        if stripped.is_empty() {
            return Err("Empty segment for angular color hint".to_string());
        }

        let meaningful = meaningful_cvs(stripped);
        if meaningful.len() != 1 {
            return Err(format!(
                "Angular color hint must be a single angle or percentage, got {} tokens",
                meaningful.len()
            ));
        }

        let cv = meaningful[0];
        match cv {
            ComponentValue::Token(token) => match &token.kind {
                CssTokenKind::Number(n) if n.to_f64() == 0.0 => Ok(AngularColorHint::Zero),
                CssTokenKind::Dimension { .. } | CssTokenKind::Number(_) => {
                    let angle = Angle::try_from(token)?;
                    Ok(AngularColorHint::AnglePercentage(AnglePercentage::Angle(angle)))
                }
                CssTokenKind::Percentage(value) => {
                    let pct = Percentage::new(value.to_f64() as f32);
                    Ok(AngularColorHint::AnglePercentage(AnglePercentage::Percentage(pct)))
                }
                _ => Err(format!("Expected angle or percentage for angular hint, got {:?}", token.kind)),
            },
            _ => Err("Expected a token for angular color hint".to_string()),
        }
    }
}

impl TryFrom<&[ComponentValue]> for AngularColorStop {
    type Error = String;

    /// Parse a comma-separated segment into an `AngularColorStop`.
    ///
    /// Accepted forms:
    /// - `<color>`
    /// - `<color> <angle-percentage>`
    /// - `<color> <angle-percentage> <angle-percentage>`
    fn try_from(value: &[ComponentValue]) -> Result<Self, Self::Error> {
        let stripped = strip_whitespace(value);
        if stripped.is_empty() {
            return Err("Empty segment for angular color stop".to_string());
        }

        let color_count = color_cv_count(stripped).ok_or_else(|| "Segment does not start with a color".to_string())?;
        let color = Color::try_from(&stripped[..color_count])?;

        let rest = &stripped[color_count..];
        let angle_cvs = meaningful_cvs(rest);

        let angle = match angle_cvs.len() {
            0 => None,
            1 => {
                let ap = try_parse_angle_percentage_or_zero(angle_cvs[0])?;
                Some(ColorStopAngle(ap, None))
            }
            2 => {
                let ap1 = try_parse_angle_percentage_or_zero(angle_cvs[0])?;
                let ap2 = try_parse_angle_percentage_or_zero(angle_cvs[1])?;
                Some(ColorStopAngle(ap1, Some(ap2)))
            }
            n => {
                return Err(format!(
                    "Too many angle/percentage values in angular color stop (expected 0-2, got {})",
                    n
                ));
            }
        };

        Ok(AngularColorStop { color, angle })
    }
}

impl TryFrom<&[ComponentValue]> for AngularColorStopList {
    type Error = String;

    /// Parse an `<angular-color-stop-list>` from comma-separated
    /// `ComponentValue`s.
    ///
    /// ```text
    /// <angular-color-stop-list> =
    ///   <angular-color-stop> , [ <angular-color-hint>? , <angular-color-stop> ]*
    /// ```
    fn try_from(value: &[ComponentValue]) -> Result<Self, Self::Error> {
        let segments = split_on_commas(value);

        if segments.is_empty() {
            return Err("Empty angular color stop list".to_string());
        }

        let mut iter = segments.iter();

        let first_segment = iter.next().unwrap();
        let first = AngularColorStop::try_from(first_segment.as_slice())
            .map_err(|e| format!("First angular color stop: {}", e))?;

        let mut rest: Vec<(Option<AngularColorHint>, AngularColorStop)> = Vec::new();
        let mut pending_hint: Option<AngularColorHint> = None;

        for segment in iter {
            let stripped = strip_whitespace(segment);

            if stripped.is_empty() {
                continue;
            }

            if segment_is_color_stop(stripped) {
                let stop =
                    AngularColorStop::try_from(segment.as_slice()).map_err(|e| format!("Angular color stop: {}", e))?;
                rest.push((pending_hint.take(), stop));
            } else {
                if pending_hint.is_some() {
                    return Err("Two consecutive angular color hints without a color stop in between".to_string());
                }
                let hint =
                    AngularColorHint::try_from(segment.as_slice()).map_err(|e| format!("Angular color hint: {}", e))?;
                pending_hint = Some(hint);
            }
        }

        if pending_hint.is_some() {
            return Err("Trailing angular color hint without a following color stop".to_string());
        }

        if rest.is_empty() {
            return Err("Angular color stop list must have at least 2 color stops".to_string());
        }

        Ok(AngularColorStopList { first, rest })
    }
}

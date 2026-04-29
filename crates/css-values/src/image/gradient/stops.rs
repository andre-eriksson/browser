use css_cssom::{ComponentValue, ComponentValueStream, CssTokenKind};

use crate::{
    CSSParsable,
    calc::{CalcDomain, CalcExpression, is_math_function},
    color::Color,
    combination::{AnglePercentage, AnglePercentageZero, LengthPercentage},
    error::CssValueError,
    image::Gradient,
    numeric::Percentage,
    quantity::Angle,
};

#[derive(Debug, Clone, PartialEq)]
pub struct ColorStopLength(pub LengthPercentage, pub Option<LengthPercentage>);

#[derive(Debug, Clone, PartialEq)]
pub struct LinearColorStop {
    pub color: Color,
    pub length: Option<ColorStopLength>,
}

impl CSSParsable for LinearColorStop {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, CssValueError> {
        let stripped = Gradient::strip_whitespace(stream.remaining());
        if stripped.is_empty() {
            return Err(CssValueError::UnexpectedEndOfInput);
        }

        let color_count = Gradient::color_cv_count(stripped)
            .ok_or_else(|| CssValueError::InvalidValue("Segment does not start with a color".into()))?;
        let color = Color::parse(&mut stripped[..color_count].into())?;

        let rest = &stripped[color_count..];
        let length_cvs = Gradient::meaningful_cvs(rest);

        let length = match length_cvs.len() {
            0 => None,
            1 => {
                let lp = LengthPercentage::try_from(length_cvs[0])?;
                Some(ColorStopLength(lp, None))
            }
            2 => {
                let lp1 = LengthPercentage::try_from(length_cvs[0])?;
                let lp2 = LengthPercentage::try_from(length_cvs[1])?;
                Some(ColorStopLength(lp1, Some(lp2)))
            }
            n => {
                return Err(CssValueError::InvalidValue(format!(
                    "Too many length/percentage values in linear color stop (expected 0-2, got {n})"
                )));
            }
        };

        Ok(Self { color, length })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LinearColorHint(pub LengthPercentage);

impl CSSParsable for LinearColorHint {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, CssValueError> {
        let stripped = Gradient::strip_whitespace(stream.remaining());
        if stripped.is_empty() {
            return Err(CssValueError::UnexpectedEndOfInput);
        }

        let meaningful = Gradient::meaningful_cvs(stripped);
        if meaningful.len() != 1 {
            return Err(CssValueError::InvalidValue(format!(
                "Linear color hint must be a single length or percentage, got {} tokens",
                meaningful.len()
            )));
        }

        let lp = LengthPercentage::try_from(meaningful[0])?;
        Ok(Self(lp))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ColorStopList(pub LinearColorStop, pub Vec<(Option<LinearColorHint>, LinearColorStop)>);

impl CSSParsable for ColorStopList {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, CssValueError> {
        let segments = Gradient::split_on_commas(stream.remaining());

        if segments.is_empty() {
            return Err(CssValueError::UnexpectedEndOfInput);
        }

        let mut iter = segments.iter();

        let first_segment = iter.next().unwrap();
        let first = LinearColorStop::parse(&mut first_segment.as_slice().into())
            .map_err(|e| CssValueError::InvalidValue(format!("First color stop: {e}")))?;

        let mut rest: Vec<(Option<LinearColorHint>, LinearColorStop)> = Vec::new();
        let mut pending_hint: Option<LinearColorHint> = None;

        for segment in iter {
            let stripped = Gradient::strip_whitespace(segment);

            if stripped.is_empty() {
                continue;
            }

            if Gradient::segment_is_color_stop(stripped) {
                let stop = LinearColorStop::parse(&mut segment.as_slice().into())?;
                rest.push((pending_hint.take(), stop));
            } else {
                if pending_hint.is_some() {
                    return Err(CssValueError::InvalidValue(
                        "Two consecutive color hints without a color stop in between".into(),
                    ));
                }
                let hint = LinearColorHint::parse(&mut segment.as_slice().into())?;
                pending_hint = Some(hint);
            }
        }

        if pending_hint.is_some() {
            return Err(CssValueError::InvalidValue("Trailing color hint without a following color stop".into()));
        }

        if rest.is_empty() {
            return Err(CssValueError::InvalidValue("Color stop list must have at least 2 color stops".into()));
        }

        Ok(Self(first, rest))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ColorStopAngle(pub AnglePercentageZero, pub Option<AnglePercentageZero>);

#[derive(Debug, Clone, PartialEq)]
pub struct AngularColorStop {
    pub color: Color,
    pub angle: Option<ColorStopAngle>,
}

impl CSSParsable for AngularColorStop {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, CssValueError> {
        let stripped = Gradient::strip_whitespace(stream.remaining());
        if stripped.is_empty() {
            return Err(CssValueError::UnexpectedEndOfInput);
        }

        let color_count = Gradient::color_cv_count(stripped)
            .ok_or_else(|| CssValueError::InvalidValue("Segment does not start with a color".into()))?;
        let color = Color::parse(&mut stripped[..color_count].into())?;

        let rest = &stripped[color_count..];
        let angle_cvs = Gradient::meaningful_cvs(rest);

        let angle = match angle_cvs.len() {
            0 => None,
            1 => {
                let ap = AnglePercentageZero::try_from(angle_cvs[0])?;
                Some(ColorStopAngle(ap, None))
            }
            2 => {
                let ap1 = AnglePercentageZero::try_from(angle_cvs[0])?;
                let ap2 = AnglePercentageZero::try_from(angle_cvs[1])?;
                Some(ColorStopAngle(ap1, Some(ap2)))
            }
            n => {
                return Err(CssValueError::InvalidValue(format!(
                    "Too many angle/percentage values in angular color stop (expected 0-2, got {n})"
                )));
            }
        };

        Ok(Self { color, angle })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AngularColorHint {
    AnglePercentage(AnglePercentage),
    Zero,
}

impl CSSParsable for AngularColorHint {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, CssValueError> {
        let stripped = Gradient::strip_whitespace(stream.remaining());
        if stripped.is_empty() {
            return Err(CssValueError::UnexpectedEndOfInput);
        }

        let meaningful = Gradient::meaningful_cvs(stripped);
        if meaningful.len() != 1 {
            return Err(CssValueError::InvalidValue(format!(
                "Angular color hint must be a single angle or percentage, got {} tokens",
                meaningful.len()
            )));
        }

        let cv = meaningful[0];
        match cv {
            ComponentValue::Function(func) => {
                if is_math_function(&func.name) {
                    let expr = CalcExpression::parse(&func.name, &func.value)?;
                    let domain = expr.resolve_domain()?;

                    if !matches!(domain, CalcDomain::Angle | CalcDomain::Percentage) {
                        return Err(CssValueError::InvalidCalcDomain {
                            expected: vec![CalcDomain::Angle, CalcDomain::Percentage],
                            found: domain,
                        });
                    }

                    Ok(Self::AnglePercentage(AnglePercentage::Calc(expr)))
                } else {
                    // TODO: Implement `from` and `to` functions for angular color hints, which allow specifying angles relative to the gradient's start angle.
                    Err(CssValueError::InvalidFunction(func.name.clone()))
                }
            }
            ComponentValue::Token(token) => match &token.kind {
                CssTokenKind::Number(n) if n.to_f64() == 0.0 => Ok(Self::Zero),
                CssTokenKind::Dimension { .. } | CssTokenKind::Number(_) => {
                    let angle = Angle::try_from(token)?;
                    Ok(Self::AnglePercentage(AnglePercentage::Angle(angle)))
                }
                CssTokenKind::Percentage(value) => {
                    let pct = Percentage::new(value.to_f64());
                    Ok(Self::AnglePercentage(AnglePercentage::Percentage(pct)))
                }
                _ => Err(CssValueError::InvalidToken(token.kind.clone())),
            },
            cvs => Err(CssValueError::InvalidComponentValue(cvs.clone())),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AngularColorStopList(pub AngularColorStop, pub Vec<(Option<AngularColorHint>, AngularColorStop)>);

impl CSSParsable for AngularColorStopList {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, CssValueError> {
        let segments = Gradient::split_on_commas(stream.remaining());

        if segments.is_empty() {
            return Err(CssValueError::UnexpectedEndOfInput);
        }

        let mut iter = segments.iter();

        let first_segment = iter.next().unwrap();
        let first = AngularColorStop::parse(&mut first_segment.as_slice().into())?;

        let mut rest: Vec<(Option<AngularColorHint>, AngularColorStop)> = Vec::new();
        let mut pending_hint: Option<AngularColorHint> = None;

        for segment in iter {
            let stripped = Gradient::strip_whitespace(segment);

            if stripped.is_empty() {
                continue;
            }

            if Gradient::segment_is_color_stop(stripped) {
                let stop = AngularColorStop::parse(&mut segment.as_slice().into())?;
                rest.push((pending_hint.take(), stop));
            } else {
                if pending_hint.is_some() {
                    return Err(CssValueError::InvalidValue(
                        "Two consecutive angular color hints without a color stop in between".into(),
                    ));
                }
                let hint = AngularColorHint::parse(&mut segment.as_slice().into())?;
                pending_hint = Some(hint);
            }
        }

        if pending_hint.is_some() {
            return Err(CssValueError::InvalidValue(
                "Trailing angular color hint without a following color stop".into(),
            ));
        }

        if rest.is_empty() {
            return Err(CssValueError::InvalidValue("Angular color stop list must have at least 2 color stops".into()));
        }

        Ok(Self(first, rest))
    }
}

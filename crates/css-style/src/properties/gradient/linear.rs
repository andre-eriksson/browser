use css_cssom::{ComponentValue, CssTokenKind};

use crate::{
    gradient::AngleOrZero,
    properties::gradient::{
        SideOrCorner, collect_idents, interpolation::ColorInterpolationMethod, reassemble_to_comma_separated,
        split_on_commas, stops::ColorStopList, strip_whitespace, try_consume_interpolation,
    },
};

#[derive(Debug, Clone, PartialEq)]
pub enum LinearDirection {
    Angle(AngleOrZero),
    Side(SideOrCorner),
}

#[derive(Debug, Clone, PartialEq)]
pub struct LinearGradientSyntax {
    pub direction: Option<LinearDirection>,
    pub interpolation: Option<ColorInterpolationMethod>,
    pub stops: ColorStopList,
}

/// Try to parse a segment as a `LinearDirection`.
///
/// Accepted forms:
///   - `<angle>`                        → `LinearDirection::Angle`
///   - `to <side-or-corner>`            → `LinearDirection::Side`
fn try_parse_direction(segment: &[ComponentValue]) -> Result<LinearDirection, String> {
    let stripped = strip_whitespace(segment);
    if stripped.is_empty() {
        return Err("Empty direction segment".into());
    }

    let meaningful: Vec<&ComponentValue> = stripped
        .iter()
        .filter(|cv| !matches!(cv, ComponentValue::Token(t) if matches!(t.kind, CssTokenKind::Whitespace)))
        .collect();

    if meaningful.len() == 1
        && let ComponentValue::Token(token) = meaningful[0]
        && let Ok(angle) = AngleOrZero::try_from(token)
    {
        return Ok(LinearDirection::Angle(angle));
    }

    let idents = collect_idents(stripped);
    if let Some(first) = idents.first()
        && first.eq_ignore_ascii_case("to")
    {
        let to_pos = stripped
                .iter()
                .position(|cv| {
                    matches!(cv, ComponentValue::Token(t) if matches!(&t.kind, CssTokenKind::Ident(s) if s.eq_ignore_ascii_case("to")))
                })
                .unwrap();

        let after_to = &stripped[to_pos + 1..];
        let side_or_corner = SideOrCorner::try_from(after_to)?;
        return Ok(LinearDirection::Side(side_or_corner));
    }

    Err("Segment is not a direction".into())
}

/// Checks whether the first segment (before the first comma) constitutes a
/// direction and/or an `in <color-interpolation>` clause, or whether it is
/// already part of the color-stop list.
///
/// Heuristic: if the segment starts with an angle token, a `to` keyword, or
/// an `in` keyword, it is *not* a color-stop.
fn segment_is_direction_or_interpolation(segment: &[ComponentValue]) -> bool {
    let stripped = strip_whitespace(segment);
    if stripped.is_empty() {
        return false;
    }

    match &stripped[0] {
        ComponentValue::Token(token) => match &token.kind {
            CssTokenKind::Ident(s) => s.eq_ignore_ascii_case("to") || s.eq_ignore_ascii_case("in"),
            CssTokenKind::Dimension { .. } => true,
            CssTokenKind::Number(n) if n.to_f64() == 0.0 => true,
            _ => false,
        },
        _ => false,
    }
}

impl TryFrom<&[ComponentValue]> for LinearGradientSyntax {
    type Error = String;

    /// Parse the inner component values of a `linear-gradient()` function.
    ///
    /// CSS grammar:
    /// ```text
    /// linear-gradient(
    ///   [ <angle> | to <side-or-corner> ]? ,
    ///   [ in <color-interpolation-method> , ]?
    ///   <color-stop-list>
    /// )
    /// ```
    ///
    /// Strategy:
    /// 1. Split the entire value list on commas.
    /// 2. Examine the first segment to determine if it is a direction
    ///    (angle or `to <side-or-corner>`).  If so, consume it.
    /// 3. Check whether the next segment starts with `in` (color
    ///    interpolation).  If so, consume it.
    /// 4. The remaining segments form the `<color-stop-list>`.
    fn try_from(value: &[ComponentValue]) -> Result<Self, Self::Error> {
        let segments = split_on_commas(value);

        if segments.is_empty() {
            return Err("Empty linear-gradient arguments".into());
        }

        let mut idx = 0;
        let mut direction: Option<LinearDirection> = None;
        let mut interpolation: Option<ColorInterpolationMethod> = None;

        if idx < segments.len() && segment_is_direction_or_interpolation(&segments[idx]) {
            let seg = &segments[idx];
            let stripped = strip_whitespace(seg);

            if let Ok(Some(method)) = try_consume_interpolation(stripped) {
                interpolation = Some(method);
                idx += 1;
            } else {
                direction = Some(try_parse_direction(seg)?);
                idx += 1;
            }
        }

        if interpolation.is_none() && idx < segments.len() {
            let seg = &segments[idx];
            let stripped = strip_whitespace(seg);

            if let Ok(Some(method)) = try_consume_interpolation(stripped) {
                interpolation = Some(method);
                idx += 1;
            }
        }

        if idx >= segments.len() {
            return Err("Missing color stop list in linear-gradient".into());
        }

        let stop_cvs = reassemble_to_comma_separated(&segments[idx..]);
        let stops = ColorStopList::try_from(stop_cvs.as_slice())?;

        Ok(LinearGradientSyntax {
            direction,
            interpolation,
            stops,
        })
    }
}

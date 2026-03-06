use css_cssom::{ComponentValue, ComponentValueStream, CssTokenKind};

use crate::{
    CSSParsable,
    combination::AngleZero,
    image::{
        Gradient,
        gradient::{interpolation::ColorInterpolationMethod, stops::ColorStopList},
    },
    position::SideOrCorner,
};

#[derive(Debug, Clone, PartialEq)]
pub enum LinearDirection {
    Angle(AngleZero),
    Side(SideOrCorner),
}

#[derive(Debug, Clone, PartialEq)]
pub struct LinearGradientSyntax {
    pub direction: Option<LinearDirection>,
    pub interpolation: Option<ColorInterpolationMethod>,
    pub stops: ColorStopList,
}

/// Parsing helpers for `LinearGradientSyntax`.
impl LinearGradientSyntax {
    /// Try to parse a segment as a `LinearDirection`.
    ///
    /// Accepted forms:
    ///   - `<angle>`                        → `LinearDirection::Angle`
    ///   - `to <side-or-corner>`            → `LinearDirection::Side`
    fn try_parse_direction(segment: &[ComponentValue]) -> Result<LinearDirection, String> {
        let stripped = Gradient::strip_whitespace(segment);
        if stripped.is_empty() {
            return Err("Empty direction segment".into());
        }

        let meaningful: Vec<&ComponentValue> = stripped
            .iter()
            .filter(|cv| !matches!(cv, ComponentValue::Token(t) if matches!(t.kind, CssTokenKind::Whitespace)))
            .collect();

        if meaningful.len() == 1
            && let ComponentValue::Token(token) = meaningful[0]
            && let Ok(angle) = AngleZero::try_from(token)
        {
            return Ok(LinearDirection::Angle(angle));
        }

        let idents = Gradient::collect_idents(stripped);
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
            let side_or_corner = SideOrCorner::parse(&mut after_to.into())?;
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
        let stripped = Gradient::strip_whitespace(segment);
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
}

impl CSSParsable for LinearGradientSyntax {
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
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, String> {
        let segments = Gradient::split_on_commas(stream.remaining());

        if segments.is_empty() {
            return Err("Empty linear-gradient arguments".into());
        }

        let mut idx = 0;
        let mut direction: Option<LinearDirection> = None;
        let mut interpolation: Option<ColorInterpolationMethod> = None;

        if idx < segments.len() && Self::segment_is_direction_or_interpolation(&segments[idx]) {
            let seg = &segments[idx];
            let stripped = Gradient::strip_whitespace(seg);

            if let Ok(Some(method)) = Gradient::try_consume_interpolation(stripped) {
                interpolation = Some(method);
                idx += 1;
            } else {
                direction = Some(Self::try_parse_direction(seg)?);
                idx += 1;
            }
        }

        if interpolation.is_none() && idx < segments.len() {
            let seg = &segments[idx];
            let stripped = Gradient::strip_whitespace(seg);

            if let Ok(Some(method)) = Gradient::try_consume_interpolation(stripped) {
                interpolation = Some(method);
                idx += 1;
            }
        }

        if idx >= segments.len() {
            return Err("Missing color stop list in linear-gradient".into());
        }

        let stop_cvs = Gradient::reassemble_to_comma_separated(&segments[idx..]);
        let stops = ColorStopList::parse(&mut stop_cvs.as_slice().into())?;

        Ok(LinearGradientSyntax {
            direction,
            interpolation,
            stops,
        })
    }
}

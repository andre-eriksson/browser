use css_cssom::{ComponentValue, ComponentValueStream, CssTokenKind};

use crate::{
    CSSParsable,
    combination::AngleZero,
    error::CssValueError,
    image::{
        Gradient,
        gradient::{interpolation::ColorInterpolationMethod, stops::AngularColorStopList},
    },
    position::Position,
};

type ConicConfig = (Option<AngleZero>, Option<Position>, Option<ColorInterpolationMethod>);

#[derive(Debug, Clone, PartialEq)]
pub struct ConicGradientSyntax {
    pub from_angle: Option<AngleZero>,
    pub position: Option<Position>,
    pub interpolation: Option<ColorInterpolationMethod>,
    pub stops: AngularColorStopList,
}

/// Parsing helpers for `ConicGradientSyntax`.
impl ConicGradientSyntax {
    /// Checks whether a segment looks like conic-gradient configuration
    /// (`from <angle>`, `at <position>`, or `in <interpolation>`) rather than
    /// a color stop.
    ///
    /// Conic config segments can start with:
    ///   - `from` keyword (angle)
    ///   - `at` keyword (position)
    ///   - `in` keyword (color interpolation)
    ///   - A dimension token (bare angle like `45deg`)
    fn segment_is_conic_config(segment: &[ComponentValue]) -> bool {
        let stripped = Gradient::strip_whitespace(segment);
        if stripped.is_empty() {
            return false;
        }

        match &stripped[0] {
            ComponentValue::Token(token) => match &token.kind {
                CssTokenKind::Ident(s) => {
                    s.eq_ignore_ascii_case("from") || s.eq_ignore_ascii_case("at") || s.eq_ignore_ascii_case("in")
                }
                _ => false,
            },
            _ => false,
        }
    }

    /// Parse the conic configuration segment which may contain any combination
    /// of `from <angle>` and `at <position>` (and possibly `in <interpolation>`).
    ///
    /// ```text
    /// [ from <angle> ]? [ at <position> ]?
    /// ```
    ///
    /// The `from` and `at` parts may appear in the same comma-segment.
    /// The `in` part is handled separately in the main parser, but if it
    /// appears here we also handle it.
    fn parse_conic_config(segment: &[ComponentValue]) -> Result<ConicConfig, CssValueError> {
        let stripped = Gradient::strip_whitespace(segment);
        if stripped.is_empty() {
            return Ok((None, None, None));
        }

        let mut from_angle: Option<AngleZero> = None;
        let mut position: Option<Position> = None;
        let mut interpolation: Option<ColorInterpolationMethod> = None;

        let from_pos = Gradient::find_ident_position(stripped, |s| s.eq_ignore_ascii_case("from"));
        let at_pos = Gradient::find_ident_position(stripped, |s| s.eq_ignore_ascii_case("at"));
        let in_pos = Gradient::find_ident_position(stripped, |s| s.eq_ignore_ascii_case("in"));

        if let Some(fp) = from_pos {
            let end = [at_pos, in_pos]
                .iter()
                .filter_map(|p| *p)
                .filter(|&p| p > fp)
                .min()
                .unwrap_or(stripped.len());
            let angle_cvs = &stripped[fp + 1..end];
            let meaningful: Vec<&ComponentValue> = angle_cvs
                .iter()
                .filter(|cv| !matches!(cv, ComponentValue::Token(t) if matches!(t.kind, CssTokenKind::Whitespace)))
                .collect();
            if meaningful.len() == 1 {
                if let ComponentValue::Token(token) = meaningful[0] {
                    from_angle = Some(AngleZero::try_from(token)?);
                }
            } else if !meaningful.is_empty() {
                return Err(CssValueError::InvalidValue(format!(
                    "Expected a single angle after 'from', got {} tokens",
                    meaningful.len()
                )));
            }
        }

        if let Some(ap) = at_pos {
            let end = in_pos.filter(|&p| p > ap).unwrap_or(stripped.len());
            let pos_cvs = &stripped[ap + 1..end];
            position = Some(Position::parse(&mut pos_cvs.into())?);
        }

        if let Some(ip) = in_pos {
            let interp_cvs = &stripped[ip + 1..];
            interpolation = Some(Gradient::try_parse_interpolation(interp_cvs)?);
        }

        Ok((from_angle, position, interpolation))
    }
}

impl CSSParsable for ConicGradientSyntax {
    /// Parse the inner component values of a `conic-gradient()` function.
    ///
    /// CSS grammar:
    /// ```text
    /// conic-gradient(
    ///   [ from <angle> ]? [ at <position> ]? ,
    ///   [ in <color-interpolation-method> , ]?
    ///   <angular-color-stop-list>
    /// )
    /// ```
    ///
    /// Strategy:
    /// 1. Split the entire value list on commas.
    /// 2. Examine the first segment – if it looks like conic config
    ///    (`from`, `at`, or `in`) consume it.
    /// 3. Check the next segment for `in <color-interpolation-method>`.
    /// 4. The remaining segments form the `<angular-color-stop-list>`.
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, CssValueError> {
        let segments = Gradient::split_on_commas(stream.remaining());

        if segments.is_empty() {
            return Err(CssValueError::InvalidValue("Empty conic-gradient arguments".into()));
        }

        let mut idx = 0;
        let mut from_angle: Option<AngleZero> = None;
        let mut position: Option<Position> = None;
        let mut interpolation: Option<ColorInterpolationMethod> = None;

        if idx < segments.len() && Self::segment_is_conic_config(&segments[idx]) {
            let seg = &segments[idx];
            let stripped = Gradient::strip_whitespace(seg);
            let idents = Gradient::collect_idents(stripped);

            if let Some(first) = idents.first() {
                if first.eq_ignore_ascii_case("in")
                    && !idents.iter().any(|i| i.eq_ignore_ascii_case("from"))
                    && !idents.iter().any(|i| i.eq_ignore_ascii_case("at"))
                {
                    if let Ok(Some(method)) = Gradient::try_consume_interpolation(stripped) {
                        interpolation = Some(method);
                        idx += 1;
                    }
                } else {
                    let (angle, pos, interp) = Self::parse_conic_config(seg)?;
                    from_angle = angle;
                    position = pos;
                    if interp.is_some() {
                        interpolation = interp;
                    }
                    idx += 1;
                }
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
            return Err(CssValueError::InvalidValue("Missing angular color stop list in conic-gradient".into()));
        }

        let stop_cvs = Gradient::reassemble_to_comma_separated(&segments[idx..]);
        let stops = AngularColorStopList::parse(&mut stop_cvs.as_slice().into())?;

        Ok(ConicGradientSyntax {
            from_angle,
            position,
            interpolation,
            stops,
        })
    }
}

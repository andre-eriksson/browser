use css_cssom::{ComponentValue, CssTokenKind};

use crate::{
    gradient::AngleOrZero,
    position::Position,
    properties::gradient::{
        collect_idents, find_ident_position, interpolation::ColorInterpolationMethod, reassemble_to_comma_separated,
        split_on_commas, stops::AngularColorStopList, strip_whitespace, try_consume_interpolation,
    },
};

/// Parsed conic-gradient configuration: `(from_angle, position, interpolation)`.
type ConicConfig = (Option<AngleOrZero>, Option<Position>, Option<ColorInterpolationMethod>);

#[derive(Debug, Clone, PartialEq)]
pub struct ConicGradientSyntax {
    pub from_angle: Option<AngleOrZero>,
    pub position: Option<Position>,
    pub interpolation: Option<ColorInterpolationMethod>,
    pub stops: AngularColorStopList,
}

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
    let stripped = strip_whitespace(segment);
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
fn parse_conic_config(segment: &[ComponentValue]) -> Result<ConicConfig, String> {
    let stripped = strip_whitespace(segment);
    if stripped.is_empty() {
        return Ok((None, None, None));
    }

    let mut from_angle: Option<AngleOrZero> = None;
    let mut position: Option<Position> = None;
    let mut interpolation: Option<ColorInterpolationMethod> = None;

    let from_pos = find_ident_position(stripped, |s| s.eq_ignore_ascii_case("from"));
    let at_pos = find_ident_position(stripped, |s| s.eq_ignore_ascii_case("at"));
    let in_pos = find_ident_position(stripped, |s| s.eq_ignore_ascii_case("in"));

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
                from_angle = Some(AngleOrZero::try_from(token)?);
            }
        } else if !meaningful.is_empty() {
            return Err(format!("Expected a single angle after 'from', got {} tokens", meaningful.len()));
        }
    }

    if let Some(ap) = at_pos {
        let end = in_pos.filter(|&p| p > ap).unwrap_or(stripped.len());
        let pos_cvs = &stripped[ap + 1..end];
        position = Some(Position::try_from(pos_cvs)?);
    }

    // `in <color-interpolation-method>`
    if let Some(ip) = in_pos {
        let interp_cvs = &stripped[ip + 1..];
        interpolation = Some(crate::properties::gradient::try_parse_interpolation(interp_cvs)?);
    }

    Ok((from_angle, position, interpolation))
}

impl TryFrom<&[ComponentValue]> for ConicGradientSyntax {
    type Error = String;

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
    fn try_from(value: &[ComponentValue]) -> Result<Self, Self::Error> {
        let segments = split_on_commas(value);

        if segments.is_empty() {
            return Err("Empty conic-gradient arguments".into());
        }

        let mut idx = 0;
        let mut from_angle: Option<AngleOrZero> = None;
        let mut position: Option<Position> = None;
        let mut interpolation: Option<ColorInterpolationMethod> = None;

        if idx < segments.len() && segment_is_conic_config(&segments[idx]) {
            let seg = &segments[idx];
            let stripped = strip_whitespace(seg);
            let idents = collect_idents(stripped);

            if let Some(first) = idents.first() {
                if first.eq_ignore_ascii_case("in")
                    && !idents.iter().any(|i| i.eq_ignore_ascii_case("from"))
                    && !idents.iter().any(|i| i.eq_ignore_ascii_case("at"))
                {
                    if let Ok(Some(method)) = try_consume_interpolation(stripped) {
                        interpolation = Some(method);
                        idx += 1;
                    }
                } else {
                    let (angle, pos, interp) = parse_conic_config(seg)?;
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
            let stripped = strip_whitespace(seg);

            if let Ok(Some(method)) = try_consume_interpolation(stripped) {
                interpolation = Some(method);
                idx += 1;
            }
        }

        if idx >= segments.len() {
            return Err("Missing angular color stop list in conic-gradient".into());
        }

        let stop_cvs = reassemble_to_comma_separated(&segments[idx..]);
        let stops = AngularColorStopList::try_from(stop_cvs.as_slice())?;

        Ok(ConicGradientSyntax {
            from_angle,
            position,
            interpolation,
            stops,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use css_cssom::CSSStyleSheet;

    /// Helper: parse an inline CSS declaration and return the component values.
    fn parse_value(css: &str) -> Vec<ComponentValue> {
        let decls = CSSStyleSheet::from_inline(css);
        assert!(!decls.is_empty(), "No declarations parsed from: {css}");
        decls[0].original_values.clone()
    }

    /// Helper: extract the first Function from parsed component values.
    fn extract_function(cvs: &[ComponentValue]) -> &css_cssom::Function {
        cvs.iter()
            .find_map(|cv| match cv {
                ComponentValue::Function(f) => Some(f),
                _ => None,
            })
            .expect("No function found in component values")
    }

    #[test]
    fn conic_two_colors() {
        let cvs = parse_value("background-image: conic-gradient(red, blue)");
        let func = extract_function(&cvs);
        let syn = ConicGradientSyntax::try_from(func.value.as_slice()).unwrap();
        assert!(syn.from_angle.is_none());
        assert!(syn.position.is_none());
        assert!(syn.interpolation.is_none());
        assert_eq!(syn.stops.rest.len(), 1);
    }

    #[test]
    fn conic_three_colors() {
        let cvs = parse_value("background-image: conic-gradient(red, green, blue)");
        let func = extract_function(&cvs);
        let syn = ConicGradientSyntax::try_from(func.value.as_slice()).unwrap();
        assert_eq!(syn.stops.rest.len(), 2);
    }

    #[test]
    fn conic_from_angle() {
        let cvs = parse_value("background-image: conic-gradient(from 45deg, red, blue)");
        let func = extract_function(&cvs);
        let syn = ConicGradientSyntax::try_from(func.value.as_slice()).unwrap();
        assert!(syn.from_angle.is_some());
        assert!(syn.position.is_none());
    }

    #[test]
    fn conic_from_turn() {
        let cvs = parse_value("background-image: conic-gradient(from 0.25turn, red, blue)");
        let func = extract_function(&cvs);
        let syn = ConicGradientSyntax::try_from(func.value.as_slice()).unwrap();
        assert!(matches!(syn.from_angle, Some(AngleOrZero::Angle(_))));
    }

    #[test]
    fn conic_at_center() {
        let cvs = parse_value("background-image: conic-gradient(at center, red, blue)");
        let func = extract_function(&cvs);
        let syn = ConicGradientSyntax::try_from(func.value.as_slice()).unwrap();
        assert!(syn.position.is_some());
        assert!(syn.from_angle.is_none());
    }

    #[test]
    fn conic_from_angle_at_position() {
        let cvs = parse_value("background-image: conic-gradient(from 90deg at center, red, blue)");
        let func = extract_function(&cvs);
        let syn = ConicGradientSyntax::try_from(func.value.as_slice()).unwrap();
        assert!(syn.from_angle.is_some());
        assert!(syn.position.is_some());
    }

    #[test]
    fn conic_stops_with_angles() {
        let cvs = parse_value("background-image: conic-gradient(red 0deg, blue 360deg)");
        let func = extract_function(&cvs);
        let syn = ConicGradientSyntax::try_from(func.value.as_slice()).unwrap();
        assert!(syn.stops.first.angle.is_some());
        assert!(syn.stops.rest[0].1.angle.is_some());
    }

    #[test]
    fn conic_stops_with_percentages() {
        let cvs = parse_value("background-image: conic-gradient(red 0%, blue 100%)");
        let func = extract_function(&cvs);
        let syn = ConicGradientSyntax::try_from(func.value.as_slice()).unwrap();
        assert!(syn.stops.first.angle.is_some());
        assert!(syn.stops.rest[0].1.angle.is_some());
    }

    #[test]
    fn conic_hex_colors() {
        let cvs = parse_value("background-image: conic-gradient(#ff0000, #0000ff)");
        let func = extract_function(&cvs);
        let syn = ConicGradientSyntax::try_from(func.value.as_slice()).unwrap();
        assert_eq!(syn.stops.rest.len(), 1);
    }

    #[test]
    fn conic_single_stop_fails() {
        let cvs = parse_value("background-image: conic-gradient(red)");
        let func = extract_function(&cvs);
        assert!(ConicGradientSyntax::try_from(func.value.as_slice()).is_err());
    }

    #[test]
    fn conic_empty_fails() {
        let empty: &[ComponentValue] = &[];
        assert!(ConicGradientSyntax::try_from(empty).is_err());
    }

    #[test]
    fn conic_many_stops() {
        let cvs = parse_value("background-image: conic-gradient(red, orange, yellow, green, blue)");
        let func = extract_function(&cvs);
        let syn = ConicGradientSyntax::try_from(func.value.as_slice()).unwrap();
        assert_eq!(syn.stops.rest.len(), 4);
    }
}

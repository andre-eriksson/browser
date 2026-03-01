use css_cssom::{ComponentValue, CssTokenKind};

use crate::{
    gradient::{RadialExtent, RadialShape},
    length::{Length, LengthUnit},
    percentage::LengthPercentage,
    position::Position,
    properties::gradient::{
        find_ident_position, interpolation::ColorInterpolationMethod, meaningful_cvs, reassemble_to_comma_separated,
        split_on_commas, stops::ColorStopList, strip_whitespace, try_consume_interpolation,
        try_parse_length_percentage,
    },
};

/// Result of parsing the radial gradient configuration segment (shape, size, position).
type RadialConfig = (Option<RadialShape>, Option<RadialSize>, Option<Position>);

#[derive(Debug, Clone, PartialEq)]
pub enum RadialSize {
    Extent(RadialExtent),
    Length(Length),
    LengthPercentagePair(LengthPercentage, LengthPercentage),
}

#[derive(Debug, Clone, PartialEq)]
pub struct RadialGradientSyntax {
    pub shape: Option<RadialShape>,
    pub size: Option<RadialSize>,
    pub position: Option<Position>,
    pub interpolation: Option<ColorInterpolationMethod>,
    pub stops: ColorStopList,
}

/// Try to parse a single `ComponentValue` as a `Length`.
fn try_parse_length(cv: &ComponentValue) -> Result<Length, String> {
    match cv {
        ComponentValue::Token(token) => match &token.kind {
            CssTokenKind::Dimension { value, unit } => {
                let len_unit = unit
                    .parse::<LengthUnit>()
                    .map_err(|_| format!("Invalid length unit: '{}'", unit))?;
                Ok(Length::new(value.to_f64() as f32, len_unit))
            }
            CssTokenKind::Number(value) if value.to_f64() == 0.0 => Ok(Length::new(0.0, LengthUnit::Px)),
            _ => Err(format!("Expected dimension, got {:?}", token.kind)),
        },
        _ => Err("Expected a token for length".to_string()),
    }
}

/// Checks whether the first segment (before the first comma) contains
/// radial-gradient configuration (shape, size, `at`, or `in`) rather than
/// being a color stop.
///
/// Radial config segments can start with:
///   - A shape keyword (`circle`, `ellipse`)
///   - An extent keyword (`closest-side`, `farthest-corner`, etc.)
///   - A dimension (explicit size like `50px`)
///   - A percentage (for ellipse sizes)
///   - The `at` keyword (position)
///   - The `in` keyword (color interpolation)
fn segment_is_radial_config(segment: &[ComponentValue]) -> bool {
    let stripped = strip_whitespace(segment);
    if stripped.is_empty() {
        return false;
    }

    match &stripped[0] {
        ComponentValue::Token(token) => match &token.kind {
            CssTokenKind::Ident(s) => {
                s.eq_ignore_ascii_case("circle")
                    || s.eq_ignore_ascii_case("ellipse")
                    || s.eq_ignore_ascii_case("at")
                    || s.eq_ignore_ascii_case("in")
                    || s.parse::<RadialExtent>().is_ok()
            }
            CssTokenKind::Dimension { .. } => true,
            CssTokenKind::Percentage(_) => true,
            CssTokenKind::Number(n) if n.to_f64() == 0.0 => true,
            _ => false,
        },
        _ => false,
    }
}

/// Parse the radial shape/size/position configuration from a segment.
///
/// CSS grammar for the configuration part:
/// ```text
/// [ [ <radial-shape> || <radial-size> ] [ at <position> ]? ]
/// | [ at <position> ]
/// ```
///
/// Returns `(shape, size, position)`.
fn parse_radial_config(segment: &[ComponentValue]) -> Result<RadialConfig, String> {
    let stripped = strip_whitespace(segment);
    if stripped.is_empty() {
        return Ok((None, None, None));
    }

    let at_pos = find_ident_position(stripped, |s| s.eq_ignore_ascii_case("at"));

    let (shape_size_cvs, position_cvs) = if let Some(pos) = at_pos {
        (&stripped[..pos], Some(&stripped[pos + 1..]))
    } else {
        (stripped, None)
    };

    let mut shape: Option<RadialShape> = None;
    let mut size: Option<RadialSize> = None;

    let tokens = meaningful_cvs(shape_size_cvs);

    let mut length_values: Vec<&ComponentValue> = Vec::new();

    for cv in &tokens {
        match cv {
            ComponentValue::Token(token) => match &token.kind {
                CssTokenKind::Ident(ident) => {
                    if let Ok(s) = ident.parse::<RadialShape>() {
                        if shape.is_some() {
                            return Err("Duplicate shape keyword".to_string());
                        }
                        shape = Some(s);
                    } else if let Ok(extent) = ident.parse::<RadialExtent>() {
                        if size.is_some() {
                            return Err("Duplicate size value".to_string());
                        }
                        size = Some(RadialSize::Extent(extent));
                    } else {
                        return Err(format!("Unexpected ident in radial config: '{}'", ident));
                    }
                }
                CssTokenKind::Dimension { .. } | CssTokenKind::Percentage(_) | CssTokenKind::Number(_) => {
                    length_values.push(cv);
                }
                _ => {
                    return Err(format!("Unexpected token in radial config: {:?}", token.kind));
                }
            },
            _ => {
                return Err("Unexpected non-token in radial config".to_string());
            }
        }
    }

    if !length_values.is_empty() {
        if size.is_some() {
            return Err("Cannot combine extent keyword with explicit size".to_string());
        }
        match length_values.len() {
            1 => {
                let len = try_parse_length(length_values[0])?;
                size = Some(RadialSize::Length(len));
            }
            2 => {
                let lp1 = try_parse_length_percentage(length_values[0])?;
                let lp2 = try_parse_length_percentage(length_values[1])?;
                size = Some(RadialSize::LengthPercentagePair(lp1, lp2));
            }
            n => {
                return Err(format!("Too many size values in radial config (expected 1-2, got {})", n));
            }
        }
    }

    let position = if let Some(pos_cvs) = position_cvs {
        Some(Position::try_from(pos_cvs)?)
    } else {
        None
    };

    Ok((shape, size, position))
}

impl TryFrom<&[ComponentValue]> for RadialGradientSyntax {
    type Error = String;

    /// Parse the inner component values of a `radial-gradient()` function.
    ///
    /// CSS grammar:
    /// ```text
    /// radial-gradient(
    ///   [ [ <radial-shape> || <radial-size> ] [ at <position> ]? ]? ,
    ///   [ in <color-interpolation-method> , ]?
    ///   <color-stop-list>
    /// )
    /// ```
    ///
    /// Strategy:
    /// 1. Split the entire value list on commas.
    /// 2. Examine the first segment – if it looks like radial configuration
    ///    (shape, size, `at <position>`) consume it.
    /// 3. Check the next segment for `in <color-interpolation-method>`.
    /// 4. The remaining segments form the `<color-stop-list>`.
    fn try_from(value: &[ComponentValue]) -> Result<Self, Self::Error> {
        let segments = split_on_commas(value);

        if segments.is_empty() {
            return Err("Empty radial-gradient arguments".into());
        }

        let mut idx = 0;
        let mut shape: Option<RadialShape> = None;
        let mut size: Option<RadialSize> = None;
        let mut position: Option<Position> = None;
        let mut interpolation: Option<ColorInterpolationMethod> = None;

        if idx < segments.len() {
            let seg = &segments[idx];
            let stripped = strip_whitespace(seg);

            if let Ok(Some(method)) = try_consume_interpolation(stripped) {
                interpolation = Some(method);
                idx += 1;
            } else if segment_is_radial_config(seg) {
                let (s, sz, pos) = parse_radial_config(seg)?;
                shape = s;
                size = sz;
                position = pos;
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
            return Err("Missing color stop list in radial-gradient".into());
        }

        let stop_cvs = reassemble_to_comma_separated(&segments[idx..]);
        let stops = ColorStopList::try_from(stop_cvs.as_slice())?;

        Ok(RadialGradientSyntax {
            shape,
            size,
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
    fn radial_two_colors() {
        let cvs = parse_value("background-image: radial-gradient(red, blue)");
        let func = extract_function(&cvs);
        let syn = RadialGradientSyntax::try_from(func.value.as_slice()).unwrap();
        assert!(syn.shape.is_none());
        assert!(syn.size.is_none());
        assert!(syn.position.is_none());
        assert!(syn.interpolation.is_none());
        assert_eq!(syn.stops.rest.len(), 1);
    }

    #[test]
    fn radial_three_colors() {
        let cvs = parse_value("background-image: radial-gradient(red, green, blue)");
        let func = extract_function(&cvs);
        let syn = RadialGradientSyntax::try_from(func.value.as_slice()).unwrap();
        assert_eq!(syn.stops.rest.len(), 2);
    }

    #[test]
    fn radial_circle() {
        let cvs = parse_value("background-image: radial-gradient(circle, red, blue)");
        let func = extract_function(&cvs);
        let syn = RadialGradientSyntax::try_from(func.value.as_slice()).unwrap();
        assert_eq!(syn.shape, Some(RadialShape::Circle));
        assert!(syn.size.is_none());
    }

    #[test]
    fn radial_ellipse() {
        let cvs = parse_value("background-image: radial-gradient(ellipse, red, blue)");
        let func = extract_function(&cvs);
        let syn = RadialGradientSyntax::try_from(func.value.as_slice()).unwrap();
        assert_eq!(syn.shape, Some(RadialShape::Ellipse));
    }

    #[test]
    fn radial_closest_side() {
        let cvs = parse_value("background-image: radial-gradient(closest-side, red, blue)");
        let func = extract_function(&cvs);
        let syn = RadialGradientSyntax::try_from(func.value.as_slice()).unwrap();
        assert_eq!(syn.size, Some(RadialSize::Extent(RadialExtent::ClosestSide)));
    }

    #[test]
    fn radial_farthest_corner() {
        let cvs = parse_value("background-image: radial-gradient(farthest-corner, red, blue)");
        let func = extract_function(&cvs);
        let syn = RadialGradientSyntax::try_from(func.value.as_slice()).unwrap();
        assert_eq!(syn.size, Some(RadialSize::Extent(RadialExtent::FarthestCorner)));
    }

    #[test]
    fn radial_explicit_length() {
        let cvs = parse_value("background-image: radial-gradient(50px, red, blue)");
        let func = extract_function(&cvs);
        let syn = RadialGradientSyntax::try_from(func.value.as_slice()).unwrap();
        assert!(matches!(syn.size, Some(RadialSize::Length(_))));
    }

    #[test]
    fn radial_explicit_two_lengths() {
        let cvs = parse_value("background-image: radial-gradient(50px 100px, red, blue)");
        let func = extract_function(&cvs);
        let syn = RadialGradientSyntax::try_from(func.value.as_slice()).unwrap();
        assert!(matches!(syn.size, Some(RadialSize::LengthPercentagePair(_, _))));
    }

    #[test]
    fn radial_circle_closest_side() {
        let cvs = parse_value("background-image: radial-gradient(circle closest-side, red, blue)");
        let func = extract_function(&cvs);
        let syn = RadialGradientSyntax::try_from(func.value.as_slice()).unwrap();
        assert_eq!(syn.shape, Some(RadialShape::Circle));
        assert_eq!(syn.size, Some(RadialSize::Extent(RadialExtent::ClosestSide)));
    }

    #[test]
    fn radial_at_center() {
        let cvs = parse_value("background-image: radial-gradient(at center, red, blue)");
        let func = extract_function(&cvs);
        let syn = RadialGradientSyntax::try_from(func.value.as_slice()).unwrap();
        assert!(syn.position.is_some());
    }

    #[test]
    fn radial_circle_at_top_left() {
        let cvs = parse_value("background-image: radial-gradient(circle at top left, red, blue)");
        let func = extract_function(&cvs);
        let syn = RadialGradientSyntax::try_from(func.value.as_slice()).unwrap();
        assert_eq!(syn.shape, Some(RadialShape::Circle));
        assert!(syn.position.is_some());
    }

    #[test]
    fn radial_stops_with_percentages() {
        let cvs = parse_value("background-image: radial-gradient(red 0%, blue 100%)");
        let func = extract_function(&cvs);
        let syn = RadialGradientSyntax::try_from(func.value.as_slice()).unwrap();
        assert!(syn.stops.first.length.is_some());
        assert!(syn.stops.rest[0].1.length.is_some());
    }

    #[test]
    fn radial_single_stop_fails() {
        let cvs = parse_value("background-image: radial-gradient(red)");
        let func = extract_function(&cvs);
        assert!(RadialGradientSyntax::try_from(func.value.as_slice()).is_err());
    }

    #[test]
    fn radial_empty_fails() {
        let empty: &[ComponentValue] = &[];
        assert!(RadialGradientSyntax::try_from(empty).is_err());
    }

    #[test]
    fn radial_hex_colors() {
        let cvs = parse_value("background-image: radial-gradient(#ff0000, #0000ff)");
        let func = extract_function(&cvs);
        let syn = RadialGradientSyntax::try_from(func.value.as_slice()).unwrap();
        assert_eq!(syn.stops.rest.len(), 1);
    }

    #[test]
    fn radial_many_stops() {
        let cvs = parse_value("background-image: radial-gradient(red, orange, yellow, green, blue)");
        let func = extract_function(&cvs);
        let syn = RadialGradientSyntax::try_from(func.value.as_slice()).unwrap();
        assert_eq!(syn.stops.rest.len(), 4);
    }
}

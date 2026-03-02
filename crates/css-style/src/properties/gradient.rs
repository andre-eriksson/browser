use css_cssom::{ComponentValue, CssToken, CssTokenKind, Function};

use crate::{
    length::{Length, LengthUnit},
    percentage::LengthPercentage,
    position::{HorizontalSide, VerticalSide},
    primitives::percentage::Percentage,
    properties::gradient::{
        conic::ConicGradientSyntax,
        interpolation::{ColorInterpolationMethod, HueInterpolationMethod, PolarColorSpace},
        linear::LinearGradientSyntax,
        radial::RadialGradientSyntax,
    },
};

pub mod conic;
pub mod interpolation;
pub mod linear;
pub mod radial;
pub mod stops;

/// Split a slice of `ComponentValue` on `CssTokenKind::Comma`, returning the
/// segments between commas.  Each segment is a `Vec<ComponentValue>` that does
/// **not** contain the comma itself.
pub(crate) fn split_on_commas(input: &[ComponentValue]) -> Vec<Vec<ComponentValue>> {
    let mut segments: Vec<Vec<ComponentValue>> = Vec::new();
    let mut current: Vec<ComponentValue> = Vec::new();

    for cv in input {
        if matches!(cv, ComponentValue::Token(t) if matches!(t.kind, CssTokenKind::Comma)) {
            segments.push(std::mem::take(&mut current));
        } else {
            current.push(cv.clone());
        }
    }
    segments.push(current);

    segments
}

/// Strip leading and trailing whitespace tokens from a `ComponentValue` slice.
pub(crate) fn strip_whitespace(cvs: &[ComponentValue]) -> &[ComponentValue] {
    let is_ws =
        |cv: &ComponentValue| matches!(cv, ComponentValue::Token(t) if matches!(t.kind, CssTokenKind::Whitespace));
    let start = cvs.iter().position(|cv| !is_ws(cv)).unwrap_or(cvs.len());
    let end = cvs
        .iter()
        .rposition(|cv| !is_ws(cv))
        .map(|i| i + 1)
        .unwrap_or(0);
    if start >= end { &[] } else { &cvs[start..end] }
}

/// Filter out whitespace `ComponentValue`s, returning references to the
/// meaningful (non-whitespace) ones.
pub(crate) fn meaningful_cvs(cvs: &[ComponentValue]) -> Vec<&ComponentValue> {
    cvs.iter()
        .filter(|cv| !matches!(cv, ComponentValue::Token(t) if matches!(t.kind, CssTokenKind::Whitespace)))
        .collect()
}

/// Try to parse a single `ComponentValue` as a `LengthPercentage`.
pub(crate) fn try_parse_length_percentage(cv: &ComponentValue) -> Result<LengthPercentage, String> {
    match cv {
        ComponentValue::Token(token) => match &token.kind {
            CssTokenKind::Dimension { value, unit } => {
                let len_unit = unit
                    .parse::<LengthUnit>()
                    .map_err(|_| format!("Invalid length unit: '{}'", unit))?;
                Ok(LengthPercentage::Length(Length::new(value.to_f64() as f32, len_unit)))
            }
            CssTokenKind::Percentage(value) => Ok(LengthPercentage::Percentage(Percentage::new(value.to_f64() as f32))),
            CssTokenKind::Number(value) if value.to_f64() == 0.0 => {
                Ok(LengthPercentage::Length(Length::new(0.0, LengthUnit::Px)))
            }
            _ => Err(format!("Expected dimension or percentage, got {:?}", token.kind)),
        },
        _ => Err("Expected a token for length/percentage".to_string()),
    }
}

/// Collect all non-whitespace ident strings from a segment.
pub(crate) fn collect_idents(segment: &[ComponentValue]) -> Vec<String> {
    segment
        .iter()
        .filter_map(|cv| match cv {
            ComponentValue::Token(t) => match &t.kind {
                CssTokenKind::Ident(s) => Some(s.clone()),
                _ => None,
            },
            _ => None,
        })
        .collect()
}

/// Try to parse a `ColorInterpolationMethod` from a segment whose first
/// non-whitespace ident is the color-space keyword (i.e. the `in` keyword has
/// already been identified and stripped by the caller).
///
/// Accepted forms:
///   - `<rectangular-color-space>`
///   - `<polar-color-space> [<hue-interpolation-method> hue]?`
///   - any other single ident → `ColorInterpolationMethod::Custom`
pub(crate) fn try_parse_interpolation(segment: &[ComponentValue]) -> Result<ColorInterpolationMethod, String> {
    let stripped = strip_whitespace(segment);
    let idents = collect_idents(stripped);

    if idents.is_empty() {
        return Err("Empty interpolation segment".into());
    }

    let first = &idents[0];

    if let Ok(rect) = first.parse() {
        return Ok(ColorInterpolationMethod::Rectangular(rect));
    }

    if let Ok(polar) = first.parse::<PolarColorSpace>() {
        let hue_method = if idents.len() >= 2 {
            idents[1].parse::<HueInterpolationMethod>().ok()
        } else {
            None
        };
        return Ok(ColorInterpolationMethod::Polar(polar, hue_method));
    }

    Ok(ColorInterpolationMethod::Custom(first.to_string()))
}

/// Reassemble a set of already-split segments back into a flat
/// `Vec<ComponentValue>` separated by synthetic comma tokens.
///
/// This is used so that the various `TryFrom<&[ComponentValue]>`
/// implementations for stop lists can receive the raw comma-separated form
/// they expect.
pub(crate) fn reassemble_to_comma_separated(segments: &[Vec<ComponentValue>]) -> Vec<ComponentValue> {
    let mut out: Vec<ComponentValue> = Vec::new();
    for (i, seg) in segments.iter().enumerate() {
        if i > 0 {
            out.push(ComponentValue::Token(CssToken {
                kind: CssTokenKind::Comma,
                position: None,
            }));
        }
        out.extend(seg.iter().cloned());
    }
    out
}

/// Find the position of the first ident token matching the given predicate in
/// a (possibly whitespace-containing) slice, returning its index.
pub(crate) fn find_ident_position(cvs: &[ComponentValue], predicate: impl Fn(&str) -> bool) -> Option<usize> {
    cvs.iter().position(|cv| {
        matches!(
            cv,
            ComponentValue::Token(t) if matches!(&t.kind, CssTokenKind::Ident(s) if predicate(s))
        )
    })
}

/// Try to consume an `in <color-interpolation-method>` clause from a stripped
/// segment.  Returns `Some(method)` if the segment starts with the `in`
/// keyword, `None` otherwise.
pub(crate) fn try_consume_interpolation(
    stripped: &[ComponentValue],
) -> Result<Option<ColorInterpolationMethod>, String> {
    let idents = collect_idents(stripped);
    if let Some(first) = idents.first()
        && first.eq_ignore_ascii_case("in")
    {
        let in_pos = find_ident_position(stripped, |s| s.eq_ignore_ascii_case("in")).unwrap();
        let method = try_parse_interpolation(&stripped[in_pos + 1..])?;
        return Ok(Some(method));
    }
    Ok(None)
}

#[derive(Debug, Clone, PartialEq)]
pub struct SideOrCorner {
    pub horizontal: Option<HorizontalSide>,
    pub vertical: Option<VerticalSide>,
}

impl TryFrom<&[ComponentValue]> for SideOrCorner {
    type Error = String;

    /// Parse a `<side-or-corner>` from a slice of `ComponentValue`s.
    ///
    /// The CSS grammar is:
    /// ```text
    /// <side-or-corner> = [ to [ left | right ] || [ top | bottom ] ]
    /// ```
    ///
    /// The `to` keyword is expected to have already been consumed by the caller.
    /// This parses the remaining ident tokens for horizontal (`left`/`right`)
    /// and/or vertical (`top`/`bottom`) sides.
    fn try_from(value: &[ComponentValue]) -> Result<Self, Self::Error> {
        let mut horizontal: Option<HorizontalSide> = None;
        let mut vertical: Option<VerticalSide> = None;

        for cv in value {
            match cv {
                ComponentValue::Token(token) => match &token.kind {
                    CssTokenKind::Ident(ident) => {
                        if let Ok(h) = ident.parse::<HorizontalSide>() {
                            if horizontal.is_some() {
                                return Err("Duplicate horizontal side".to_string());
                            }
                            horizontal = Some(h);
                        } else if let Ok(v) = ident.parse::<VerticalSide>() {
                            if vertical.is_some() {
                                return Err("Duplicate vertical side".to_string());
                            }
                            vertical = Some(v);
                        } else {
                            return Err(format!("Invalid side-or-corner keyword: '{}'", ident));
                        }
                    }
                    CssTokenKind::Whitespace => continue,
                    _ => return Err(format!("Unexpected token in side-or-corner: {:?}", token.kind)),
                },
                _ => return Err("Expected a token in side-or-corner".to_string()),
            }
        }

        if horizontal.is_none() && vertical.is_none() {
            return Err("Expected at least one side keyword (left, right, top, bottom)".to_string());
        }

        Ok(SideOrCorner {
            horizontal,
            vertical,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Gradient {
    Linear(LinearGradientSyntax),
    RepeatingLinear(LinearGradientSyntax),
    Radial(RadialGradientSyntax),
    RepeatingRadial(RadialGradientSyntax),
    Conic(ConicGradientSyntax),
    RepeatingConic(ConicGradientSyntax),
}

impl TryFrom<&Function> for Gradient {
    type Error = String;

    fn try_from(func: &Function) -> Result<Self, Self::Error> {
        match func.name.as_str() {
            "linear-gradient" => Ok(Self::Linear(LinearGradientSyntax::try_from(func.value.as_slice())?)),
            "repeating-linear-gradient" => {
                Ok(Self::RepeatingLinear(LinearGradientSyntax::try_from(func.value.as_slice())?))
            }
            "radial-gradient" => Ok(Self::Radial(RadialGradientSyntax::try_from(func.value.as_slice())?)),
            "repeating-radial-gradient" => {
                Ok(Self::RepeatingRadial(RadialGradientSyntax::try_from(func.value.as_slice())?))
            }
            "conic-gradient" => Ok(Self::Conic(ConicGradientSyntax::try_from(func.value.as_slice())?)),
            "repeating-conic-gradient" => {
                Ok(Self::RepeatingConic(ConicGradientSyntax::try_from(func.value.as_slice())?))
            }
            _ => Err(format!("Unknown gradient function: '{}'", func.name)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use css_cssom::CSSStyleSheet;

    /// Helper: parse an inline CSS declaration and return the component values
    /// for the first declaration found.
    fn parse_value(css: &str) -> Vec<ComponentValue> {
        let decls = CSSStyleSheet::from_inline(css);
        assert!(!decls.is_empty(), "No declarations parsed from: {css}");
        decls[0].original_values.clone()
    }

    /// Helper: extract the first Function from parsed component values.
    fn extract_function(cvs: &[ComponentValue]) -> &Function {
        cvs.iter()
            .find_map(|cv| match cv {
                ComponentValue::Function(f) => Some(f),
                _ => None,
            })
            .expect("No function found in component values")
    }

    #[test]
    fn side_or_corner_left() {
        let cvs = parse_value("x: to left");
        let stripped = strip_whitespace(&cvs);
        let to_pos = stripped
            .iter()
            .position(|cv| matches!(cv, ComponentValue::Token(t) if matches!(&t.kind, CssTokenKind::Ident(s) if s.eq_ignore_ascii_case("to"))))
            .unwrap();
        let after_to = &stripped[to_pos + 1..];
        let sc = SideOrCorner::try_from(after_to).unwrap();
        assert_eq!(sc.horizontal, Some(HorizontalSide::Left));
        assert_eq!(sc.vertical, None);
    }

    #[test]
    fn side_or_corner_top_right() {
        let cvs = parse_value("x: to top right");
        let stripped = strip_whitespace(&cvs);
        let to_pos = stripped
            .iter()
            .position(|cv| matches!(cv, ComponentValue::Token(t) if matches!(&t.kind, CssTokenKind::Ident(s) if s.eq_ignore_ascii_case("to"))))
            .unwrap();
        let after_to = &stripped[to_pos + 1..];
        let sc = SideOrCorner::try_from(after_to).unwrap();
        assert_eq!(sc.horizontal, Some(HorizontalSide::Right));
        assert_eq!(sc.vertical, Some(VerticalSide::Top));
    }

    #[test]
    fn side_or_corner_bottom() {
        let cvs = parse_value("x: to bottom");
        let stripped = strip_whitespace(&cvs);
        let to_pos = stripped
            .iter()
            .position(|cv| matches!(cv, ComponentValue::Token(t) if matches!(&t.kind, CssTokenKind::Ident(s) if s.eq_ignore_ascii_case("to"))))
            .unwrap();
        let after_to = &stripped[to_pos + 1..];
        let sc = SideOrCorner::try_from(after_to).unwrap();
        assert_eq!(sc.horizontal, None);
        assert_eq!(sc.vertical, Some(VerticalSide::Bottom));
    }

    #[test]
    fn side_or_corner_empty_fails() {
        let empty: &[ComponentValue] = &[];
        assert!(SideOrCorner::try_from(empty).is_err());
    }

    #[test]
    fn gradient_linear() {
        let cvs = parse_value("background-image: linear-gradient(red, blue)");
        let func = extract_function(&cvs);
        let grad = Gradient::try_from(func).unwrap();
        assert!(matches!(grad, Gradient::Linear(_)));
    }

    #[test]
    fn gradient_repeating_linear() {
        let cvs = parse_value("background-image: repeating-linear-gradient(red, blue)");
        let func = extract_function(&cvs);
        let grad = Gradient::try_from(func).unwrap();
        assert!(matches!(grad, Gradient::RepeatingLinear(_)));
    }

    #[test]
    fn gradient_radial() {
        let cvs = parse_value("background-image: radial-gradient(red, blue)");
        let func = extract_function(&cvs);
        let grad = Gradient::try_from(func).unwrap();
        assert!(matches!(grad, Gradient::Radial(_)));
    }

    #[test]
    fn gradient_repeating_radial() {
        let cvs = parse_value("background-image: repeating-radial-gradient(red, blue)");
        let func = extract_function(&cvs);
        let grad = Gradient::try_from(func).unwrap();
        assert!(matches!(grad, Gradient::RepeatingRadial(_)));
    }

    #[test]
    fn gradient_conic() {
        let cvs = parse_value("background-image: conic-gradient(red, blue)");
        let func = extract_function(&cvs);
        let grad = Gradient::try_from(func).unwrap();
        assert!(matches!(grad, Gradient::Conic(_)));
    }

    #[test]
    fn gradient_repeating_conic() {
        let cvs = parse_value("background-image: repeating-conic-gradient(red, blue)");
        let func = extract_function(&cvs);
        let grad = Gradient::try_from(func).unwrap();
        assert!(matches!(grad, Gradient::RepeatingConic(_)));
    }

    #[test]
    fn gradient_unknown_function_fails() {
        let cvs = parse_value("background-image: fancy-gradient(red, blue)");
        let func = extract_function(&cvs);
        assert!(Gradient::try_from(func).is_err());
    }

    #[test]
    fn gradient_linear_with_direction() {
        let cvs = parse_value("background-image: linear-gradient(to right, red, blue)");
        let func = extract_function(&cvs);
        let grad = Gradient::try_from(func).unwrap();
        if let Gradient::Linear(syn) = grad {
            assert!(syn.direction.is_some());
        } else {
            panic!("Expected Gradient::Linear");
        }
    }

    #[test]
    fn gradient_linear_with_angle() {
        let cvs = parse_value("background-image: linear-gradient(45deg, red, blue)");
        let func = extract_function(&cvs);
        let grad = Gradient::try_from(func).unwrap();
        if let Gradient::Linear(syn) = grad {
            assert!(syn.direction.is_some());
        } else {
            panic!("Expected Gradient::Linear");
        }
    }

    #[test]
    fn split_on_commas_basic() {
        let cvs = parse_value("x: red, blue, green");
        let segments = split_on_commas(&cvs);
        assert_eq!(segments.len(), 3);
    }

    #[test]
    fn strip_whitespace_trims() {
        let cvs = parse_value("x: hello");
        let stripped = strip_whitespace(&cvs);
        assert!(!stripped.is_empty());
        assert!(!matches!(&stripped[0], ComponentValue::Token(t) if matches!(t.kind, CssTokenKind::Whitespace)));
    }

    #[test]
    fn try_parse_length_percentage_dimension() {
        let cvs = parse_value("x: 10px");
        let stripped = strip_whitespace(&cvs);
        let meaningful = meaningful_cvs(stripped);
        let lp = try_parse_length_percentage(meaningful[0]).unwrap();
        assert!(matches!(lp, crate::percentage::LengthPercentage::Length(_)));
    }

    #[test]
    fn try_parse_length_percentage_pct() {
        let cvs = parse_value("x: 50%");
        let stripped = strip_whitespace(&cvs);
        let meaningful = meaningful_cvs(stripped);
        let lp = try_parse_length_percentage(meaningful[0]).unwrap();
        assert!(matches!(lp, crate::percentage::LengthPercentage::Percentage(_)));
    }

    #[test]
    fn try_parse_length_percentage_zero() {
        let cvs = parse_value("x: 0");
        let stripped = strip_whitespace(&cvs);
        let meaningful = meaningful_cvs(stripped);
        let lp = try_parse_length_percentage(meaningful[0]).unwrap();
        assert!(matches!(lp, crate::percentage::LengthPercentage::Length(_)));
    }
}

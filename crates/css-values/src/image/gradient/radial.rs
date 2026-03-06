use css_cssom::{ComponentValue, ComponentValueStream, CssTokenKind};
use strum::EnumString;

use crate::{
    CSSParsable,
    combination::LengthPercentage,
    image::{
        Gradient,
        gradient::{interpolation::ColorInterpolationMethod, stops::ColorStopList},
    },
    position::Position,
    quantity::{Length, LengthUnit},
};

#[derive(Debug, Clone, PartialEq, EnumString)]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
pub enum RadialShape {
    Circle,
    Ellipse,
}

#[derive(Debug, Clone, PartialEq, EnumString)]
#[strum(serialize_all = "kebab-case", ascii_case_insensitive)]
pub enum RadialExtent {
    ClosestCorner,
    ClosestSide,
    FarthestCorner,
    FarthestSide,
}

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

/// Parsing helpers for `RadialGradientSyntax`.
impl RadialGradientSyntax {
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
        let stripped = Gradient::strip_whitespace(segment);
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
        let stripped = Gradient::strip_whitespace(segment);
        if stripped.is_empty() {
            return Ok((None, None, None));
        }

        let at_pos = Gradient::find_ident_position(stripped, |s| s.eq_ignore_ascii_case("at"));

        let (shape_size_cvs, position_cvs) = if let Some(pos) = at_pos {
            (&stripped[..pos], Some(&stripped[pos + 1..]))
        } else {
            (stripped, None)
        };

        let mut shape: Option<RadialShape> = None;
        let mut size: Option<RadialSize> = None;

        let tokens = Gradient::meaningful_cvs(shape_size_cvs);

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
                    let len = Self::try_parse_length(length_values[0])?;
                    size = Some(RadialSize::Length(len));
                }
                2 => {
                    let lp1 = Gradient::try_parse_length_percentage(length_values[0])?;
                    let lp2 = Gradient::try_parse_length_percentage(length_values[1])?;
                    size = Some(RadialSize::LengthPercentagePair(lp1, lp2));
                }
                n => {
                    return Err(format!("Too many size values in radial config (expected 1-2, got {})", n));
                }
            }
        }

        let position = if let Some(pos_cvs) = position_cvs {
            Some(Position::parse(&mut pos_cvs.into())?)
        } else {
            None
        };

        Ok((shape, size, position))
    }
}

impl CSSParsable for RadialGradientSyntax {
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
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, String> {
        let segments = Gradient::split_on_commas(stream.remaining());

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
            let stripped = Gradient::strip_whitespace(seg);

            if let Ok(Some(method)) = Gradient::try_consume_interpolation(stripped) {
                interpolation = Some(method);
                idx += 1;
            } else if Self::segment_is_radial_config(seg) {
                let (s, sz, pos) = Self::parse_radial_config(seg)?;
                shape = s;
                size = sz;
                position = pos;
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
            return Err("Missing color stop list in radial-gradient".into());
        }

        let stop_cvs = Gradient::reassemble_to_comma_separated(&segments[idx..]);
        let stops = ColorStopList::parse(&mut stop_cvs.as_slice().into())?;

        Ok(RadialGradientSyntax {
            shape,
            size,
            position,
            interpolation,
            stops,
        })
    }
}

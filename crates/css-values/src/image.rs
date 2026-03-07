use css_cssom::{ComponentValue, ComponentValueStream, CssToken, CssTokenKind, Function};

use crate::{
    CSSParsable,
    combination::{AnglePercentage, AnglePercentageZero, LengthPercentage},
    image::gradient::{
        conic::ConicGradientSyntax,
        interpolation::{ColorInterpolationMethod, HueInterpolationMethod, PolarColorSpace},
        linear::LinearGradientSyntax,
        radial::RadialGradientSyntax,
    },
    numeric::Percentage,
    quantity::{Angle, Length, LengthUnit},
};

pub mod gradient;

/// Represents the various types of gradients in CSS, including linear, radial, and conic gradients,
/// as well as their repeating variants. Each variant holds the specific syntax structure for that
/// gradient type.
#[derive(Debug, Clone, PartialEq)]
pub enum Gradient {
    Linear(LinearGradientSyntax),
    RepeatingLinear(LinearGradientSyntax),
    Radial(RadialGradientSyntax),
    RepeatingRadial(RadialGradientSyntax),
    Conic(ConicGradientSyntax),
    RepeatingConic(ConicGradientSyntax),
}

impl Gradient {
    /// Parse a gradient from a `Function` node directly.
    ///
    /// This is the core dispatch that maps function names to their
    /// respective syntax parsers.
    pub fn parse_function(func: &Function) -> Result<Self, String> {
        if func.name.eq_ignore_ascii_case("linear-gradient") {
            Ok(Self::Linear(LinearGradientSyntax::parse(&mut func.value.as_slice().into())?))
        } else if func.name.eq_ignore_ascii_case("repeating-linear-gradient") {
            Ok(Self::RepeatingLinear(LinearGradientSyntax::parse(&mut func.value.as_slice().into())?))
        } else if func.name.eq_ignore_ascii_case("radial-gradient") {
            Ok(Self::Radial(RadialGradientSyntax::parse(&mut func.value.as_slice().into())?))
        } else if func.name.eq_ignore_ascii_case("repeating-radial-gradient") {
            Ok(Self::RepeatingRadial(RadialGradientSyntax::parse(&mut func.value.as_slice().into())?))
        } else if func.name.eq_ignore_ascii_case("conic-gradient") {
            Ok(Self::Conic(ConicGradientSyntax::parse(&mut func.value.as_slice().into())?))
        } else if func.name.eq_ignore_ascii_case("repeating-conic-gradient") {
            Ok(Self::RepeatingConic(ConicGradientSyntax::parse(&mut func.value.as_slice().into())?))
        } else {
            Err(format!("Unknown gradient function: '{}'", func.name))
        }
    }

    /// Split a slice of `ComponentValue` on `CssTokenKind::Comma`, returning
    /// the segments between commas.  Each segment is a `Vec<ComponentValue>`
    /// that does **not** contain the comma itself.
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

    /// Strip leading and trailing whitespace tokens from a `ComponentValue`
    /// slice.
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
                CssTokenKind::Percentage(value) => {
                    Ok(LengthPercentage::Percentage(Percentage::new(value.to_f64() as f32)))
                }
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
    /// non-whitespace ident is the color-space keyword (i.e. the `in` keyword
    /// has already been identified and stripped by the caller).
    ///
    /// Accepted forms:
    ///   - `<rectangular-color-space>`
    ///   - `<polar-color-space> [<hue-interpolation-method> hue]?`
    ///   - any other single ident → `ColorInterpolationMethod::Custom`
    pub(crate) fn try_parse_interpolation(segment: &[ComponentValue]) -> Result<ColorInterpolationMethod, String> {
        let stripped = Self::strip_whitespace(segment);
        let idents = Self::collect_idents(stripped);

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
    /// This is used so that the various `CSSParsable::try_parse`
    /// implementations for stop lists can receive the raw comma-separated
    /// form they expect.
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

    /// Find the position of the first ident token matching the given predicate
    /// in a (possibly whitespace-containing) slice, returning its index.
    pub(crate) fn find_ident_position(cvs: &[ComponentValue], predicate: impl Fn(&str) -> bool) -> Option<usize> {
        cvs.iter().position(|cv| {
            matches!(
                cv,
                ComponentValue::Token(t) if matches!(&t.kind, CssTokenKind::Ident(s) if predicate(s))
            )
        })
    }

    /// Try to consume an `in <color-interpolation-method>` clause from a
    /// stripped segment.  Returns `Some(method)` if the segment starts with
    /// the `in` keyword, `None` otherwise.
    pub(crate) fn try_consume_interpolation(
        stripped: &[ComponentValue],
    ) -> Result<Option<ColorInterpolationMethod>, String> {
        let idents = Self::collect_idents(stripped);
        if let Some(first) = idents.first()
            && first.eq_ignore_ascii_case("in")
        {
            let in_pos = Self::find_ident_position(stripped, |s| s.eq_ignore_ascii_case("in")).unwrap();
            let method = Self::try_parse_interpolation(&stripped[in_pos + 1..])?;
            return Ok(Some(method));
        }
        Ok(None)
    }

    /// Try to parse a single `ComponentValue` as an `AnglePercentageOrZero`.
    pub(crate) fn try_parse_angle_percentage_or_zero(cv: &ComponentValue) -> Result<AnglePercentageZero, String> {
        match cv {
            ComponentValue::Token(token) => match &token.kind {
                CssTokenKind::Dimension { .. } | CssTokenKind::Number(_) => {
                    let angle = Angle::try_from(token)?;
                    Ok(AnglePercentageZero::from(AnglePercentage::Angle(angle)))
                }
                CssTokenKind::Percentage(value) => {
                    let pct = Percentage::new(value.to_f64() as f32);
                    Ok(AnglePercentageZero::from(AnglePercentage::Percentage(pct)))
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
    pub(crate) fn color_cv_count(segment: &[ComponentValue]) -> Option<usize> {
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
    pub(crate) fn segment_is_color_stop(segment: &[ComponentValue]) -> bool {
        Self::color_cv_count(segment).is_some()
    }
}

impl CSSParsable for Gradient {
    /// Parse a gradient from a `ComponentValueStream`.
    ///
    /// Expects the stream to contain a gradient function
    /// (`linear-gradient(…)`, `radial-gradient(…)`, etc.).
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, String> {
        stream.skip_whitespace();

        if let Some(cv) = stream.peek()
            && let ComponentValue::Function(func) = cv
        {
            let result = Self::parse_function(func)?;
            stream.next_cv();
            return Ok(result);
        }

        Err("Expected a gradient function".to_string())
    }
}

/// Represents the various ways to specify an image in CSS, including URLs, gradients,
/// and more complex constructs like cross-fades and image sets. Each variant holds the
/// specific data structure relevant to that type of image.
#[derive(Debug, Clone, PartialEq)]
pub enum Image {
    None,
    Url(String),
    Gradient(Gradient),
    // TODO: Element()
    // TODO: Image()
    // TODO: CrossFade()
    // TODO: ImageSet()
    // TODO: Paint()
}

impl TryFrom<&Function> for Image {
    type Error = String;

    fn try_from(value: &Function) -> Result<Self, Self::Error> {
        if let Ok(gradient) = Gradient::parse_function(value) {
            Ok(Image::Gradient(gradient))
        } else if value.name.eq_ignore_ascii_case("url") {
            if let Some(ComponentValue::Token(token)) = value.value.first() {
                if let CssTokenKind::String(s) = &token.kind {
                    Ok(Image::Url(s.clone()))
                } else {
                    Err("Expected a string token in url() function".to_string())
                }
            } else {
                Err("Expected at least one argument in url() function".to_string())
            }
        } else {
            Err(format!("Unknown image function: '{}'", value.name))
        }
    }
}

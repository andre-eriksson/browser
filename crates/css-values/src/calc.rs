use std::str::FromStr;

use css_cssom::{AssociatedToken, ComponentValue, ComponentValueStream, CssTokenKind};

use crate::{
    error::CssValueError,
    numeric::Percentage,
    quantity::{Angle, Dimension, Frequency, Length, Resolution, Time},
};

/// The set of CSS Calc function names that can be parsed as calc expressions.
const MATH_FUNCTION_NAMES: &[&str] = &["calc", "min", "max", "clamp"];

/// Returns true if the given function name is a CSS Calc function (calc, min, max, clamp).
#[must_use]
pub fn is_math_function(name: &str) -> bool {
    MATH_FUNCTION_NAMES
        .iter()
        .any(|n| name.eq_ignore_ascii_case(n))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalcDomain {
    Number,
    Length,
    Angle,
    Time,
    Frequency,
    Resolution,

    /// Should **always** resolve to a fraction of 1 (e.g. 50% resolves to 0.5).
    Percentage,
    All,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CalcKind {
    Number(f64),
    Length(Length),
    Angle(Angle),
    Time(Time),
    Frequency(Frequency),
    Resolution(Resolution),
    Percentage(Percentage),
}

/// Represents the special keywords that can be used in `calc()` expressions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalcKeyword {
    E,
    PI,
    Infinity,
    NegativeInfinity,
    NaN,
}

impl CalcKeyword {
    #[must_use]
    pub const fn to_f64(self) -> f64 {
        match self {
            Self::E => std::f64::consts::E,
            Self::PI => std::f64::consts::PI,
            Self::Infinity => f64::INFINITY,
            Self::NegativeInfinity => f64::NEG_INFINITY,
            Self::NaN => f64::NAN,
        }
    }
}

impl FromStr for CalcKeyword {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "e" => Ok(Self::E),
            "pi" => Ok(Self::PI),
            "infinity" => Ok(Self::Infinity),
            "-infinity" => Ok(Self::NegativeInfinity),
            "nan" => Ok(Self::NaN),
            _ => Err(format!("Invalid calculate keyword: {s}")),
        }
    }
}

/// Represents the arguments to a `clamp()` function, which consists of an optional minimum value, a required value, and an optional maximum value.
#[derive(Debug, Clone, PartialEq)]
pub struct ClampArgs {
    pub min: Option<Box<CalcSum>>,
    pub val: Box<CalcSum>,
    pub max: Option<Box<CalcSum>>,
}

/// Represents a single value in a `calc()` expression, which can be a number, length, percentage, keyword, or a nested `calc()` expression.
#[derive(Debug, Clone, PartialEq)]
pub enum CalcValue {
    Number(f64),
    Dimension(Dimension),
    Percentage(Percentage),
    Keyword(CalcKeyword),
    NestedSum(Box<CalcSum>),
    Min(Vec<CalcSum>),
    Max(Vec<CalcSum>),
    Clamp(ClampArgs),
}

impl CalcDomain {
    /// Checks if this domain is a dimension (not a number, percentage, or All).
    pub fn is_dimension(&self) -> bool {
        !matches!(self, Self::Number | Self::Percentage | Self::All)
    }

    /// Attempts to combine two domains, returning the resulting domain if they are compatible.
    /// This is used for addition, subtraction, min, max, and clamp.
    pub fn combine(&self, other: &Self) -> Option<Self> {
        if self == other {
            Some(*self)
        } else if *self == CalcDomain::Percentage && other.is_dimension() {
            Some(*other)
        } else if self.is_dimension() && *other == CalcDomain::Percentage {
            Some(*self)
        } else if *self == CalcDomain::All {
            Some(*other)
        } else if *other == CalcDomain::All {
            Some(*self)
        } else {
            None
        }
    }
}

impl CalcValue {
    pub fn kind(self) -> Result<CalcKind, CssValueError> {
        match self {
            Self::Number(n) => Ok(CalcKind::Number(n)),
            Self::Keyword(k) => Ok(CalcKind::Number(k.to_f64())),
            Self::Percentage(pct) => Ok(CalcKind::Percentage(pct)),
            Self::Dimension(dim) => match dim {
                Dimension::Length(l) => Ok(CalcKind::Length(l)),
                Dimension::Angle(a) => Ok(CalcKind::Angle(a)),
                Dimension::Time(t) => Ok(CalcKind::Time(t)),
                Dimension::Frequency(f) => Ok(CalcKind::Frequency(f)),
                Dimension::Resolution(r) => Ok(CalcKind::Resolution(r)),
            },
            Self::NestedSum(_) | Self::Min(_) | Self::Max(_) | Self::Clamp(_) => Err(CssValueError::InvalidValue(
                "Cannot determine kind of a nested calc() expression without evaluating it".into(),
            )),
        }
    }

    pub fn evaluate(&self) -> Result<(f64, CalcDomain), CssValueError> {
        match self {
            Self::Number(n) => Ok((*n, CalcDomain::Number)),
            Self::Keyword(k) => Ok((k.to_f64(), CalcDomain::Number)),
            Self::Percentage(pct) => Ok((pct.as_fraction(), CalcDomain::Percentage)),
            Self::Dimension(dim) => match dim {
                Dimension::Length(_) => Err(CssValueError::InvalidValue(
                    "Cannot evaluate a length dimension without context (PixelRepr)".into(),
                )),
                Dimension::Angle(a) => Ok((a.to_degrees(), CalcDomain::Angle)),
                Dimension::Time(t) => Ok((t.to_seconds(), CalcDomain::Time)),
                Dimension::Frequency(f) => Ok((f.to_hertz(), CalcDomain::Frequency)),
                Dimension::Resolution(r) => Ok((r.to_dppx(), CalcDomain::Resolution)),
            },
            Self::NestedSum(sum) => sum.evaluate(),
            Self::Min(args) => {
                let mut min_val = f64::INFINITY;
                let mut min_domain = CalcDomain::Number;

                for arg in args {
                    let (val, domain) = arg.evaluate()?;
                    if !matches!(domain, CalcDomain::Number | CalcDomain::Percentage | CalcDomain::All) {
                        return Err(CssValueError::InvalidValue(format!(
                            "min() arguments must be numbers, percentages, or All, but got {domain:?}"
                        )));
                    }

                    min_domain = min_domain.combine(&domain).unwrap_or(CalcDomain::Number);

                    if val < min_val {
                        min_val = val;
                    }
                }

                Ok((min_val, min_domain))
            }

            Self::Max(args) => {
                let mut max_val = f64::NEG_INFINITY;
                let mut max_domain = CalcDomain::Number;

                for arg in args {
                    let (val, domain) = arg.evaluate()?;
                    if !matches!(domain, CalcDomain::Number | CalcDomain::Percentage | CalcDomain::All) {
                        return Err(CssValueError::InvalidValue(format!(
                            "max() arguments must be numbers, percentages, or All, but got {domain:?}"
                        )));
                    }

                    max_domain = max_domain.combine(&domain).unwrap_or(CalcDomain::Number);

                    if val > max_val {
                        max_val = val;
                    }
                }

                Ok((max_val, max_domain))
            }
            Self::Clamp(args) => {
                let min_val = args
                    .min
                    .as_ref()
                    .map_or(Ok((f64::NEG_INFINITY, CalcDomain::Number)), |s| s.evaluate())?;

                let val_val = args.val.evaluate()?;
                let max_val = args
                    .max
                    .as_ref()
                    .map_or(Ok((f64::INFINITY, CalcDomain::Number)), |s| s.evaluate())?;

                Ok((val_val.0.clamp(min_val.0, max_val.0), val_val.1))
            }
        }
    }

    pub fn resolve_type(&self) -> Result<CalcDomain, CssValueError> {
        match self {
            CalcValue::Number(_) => Ok(CalcDomain::Number),
            CalcValue::Dimension(dim) => match dim {
                Dimension::Length(_) => Ok(CalcDomain::Length),
                Dimension::Angle(_) => Ok(CalcDomain::Angle),
                Dimension::Time(_) => Ok(CalcDomain::Time),
                Dimension::Frequency(_) => Ok(CalcDomain::Frequency),
                Dimension::Resolution(_) => Ok(CalcDomain::Resolution),
            },
            CalcValue::Percentage(_) => Ok(CalcDomain::Percentage),
            CalcValue::Keyword(_) => Ok(CalcDomain::Number),
            CalcValue::NestedSum(sum) => sum.resolve_domain(),
            CalcValue::Min(args) | CalcValue::Max(args) => {
                let mut domain = args[0].resolve_domain()?;
                for arg in args.iter().skip(1) {
                    let arg_domain = arg.resolve_domain()?;
                    domain = domain.combine(&arg_domain).ok_or_else(|| {
                        CssValueError::InvalidValue(format!(
                            "All arguments to min() and max() must be compatible, but got {domain:?} and {arg_domain:?}"
                        ))
                    })?;
                }
                Ok(domain)
            }
            CalcValue::Clamp(args) => {
                let mut domain = args.val.resolve_domain()?;

                if let Some(min) = &args.min {
                    let min_domain = min.resolve_domain()?;
                    domain = domain.combine(&min_domain).ok_or_else(|| {
                        CssValueError::InvalidValue(format!(
                            "clamp() min argument must be compatible with value, but got {min_domain:?} and {domain:?}"
                        ))
                    })?;
                }

                if let Some(max) = &args.max {
                    let max_domain = max.resolve_domain()?;
                    domain = domain.combine(&max_domain).ok_or_else(|| {
                        CssValueError::InvalidValue(format!(
                            "clamp() max argument must be compatible with value, but got {max_domain:?} and {domain:?}"
                        ))
                    })?;
                }

                Ok(domain)
            }
        }
    }
}

/// Represents a product of values in a `calc()` expression, which can be a single value, a multiplication, or a division.
#[derive(Debug, Clone, PartialEq)]
pub enum CalcProduct {
    Value(CalcValue),
    Multiply(Box<Self>, Box<Self>),
    Divide(Box<Self>, Box<Self>),
}

impl CalcProduct {
    pub fn kind(self) -> Result<CalcKind, CssValueError> {
        match self {
            Self::Value(val) => val.kind(),
            Self::Multiply(l, r) => {
                let left_kind = l.kind()?;
                let right_kind = r.kind()?;

                match (left_kind, right_kind) {
                    (CalcKind::Number(l), CalcKind::Number(r)) => Ok(CalcKind::Number(l * r)),
                    (CalcKind::Number(n), CalcKind::Percentage(p)) | (CalcKind::Percentage(p), CalcKind::Number(n)) => {
                        Ok(CalcKind::Percentage(Percentage::new(n * p.as_fraction())))
                    }
                    (CalcKind::Percentage(p1), CalcKind::Percentage(p2)) => {
                        Ok(CalcKind::Percentage(Percentage::new(p1.as_fraction() * p2.as_fraction())))
                    }
                    _ => Err(CssValueError::InvalidValue(format!(
                        "At least one side of a multiplication must be a number or percentage, but got {left_kind:?} * {right_kind:?}"
                    ))),
                }
            }
            Self::Divide(l, r) => {
                let left_kind = l.kind()?;
                let right_kind = r.kind()?;

                match (left_kind, right_kind) {
                    (CalcKind::Number(l), CalcKind::Number(r)) => {
                        if r == 0.0 {
                            Err(CssValueError::InvalidValue("Division by zero in calc()".into()))
                        } else {
                            Ok(CalcKind::Number(l / r))
                        }
                    }
                    (CalcKind::Number(n), CalcKind::Percentage(p)) => {
                        if p.as_fraction() == 0.0 {
                            Err(CssValueError::InvalidValue("Division by zero in calc()".into()))
                        } else {
                            Ok(CalcKind::Percentage(Percentage::new(n / p.as_fraction())))
                        }
                    }
                    (CalcKind::Percentage(p), CalcKind::Number(n)) => {
                        if n == 0.0 {
                            Err(CssValueError::InvalidValue("Division by zero in calc()".into()))
                        } else {
                            Ok(CalcKind::Percentage(Percentage::new(p.as_fraction() / n)))
                        }
                    }
                    _ => Err(CssValueError::InvalidValue(format!(
                        "The right side of a division must be a number or percentage, but got {right_kind:?}"
                    ))),
                }
            }
        }
    }

    pub fn evaluate(&self) -> Result<(f64, CalcDomain), CssValueError> {
        match self {
            Self::Value(v) => v.evaluate(),
            Self::Multiply(l, r) => {
                let (left_val, left_domain) = l.evaluate()?;
                let (right_val, right_domain) = r.evaluate()?;

                match (left_domain, right_domain) {
                    (CalcDomain::Number, d) | (d, CalcDomain::Number) => Ok((left_val * right_val, d)),
                    _ => Err(CssValueError::InvalidValue(format!(
                        "At least one side of a multiplication must be a number, but got {left_domain:?} * {right_domain:?}"
                    ))),
                }
            }
            Self::Divide(l, r) => {
                let divisor = r.evaluate()?;
                if divisor.0 == 0.0 {
                    Err(CssValueError::InvalidValue("Division by zero in calc()".into()))
                } else {
                    let dividend = l.evaluate()?;
                    Ok((dividend.0 / divisor.0, dividend.1))
                }
            }
        }
    }

    pub fn resolve_domain(&self) -> Result<CalcDomain, CssValueError> {
        match self {
            CalcProduct::Value(val) => val.resolve_type(),
            CalcProduct::Multiply(left, right) => {
                let left_domain = left.resolve_domain()?;
                let right_domain = right.resolve_domain()?;

                match (left_domain, right_domain) {
                    (CalcDomain::Number, CalcDomain::Number) => Ok(CalcDomain::Number),
                    (d, CalcDomain::Number) | (CalcDomain::Number, d) => Ok(d),
                    _ => Err(CssValueError::InvalidValue(format!(
                        "At least one side of a multiplication must be a number, but got {left_domain:?} * {right_domain:?}"
                    ))),
                }
            }
            CalcProduct::Divide(left, right) => {
                let left_domain = left.resolve_domain()?;
                let right_domain = right.resolve_domain()?;

                if right_domain != CalcDomain::Number {
                    return Err(CssValueError::InvalidValue(format!(
                        "The right side of a division must be a number, but got {right_domain:?}"
                    )));
                }
                Ok(left_domain)
            }
        }
    }
}

/// Represents a sum of products in a `calc()` expression, which can be a single product, an addition, or a subtraction.
#[derive(Debug, Clone, PartialEq)]
pub enum CalcSum {
    Product(CalcProduct),
    Add(Box<Self>, Box<Self>),
    Subtract(Box<Self>, Box<Self>),
}

impl CalcSum {
    pub fn kind(self) -> Result<CalcKind, CssValueError> {
        match self {
            Self::Product(p) => p.kind(),
            Self::Add(l, r) => {
                let left_kind = l.kind()?;
                let right_kind = r.kind()?;

                if left_kind == right_kind {
                    Ok(left_kind)
                } else {
                    Err(CssValueError::InvalidValue(format!(
                        "Cannot add or subtract incompatible types in calc(): {left_kind:?} + {right_kind:?}"
                    )))
                }
            }
            Self::Subtract(l, r) => {
                let left_kind = l.kind()?;
                let right_kind = r.kind()?;

                if left_kind == right_kind {
                    Ok(left_kind)
                } else {
                    Err(CssValueError::InvalidValue(format!(
                        "Cannot add or subtract incompatible types in calc(): {left_kind:?} - {right_kind:?}"
                    )))
                }
            }
        }
    }

    pub fn evaluate(&self) -> Result<(f64, CalcDomain), CssValueError> {
        match self {
            Self::Product(p) => p.evaluate(),
            Self::Add(l, r) => {
                let (left_val, left_domain) = l.evaluate()?;
                let (right_val, right_domain) = r.evaluate()?;

                if let Some(result_domain) = left_domain.combine(&right_domain) {
                    Ok((left_val + right_val, result_domain))
                } else {
                    Err(CssValueError::InvalidValue(format!(
                        "Cannot add incompatible types in calc(): {left_domain:?} + {right_domain:?}"
                    )))
                }
            }
            Self::Subtract(l, r) => {
                let (left_val, left_domain) = l.evaluate()?;
                let (right_val, right_domain) = r.evaluate()?;

                if let Some(result_domain) = left_domain.combine(&right_domain) {
                    Ok((left_val - right_val, result_domain))
                } else {
                    Err(CssValueError::InvalidValue(format!(
                        "Cannot subtract incompatible types in calc(): {left_domain:?} - {right_domain:?}"
                    )))
                }
            }
        }
    }

    pub fn resolve_domain(&self) -> Result<CalcDomain, CssValueError> {
        match self {
            CalcSum::Product(product) => product.resolve_domain(),
            CalcSum::Add(left, right) | CalcSum::Subtract(left, right) => {
                let left_domain = left.resolve_domain()?;
                let right_domain = right.resolve_domain()?;

                left_domain.combine(&right_domain).ok_or_else(|| {
                    CssValueError::InvalidValue(format!(
                        "Invalid addition/subtraction between incompatible types: {left_domain:?} and {right_domain:?}"
                    ))
                })
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CalcExpression {
    sum: CalcSum,
}

impl CalcExpression {
    /// Parse any CSS Calc function (calc, min, max, clamp) from its inner component values and function name.
    /// This dispatches to the appropriate parser based on the function name.
    pub fn parse(name: &str, value: &[ComponentValue]) -> Result<Self, CssValueError> {
        if name.eq_ignore_ascii_case("calc") {
            Self::parse_calc(value)
        } else if name.eq_ignore_ascii_case("min") {
            let args = Self::parse_comma_separated_sums(value)?;
            if args.is_empty() {
                return Err(CssValueError::InvalidValue("min() requires at least one argument".into()));
            }
            Ok(Self {
                sum: CalcSum::Product(CalcProduct::Value(CalcValue::Min(args))),
            })
        } else if name.eq_ignore_ascii_case("max") {
            let args = Self::parse_comma_separated_sums(value)?;
            if args.is_empty() {
                return Err(CssValueError::InvalidValue("max() requires at least one argument".into()));
            }
            Ok(Self {
                sum: CalcSum::Product(CalcProduct::Value(CalcValue::Max(args))),
            })
        } else if name.eq_ignore_ascii_case("clamp") {
            let args = Self::parse_clamp_args(value)?;
            Ok(Self {
                sum: CalcSum::Product(CalcProduct::Value(CalcValue::Clamp(args))),
            })
        } else {
            Err(CssValueError::InvalidFunction(format!("Math function: {name}")))
        }
    }

    /// Resolves the type of the calc expression, returning an error if it is invalid (e.g. adding a length to a time).
    pub fn resolve_domain(&self) -> Result<CalcDomain, CssValueError> {
        self.sum.resolve_domain()
    }

    /// Evaluates a context-free calc expression into a canonical f64 value.
    /// Returns degrees for Angles, seconds for Time, Hertz for Frequency, etc.
    ///
    /// Panics if called on a Length (which requires PixelRepr context).
    pub fn evaluate(&self) -> Result<(f64, CalcDomain), CssValueError> {
        self.sum.evaluate()
    }

    /// Returns a reference to the root `CalcSum` of this expression, which represents the entire parsed expression tree.
    pub const fn sum(&self) -> &CalcSum {
        &self.sum
    }

    pub fn into_sum(self) -> CalcSum {
        self.sum
    }

    /// Splits a token slice on `CssTokenKind::Comma`, returning the segments between commas.
    fn split_on_commas(input: &[ComponentValue]) -> Vec<Vec<ComponentValue>> {
        let mut segments: Vec<Vec<ComponentValue>> = Vec::new();
        let mut current_segment: Vec<ComponentValue> = Vec::new();

        for cv in input {
            if matches!(cv, ComponentValue::Token(t) if matches!(t.kind, CssTokenKind::Comma)) {
                segments.push(std::mem::take(&mut current_segment));
            } else {
                current_segment.push(cv.clone());
            }
        }
        if !current_segment.is_empty() {
            segments.push(current_segment);
        }

        segments
    }

    /// Skips whitespace in the stream and returns whether any was consumed.
    fn skip_whitespace_check(stream: &mut ComponentValueStream) -> bool {
        let checkpoint = stream.checkpoint();
        stream.skip_whitespace();
        stream.checkpoint() > checkpoint
    }

    /// Peeks at the current token kind without consuming it.
    fn peek_token_kind<'css>(stream: &'css ComponentValueStream) -> Option<&'css CssTokenKind> {
        if let Some(ComponentValue::Token(token)) = stream.peek() {
            Some(&token.kind)
        } else {
            None
        }
    }

    fn parse_sum(stream: &mut ComponentValueStream) -> Result<CalcSum, CssValueError> {
        let mut left = CalcSum::Product(Self::parse_product(stream)?);

        loop {
            let had_whitespace_before = Self::skip_whitespace_check(stream);
            if stream.peek().is_none() {
                break;
            }

            let is_plus_or_minus = matches!(Self::peek_token_kind(stream), Some(CssTokenKind::Delim('+' | '-')));
            if !is_plus_or_minus {
                break;
            }

            if !had_whitespace_before {
                return Err(CssValueError::InvalidValue(
                    "Whitespace is required before '+' or '-' operator in calc()".into(),
                ));
            }

            let op = match stream.next_cv() {
                Some(ComponentValue::Token(token)) => match &token.kind {
                    CssTokenKind::Delim('+') => '+',
                    CssTokenKind::Delim('-') => '-',
                    _ => break,
                },
                _ => break,
            };

            let had_whitespace_after = Self::skip_whitespace_check(stream);
            if !had_whitespace_after {
                return Err(CssValueError::InvalidValue(
                    "Whitespace is required after '+' or '-' operator in calc()".into(),
                ));
            }

            let next_product = Self::parse_product(stream)?;
            let right = CalcSum::Product(next_product);

            if op == '+' {
                left = CalcSum::Add(Box::new(left), Box::new(right));
            } else {
                left = CalcSum::Subtract(Box::new(left), Box::new(right));
            }
        }

        Ok(left)
    }

    fn parse_product(stream: &mut ComponentValueStream) -> Result<CalcProduct, CssValueError> {
        let mut left = CalcProduct::Value(Self::parse_value(stream)?);

        loop {
            let checkpoint_before_ws = stream.checkpoint();
            stream.skip_whitespace();

            if stream.peek().is_none() {
                break;
            }

            let op_char = match Self::peek_token_kind(stream) {
                Some(CssTokenKind::Delim('*')) => '*',
                Some(CssTokenKind::Delim('/')) => '/',
                _ => {
                    stream.restore(checkpoint_before_ws);
                    break;
                }
            };

            stream.next_cv();
            stream.skip_whitespace();

            let next_value = Self::parse_value(stream)?;
            let right = CalcProduct::Value(next_value);

            if op_char == '*' {
                left = CalcProduct::Multiply(Box::new(left), Box::new(right));
            } else {
                left = CalcProduct::Divide(Box::new(left), Box::new(right));
            }
        }

        Ok(left)
    }

    /// Parse a `<calc-sum>` from a flat list of component values (i.e. the contents inside a `calc()` function).
    fn parse_calc(input: &[ComponentValue]) -> Result<Self, CssValueError> {
        let mut stream = ComponentValueStream::new(input);
        let sum = Self::parse_sum(&mut stream)?;

        stream.skip_whitespace();
        if stream.peek().is_some() {
            return Err(CssValueError::UnexpectedRemainingInput);
        }

        Ok(Self { sum })
    }

    fn parse_value(stream: &mut ComponentValueStream) -> Result<CalcValue, CssValueError> {
        let Some(cv) = stream.next_non_whitespace() else {
            return Err(CssValueError::UnexpectedEndOfInput);
        };

        match cv {
            ComponentValue::Function(func) if func.name.eq_ignore_ascii_case("calc") => {
                let func = func.clone();
                let nested = Self::parse_calc(&func.value)?;
                Ok(CalcValue::NestedSum(Box::new(nested.sum)))
            }

            ComponentValue::Function(func) if func.name.eq_ignore_ascii_case("min") => {
                let func = func.clone();
                let args = Self::parse_comma_separated_sums(&func.value)?;
                if args.is_empty() {
                    return Err(CssValueError::InvalidValue("min() requires at least one argument".into()));
                }
                Ok(CalcValue::Min(args))
            }

            ComponentValue::Function(func) if func.name.eq_ignore_ascii_case("max") => {
                let func = func.clone();
                let args = Self::parse_comma_separated_sums(&func.value)?;
                if args.is_empty() {
                    return Err(CssValueError::InvalidValue("max() requires at least one argument".into()));
                }
                Ok(CalcValue::Max(args))
            }

            ComponentValue::Function(func) if func.name.eq_ignore_ascii_case("clamp") => {
                let func = func.clone();
                let args = Self::parse_clamp_args(&func.value)?;
                Ok(CalcValue::Clamp(args))
            }

            ComponentValue::SimpleBlock(block) if matches!(block.associated_token, AssociatedToken::Parenthesis) => {
                let block = block.clone();
                let nested = Self::parse_calc(&block.value)?;
                Ok(CalcValue::NestedSum(Box::new(nested.sum)))
            }

            ComponentValue::Token(token) => match &token.kind {
                CssTokenKind::Number(num) => {
                    let val = num.to_f64();
                    Ok(CalcValue::Number(val))
                }

                CssTokenKind::Dimension { value, unit } => {
                    let dimension = Dimension::parse(value, unit)?;
                    Ok(CalcValue::Dimension(dimension))
                }

                CssTokenKind::Percentage(num) => {
                    let val = num.to_f64();
                    Ok(CalcValue::Percentage(Percentage::new(val)))
                }

                CssTokenKind::Ident(ident) => CalcKeyword::from_str(ident)
                    .map(CalcValue::Keyword)
                    .map_err(|_| CssValueError::InvalidValue(format!("Invalid calc() keyword: {ident}"))),

                _ => Err(CssValueError::InvalidToken(token.kind.clone())),
            },
            cvs => Err(CssValueError::InvalidComponentValue(cvs.clone())),
        }
    }

    /// Parses the three comma-separated arguments of `clamp()`, where the first and third
    /// may be the keyword `none` (meaning no bound on that side).
    ///
    /// `<clamp()> = clamp( [ <calc-sum> | none ] , <calc-sum> , [ <calc-sum> | none ] )`
    fn parse_clamp_args(input: &[ComponentValue]) -> Result<ClampArgs, CssValueError> {
        let segments = Self::split_on_commas(input);
        if segments.len() != 3 {
            return Err(CssValueError::InvalidValue(format!(
                "clamp() requires exactly 3 arguments separated by commas, got {}",
                segments.len()
            )));
        }

        let min = Self::parse_clamp_bound(&segments[0])?;
        let val = Self::parse_calc(&segments[1])
            .map(|e| Box::new(e.sum))
            .map_err(|e| CssValueError::InvalidValue(format!("Invalid clamp() value argument: {e}")))?;
        let max = Self::parse_clamp_bound(&segments[2])?;

        Ok(ClampArgs { min, val, max })
    }

    /// Parses a single `clamp()` bound, which is either `none` or a `<calc-sum>`.
    fn parse_clamp_bound(segment: &[ComponentValue]) -> Result<Option<Box<CalcSum>>, CssValueError> {
        let mut stream = ComponentValueStream::new(segment);
        stream.skip_whitespace();

        if let Some(ComponentValue::Token(token)) = stream.peek()
            && let CssTokenKind::Ident(ident) = &token.kind
            && ident.eq_ignore_ascii_case("none")
        {
            stream.next_cv();
            if !stream.has_remaining_tokens() {
                return Ok(None);
            }
        }

        Self::parse_calc(segment)
            .map(|e| Some(Box::new(e.sum)))
            .map_err(|e| CssValueError::InvalidValue(format!("Invalid clamp() bound argument: {e}")))
    }

    /// Parses comma-separated `<calc-sum>` arguments from a function's value tokens.
    /// Used for `min()` and `max()` which take `<calc-sum>#` arguments.
    fn parse_comma_separated_sums(input: &[ComponentValue]) -> Result<Vec<CalcSum>, CssValueError> {
        let segments = Self::split_on_commas(input);

        let mut sums = Vec::with_capacity(segments.len());
        for segment in &segments {
            let expr = Self::parse_calc(segment)?;
            sums.push(expr.sum);
        }

        Ok(sums)
    }
}

#[cfg(test)]
mod tests {
    use crate::quantity::Length;

    use super::*;

    #[test]
    fn test_is_math_function() {
        assert!(is_math_function("calc"));
        assert!(is_math_function("min"));
        assert!(is_math_function("max"));
        assert!(is_math_function("clamp"));
        assert!(is_math_function("CALC"));
        assert!(!is_math_function("not-a-math-function"));
    }

    #[test]
    fn test_calc_keyword_from_str() {
        assert_eq!(CalcKeyword::from_str("e").unwrap(), CalcKeyword::E);
        assert_eq!(CalcKeyword::from_str("PI").unwrap(), CalcKeyword::PI);
        assert_eq!(CalcKeyword::from_str("infinity").unwrap(), CalcKeyword::Infinity);
        assert_eq!(CalcKeyword::from_str("-infinity").unwrap(), CalcKeyword::NegativeInfinity);
        assert_eq!(CalcKeyword::from_str("NaN").unwrap(), CalcKeyword::NaN);
        assert!(CalcKeyword::from_str("invalid").is_err());
    }

    #[test]
    fn test_calc_keyword_to_f64() {
        assert_eq!(CalcKeyword::E.to_f64(), std::f64::consts::E);
        assert_eq!(CalcKeyword::PI.to_f64(), std::f64::consts::PI);
        assert_eq!(CalcKeyword::Infinity.to_f64(), f64::INFINITY);
        assert_eq!(CalcKeyword::NegativeInfinity.to_f64(), f64::NEG_INFINITY);
        assert!(CalcKeyword::NaN.to_f64().is_nan());
    }

    #[test]
    fn test_calc_domain_combine() {
        assert_eq!(CalcDomain::Number.combine(&CalcDomain::Number), Some(CalcDomain::Number));
        assert_eq!(CalcDomain::Length.combine(&CalcDomain::Length), Some(CalcDomain::Length));
        assert_eq!(CalcDomain::Percentage.combine(&CalcDomain::Percentage), Some(CalcDomain::Percentage));
        assert_eq!(CalcDomain::Length.combine(&CalcDomain::Percentage), Some(CalcDomain::Length));
        assert_eq!(CalcDomain::Percentage.combine(&CalcDomain::Length), Some(CalcDomain::Length));
        assert_eq!(CalcDomain::All.combine(&CalcDomain::Length), Some(CalcDomain::Length));
        assert_eq!(CalcDomain::Length.combine(&CalcDomain::All), Some(CalcDomain::Length));
        assert_eq!(CalcDomain::Length.combine(&CalcDomain::Angle), None);
    }

    #[test]
    fn test_calc_value_resolve_type() {
        let val = CalcValue::Number(42.0);
        assert_eq!(val.resolve_type().unwrap(), CalcDomain::Number);

        let val = CalcValue::Dimension(Dimension::Length(Length::px(100.0)));
        assert_eq!(val.resolve_type().unwrap(), CalcDomain::Length);

        let val = CalcValue::Percentage(Percentage::new(50.0));
        assert_eq!(val.resolve_type().unwrap(), CalcDomain::Percentage);

        let val = CalcValue::Keyword(CalcKeyword::PI);
        assert_eq!(val.resolve_type().unwrap(), CalcDomain::Number);
    }

    #[test]
    fn test_calc_type() {
        let expr = CalcExpression {
            sum: CalcSum::Add(
                Box::new(CalcSum::Product(CalcProduct::Value(CalcValue::Dimension(Dimension::Length(Length::px(
                    100.0,
                )))))),
                Box::new(CalcSum::Product(CalcProduct::Value(CalcValue::Percentage(Percentage::new(50.0))))),
            ),
        };
        assert_eq!(expr.resolve_domain().unwrap(), CalcDomain::Length);
    }
}

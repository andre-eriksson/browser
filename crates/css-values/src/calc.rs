use std::str::FromStr;

use css_cssom::{AssociatedToken, ComponentValue, ComponentValueStream, CssTokenKind};

use crate::{
    numeric::Percentage,
    quantity::{Length, LengthUnit},
};

/// The set of CSS Calc function names that can be parsed as calc expressions.
const MATH_FUNCTION_NAMES: &[&str] = &["calc", "min", "max", "clamp"];

/// Returns true if the given function name is a CSS Calc function (calc, min, max, clamp).
pub fn is_math_function(name: &str) -> bool {
    MATH_FUNCTION_NAMES
        .iter()
        .any(|n| name.eq_ignore_ascii_case(n))
}

/// Represents the special keywords that can be used in calc() expressions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CalcKeyword {
    E,
    PI,
    Infinity,
    NegativeInfinity,
    NaN,
}

impl CalcKeyword {
    pub fn to_f32(self) -> f32 {
        match self {
            CalcKeyword::E => std::f32::consts::E,
            CalcKeyword::PI => std::f32::consts::PI,
            CalcKeyword::Infinity => f32::INFINITY,
            CalcKeyword::NegativeInfinity => f32::NEG_INFINITY,
            CalcKeyword::NaN => f32::NAN,
        }
    }
}

impl FromStr for CalcKeyword {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "e" => Ok(CalcKeyword::E),
            "pi" => Ok(CalcKeyword::PI),
            "infinity" => Ok(CalcKeyword::Infinity),
            "-infinity" => Ok(CalcKeyword::NegativeInfinity),
            "nan" => Ok(CalcKeyword::NaN),
            _ => Err(format!("Invalid calculate keyword: {}", s)),
        }
    }
}

/// Represents the arguments to a clamp() function, which consists of an optional minimum value, a required value, and an optional maximum value.
#[derive(Debug, Clone, PartialEq)]
pub struct ClampArgs {
    pub min: Option<Box<CalcSum>>,
    pub val: Box<CalcSum>,
    pub max: Option<Box<CalcSum>>,
}

/// Represents a single value in a calc() expression, which can be a number, length, percentage, keyword, or a nested calc() expression.
#[derive(Debug, Clone, PartialEq)]
pub enum CalcValue {
    Number(f32),
    Length(Length),
    Percentage(Percentage),
    Keyword(CalcKeyword),
    NestedSum(Box<CalcSum>),
    Min(Vec<CalcSum>),
    Max(Vec<CalcSum>),
    Clamp(ClampArgs),
}

/// Represents a product of values in a calc() expression, which can be a single value, a multiplication, or a division.
#[derive(Debug, Clone, PartialEq)]
pub enum CalcProduct {
    Value(CalcValue),
    Multiply(Box<CalcProduct>, Box<CalcProduct>),
    Divide(Box<CalcProduct>, Box<CalcProduct>),
}

/// Represents a sum of products in a calc() expression, which can be a single product, an addition, or a subtraction.
#[derive(Debug, Clone, PartialEq)]
pub enum CalcSum {
    Product(CalcProduct),
    Add(Box<CalcSum>, Box<CalcSum>),
    Subtract(Box<CalcSum>, Box<CalcSum>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct CalcExpression {
    pub sum: CalcSum,
}

impl CalcExpression {
    /// Parse a `<calc-sum>` from a flat list of component values (i.e. the contents inside a `calc()` function).
    pub fn parse(input: &[ComponentValue]) -> Result<Self, String> {
        let mut stream = ComponentValueStream::new(input);
        let sum = Self::parse_sum(&mut stream)?;

        stream.skip_whitespace();
        if stream.peek().is_some() {
            return Err("Unexpected trailing input in calc()".to_string());
        }

        Ok(CalcExpression { sum })
    }

    /// Parse any CSS Calc function (calc, min, max, clamp) from its inner component values and function name.
    /// This dispatches to the appropriate parser based on the function name.
    pub fn parse_math_function(name: &str, value: &[ComponentValue]) -> Result<Self, String> {
        if name.eq_ignore_ascii_case("calc") {
            Self::parse(value)
        } else if name.eq_ignore_ascii_case("min") {
            let args = Self::parse_comma_separated_sums(value)?;
            if args.is_empty() {
                return Err("min() requires at least one argument".to_string());
            }
            Ok(CalcExpression {
                sum: CalcSum::Product(CalcProduct::Value(CalcValue::Min(args))),
            })
        } else if name.eq_ignore_ascii_case("max") {
            let args = Self::parse_comma_separated_sums(value)?;
            if args.is_empty() {
                return Err("max() requires at least one argument".to_string());
            }
            Ok(CalcExpression {
                sum: CalcSum::Product(CalcProduct::Value(CalcValue::Max(args))),
            })
        } else if name.eq_ignore_ascii_case("clamp") {
            let args = Self::parse_clamp_args(value)?;
            Ok(CalcExpression {
                sum: CalcSum::Product(CalcProduct::Value(CalcValue::Clamp(args))),
            })
        } else {
            Err(format!("Unknown Calc function: {}", name))
        }
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
    fn peek_token_kind<'a>(stream: &'a ComponentValueStream) -> Option<&'a CssTokenKind> {
        if let Some(ComponentValue::Token(token)) = stream.peek() {
            Some(&token.kind)
        } else {
            None
        }
    }

    fn parse_sum(stream: &mut ComponentValueStream) -> Result<CalcSum, String> {
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
                return Err("Whitespace is required before '+' or '-' operator in calc()".to_string());
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
                return Err("Whitespace is required after '+' or '-' operator in calc()".to_string());
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

    fn parse_product(stream: &mut ComponentValueStream) -> Result<CalcProduct, String> {
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

    fn parse_value(stream: &mut ComponentValueStream) -> Result<CalcValue, String> {
        stream.skip_whitespace();

        let Some(cv) = stream.peek() else {
            return Err("Unexpected end of input".to_string());
        };

        match cv {
            ComponentValue::Function(func) if func.name.eq_ignore_ascii_case("calc") => {
                let func = func.clone();
                stream.next_cv();
                let nested = CalcExpression::parse(&func.value)?;
                Ok(CalcValue::NestedSum(Box::new(nested.sum)))
            }

            ComponentValue::Function(func) if func.name.eq_ignore_ascii_case("min") => {
                let func = func.clone();
                stream.next_cv();
                let args = Self::parse_comma_separated_sums(&func.value)?;
                if args.is_empty() {
                    return Err("min() requires at least one argument".to_string());
                }
                Ok(CalcValue::Min(args))
            }

            ComponentValue::Function(func) if func.name.eq_ignore_ascii_case("max") => {
                let func = func.clone();
                stream.next_cv();
                let args = Self::parse_comma_separated_sums(&func.value)?;
                if args.is_empty() {
                    return Err("max() requires at least one argument".to_string());
                }
                Ok(CalcValue::Max(args))
            }

            ComponentValue::Function(func) if func.name.eq_ignore_ascii_case("clamp") => {
                let func = func.clone();
                stream.next_cv();
                let args = Self::parse_clamp_args(&func.value)?;
                Ok(CalcValue::Clamp(args))
            }

            ComponentValue::SimpleBlock(block) if matches!(block.associated_token, AssociatedToken::Parenthesis) => {
                let block = block.clone();
                stream.next_cv();
                let nested = CalcExpression::parse(&block.value)?;
                Ok(CalcValue::NestedSum(Box::new(nested.sum)))
            }

            ComponentValue::Token(token) => match &token.kind {
                CssTokenKind::Number(num) => {
                    let val = num.to_f64() as f32;
                    stream.next_cv();
                    Ok(CalcValue::Number(val))
                }

                CssTokenKind::Dimension { value, unit } => {
                    let val = value.to_f64() as f32;
                    let len_unit = unit
                        .parse::<LengthUnit>()
                        .map_err(|_| format!("Invalid length unit: {}", unit))?;
                    stream.next_cv();
                    Ok(CalcValue::Length(Length::new(val, len_unit)))
                }

                CssTokenKind::Percentage(num) => {
                    let val = num.to_f64() as f32;
                    stream.next_cv();
                    Ok(CalcValue::Percentage(Percentage::new(val)))
                }

                CssTokenKind::Ident(ident) => {
                    let result = CalcKeyword::from_str(ident)
                        .map(CalcValue::Keyword)
                        .map_err(|_| format!("Invalid calc() keyword or identifier: {}", ident));
                    if result.is_ok() {
                        stream.next_cv();
                    }
                    result
                }

                _ => Err(format!("Unexpected token in calc(): {:?}", token.kind)),
            },

            _ => Err(format!("Unexpected component value in calc(): {:?}", cv)),
        }
    }

    /// Parses the three comma-separated arguments of `clamp()`, where the first and third
    /// may be the keyword `none` (meaning no bound on that side).
    ///
    /// `<clamp()> = clamp( [ <calc-sum> | none ] , <calc-sum> , [ <calc-sum> | none ] )`
    fn parse_clamp_args(input: &[ComponentValue]) -> Result<ClampArgs, String> {
        let segments = Self::split_on_commas(input);
        if segments.len() != 3 {
            return Err(format!("clamp() requires exactly 3 arguments, got {}", segments.len()));
        }

        let min = Self::parse_clamp_bound(&segments[0])?;
        let val = CalcExpression::parse(&segments[1])
            .map(|e| Box::new(e.sum))
            .map_err(|e| format!("Invalid clamp() value argument: {}", e))?;
        let max = Self::parse_clamp_bound(&segments[2])?;

        Ok(ClampArgs { min, val, max })
    }

    /// Parses a single clamp() bound, which is either `none` or a `<calc-sum>`.
    fn parse_clamp_bound(segment: &[ComponentValue]) -> Result<Option<Box<CalcSum>>, String> {
        let mut stream = ComponentValueStream::new(segment);
        stream.skip_whitespace();

        if let Some(ComponentValue::Token(token)) = stream.peek()
            && let CssTokenKind::Ident(ident) = &token.kind
            && ident.eq_ignore_ascii_case("none")
        {
            stream.next_cv();
            stream.skip_whitespace();
            if stream.peek().is_none() {
                return Ok(None);
            }
        }

        CalcExpression::parse(segment)
            .map(|e| Some(Box::new(e.sum)))
            .map_err(|e| format!("Invalid clamp() bound: {}", e))
    }

    /// Parses comma-separated `<calc-sum>` arguments from a function's value tokens.
    /// Used for `min()` and `max()` which take `<calc-sum>#` arguments.
    fn parse_comma_separated_sums(input: &[ComponentValue]) -> Result<Vec<CalcSum>, String> {
        let segments = Self::split_on_commas(input);

        let mut sums = Vec::with_capacity(segments.len());
        for segment in &segments {
            let expr = CalcExpression::parse(segment)?;
            sums.push(expr.sum);
        }

        Ok(sums)
    }
}

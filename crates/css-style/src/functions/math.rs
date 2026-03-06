use std::str::FromStr;

use css_cssom::{AssociatedToken, ComponentValue, ComponentValueStream, CssTokenKind};

use crate::{
    length::LengthUnit,
    primitives::{length::Length, percentage::Percentage},
    properties::{AbsoluteContext, RelativeContext, RelativeType},
};

/// Represents the special keywords that can be used in calc() expressions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MathKeyword {
    E,
    PI,
    Infinity,
    NegativeInfinity,
    NaN,
}

impl MathKeyword {
    pub fn to_f32(self) -> f32 {
        match self {
            MathKeyword::E => std::f32::consts::E,
            MathKeyword::PI => std::f32::consts::PI,
            MathKeyword::Infinity => f32::INFINITY,
            MathKeyword::NegativeInfinity => f32::NEG_INFINITY,
            MathKeyword::NaN => f32::NAN,
        }
    }
}

impl FromStr for MathKeyword {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "e" => Ok(MathKeyword::E),
            "pi" => Ok(MathKeyword::PI),
            "infinity" => Ok(MathKeyword::Infinity),
            "-infinity" => Ok(MathKeyword::NegativeInfinity),
            "nan" => Ok(MathKeyword::NaN),
            _ => Err(format!("Invalid calculate keyword: {}", s)),
        }
    }
}

/// Represents the arguments to a clamp() function, which consists of an optional minimum value, a required value, and an optional maximum value.
#[derive(Debug, Clone, PartialEq)]
pub struct ClampArgs {
    pub min: Option<Box<MathSum>>,
    pub val: Box<MathSum>,
    pub max: Option<Box<MathSum>>,
}

/// Represents a single value in a calc() expression, which can be a number, length, percentage, keyword, or a nested calc() expression.
#[derive(Debug, Clone, PartialEq)]
pub enum MathValue {
    Number(f32),
    Length(Length),
    Percentage(Percentage),
    Keyword(MathKeyword),
    NestedSum(Box<MathSum>),
    Min(Vec<MathSum>),
    Max(Vec<MathSum>),
    Clamp(ClampArgs),
}

impl MathValue {
    pub fn to_px(&self, rel_type: Option<RelativeType>, rel_ctx: &RelativeContext, abs_ctx: &AbsoluteContext) -> f32 {
        match self {
            MathValue::Number(n) => *n,
            MathValue::Length(l) => l.to_px(rel_ctx, abs_ctx),
            MathValue::Keyword(k) => k.to_f32(),
            MathValue::NestedSum(sum) => sum.to_px(rel_type, rel_ctx, abs_ctx),
            MathValue::Percentage(p) => match rel_type {
                Some(RelativeType::FontSize) => rel_ctx.parent.font_size * p.as_fraction(),
                Some(RelativeType::ParentHeight) => rel_ctx.parent.intrinsic_height * p.as_fraction(),
                Some(RelativeType::ParentWidth) => rel_ctx.parent.intrinsic_width * p.as_fraction(),
                Some(RelativeType::RootFontSize) => abs_ctx.root_font_size * p.as_fraction(),
                Some(RelativeType::ViewportHeight) => abs_ctx.viewport_height * p.as_fraction(),
                Some(RelativeType::ViewportWidth) => abs_ctx.viewport_width * p.as_fraction(),
                None => 0.0,
            },
            MathValue::Min(args) => args
                .iter()
                .map(|sum| sum.to_px(rel_type, rel_ctx, abs_ctx))
                .fold(f32::INFINITY, f32::min),
            MathValue::Max(args) => args
                .iter()
                .map(|sum| sum.to_px(rel_type, rel_ctx, abs_ctx))
                .fold(f32::NEG_INFINITY, f32::max),
            MathValue::Clamp(args) => {
                let min_val = args
                    .min
                    .as_ref()
                    .map_or(f32::NEG_INFINITY, |s| s.to_px(rel_type, rel_ctx, abs_ctx));
                let val_val = args.val.to_px(rel_type, rel_ctx, abs_ctx);
                let max_val = args
                    .max
                    .as_ref()
                    .map_or(f32::INFINITY, |s| s.to_px(rel_type, rel_ctx, abs_ctx));
                val_val.clamp(min_val, max_val)
            }
        }
    }
}

/// Represents a product of values in a calc() expression, which can be a single value, a multiplication, or a division.
#[derive(Debug, Clone, PartialEq)]
pub enum MathProduct {
    Value(MathValue),
    Multiply(Box<MathProduct>, Box<MathProduct>),
    Divide(Box<MathProduct>, Box<MathProduct>),
}

impl MathProduct {
    pub fn to_px(&self, rel_type: Option<RelativeType>, rel_ctx: &RelativeContext, abs_ctx: &AbsoluteContext) -> f32 {
        match self {
            MathProduct::Value(v) => v.to_px(rel_type, rel_ctx, abs_ctx),
            MathProduct::Multiply(left, right) => {
                left.to_px(rel_type, rel_ctx, abs_ctx) * right.to_px(rel_type, rel_ctx, abs_ctx)
            }
            MathProduct::Divide(left, right) => {
                let divisor = right.to_px(rel_type, rel_ctx, abs_ctx);
                if divisor == 0.0 {
                    f32::NAN
                } else {
                    left.to_px(rel_type, rel_ctx, abs_ctx) / divisor
                }
            }
        }
    }
}

/// Represents a sum of products in a calc() expression, which can be a single product, an addition, or a subtraction.
#[derive(Debug, Clone, PartialEq)]
pub enum MathSum {
    Product(MathProduct),
    Add(Box<MathSum>, Box<MathSum>),
    Subtract(Box<MathSum>, Box<MathSum>),
}

impl MathSum {
    pub fn to_px(&self, rel_type: Option<RelativeType>, rel_ctx: &RelativeContext, abs_ctx: &AbsoluteContext) -> f32 {
        match self {
            MathSum::Product(p) => p.to_px(rel_type, rel_ctx, abs_ctx),
            MathSum::Add(left, right) => {
                left.to_px(rel_type, rel_ctx, abs_ctx) + right.to_px(rel_type, rel_ctx, abs_ctx)
            }
            MathSum::Subtract(left, right) => {
                left.to_px(rel_type, rel_ctx, abs_ctx) - right.to_px(rel_type, rel_ctx, abs_ctx)
            }
        }
    }
}

/// Represents a CSS calc() expression (or any CSS math function: calc, min, max, clamp).
/// The top-level structure is a sum of products, which allows for proper operator precedence and associativity when evaluating the expression.
#[derive(Debug, Clone, PartialEq)]
pub struct MathExpression {
    pub sum: MathSum,
}

/// The set of CSS math function names that can be parsed as calc expressions.
const MATH_FUNCTION_NAMES: &[&str] = &["calc", "min", "max", "clamp"];

/// Returns true if the given function name is a CSS math function (calc, min, max, clamp).
pub fn is_math_function(name: &str) -> bool {
    MATH_FUNCTION_NAMES
        .iter()
        .any(|n| name.eq_ignore_ascii_case(n))
}

impl MathExpression {
    pub fn to_px(&self, rel_type: Option<RelativeType>, rel_ctx: &RelativeContext, abs_ctx: &AbsoluteContext) -> f32 {
        self.sum.to_px(rel_type, rel_ctx, abs_ctx)
    }

    /// Parse a `<calc-sum>` from a flat list of component values (i.e. the contents inside a `calc()` function).
    pub fn parse(input: &[ComponentValue]) -> Result<Self, String> {
        let mut stream = ComponentValueStream::new(input);
        let sum = Self::parse_sum(&mut stream)?;

        stream.skip_whitespace();
        if stream.peek().is_some() {
            return Err("Unexpected trailing input in calc()".to_string());
        }

        Ok(MathExpression { sum })
    }

    /// Parse any CSS math function (calc, min, max, clamp) from its inner component values and function name.
    /// This dispatches to the appropriate parser based on the function name.
    pub fn parse_math_function(name: &str, value: &[ComponentValue]) -> Result<Self, String> {
        if name.eq_ignore_ascii_case("calc") {
            Self::parse(value)
        } else if name.eq_ignore_ascii_case("min") {
            let args = Self::parse_comma_separated_sums(value)?;
            if args.is_empty() {
                return Err("min() requires at least one argument".to_string());
            }
            Ok(MathExpression {
                sum: MathSum::Product(MathProduct::Value(MathValue::Min(args))),
            })
        } else if name.eq_ignore_ascii_case("max") {
            let args = Self::parse_comma_separated_sums(value)?;
            if args.is_empty() {
                return Err("max() requires at least one argument".to_string());
            }
            Ok(MathExpression {
                sum: MathSum::Product(MathProduct::Value(MathValue::Max(args))),
            })
        } else if name.eq_ignore_ascii_case("clamp") {
            let args = Self::parse_clamp_args(value)?;
            Ok(MathExpression {
                sum: MathSum::Product(MathProduct::Value(MathValue::Clamp(args))),
            })
        } else {
            Err(format!("Unknown math function: {}", name))
        }
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

    fn parse_sum(stream: &mut ComponentValueStream) -> Result<MathSum, String> {
        let mut left = MathSum::Product(Self::parse_product(stream)?);

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
            let right = MathSum::Product(next_product);

            if op == '+' {
                left = MathSum::Add(Box::new(left), Box::new(right));
            } else {
                left = MathSum::Subtract(Box::new(left), Box::new(right));
            }
        }

        Ok(left)
    }

    fn parse_product(stream: &mut ComponentValueStream) -> Result<MathProduct, String> {
        let mut left = MathProduct::Value(Self::parse_value(stream)?);

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
            let right = MathProduct::Value(next_value);

            if op_char == '*' {
                left = MathProduct::Multiply(Box::new(left), Box::new(right));
            } else {
                left = MathProduct::Divide(Box::new(left), Box::new(right));
            }
        }

        Ok(left)
    }

    fn parse_value(stream: &mut ComponentValueStream) -> Result<MathValue, String> {
        stream.skip_whitespace();

        let Some(cv) = stream.peek() else {
            return Err("Unexpected end of input".to_string());
        };

        match cv {
            ComponentValue::Function(func) if func.name.eq_ignore_ascii_case("calc") => {
                let func = func.clone();
                stream.next_cv();
                let nested = MathExpression::parse(&func.value)?;
                Ok(MathValue::NestedSum(Box::new(nested.sum)))
            }

            ComponentValue::Function(func) if func.name.eq_ignore_ascii_case("min") => {
                let func = func.clone();
                stream.next_cv();
                let args = Self::parse_comma_separated_sums(&func.value)?;
                if args.is_empty() {
                    return Err("min() requires at least one argument".to_string());
                }
                Ok(MathValue::Min(args))
            }

            ComponentValue::Function(func) if func.name.eq_ignore_ascii_case("max") => {
                let func = func.clone();
                stream.next_cv();
                let args = Self::parse_comma_separated_sums(&func.value)?;
                if args.is_empty() {
                    return Err("max() requires at least one argument".to_string());
                }
                Ok(MathValue::Max(args))
            }

            ComponentValue::Function(func) if func.name.eq_ignore_ascii_case("clamp") => {
                let func = func.clone();
                stream.next_cv();
                let args = Self::parse_clamp_args(&func.value)?;
                Ok(MathValue::Clamp(args))
            }

            ComponentValue::SimpleBlock(block) if matches!(block.associated_token, AssociatedToken::Parenthesis) => {
                let block = block.clone();
                stream.next_cv();
                let nested = MathExpression::parse(&block.value)?;
                Ok(MathValue::NestedSum(Box::new(nested.sum)))
            }

            ComponentValue::Token(token) => match &token.kind {
                CssTokenKind::Number(num) => {
                    let val = num.to_f64() as f32;
                    stream.next_cv();
                    Ok(MathValue::Number(val))
                }

                CssTokenKind::Dimension { value, unit } => {
                    let val = value.to_f64() as f32;
                    let len_unit = unit
                        .parse::<LengthUnit>()
                        .map_err(|_| format!("Invalid length unit: {}", unit))?;
                    stream.next_cv();
                    Ok(MathValue::Length(Length::new(val, len_unit)))
                }

                CssTokenKind::Percentage(num) => {
                    let val = num.to_f64() as f32;
                    stream.next_cv();
                    Ok(MathValue::Percentage(Percentage::new(val)))
                }

                CssTokenKind::Ident(ident) => {
                    let result = MathKeyword::from_str(ident)
                        .map(MathValue::Keyword)
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
        let segments = split_on_commas(input);
        if segments.len() != 3 {
            return Err(format!("clamp() requires exactly 3 arguments, got {}", segments.len()));
        }

        let min = Self::parse_clamp_bound(&segments[0])?;
        let val = MathExpression::parse(&segments[1])
            .map(|e| Box::new(e.sum))
            .map_err(|e| format!("Invalid clamp() value argument: {}", e))?;
        let max = Self::parse_clamp_bound(&segments[2])?;

        Ok(ClampArgs { min, val, max })
    }

    /// Parses a single clamp() bound, which is either `none` or a `<calc-sum>`.
    fn parse_clamp_bound(segment: &[ComponentValue]) -> Result<Option<Box<MathSum>>, String> {
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

        MathExpression::parse(segment)
            .map(|e| Some(Box::new(e.sum)))
            .map_err(|e| format!("Invalid clamp() bound: {}", e))
    }

    /// Parses comma-separated `<calc-sum>` arguments from a function's value tokens.
    /// Used for `min()` and `max()` which take `<calc-sum>#` arguments.
    fn parse_comma_separated_sums(input: &[ComponentValue]) -> Result<Vec<MathSum>, String> {
        let segments = split_on_commas(input);

        let mut sums = Vec::with_capacity(segments.len());
        for segment in &segments {
            let expr = MathExpression::parse(segment)?;
            sums.push(expr.sum);
        }

        Ok(sums)
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

#[cfg(test)]
mod tests {
    use crate::ComputedStyle;

    use super::*;
    use css_cssom::{ComponentValue, CssToken, CssTokenKind, Function, NumericValue};

    /// Helper function to create test contexts
    fn create_test_contexts() -> (RelativeContext, AbsoluteContext<'static>) {
        let rel_ctx = RelativeContext {
            parent: ComputedStyle {
                font_size: 16.0,
                intrinsic_width: 800.0,
                intrinsic_height: 600.0,
                ..Default::default()
            }
            .into(),
        };
        let abs_ctx = AbsoluteContext {
            root_font_size: 16.0,
            viewport_width: 1024.0,
            viewport_height: 768.0,
            ..Default::default()
        };
        (rel_ctx, abs_ctx)
    }

    /// Helper function to create a number token
    fn number_token(value: f64) -> ComponentValue {
        ComponentValue::Token(CssToken {
            kind: CssTokenKind::Number(NumericValue::from(value)),
            position: None,
        })
    }

    /// Helper function to create a dimension token
    fn dimension_token(value: f64, unit: &str) -> ComponentValue {
        ComponentValue::Token(CssToken {
            kind: CssTokenKind::Dimension {
                value: NumericValue::from(value),
                unit: unit.to_string(),
            },
            position: None,
        })
    }

    /// Helper function to create a delim token
    fn delim_token(ch: char) -> ComponentValue {
        ComponentValue::Token(CssToken {
            kind: CssTokenKind::Delim(ch),
            position: None,
        })
    }

    /// Helper function to create a whitespace token
    fn whitespace_token() -> ComponentValue {
        ComponentValue::Token(CssToken {
            kind: CssTokenKind::Whitespace,
            position: None,
        })
    }

    #[test]
    fn test_simple_number() {
        let input = vec![number_token(42.0)];
        let expr = MathExpression::parse(&input).unwrap();
        let (rel_ctx, abs_ctx) = create_test_contexts();
        let result = expr.to_px(None, &rel_ctx, &abs_ctx);
        assert_eq!(result, 42.0);
    }

    #[test]
    fn test_simple_negative_number() {
        let input = vec![number_token(-42.0)];
        let expr = MathExpression::parse(&input).unwrap();
        let (rel_ctx, abs_ctx) = create_test_contexts();
        let result = expr.to_px(None, &rel_ctx, &abs_ctx);
        assert_eq!(result, -42.0);
    }

    #[test]
    fn test_simple_addition() {
        let input = vec![
            number_token(10.0),
            whitespace_token(),
            delim_token('+'),
            whitespace_token(),
            number_token(5.0),
        ];
        let expr = MathExpression::parse(&input).unwrap();
        let (rel_ctx, abs_ctx) = create_test_contexts();
        let result = expr.to_px(None, &rel_ctx, &abs_ctx);
        assert_eq!(result, 15.0);
    }

    #[test]
    fn test_simple_subtraction() {
        let input = vec![
            number_token(10.0),
            whitespace_token(),
            delim_token('-'),
            whitespace_token(),
            number_token(5.0),
        ];
        let expr = MathExpression::parse(&input).unwrap();
        let (rel_ctx, abs_ctx) = create_test_contexts();
        let result = expr.to_px(None, &rel_ctx, &abs_ctx);
        assert_eq!(result, 5.0);
    }

    #[test]
    fn test_multiple_additions() {
        let input = vec![
            number_token(10.0),
            whitespace_token(),
            delim_token('+'),
            whitespace_token(),
            number_token(5.0),
            whitespace_token(),
            delim_token('+'),
            whitespace_token(),
            number_token(3.0),
        ];
        let expr = MathExpression::parse(&input).unwrap();
        let (rel_ctx, abs_ctx) = create_test_contexts();
        let result = expr.to_px(None, &rel_ctx, &abs_ctx);
        assert_eq!(result, 18.0);
    }

    #[test]
    fn test_addition_and_subtraction() {
        let input = vec![
            number_token(10.0),
            whitespace_token(),
            delim_token('+'),
            whitespace_token(),
            number_token(5.0),
            whitespace_token(),
            delim_token('-'),
            whitespace_token(),
            number_token(3.0),
        ];
        let expr = MathExpression::parse(&input).unwrap();
        let (rel_ctx, abs_ctx) = create_test_contexts();
        let result = expr.to_px(None, &rel_ctx, &abs_ctx);
        assert_eq!(result, 12.0);
    }

    #[test]
    fn test_simple_multiplication() {
        let input = vec![
            number_token(10.0),
            whitespace_token(),
            delim_token('*'),
            whitespace_token(),
            number_token(5.0),
        ];
        let expr = MathExpression::parse(&input).unwrap();
        let (rel_ctx, abs_ctx) = create_test_contexts();
        let result = expr.to_px(None, &rel_ctx, &abs_ctx);
        assert_eq!(result, 50.0);
    }

    #[test]
    fn test_simple_division() {
        let input = vec![
            number_token(10.0),
            whitespace_token(),
            delim_token('/'),
            whitespace_token(),
            number_token(5.0),
        ];
        let expr = MathExpression::parse(&input).unwrap();
        let (rel_ctx, abs_ctx) = create_test_contexts();
        let result = expr.to_px(None, &rel_ctx, &abs_ctx);
        assert_eq!(result, 2.0);
    }

    #[test]
    fn test_division_by_zero() {
        let input = vec![
            number_token(10.0),
            whitespace_token(),
            delim_token('/'),
            whitespace_token(),
            number_token(0.0),
        ];
        let expr = MathExpression::parse(&input).unwrap();
        let (rel_ctx, abs_ctx) = create_test_contexts();
        let result = expr.to_px(None, &rel_ctx, &abs_ctx);
        assert!(result.is_nan());
    }

    #[test]
    fn test_operator_precedence() {
        let input = vec![
            number_token(10.0),
            whitespace_token(),
            delim_token('+'),
            whitespace_token(),
            number_token(5.0),
            whitespace_token(),
            delim_token('*'),
            whitespace_token(),
            number_token(2.0),
        ];
        let expr = MathExpression::parse(&input).unwrap();
        let (rel_ctx, abs_ctx) = create_test_contexts();
        let result = expr.to_px(None, &rel_ctx, &abs_ctx);
        assert_eq!(result, 20.0);
    }

    #[test]
    fn test_associativity() {
        let input = vec![
            number_token(24.0),
            whitespace_token(),
            delim_token('/'),
            whitespace_token(),
            number_token(3.0),
            whitespace_token(),
            delim_token('*'),
            whitespace_token(),
            number_token(2.0),
        ];

        let expr = MathExpression::parse(&input).unwrap();
        let (rel_ctx, abs_ctx) = create_test_contexts();
        let result = expr.to_px(None, &rel_ctx, &abs_ctx);
        assert_eq!(result, 16.0);
    }

    #[test]
    fn test_complex_expression() {
        let input = vec![
            number_token(10.0),
            whitespace_token(),
            delim_token('+'),
            whitespace_token(),
            number_token(5.0),
            whitespace_token(),
            delim_token('*'),
            whitespace_token(),
            number_token(2.0),
            whitespace_token(),
            delim_token('-'),
            whitespace_token(),
            number_token(8.0),
            whitespace_token(),
            delim_token('/'),
            whitespace_token(),
            number_token(4.0),
        ];
        let expr = MathExpression::parse(&input).unwrap();
        let (rel_ctx, abs_ctx) = create_test_contexts();
        let result = expr.to_px(None, &rel_ctx, &abs_ctx);
        assert_eq!(result, 18.0);
    }

    #[test]
    fn test_negative_numbers() {
        let input = vec![
            number_token(10.0),
            whitespace_token(),
            delim_token('+'),
            whitespace_token(),
            number_token(-5.0),
        ];
        let expr = MathExpression::parse(&input).unwrap();
        let (rel_ctx, abs_ctx) = create_test_contexts();
        let result = expr.to_px(None, &rel_ctx, &abs_ctx);
        assert_eq!(result, 5.0);
    }

    #[test]
    fn test_pi() {
        let input = vec![
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("pi".to_string()),
                position: None,
            }),
            whitespace_token(),
            delim_token('*'),
            whitespace_token(),
            number_token(2.0),
        ];
        let expr = MathExpression::parse(&input).unwrap();
        let (rel_ctx, abs_ctx) = create_test_contexts();
        let result = expr.to_px(None, &rel_ctx, &abs_ctx);
        assert!((result - (std::f32::consts::PI * 2.0)).abs() < 0.001);
    }

    #[test]
    fn test_e() {
        let input = vec![
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("e".to_string()),
                position: None,
            }),
            whitespace_token(),
            delim_token('*'),
            whitespace_token(),
            number_token(2.0),
        ];
        let expr = MathExpression::parse(&input).unwrap();
        let (rel_ctx, abs_ctx) = create_test_contexts();
        let result = expr.to_px(None, &rel_ctx, &abs_ctx);
        assert!((result - (std::f32::consts::E * 2.0)).abs() < 0.001);
    }

    #[test]
    fn test_infinity() {
        let input = vec![
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("infinity".to_string()),
                position: None,
            }),
            whitespace_token(),
            delim_token('*'),
            whitespace_token(),
            number_token(2.0),
        ];
        let expr = MathExpression::parse(&input).unwrap();
        let (rel_ctx, abs_ctx) = create_test_contexts();
        let result = expr.to_px(None, &rel_ctx, &abs_ctx);
        assert!(result.is_infinite() && result.is_sign_positive());
    }

    #[test]
    fn test_negative_infinity() {
        let input = vec![
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("-infinity".to_string()),
                position: None,
            }),
            whitespace_token(),
            delim_token('*'),
            whitespace_token(),
            number_token(2.0),
        ];
        let expr = MathExpression::parse(&input).unwrap();
        let (rel_ctx, abs_ctx) = create_test_contexts();
        let result = expr.to_px(None, &rel_ctx, &abs_ctx);
        assert!(result.is_infinite() && result.is_sign_negative());
    }

    #[test]
    fn test_nan() {
        let input = vec![
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("nan".to_string()),
                position: None,
            }),
            whitespace_token(),
            delim_token('*'),
            whitespace_token(),
            number_token(2.0),
        ];
        let expr = MathExpression::parse(&input).unwrap();
        let (rel_ctx, abs_ctx) = create_test_contexts();
        let result = expr.to_px(None, &rel_ctx, &abs_ctx);
        assert!(result.is_nan());
    }

    #[test]
    fn test_chained_multiplication() {
        let input = vec![
            number_token(2.0),
            whitespace_token(),
            delim_token('*'),
            whitespace_token(),
            number_token(3.0),
            whitespace_token(),
            delim_token('*'),
            whitespace_token(),
            number_token(4.0),
        ];
        let expr = MathExpression::parse(&input).unwrap();
        let (rel_ctx, abs_ctx) = create_test_contexts();
        let result = expr.to_px(None, &rel_ctx, &abs_ctx);
        assert_eq!(result, 24.0);
    }

    #[test]
    fn test_chained_division() {
        let input = vec![
            number_token(24.0),
            whitespace_token(),
            delim_token('/'),
            whitespace_token(),
            number_token(3.0),
            whitespace_token(),
            delim_token('/'),
            whitespace_token(),
            number_token(2.0),
        ];
        let expr = MathExpression::parse(&input).unwrap();
        let (rel_ctx, abs_ctx) = create_test_contexts();
        let result = expr.to_px(None, &rel_ctx, &abs_ctx);
        assert_eq!(result, 4.0);
    }

    #[test]
    fn test_whitespace_required_around_plus() {
        let input = vec![number_token(10.0), delim_token('+'), number_token(5.0)];
        let result = MathExpression::parse(&input);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("Whitespace is required before")
        );
    }

    #[test]
    fn test_whitespace_required_around_minus() {
        let input = vec![number_token(10.0), delim_token('-'), number_token(5.0)];
        let result = MathExpression::parse(&input);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("Whitespace is required before")
        );
    }

    #[test]
    fn test_whitespace_required_after_plus() {
        let input = vec![
            number_token(10.0),
            whitespace_token(),
            delim_token('+'),
            number_token(5.0),
        ];
        let result = MathExpression::parse(&input);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Whitespace is required after"));
    }

    #[test]
    fn test_whitespace_required_after_minus() {
        let input = vec![
            number_token(10.0),
            whitespace_token(),
            delim_token('-'),
            number_token(5.0),
        ];
        let result = MathExpression::parse(&input);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Whitespace is required after"));
    }

    #[test]
    fn test_negative_number_subtraction() {
        let input = vec![
            number_token(50.0),
            whitespace_token(),
            delim_token('-'),
            whitespace_token(),
            number_token(-20.0),
        ];
        let expr = MathExpression::parse(&input).unwrap();
        let (rel_ctx, abs_ctx) = create_test_contexts();
        let result = expr.to_px(None, &rel_ctx, &abs_ctx);
        assert_eq!(result, 70.0);
    }

    #[test]
    fn test_multiple_negative_operations() {
        let input = vec![
            number_token(100.0),
            whitespace_token(),
            delim_token('-'),
            whitespace_token(),
            number_token(-20.0),
            whitespace_token(),
            delim_token('+'),
            whitespace_token(),
            number_token(-10.0),
        ];
        let expr = MathExpression::parse(&input).unwrap();
        let (rel_ctx, abs_ctx) = create_test_contexts();
        let result = expr.to_px(None, &rel_ctx, &abs_ctx);
        assert_eq!(result, 110.0);
    }

    #[test]
    fn test_negative_number_multiplication() {
        let input = vec![
            number_token(10.0),
            whitespace_token(),
            delim_token('*'),
            whitespace_token(),
            number_token(-5.0),
        ];
        let expr = MathExpression::parse(&input).unwrap();
        let (rel_ctx, abs_ctx) = create_test_contexts();
        let result = expr.to_px(None, &rel_ctx, &abs_ctx);
        assert_eq!(result, -50.0);
    }

    #[test]
    fn test_no_whitespace_required_for_multiply() {
        let input = vec![number_token(10.0), delim_token('*'), number_token(5.0)];
        let expr = MathExpression::parse(&input).unwrap();
        let (rel_ctx, abs_ctx) = create_test_contexts();
        let result = expr.to_px(None, &rel_ctx, &abs_ctx);
        assert_eq!(result, 50.0);
    }

    #[test]
    fn test_no_whitespace_required_for_divide() {
        let input = vec![number_token(10.0), delim_token('/'), number_token(5.0)];
        let expr = MathExpression::parse(&input).unwrap();
        let (rel_ctx, abs_ctx) = create_test_contexts();
        let result = expr.to_px(None, &rel_ctx, &abs_ctx);
        assert_eq!(result, 2.0);
    }

    #[test]
    fn test_mixed_whitespace_requirements() {
        let input = vec![
            number_token(10.0),
            delim_token('*'),
            number_token(5.0),
            whitespace_token(),
            delim_token('+'),
            whitespace_token(),
            number_token(20.0),
        ];
        let expr = MathExpression::parse(&input).unwrap();
        let (rel_ctx, abs_ctx) = create_test_contexts();
        let result = expr.to_px(None, &rel_ctx, &abs_ctx);
        assert_eq!(result, 70.0);
    }

    #[test]
    fn test_mixed_whitespace_requirements_no_space_around_plus() {
        let input = vec![
            number_token(10.0),
            delim_token('*'),
            number_token(5.0),
            delim_token('+'),
            number_token(20.0),
        ];
        let result = MathExpression::parse(&input);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("Whitespace is required before")
        );
    }

    #[test]
    fn test_nested_calc() {
        let input = vec![ComponentValue::Function(Function {
            name: "calc".to_string(),
            value: vec![
                number_token(10.0),
                whitespace_token(),
                delim_token('+'),
                whitespace_token(),
                ComponentValue::Function(Function {
                    name: "calc".to_string(),
                    value: vec![
                        number_token(5.0),
                        whitespace_token(),
                        delim_token('*'),
                        whitespace_token(),
                        number_token(2.0),
                    ],
                }),
            ],
        })];
        let expr = MathExpression::parse(&input).unwrap();
        let (rel_ctx, abs_ctx) = create_test_contexts();
        let result = expr.to_px(None, &rel_ctx, &abs_ctx);
        assert_eq!(result, 20.0);
    }

    #[test]
    fn test_mixed_units_px_em() {
        let input = vec![
            dimension_token(10.0, "px"),
            whitespace_token(),
            delim_token('+'),
            whitespace_token(),
            dimension_token(2.0, "em"),
        ];
        let expr = MathExpression::parse(&input).unwrap();
        let (rel_ctx, abs_ctx) = create_test_contexts();
        let result = expr.to_px(Some(RelativeType::FontSize), &rel_ctx, &abs_ctx);
        assert_eq!(result, 10.0 + (2.0 * rel_ctx.parent.font_size));
    }

    #[test]
    fn test_mixed_units_px_rem() {
        let input = vec![
            dimension_token(10.0, "px"),
            whitespace_token(),
            delim_token('+'),
            whitespace_token(),
            dimension_token(2.0, "rem"),
        ];
        let expr = MathExpression::parse(&input).unwrap();
        let (rel_ctx, abs_ctx) = create_test_contexts();
        let result = expr.to_px(Some(RelativeType::RootFontSize), &rel_ctx, &abs_ctx);
        assert_eq!(result, 10.0 + (2.0 * abs_ctx.root_font_size));
    }

    #[test]
    fn test_mixed_units_px_percent() {
        let input = vec![
            dimension_token(10.0, "px"),
            whitespace_token(),
            delim_token('+'),
            whitespace_token(),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Percentage(NumericValue::from(50.0)),
                position: None,
            }),
        ];
        let expr = MathExpression::parse(&input).unwrap();
        let (rel_ctx, abs_ctx) = create_test_contexts();
        let result = expr.to_px(Some(RelativeType::ParentWidth), &rel_ctx, &abs_ctx);
        let width = rel_ctx.parent.intrinsic_width;
        assert_eq!(result, 10.0 + (0.5 * width));
    }

    #[test]
    fn test_mixed_units_px_vw() {
        let input = vec![
            dimension_token(10.0, "px"),
            whitespace_token(),
            delim_token('+'),
            whitespace_token(),
            dimension_token(2.0, "vw"),
        ];
        let expr = MathExpression::parse(&input).unwrap();
        let (rel_ctx, abs_ctx) = create_test_contexts();
        let result = expr.to_px(Some(RelativeType::ViewportWidth), &rel_ctx, &abs_ctx);
        assert_eq!(result, 10.0 + (0.02 * abs_ctx.viewport_width));
    }

    #[test]
    fn test_mixed_units_px_vh() {
        let input = vec![
            dimension_token(100.0, "px"),
            whitespace_token(),
            delim_token('+'),
            whitespace_token(),
            dimension_token(10.0, "vh"),
        ];
        let expr = MathExpression::parse(&input).unwrap();
        let (rel_ctx, abs_ctx) = create_test_contexts();
        let result = expr.to_px(None, &rel_ctx, &abs_ctx);
        assert_eq!(result, 176.8);
    }

    fn comma_token() -> ComponentValue {
        ComponentValue::Token(CssToken {
            kind: CssTokenKind::Comma,
            position: None,
        })
    }

    #[test]
    fn test_min_two_numbers() {
        let input = vec![ComponentValue::Function(Function {
            name: "min".to_string(),
            value: vec![number_token(100.0), comma_token(), number_token(200.0)],
        })];
        let expr = MathExpression::parse(&input).unwrap();
        let (rel_ctx, abs_ctx) = create_test_contexts();
        let result = expr.to_px(None, &rel_ctx, &abs_ctx);
        assert_eq!(result, 100.0);
    }

    #[test]
    fn test_min_three_numbers() {
        let input = vec![ComponentValue::Function(Function {
            name: "min".to_string(),
            value: vec![
                number_token(300.0),
                comma_token(),
                number_token(100.0),
                comma_token(),
                number_token(200.0),
            ],
        })];
        let expr = MathExpression::parse(&input).unwrap();
        let (rel_ctx, abs_ctx) = create_test_contexts();
        let result = expr.to_px(None, &rel_ctx, &abs_ctx);
        assert_eq!(result, 100.0);
    }

    #[test]
    fn test_min_with_calc_sum() {
        let input = vec![ComponentValue::Function(Function {
            name: "min".to_string(),
            value: vec![
                dimension_token(200.0, "px"),
                whitespace_token(),
                delim_token('-'),
                whitespace_token(),
                dimension_token(50.0, "px"),
                comma_token(),
                whitespace_token(),
                dimension_token(100.0, "px"),
            ],
        })];
        let expr = MathExpression::parse(&input).unwrap();
        let (rel_ctx, abs_ctx) = create_test_contexts();
        let result = expr.to_px(None, &rel_ctx, &abs_ctx);
        assert_eq!(result, 100.0);
    }

    #[test]
    fn test_min_expression_standalone() {
        let input = vec![number_token(100.0), comma_token(), number_token(200.0)];
        let expr = MathExpression::parse_math_function("min", &input).unwrap();
        let (rel_ctx, abs_ctx) = create_test_contexts();
        let result = expr.to_px(None, &rel_ctx, &abs_ctx);
        assert_eq!(result, 100.0);
    }

    #[test]
    fn test_max_two_numbers() {
        let input = vec![ComponentValue::Function(Function {
            name: "max".to_string(),
            value: vec![number_token(100.0), comma_token(), number_token(200.0)],
        })];
        let expr = MathExpression::parse(&input).unwrap();
        let (rel_ctx, abs_ctx) = create_test_contexts();
        let result = expr.to_px(None, &rel_ctx, &abs_ctx);
        assert_eq!(result, 200.0);
    }

    #[test]
    fn test_max_three_numbers() {
        let input = vec![ComponentValue::Function(Function {
            name: "max".to_string(),
            value: vec![
                number_token(100.0),
                comma_token(),
                number_token(300.0),
                comma_token(),
                number_token(200.0),
            ],
        })];
        let expr = MathExpression::parse(&input).unwrap();
        let (rel_ctx, abs_ctx) = create_test_contexts();
        let result = expr.to_px(None, &rel_ctx, &abs_ctx);
        assert_eq!(result, 300.0);
    }

    #[test]
    fn test_max_with_calc_sum() {
        let input = vec![ComponentValue::Function(Function {
            name: "max".to_string(),
            value: vec![
                dimension_token(50.0, "px"),
                whitespace_token(),
                delim_token('+'),
                whitespace_token(),
                dimension_token(30.0, "px"),
                comma_token(),
                whitespace_token(),
                dimension_token(100.0, "px"),
            ],
        })];
        let expr = MathExpression::parse(&input).unwrap();
        let (rel_ctx, abs_ctx) = create_test_contexts();
        let result = expr.to_px(None, &rel_ctx, &abs_ctx);
        assert_eq!(result, 100.0);
    }

    #[test]
    fn test_max_expression_standalone() {
        let input = vec![number_token(100.0), comma_token(), number_token(200.0)];
        let expr = MathExpression::parse_math_function("max", &input).unwrap();
        let (rel_ctx, abs_ctx) = create_test_contexts();
        let result = expr.to_px(None, &rel_ctx, &abs_ctx);
        assert_eq!(result, 200.0);
    }

    #[test]
    fn test_clamp_value_within_range() {
        let input = vec![ComponentValue::Function(Function {
            name: "clamp".to_string(),
            value: vec![
                number_token(100.0),
                comma_token(),
                whitespace_token(),
                number_token(150.0),
                comma_token(),
                whitespace_token(),
                number_token(200.0),
            ],
        })];
        let expr = MathExpression::parse(&input).unwrap();
        let (rel_ctx, abs_ctx) = create_test_contexts();
        let result = expr.to_px(None, &rel_ctx, &abs_ctx);
        assert_eq!(result, 150.0);
    }

    #[test]
    fn test_clamp_value_below_min() {
        let input = vec![ComponentValue::Function(Function {
            name: "clamp".to_string(),
            value: vec![
                number_token(100.0),
                comma_token(),
                whitespace_token(),
                number_token(50.0),
                comma_token(),
                whitespace_token(),
                number_token(200.0),
            ],
        })];
        let expr = MathExpression::parse(&input).unwrap();
        let (rel_ctx, abs_ctx) = create_test_contexts();
        let result = expr.to_px(None, &rel_ctx, &abs_ctx);
        assert_eq!(result, 100.0);
    }

    #[test]
    fn test_clamp_value_above_max() {
        let input = vec![ComponentValue::Function(Function {
            name: "clamp".to_string(),
            value: vec![
                number_token(100.0),
                comma_token(),
                whitespace_token(),
                number_token(300.0),
                comma_token(),
                whitespace_token(),
                number_token(200.0),
            ],
        })];
        let expr = MathExpression::parse(&input).unwrap();
        let (rel_ctx, abs_ctx) = create_test_contexts();
        let result = expr.to_px(None, &rel_ctx, &abs_ctx);
        assert_eq!(result, 200.0);
    }

    #[test]
    fn test_clamp_with_calc_sums() {
        let input = vec![ComponentValue::Function(Function {
            name: "clamp".to_string(),
            value: vec![
                dimension_token(50.0, "px"),
                whitespace_token(),
                delim_token('+'),
                whitespace_token(),
                dimension_token(50.0, "px"),
                comma_token(),
                whitespace_token(),
                dimension_token(200.0, "px"),
                comma_token(),
                whitespace_token(),
                dimension_token(300.0, "px"),
                whitespace_token(),
                delim_token('-'),
                whitespace_token(),
                dimension_token(50.0, "px"),
            ],
        })];
        let expr = MathExpression::parse(&input).unwrap();
        let (rel_ctx, abs_ctx) = create_test_contexts();
        let result = expr.to_px(None, &rel_ctx, &abs_ctx);
        assert_eq!(result, 200.0);
    }

    #[test]
    fn test_clamp_expression_standalone() {
        let input = vec![
            number_token(100.0),
            comma_token(),
            whitespace_token(),
            number_token(150.0),
            comma_token(),
            whitespace_token(),
            number_token(200.0),
        ];
        let expr = MathExpression::parse_math_function("clamp", &input).unwrap();
        let (rel_ctx, abs_ctx) = create_test_contexts();
        let result = expr.to_px(None, &rel_ctx, &abs_ctx);
        assert_eq!(result, 150.0);
    }

    #[test]
    fn test_clamp_wrong_arg_count_two() {
        let input = vec![number_token(100.0), comma_token(), number_token(200.0)];
        let result = MathExpression::parse_math_function("clamp", &input);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("exactly 3 arguments"));
    }

    #[test]
    fn test_clamp_wrong_arg_count_four() {
        let input = vec![
            number_token(100.0),
            comma_token(),
            number_token(150.0),
            comma_token(),
            number_token(200.0),
            comma_token(),
            number_token(250.0),
        ];
        let result = MathExpression::parse_math_function("clamp", &input);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("exactly 3 arguments"));
    }

    #[test]
    fn test_clamp_none_min() {
        let input = vec![ComponentValue::Function(Function {
            name: "clamp".to_string(),
            value: vec![
                ComponentValue::Token(CssToken {
                    kind: CssTokenKind::Ident("none".to_string()),
                    position: None,
                }),
                comma_token(),
                whitespace_token(),
                number_token(50.0),
                comma_token(),
                whitespace_token(),
                number_token(200.0),
            ],
        })];
        let expr = MathExpression::parse(&input).unwrap();
        let (rel_ctx, abs_ctx) = create_test_contexts();
        let result = expr.to_px(None, &rel_ctx, &abs_ctx);
        assert_eq!(result, 50.0);
    }

    #[test]
    fn test_clamp_none_max() {
        let input = vec![ComponentValue::Function(Function {
            name: "clamp".to_string(),
            value: vec![
                number_token(100.0),
                comma_token(),
                whitespace_token(),
                number_token(300.0),
                comma_token(),
                whitespace_token(),
                ComponentValue::Token(CssToken {
                    kind: CssTokenKind::Ident("none".to_string()),
                    position: None,
                }),
            ],
        })];
        let expr = MathExpression::parse(&input).unwrap();
        let (rel_ctx, abs_ctx) = create_test_contexts();
        let result = expr.to_px(None, &rel_ctx, &abs_ctx);
        assert_eq!(result, 300.0);
    }

    #[test]
    fn test_clamp_none_both() {
        let input = vec![ComponentValue::Function(Function {
            name: "clamp".to_string(),
            value: vec![
                ComponentValue::Token(CssToken {
                    kind: CssTokenKind::Ident("none".to_string()),
                    position: None,
                }),
                comma_token(),
                whitespace_token(),
                number_token(42.0),
                comma_token(),
                whitespace_token(),
                ComponentValue::Token(CssToken {
                    kind: CssTokenKind::Ident("none".to_string()),
                    position: None,
                }),
            ],
        })];
        let expr = MathExpression::parse(&input).unwrap();
        let (rel_ctx, abs_ctx) = create_test_contexts();
        let result = expr.to_px(None, &rel_ctx, &abs_ctx);
        assert_eq!(result, 42.0);
    }

    #[test]
    fn test_clamp_none_min_value_below_max() {
        let input = vec![ComponentValue::Function(Function {
            name: "clamp".to_string(),
            value: vec![
                ComponentValue::Token(CssToken {
                    kind: CssTokenKind::Ident("none".to_string()),
                    position: None,
                }),
                comma_token(),
                whitespace_token(),
                number_token(50.0),
                comma_token(),
                whitespace_token(),
                number_token(30.0),
            ],
        })];
        let expr = MathExpression::parse(&input).unwrap();
        let (rel_ctx, abs_ctx) = create_test_contexts();
        let result = expr.to_px(None, &rel_ctx, &abs_ctx);
        assert_eq!(result, 30.0);
    }

    #[test]
    fn test_nested_max_in_min() {
        let input = vec![ComponentValue::Function(Function {
            name: "min".to_string(),
            value: vec![
                ComponentValue::Function(Function {
                    name: "max".to_string(),
                    value: vec![number_token(50.0), comma_token(), number_token(150.0)],
                }),
                comma_token(),
                whitespace_token(),
                number_token(200.0),
            ],
        })];
        let expr = MathExpression::parse(&input).unwrap();
        let (rel_ctx, abs_ctx) = create_test_contexts();
        let result = expr.to_px(None, &rel_ctx, &abs_ctx);
        assert_eq!(result, 150.0);
    }

    #[test]
    fn test_max_with_nested_calc() {
        let input = vec![ComponentValue::Function(Function {
            name: "max".to_string(),
            value: vec![
                ComponentValue::Function(Function {
                    name: "calc".to_string(),
                    value: vec![
                        number_token(50.0),
                        whitespace_token(),
                        delim_token('+'),
                        whitespace_token(),
                        number_token(50.0),
                    ],
                }),
                comma_token(),
                whitespace_token(),
                number_token(80.0),
            ],
        })];
        let expr = MathExpression::parse(&input).unwrap();
        let (rel_ctx, abs_ctx) = create_test_contexts();
        let result = expr.to_px(None, &rel_ctx, &abs_ctx);
        assert_eq!(result, 100.0);
    }

    #[test]
    fn test_min_with_mixed_units() {
        let input = vec![ComponentValue::Function(Function {
            name: "min".to_string(),
            value: vec![
                dimension_token(200.0, "px"),
                comma_token(),
                whitespace_token(),
                dimension_token(10.0, "vw"),
            ],
        })];
        let expr = MathExpression::parse(&input).unwrap();
        let (rel_ctx, abs_ctx) = create_test_contexts();
        let result = expr.to_px(None, &rel_ctx, &abs_ctx);
        assert_eq!(result, 102.4);
    }

    #[test]
    fn test_min_in_calc() {
        let input = vec![
            ComponentValue::Function(Function {
                name: "min".to_string(),
                value: vec![number_token(200.0), comma_token(), number_token(100.0)],
            }),
            whitespace_token(),
            delim_token('+'),
            whitespace_token(),
            number_token(50.0),
        ];
        let expr = MathExpression::parse(&input).unwrap();
        let (rel_ctx, abs_ctx) = create_test_contexts();
        let result = expr.to_px(None, &rel_ctx, &abs_ctx);
        assert_eq!(result, 150.0);
    }
}

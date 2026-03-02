use std::str::FromStr;

use css_cssom::{AssociatedToken, ComponentValue, CssTokenKind};

use crate::{
    length::LengthUnit,
    primitives::{length::Length, percentage::Percentage},
    properties::{AbsoluteContext, RelativeContext, RelativeType},
};

/// Represents the special keywords that can be used in calc() expressions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CalculateKeyword {
    E,
    PI,
    Infinity,
    NegativeInfinity,
    NaN,
}

impl CalculateKeyword {
    pub fn to_f32(self) -> f32 {
        match self {
            CalculateKeyword::E => std::f32::consts::E,
            CalculateKeyword::PI => std::f32::consts::PI,
            CalculateKeyword::Infinity => f32::INFINITY,
            CalculateKeyword::NegativeInfinity => f32::NEG_INFINITY,
            CalculateKeyword::NaN => f32::NAN,
        }
    }
}

impl FromStr for CalculateKeyword {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "e" => Ok(CalculateKeyword::E),
            "pi" => Ok(CalculateKeyword::PI),
            "infinity" => Ok(CalculateKeyword::Infinity),
            "-infinity" => Ok(CalculateKeyword::NegativeInfinity),
            "nan" => Ok(CalculateKeyword::NaN),
            _ => Err(format!("Invalid calculate keyword: {}", s)),
        }
    }
}

/// Represents the arguments to a clamp() function, which consists of an optional minimum value, a required value, and an optional maximum value.
#[derive(Debug, Clone, PartialEq)]
pub struct ClampArgs {
    pub min: Option<Box<CalculateSum>>,
    pub val: Box<CalculateSum>,
    pub max: Option<Box<CalculateSum>>,
}

/// Represents a single value in a calc() expression, which can be a number, length, percentage, keyword, or a nested calc() expression.
#[derive(Debug, Clone, PartialEq)]
pub enum CalculateValue {
    Number(f32),
    Length(Length),
    Percentage(Percentage),
    Keyword(CalculateKeyword),
    NestedSum(Box<CalculateSum>),
    Min(Vec<CalculateSum>),
    Max(Vec<CalculateSum>),
    Clamp(ClampArgs),
}

impl CalculateValue {
    pub fn to_px(&self, rel_type: Option<RelativeType>, rel_ctx: &RelativeContext, abs_ctx: &AbsoluteContext) -> f32 {
        match self {
            CalculateValue::Number(n) => *n,
            CalculateValue::Length(l) => l.to_px(rel_ctx, abs_ctx),
            CalculateValue::Keyword(k) => k.to_f32(),
            CalculateValue::NestedSum(sum) => sum.to_px(rel_type, rel_ctx, abs_ctx),
            CalculateValue::Percentage(p) => match rel_type {
                Some(RelativeType::FontSize) => rel_ctx.parent.font_size * p.as_fraction(),
                Some(RelativeType::ParentHeight) => rel_ctx.parent.intrinsic_height * p.as_fraction(),
                Some(RelativeType::ParentWidth) => rel_ctx.parent.intrinsic_width * p.as_fraction(),
                Some(RelativeType::RootFontSize) => abs_ctx.root_font_size * p.as_fraction(),
                Some(RelativeType::ViewportHeight) => abs_ctx.viewport_height * p.as_fraction(),
                Some(RelativeType::ViewportWidth) => abs_ctx.viewport_width * p.as_fraction(),
                None => 0.0,
            },
            CalculateValue::Min(args) => args
                .iter()
                .map(|sum| sum.to_px(rel_type, rel_ctx, abs_ctx))
                .fold(f32::INFINITY, f32::min),
            CalculateValue::Max(args) => args
                .iter()
                .map(|sum| sum.to_px(rel_type, rel_ctx, abs_ctx))
                .fold(f32::NEG_INFINITY, f32::max),
            CalculateValue::Clamp(args) => {
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
pub enum CalculateProduct {
    Value(CalculateValue),
    Multiply(Box<CalculateProduct>, Box<CalculateProduct>),
    Divide(Box<CalculateProduct>, Box<CalculateProduct>),
}

impl CalculateProduct {
    pub fn to_px(&self, rel_type: Option<RelativeType>, rel_ctx: &RelativeContext, abs_ctx: &AbsoluteContext) -> f32 {
        match self {
            CalculateProduct::Value(v) => v.to_px(rel_type, rel_ctx, abs_ctx),
            CalculateProduct::Multiply(left, right) => {
                left.to_px(rel_type, rel_ctx, abs_ctx) * right.to_px(rel_type, rel_ctx, abs_ctx)
            }
            CalculateProduct::Divide(left, right) => {
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
pub enum CalculateSum {
    Product(CalculateProduct),
    Add(Box<CalculateSum>, Box<CalculateSum>),
    Subtract(Box<CalculateSum>, Box<CalculateSum>),
}

impl CalculateSum {
    pub fn to_px(&self, rel_type: Option<RelativeType>, rel_ctx: &RelativeContext, abs_ctx: &AbsoluteContext) -> f32 {
        match self {
            CalculateSum::Product(p) => p.to_px(rel_type, rel_ctx, abs_ctx),
            CalculateSum::Add(left, right) => {
                left.to_px(rel_type, rel_ctx, abs_ctx) + right.to_px(rel_type, rel_ctx, abs_ctx)
            }
            CalculateSum::Subtract(left, right) => {
                left.to_px(rel_type, rel_ctx, abs_ctx) - right.to_px(rel_type, rel_ctx, abs_ctx)
            }
        }
    }
}

/// Represents a CSS calc() expression (or any CSS math function: calc, min, max, clamp).
/// The top-level structure is a sum of products, which allows for proper operator precedence and associativity when evaluating the expression.
#[derive(Debug, Clone, PartialEq)]
pub struct CalcExpression {
    pub sum: CalculateSum,
}

/// The set of CSS math function names that can be parsed as calc expressions.
const MATH_FUNCTION_NAMES: &[&str] = &["calc", "min", "max", "clamp"];

/// Returns true if the given function name is a CSS math function (calc, min, max, clamp).
pub fn is_math_function(name: &str) -> bool {
    MATH_FUNCTION_NAMES
        .iter()
        .any(|n| name.eq_ignore_ascii_case(n))
}

impl CalcExpression {
    pub fn to_px(&self, rel_type: Option<RelativeType>, rel_ctx: &RelativeContext, abs_ctx: &AbsoluteContext) -> f32 {
        self.sum.to_px(rel_type, rel_ctx, abs_ctx)
    }

    /// Parse a `<calc-sum>` from a flat list of component values (i.e. the contents inside a `calc()` function).
    pub fn parse(input: &[ComponentValue]) -> Result<Self, String> {
        let mut parser = CalcParser::new(input);
        let sum = parser.parse_sum()?;

        parser.skip_whitespace();
        if parser.current_pos < parser.input.len() {
            return Err(format!("Unexpected trailing input at position {}", parser.current_pos));
        }

        Ok(CalcExpression { sum })
    }

    /// Parse any CSS math function (calc, min, max, clamp) from its inner component values and function name.
    /// This dispatches to the appropriate parser based on the function name.
    pub fn parse_math_function(name: &str, value: &[ComponentValue]) -> Result<Self, String> {
        if name.eq_ignore_ascii_case("calc") {
            Self::parse(value)
        } else if name.eq_ignore_ascii_case("min") {
            let args = CalcParser::parse_comma_separated_sums(value)?;
            if args.is_empty() {
                return Err("min() requires at least one argument".to_string());
            }
            Ok(CalcExpression {
                sum: CalculateSum::Product(CalculateProduct::Value(CalculateValue::Min(args))),
            })
        } else if name.eq_ignore_ascii_case("max") {
            let args = CalcParser::parse_comma_separated_sums(value)?;
            if args.is_empty() {
                return Err("max() requires at least one argument".to_string());
            }
            Ok(CalcExpression {
                sum: CalculateSum::Product(CalculateProduct::Value(CalculateValue::Max(args))),
            })
        } else if name.eq_ignore_ascii_case("clamp") {
            let args = CalcParser::parse_clamp_args(value)?;
            Ok(CalcExpression {
                sum: CalculateSum::Product(CalculateProduct::Value(CalculateValue::Clamp(args))),
            })
        } else {
            Err(format!("Unknown math function: {}", name))
        }
    }
}

/// A simple recursive descent parser for calc() expressions that respects operator precedence and associativity.
struct CalcParser<'a> {
    input: &'a [ComponentValue],
    current_pos: usize,
}

impl<'a> CalcParser<'a> {
    fn new(input: &'a [ComponentValue]) -> Self {
        Self {
            input,
            current_pos: 0,
        }
    }

    fn skip_whitespace(&mut self) -> bool {
        let start_pos = self.current_pos;
        while self.current_pos < self.input.len() {
            if let ComponentValue::Token(token) = &self.input[self.current_pos]
                && token.kind == CssTokenKind::Whitespace
            {
                self.current_pos += 1;
                continue;
            }
            break;
        }
        self.current_pos > start_pos
    }

    fn peek_token(&self) -> Option<&CssTokenKind> {
        if self.current_pos >= self.input.len() {
            return None;
        }

        if let ComponentValue::Token(token) = &self.input[self.current_pos] {
            Some(&token.kind)
        } else {
            None
        }
    }

    fn consume_token(&mut self) -> Option<&ComponentValue> {
        if self.current_pos >= self.input.len() {
            return None;
        }
        let token = &self.input[self.current_pos];
        self.current_pos += 1;
        Some(token)
    }

    fn parse_sum(&mut self) -> Result<CalculateSum, String> {
        let mut left = CalculateSum::Product(self.parse_product()?);

        loop {
            let had_whitespace_before = self.skip_whitespace();
            if self.current_pos >= self.input.len() {
                break;
            }

            let is_plus_or_minus = matches!(self.peek_token(), Some(CssTokenKind::Delim('+' | '-')));
            if !is_plus_or_minus {
                break;
            }

            if !had_whitespace_before {
                return Err("Whitespace is required before '+' or '-' operator in calc()".to_string());
            }

            let op = match self.consume_token() {
                Some(ComponentValue::Token(token)) => match &token.kind {
                    CssTokenKind::Delim('+') => '+',
                    CssTokenKind::Delim('-') => '-',
                    _ => break,
                },
                _ => break,
            };

            let had_whitespace_after = self.skip_whitespace();
            if !had_whitespace_after {
                return Err("Whitespace is required after '+' or '-' operator in calc()".to_string());
            }

            let next_product = self.parse_product()?;
            let right = CalculateSum::Product(next_product);

            if op == '+' {
                left = CalculateSum::Add(Box::new(left), Box::new(right));
            } else {
                left = CalculateSum::Subtract(Box::new(left), Box::new(right));
            }
        }

        Ok(left)
    }

    fn parse_product(&mut self) -> Result<CalculateProduct, String> {
        let mut left = CalculateProduct::Value(self.parse_value()?);

        loop {
            let pos_before_ws = self.current_pos;
            self.skip_whitespace();

            if self.current_pos >= self.input.len() {
                break;
            }

            let op_char = match self.peek_token() {
                Some(CssTokenKind::Delim('*')) => '*',
                Some(CssTokenKind::Delim('/')) => '/',
                _ => {
                    self.current_pos = pos_before_ws;
                    break;
                }
            };

            self.current_pos += 1;

            self.skip_whitespace();

            let next_value = self.parse_value()?;
            let right = CalculateProduct::Value(next_value);

            if op_char == '*' {
                left = CalculateProduct::Multiply(Box::new(left), Box::new(right));
            } else {
                left = CalculateProduct::Divide(Box::new(left), Box::new(right));
            }
        }

        Ok(left)
    }

    fn parse_value(&mut self) -> Result<CalculateValue, String> {
        self.skip_whitespace();
        if self.current_pos >= self.input.len() {
            return Err("Unexpected end of input".to_string());
        }

        let cv = &self.input[self.current_pos];

        match cv {
            ComponentValue::Function(func) if func.name.eq_ignore_ascii_case("calc") => {
                self.current_pos += 1;
                let nested = CalcExpression::parse(&func.value)?;
                Ok(CalculateValue::NestedSum(Box::new(nested.sum)))
            }

            ComponentValue::Function(func) if func.name.eq_ignore_ascii_case("min") => {
                self.current_pos += 1;
                let args = Self::parse_comma_separated_sums(&func.value)?;
                if args.is_empty() {
                    return Err("min() requires at least one argument".to_string());
                }
                Ok(CalculateValue::Min(args))
            }

            ComponentValue::Function(func) if func.name.eq_ignore_ascii_case("max") => {
                self.current_pos += 1;
                let args = Self::parse_comma_separated_sums(&func.value)?;
                if args.is_empty() {
                    return Err("max() requires at least one argument".to_string());
                }
                Ok(CalculateValue::Max(args))
            }

            ComponentValue::Function(func) if func.name.eq_ignore_ascii_case("clamp") => {
                self.current_pos += 1;
                let args = Self::parse_clamp_args(&func.value)?;
                Ok(CalculateValue::Clamp(args))
            }

            ComponentValue::SimpleBlock(block) if matches!(block.associated_token, AssociatedToken::Parenthesis) => {
                self.current_pos += 1;
                let nested = CalcExpression::parse(&block.value)?;
                Ok(CalculateValue::NestedSum(Box::new(nested.sum)))
            }

            ComponentValue::Token(token) => match &token.kind {
                CssTokenKind::Number(num) => {
                    self.current_pos += 1;
                    Ok(CalculateValue::Number(num.to_f64() as f32))
                }

                CssTokenKind::Dimension { value, unit } => {
                    self.current_pos += 1;
                    let len_unit = unit
                        .parse::<LengthUnit>()
                        .map_err(|_| format!("Invalid length unit: {}", unit))?;
                    Ok(CalculateValue::Length(Length::new(value.to_f64() as f32, len_unit)))
                }

                CssTokenKind::Percentage(num) => {
                    self.current_pos += 1;
                    Ok(CalculateValue::Percentage(Percentage::new(num.to_f64() as f32)))
                }

                CssTokenKind::Ident(ident) => {
                    self.current_pos += 1;
                    CalculateKeyword::from_str(ident)
                        .map(CalculateValue::Keyword)
                        .map_err(|_| format!("Invalid calc() keyword or identifier: {}", ident))
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
    fn parse_clamp_bound(segment: &[ComponentValue]) -> Result<Option<Box<CalculateSum>>, String> {
        let non_ws: Vec<&ComponentValue> = segment
            .iter()
            .filter(|cv| !matches!(cv, ComponentValue::Token(t) if t.kind == CssTokenKind::Whitespace))
            .collect();

        if non_ws.len() == 1
            && let ComponentValue::Token(token) = non_ws[0]
            && let CssTokenKind::Ident(ident) = &token.kind
            && ident.eq_ignore_ascii_case("none")
        {
            return Ok(None);
        }

        CalcExpression::parse(segment)
            .map(|e| Some(Box::new(e.sum)))
            .map_err(|e| format!("Invalid clamp() bound: {}", e))
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

    /// Parses comma-separated `<calc-sum>` arguments from a function's value tokens.
    /// Used for `min()` and `max()` which take `<calc-sum>#` arguments.
    fn parse_comma_separated_sums(input: &[ComponentValue]) -> Result<Vec<CalculateSum>, String> {
        let segments = Self::split_on_commas(input);

        let mut sums = Vec::with_capacity(segments.len());
        for segment in &segments {
            let expr = CalcExpression::parse(segment)?;
            sums.push(expr.sum);
        }

        Ok(sums)
    }
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
        let expr = CalcExpression::parse(&input).unwrap();
        let (rel_ctx, abs_ctx) = create_test_contexts();
        let result = expr.to_px(None, &rel_ctx, &abs_ctx);
        assert_eq!(result, 42.0);
    }

    #[test]
    fn test_simple_negative_number() {
        let input = vec![number_token(-42.0)];
        let expr = CalcExpression::parse(&input).unwrap();
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
        let expr = CalcExpression::parse(&input).unwrap();
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
        let expr = CalcExpression::parse(&input).unwrap();
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
        let expr = CalcExpression::parse(&input).unwrap();
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
        let expr = CalcExpression::parse(&input).unwrap();
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
        let expr = CalcExpression::parse(&input).unwrap();
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
        let expr = CalcExpression::parse(&input).unwrap();
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
        let expr = CalcExpression::parse(&input).unwrap();
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
        let expr = CalcExpression::parse(&input).unwrap();
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

        let expr = CalcExpression::parse(&input).unwrap();
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
        let expr = CalcExpression::parse(&input).unwrap();
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
        let expr = CalcExpression::parse(&input).unwrap();
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
        let expr = CalcExpression::parse(&input).unwrap();
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
        let expr = CalcExpression::parse(&input).unwrap();
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
        let expr = CalcExpression::parse(&input).unwrap();
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
        let expr = CalcExpression::parse(&input).unwrap();
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
        let expr = CalcExpression::parse(&input).unwrap();
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
        let expr = CalcExpression::parse(&input).unwrap();
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
        let expr = CalcExpression::parse(&input).unwrap();
        let (rel_ctx, abs_ctx) = create_test_contexts();
        let result = expr.to_px(None, &rel_ctx, &abs_ctx);
        assert_eq!(result, 4.0);
    }

    #[test]
    fn test_whitespace_required_around_plus() {
        let input = vec![number_token(10.0), delim_token('+'), number_token(5.0)];
        let result = CalcExpression::parse(&input);
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
        let result = CalcExpression::parse(&input);
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
        let result = CalcExpression::parse(&input);
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
        let result = CalcExpression::parse(&input);
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
        let expr = CalcExpression::parse(&input).unwrap();
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
        let expr = CalcExpression::parse(&input).unwrap();
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
        let expr = CalcExpression::parse(&input).unwrap();
        let (rel_ctx, abs_ctx) = create_test_contexts();
        let result = expr.to_px(None, &rel_ctx, &abs_ctx);
        assert_eq!(result, -50.0);
    }

    #[test]
    fn test_no_whitespace_required_for_multiply() {
        let input = vec![number_token(10.0), delim_token('*'), number_token(5.0)];
        let expr = CalcExpression::parse(&input).unwrap();
        let (rel_ctx, abs_ctx) = create_test_contexts();
        let result = expr.to_px(None, &rel_ctx, &abs_ctx);
        assert_eq!(result, 50.0);
    }

    #[test]
    fn test_no_whitespace_required_for_divide() {
        let input = vec![number_token(10.0), delim_token('/'), number_token(5.0)];
        let expr = CalcExpression::parse(&input).unwrap();
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
        let expr = CalcExpression::parse(&input).unwrap();
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
        let result = CalcExpression::parse(&input);
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
        let expr = CalcExpression::parse(&input).unwrap();
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
        let expr = CalcExpression::parse(&input).unwrap();
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
        let expr = CalcExpression::parse(&input).unwrap();
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
        let expr = CalcExpression::parse(&input).unwrap();
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
        let expr = CalcExpression::parse(&input).unwrap();
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
        let expr = CalcExpression::parse(&input).unwrap();
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
        let expr = CalcExpression::parse(&input).unwrap();
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
        let expr = CalcExpression::parse(&input).unwrap();
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
        let expr = CalcExpression::parse(&input).unwrap();
        let (rel_ctx, abs_ctx) = create_test_contexts();
        let result = expr.to_px(None, &rel_ctx, &abs_ctx);
        assert_eq!(result, 100.0);
    }

    #[test]
    fn test_min_expression_standalone() {
        let input = vec![number_token(100.0), comma_token(), number_token(200.0)];
        let expr = CalcExpression::parse_math_function("min", &input).unwrap();
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
        let expr = CalcExpression::parse(&input).unwrap();
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
        let expr = CalcExpression::parse(&input).unwrap();
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
        let expr = CalcExpression::parse(&input).unwrap();
        let (rel_ctx, abs_ctx) = create_test_contexts();
        let result = expr.to_px(None, &rel_ctx, &abs_ctx);
        assert_eq!(result, 100.0);
    }

    #[test]
    fn test_max_expression_standalone() {
        let input = vec![number_token(100.0), comma_token(), number_token(200.0)];
        let expr = CalcExpression::parse_math_function("max", &input).unwrap();
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
        let expr = CalcExpression::parse(&input).unwrap();
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
        let expr = CalcExpression::parse(&input).unwrap();
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
        let expr = CalcExpression::parse(&input).unwrap();
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
        let expr = CalcExpression::parse(&input).unwrap();
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
        let expr = CalcExpression::parse_math_function("clamp", &input).unwrap();
        let (rel_ctx, abs_ctx) = create_test_contexts();
        let result = expr.to_px(None, &rel_ctx, &abs_ctx);
        assert_eq!(result, 150.0);
    }

    #[test]
    fn test_clamp_wrong_arg_count_two() {
        let input = vec![number_token(100.0), comma_token(), number_token(200.0)];
        let result = CalcExpression::parse_math_function("clamp", &input);
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
        let result = CalcExpression::parse_math_function("clamp", &input);
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
        let expr = CalcExpression::parse(&input).unwrap();
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
        let expr = CalcExpression::parse(&input).unwrap();
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
        let expr = CalcExpression::parse(&input).unwrap();
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
        let expr = CalcExpression::parse(&input).unwrap();
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
        let expr = CalcExpression::parse(&input).unwrap();
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
        let expr = CalcExpression::parse(&input).unwrap();
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
        let expr = CalcExpression::parse(&input).unwrap();
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
        let expr = CalcExpression::parse(&input).unwrap();
        let (rel_ctx, abs_ctx) = create_test_contexts();
        let result = expr.to_px(None, &rel_ctx, &abs_ctx);
        assert_eq!(result, 150.0);
    }
}

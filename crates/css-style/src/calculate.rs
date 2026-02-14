use std::str::FromStr;

use css_cssom::{ComponentValue, CssTokenKind};

use crate::{
    length::LengthUnit,
    primitives::{length::Length, percentage::Percentage},
    properties::{AbsoluteContext, RelativeContext, RelativeType},
};

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

#[derive(Debug, Clone, PartialEq)]
pub enum CalculateValue {
    Number(f32),
    Length(Length),
    Percentage(Percentage),
    Keyword(CalculateKeyword),
    NestedSum(Box<CalculateSum>),
}

impl CalculateValue {
    pub fn to_px(
        &self,
        rel_type: Option<RelativeType>,
        rel_ctx: &RelativeContext,
        abs_ctx: &AbsoluteContext,
    ) -> f32 {
        match self {
            CalculateValue::Number(n) => *n,
            CalculateValue::Length(l) => l.to_px(rel_ctx, abs_ctx),
            CalculateValue::Keyword(k) => k.to_f32(),
            CalculateValue::NestedSum(sum) => sum.to_px(rel_type, rel_ctx, abs_ctx),
            CalculateValue::Percentage(p) => match rel_type {
                Some(RelativeType::FontSize) => rel_ctx.parent_font_size * p.as_fraction(),
                Some(RelativeType::ParentHeight) => rel_ctx.parent_height * p.as_fraction(),
                Some(RelativeType::ParentWidth) => rel_ctx.parent_width * p.as_fraction(),
                Some(RelativeType::RootFontSize) => abs_ctx.root_font_size * p.as_fraction(),
                Some(RelativeType::ViewportHeight) => abs_ctx.viewport_height * p.as_fraction(),
                Some(RelativeType::ViewportWidth) => abs_ctx.viewport_width * p.as_fraction(),
                None => 0.0,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CalculateProduct {
    Value(CalculateValue),
    Multiply(Box<CalculateProduct>, Box<CalculateProduct>),
    Divide(Box<CalculateProduct>, Box<CalculateProduct>),
}

impl CalculateProduct {
    pub fn to_px(
        &self,
        rel_type: Option<RelativeType>,
        rel_ctx: &RelativeContext,
        abs_ctx: &AbsoluteContext,
    ) -> f32 {
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

#[derive(Debug, Clone, PartialEq)]
pub enum CalculateSum {
    Product(CalculateProduct),
    Add(Box<CalculateSum>, Box<CalculateSum>),
    Subtract(Box<CalculateSum>, Box<CalculateSum>),
}

impl CalculateSum {
    pub fn to_px(
        &self,
        rel_type: Option<RelativeType>,
        rel_ctx: &RelativeContext,
        abs_ctx: &AbsoluteContext,
    ) -> f32 {
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

/// Represents a CSS calc() expression.
///
/// ## Whitespace Requirements
///
/// According to the CSS specification, whitespace is **required** on both sides
/// of the `+` and `-` operators. This is necessary to disambiguate expressions like:
/// - `calc(50px - -20px)` (valid: 50px minus negative 20px = 70px)
/// - `calc(50px--20px)` (invalid: missing whitespace)
///
/// Whitespace is **optional** around the `*` and `/` operators:
/// - `calc(10*5)` (valid)
/// - `calc(10 * 5)` (also valid)
#[derive(Debug, Clone, PartialEq)]
pub struct CalcExpression {
    pub sum: CalculateSum,
}

impl CalcExpression {
    pub fn to_px(
        &self,
        rel_type: Option<RelativeType>,
        rel_ctx: &RelativeContext,
        abs_ctx: &AbsoluteContext,
    ) -> f32 {
        self.sum.to_px(rel_type, rel_ctx, abs_ctx)
    }

    pub fn parse(input: &[ComponentValue]) -> Result<Self, String> {
        let mut parser = CalcParser::new(input);
        let sum = parser.parse_sum()?;

        if parser.current_pos < parser.input.len() {
            return Err(format!(
                "Unexpected trailing input at position {}",
                parser.current_pos
            ));
        }

        Ok(CalcExpression { sum })
    }
}

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

            let is_plus_or_minus =
                matches!(self.peek_token(), Some(CssTokenKind::Delim('+' | '-')));
            if !is_plus_or_minus {
                break;
            }

            if !had_whitespace_before {
                return Err(
                    "Whitespace is required before '+' or '-' operator in calc()".to_string(),
                );
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
                return Err(
                    "Whitespace is required after '+' or '-' operator in calc()".to_string()
                );
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

            ComponentValue::SimpleBlock(block)
                if matches!(
                    block.associated_token,
                    css_cssom::AssociatedToken::Parenthesis
                ) =>
            {
                self.current_pos += 1;
                let nested = CalcExpression::parse(&block.value)?;
                Ok(CalculateValue::NestedSum(Box::new(nested.sum)))
            }

            ComponentValue::Token(token) => match &token.kind {
                CssTokenKind::Number(num) => {
                    self.current_pos += 1;
                    Ok(CalculateValue::Number(num.value as f32))
                }

                CssTokenKind::Dimension { value, unit } => {
                    self.current_pos += 1;
                    let len_unit = unit
                        .parse::<LengthUnit>()
                        .map_err(|_| format!("Invalid length unit: {}", unit))?;
                    Ok(CalculateValue::Length(Length::new(
                        value.value as f32,
                        len_unit,
                    )))
                }

                CssTokenKind::Percentage(num) => {
                    self.current_pos += 1;
                    Ok(CalculateValue::Percentage(Percentage::new(
                        num.value as f32,
                    )))
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use css_cssom::{ComponentValue, CssToken, CssTokenKind, NumericValue};

    /// Helper function to create test contexts
    fn create_test_contexts() -> (RelativeContext, AbsoluteContext) {
        let rel_ctx = RelativeContext {
            parent_font_size: 16.0,
            parent_width: 800.0,
            parent_height: 600.0,
        };
        let abs_ctx = AbsoluteContext {
            root_font_size: 16.0,
            viewport_width: 1024.0,
            viewport_height: 768.0,
        };
        (rel_ctx, abs_ctx)
    }

    /// Helper function to create a number token
    fn number_token(value: f64) -> ComponentValue {
        ComponentValue::Token(CssToken {
            kind: CssTokenKind::Number(NumericValue {
                value,
                int_value: None,
                type_flag: css_cssom::NumberType::Number,
                repr: value.to_string(),
            }),
            position: None,
        })
    }

    /// Helper function to create a dimension token
    fn dimension_token(value: f64, unit: &str) -> ComponentValue {
        ComponentValue::Token(CssToken {
            kind: CssTokenKind::Dimension {
                value: NumericValue {
                    value,
                    int_value: None,
                    type_flag: css_cssom::NumberType::Number,
                    repr: value.to_string(),
                },
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
        let input = vec![ComponentValue::Function(css_cssom::Function {
            name: "calc".to_string(),
            value: vec![
                number_token(10.0),
                whitespace_token(),
                delim_token('+'),
                whitespace_token(),
                ComponentValue::Function(css_cssom::Function {
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
        assert_eq!(result, 10.0 + (2.0 * rel_ctx.parent_font_size));
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
                kind: CssTokenKind::Percentage(NumericValue {
                    value: 50.0,
                    int_value: None,
                    type_flag: css_cssom::NumberType::Number,
                    repr: "50%".to_string(),
                }),
                position: None,
            }),
        ];
        let expr = CalcExpression::parse(&input).unwrap();
        let (rel_ctx, abs_ctx) = create_test_contexts();
        let result = expr.to_px(Some(RelativeType::ParentWidth), &rel_ctx, &abs_ctx);
        assert_eq!(result, 10.0 + (0.5 * rel_ctx.parent_width));
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
            dimension_token(10.0, "px"),
            whitespace_token(),
            delim_token('+'),
            whitespace_token(),
            dimension_token(2.0, "vh"),
        ];
        let expr = CalcExpression::parse(&input).unwrap();
        let (rel_ctx, abs_ctx) = create_test_contexts();
        let result = expr.to_px(Some(RelativeType::ViewportHeight), &rel_ctx, &abs_ctx);
        assert_eq!(result, 10.0 + (0.02 * abs_ctx.viewport_height));
    }
}

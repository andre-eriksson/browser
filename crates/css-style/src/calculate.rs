use std::str::FromStr;

use crate::{
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
        rel_type: RelativeType,
        rel_ctx: &RelativeContext,
        abs_ctx: &AbsoluteContext,
    ) -> f32 {
        match self {
            CalculateValue::Number(n) => *n,
            CalculateValue::Length(l) => l.to_px(rel_ctx, abs_ctx),
            CalculateValue::Keyword(k) => k.to_f32(),
            CalculateValue::NestedSum(sum) => sum.to_px(rel_type, rel_ctx, abs_ctx),
            CalculateValue::Percentage(p) => match rel_type {
                RelativeType::FontSize => rel_ctx.font_size * p.as_fraction(),
                RelativeType::ParentHeight => rel_ctx.parent_height * p.as_fraction(),
                RelativeType::ParentWidth => rel_ctx.parent_width * p.as_fraction(),
                RelativeType::RootFontSize => abs_ctx.root_font_size * p.as_fraction(),
                RelativeType::ViewportHeight => abs_ctx.viewport_height * p.as_fraction(),
                RelativeType::ViewportWidth => abs_ctx.viewport_width * p.as_fraction(),
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
        rel_type: RelativeType,
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
        rel_type: RelativeType,
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

#[derive(Debug, Clone, PartialEq)]
pub struct CalcExpression {
    pub sum: CalculateSum,
}

impl CalcExpression {
    pub fn to_px(
        &self,
        rel_type: RelativeType,
        rel_ctx: &RelativeContext,
        abs_ctx: &AbsoluteContext,
    ) -> f32 {
        self.sum.to_px(rel_type, rel_ctx, abs_ctx)
    }

    pub fn parse(input: &str) -> Result<Self, String> {
        let trimmed = input.trim();

        let owned_inner;
        let inner: &str = if let Some(stripped) = trimmed.strip_prefix("calc(") {
            let mut depth: usize = 1;
            let mut close_idx = None;
            for (i, ch) in stripped.char_indices() {
                match ch {
                    '(' => depth += 1,
                    ')' => {
                        depth -= 1;
                        if depth == 0 {
                            close_idx = Some(5 + i);
                            break;
                        }
                    }
                    _ => {}
                }
            }

            let Some(close_idx) = close_idx else {
                return Err("Unclosed calc() expression".to_string());
            };

            if trimmed[close_idx + 1..].trim().is_empty() {
                owned_inner = trimmed[5..close_idx].to_string();
                &owned_inner
            } else {
                return Err(format!(
                    "Unexpected trailing input: {}",
                    trimmed[close_idx + 1..].trim()
                ));
            }
        } else if trimmed.starts_with('(') && trimmed.ends_with(')') {
            &trimmed[1..trimmed.len() - 1]
        } else {
            trimmed
        };

        let mut parser = CalcParser::new(inner);
        let sum = parser.parse_sum()?;
        parser.skip_whitespace();
        if parser.current_pos < parser.input.len() {
            return Err(format!(
                "Unexpected trailing input: {}",
                parser.input[parser.current_pos..]
                    .iter()
                    .collect::<String>()
            ));
        }

        Ok(CalcExpression { sum })
    }
}

struct CalcParser {
    input: Vec<char>,
    current_pos: usize,
}

impl CalcParser {
    fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            current_pos: 0,
        }
    }

    fn skip_whitespace(&mut self) {
        while self.current_pos < self.input.len() && self.input[self.current_pos].is_whitespace() {
            self.current_pos += 1;
        }
    }

    fn parse_sum(&mut self) -> Result<CalculateSum, String> {
        let mut left = CalculateSum::Product(self.parse_product()?);

        loop {
            self.skip_whitespace();
            if self.current_pos >= self.input.len() {
                break;
            }

            let op = self.input[self.current_pos];
            if op == '+' || op == '-' {
                self.current_pos += 1;

                let next_product = self.parse_product()?;
                let right = CalculateSum::Product(next_product);

                if op == '+' {
                    left = CalculateSum::Add(Box::new(left), Box::new(right));
                } else {
                    left = CalculateSum::Subtract(Box::new(left), Box::new(right));
                }
            } else {
                break;
            }
        }

        Ok(left)
    }

    fn parse_product(&mut self) -> Result<CalculateProduct, String> {
        let mut left = CalculateProduct::Value(self.parse_value()?);

        loop {
            self.skip_whitespace();
            if self.current_pos >= self.input.len() {
                break;
            }

            let op = self.input[self.current_pos];
            if op == '*' || op == '/' {
                self.current_pos += 1;

                let next_value = self.parse_value()?;
                let right = CalculateProduct::Value(next_value);

                if op == '*' {
                    left = CalculateProduct::Multiply(Box::new(left), Box::new(right));
                } else {
                    left = CalculateProduct::Divide(Box::new(left), Box::new(right));
                }
            } else {
                break;
            }
        }

        Ok(left)
    }

    fn parse_value(&mut self) -> Result<CalculateValue, String> {
        self.skip_whitespace();
        if self.current_pos >= self.input.len() {
            return Err("Unexpected end of input".to_string());
        }

        if self.current_pos + 5 <= self.input.len()
            && self.input[self.current_pos..self.current_pos + 5] == ['c', 'a', 'l', 'c', '(']
        {
            self.current_pos += 5;
            let nested = self.parse_sum()?;
            self.skip_whitespace();
            if self.current_pos >= self.input.len() || self.input[self.current_pos] != ')' {
                return Err("Expected closing ')' in nested calc() expression".to_string());
            }
            self.current_pos += 1;
            return Ok(CalculateValue::NestedSum(Box::new(nested)));
        }

        if self.input[self.current_pos] == '(' {
            self.current_pos += 1;
            let nested = self.parse_sum()?;
            self.skip_whitespace();
            if self.current_pos >= self.input.len() || self.input[self.current_pos] != ')' {
                return Err("Expected closing ')' in calc() expression".to_string());
            }
            self.current_pos += 1;
            return Ok(CalculateValue::NestedSum(Box::new(nested)));
        }

        let start_pos = self.current_pos;

        if ['+', '-'].contains(&self.input[self.current_pos]) {
            self.current_pos += 1;
        }

        while self.current_pos < self.input.len() {
            let ch = self.input[self.current_pos];
            if ch.is_whitespace() || ['+', '-', '*', '/', '(', ')'].contains(&ch) {
                break;
            }
            self.current_pos += 1;
        }

        let token: String = self.input[start_pos..self.current_pos].iter().collect();
        if token.is_empty() || token == "+" || token == "-" {
            return Err("Expected a value".to_string());
        }

        if let Ok(num) = token.parse::<f32>() {
            Ok(CalculateValue::Number(num))
        } else if let Ok(length) = token.parse::<Length>() {
            Ok(CalculateValue::Length(length))
        } else if let Ok(percentage) = token.parse::<Percentage>() {
            Ok(CalculateValue::Percentage(percentage))
        } else if let Ok(keyword) = token.parse::<CalculateKeyword>() {
            Ok(CalculateValue::Keyword(keyword))
        } else {
            Err(format!("Invalid calc() value token: {}", token))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_product_multiply() {
        let mut parser = CalcParser::new("10px * 2");
        let product = parser.parse_product().unwrap();
        assert_eq!(
            product,
            CalculateProduct::Multiply(
                CalculateProduct::Value(CalculateValue::Length(Length::px(10.0))).into(),
                CalculateProduct::Value(CalculateValue::Number(2.0)).into()
            )
        );
    }

    #[test]
    fn test_parse_product_divide() {
        let mut parser = CalcParser::new("20px / 4");
        let product = parser.parse_product().unwrap();
        assert_eq!(
            product,
            CalculateProduct::Divide(
                CalculateProduct::Value(CalculateValue::Length(Length::px(20.0))).into(),
                CalculateProduct::Value(CalculateValue::Number(4.0)).into()
            )
        );
    }

    #[test]
    fn test_parse_sum_add() {
        let mut parser = CalcParser::new("10px + 5px");
        let sum = parser.parse_sum().unwrap();
        assert_eq!(
            sum,
            CalculateSum::Add(
                CalculateSum::Product(CalculateProduct::Value(CalculateValue::Length(Length::px(
                    10.0
                ))))
                .into(),
                CalculateSum::Product(CalculateProduct::Value(CalculateValue::Length(Length::px(
                    5.0
                ))))
                .into()
            )
        );
    }

    #[test]
    fn test_parse_sum_subtract() {
        let mut parser = CalcParser::new("15px - 5px");
        let sum = parser.parse_sum().unwrap();
        assert_eq!(
            sum,
            CalculateSum::Subtract(
                CalculateSum::Product(CalculateProduct::Value(CalculateValue::Length(Length::px(
                    15.0
                ))))
                .into(),
                CalculateSum::Product(CalculateProduct::Value(CalculateValue::Length(Length::px(
                    5.0
                ))))
                .into()
            )
        );
    }

    #[test]
    fn test_calculate_expression() {
        let ctx = RelativeContext::default();

        let expr = CalcExpression {
            sum: CalculateSum::Add(
                CalculateSum::Product(CalculateProduct::Value(CalculateValue::Length(Length::px(
                    10.0,
                ))))
                .into(),
                CalculateSum::Product(CalculateProduct::Value(CalculateValue::Length(Length::px(
                    5.0,
                ))))
                .into(),
            ),
        };

        let result = expr.to_px(RelativeType::FontSize, &ctx, &AbsoluteContext::default());
        assert_eq!(result, 15.0);
    }

    #[test]
    fn test_calculate_expression_with_percentage() {
        let ctx = RelativeContext {
            parent_width: 400.0,
            ..Default::default()
        };

        let expr = CalcExpression {
            sum: CalculateSum::Add(
                CalculateSum::Product(CalculateProduct::Value(CalculateValue::Percentage(
                    Percentage::new(50.0),
                )))
                .into(),
                CalculateSum::Product(CalculateProduct::Value(CalculateValue::Length(Length::px(
                    20.0,
                ))))
                .into(),
            ),
        };

        let result = expr.to_px(RelativeType::ParentWidth, &ctx, &AbsoluteContext::default());
        assert_eq!(result, 220.0);
    }

    #[test]
    fn test_calculate_expression_with_keyword() {
        let ctx = RelativeContext::default();

        let expr = CalcExpression {
            sum: CalculateSum::Add(
                CalculateSum::Product(CalculateProduct::Value(CalculateValue::Keyword(
                    CalculateKeyword::PI,
                )))
                .into(),
                CalculateSum::Product(CalculateProduct::Value(CalculateValue::Length(Length::px(
                    10.0,
                ))))
                .into(),
            ),
        };

        let result = expr.to_px(RelativeType::FontSize, &ctx, &AbsoluteContext::default());
        assert!((result - (std::f32::consts::PI + 10.0)).abs() < f32::EPSILON);
    }

    #[test]
    fn test_parse_complex_calculate_expression() {
        let expr = "calc(10px + 5px * 2 - 3px / 1.5 + 50%)";
        let parsed_expr = CalcExpression::parse(expr).unwrap();
        let ctx = RelativeContext {
            parent_width: 400.0,
            ..Default::default()
        };
        let result =
            parsed_expr.to_px(RelativeType::ParentWidth, &ctx, &AbsoluteContext::default());

        assert_eq!(result, 218.0);
    }

    #[test]
    fn test_nested_parentheses() {
        let expr = CalcExpression::parse("calc((10px + 5px) * 2)").unwrap();
        let result = expr.to_px(
            RelativeType::ParentWidth,
            &RelativeContext::default(),
            &AbsoluteContext::default(),
        );
        assert_eq!(result, 30.0);
    }

    #[test]
    fn test_reject_trailing_input() {
        let err = CalcExpression::parse("calc(10px + 5px) foo").unwrap_err();
        assert!(err.contains("Unexpected trailing input"));
    }

    #[test]
    fn test_reject_dimension_keyword_token() {
        let err = CalcExpression::parse("calc(auto + 1px)").unwrap_err();
        assert!(err.contains("Invalid calc() value token"));
    }
}

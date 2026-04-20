use css_values::calc::{CalcExpression, CalcProduct, CalcSum, CalcValue};

use crate::{AbsoluteContext, RelativeContext, RelativeType, properties::PixelRepr};

impl PixelRepr for CalcValue {
    fn to_px(
        &self,
        rel_type: Option<RelativeType>,
        rel_ctx: Option<&RelativeContext>,
        abs_ctx: &AbsoluteContext,
    ) -> f64 {
        match self {
            Self::Number(n) => *n,
            Self::Length(l) => l.to_px(rel_type, rel_ctx, abs_ctx),
            Self::Keyword(k) => k.to_f64(),
            Self::NestedSum(sum) => sum.to_px(rel_type, rel_ctx, abs_ctx),
            Self::Percentage(p) => match rel_type {
                Some(RelativeType::FontSize) => rel_ctx
                    .map_or(abs_ctx.root_font_size * p.as_fraction(), |ctx| ctx.parent.font_size * p.as_fraction()),
                Some(RelativeType::ParentHeight) => rel_ctx.map_or(abs_ctx.viewport_height * p.as_fraction(), |ctx| {
                    ctx.parent.intrinsic_height * p.as_fraction()
                }),
                Some(RelativeType::ParentWidth) => rel_ctx.map_or(abs_ctx.viewport_width * p.as_fraction(), |ctx| {
                    ctx.parent.intrinsic_width * p.as_fraction()
                }),
                Some(RelativeType::RootFontSize) => abs_ctx.root_font_size * p.as_fraction(),
                Some(RelativeType::ViewportHeight) => abs_ctx.viewport_height * p.as_fraction(),
                Some(RelativeType::ViewportWidth) => abs_ctx.viewport_width * p.as_fraction(),
                None => 0.0,
            },
            Self::Min(args) => args
                .iter()
                .map(|sum| sum.to_px(rel_type, rel_ctx, abs_ctx))
                .fold(f64::INFINITY, f64::min),
            Self::Max(args) => args
                .iter()
                .map(|sum| sum.to_px(rel_type, rel_ctx, abs_ctx))
                .fold(f64::NEG_INFINITY, f64::max),
            Self::Clamp(args) => {
                let min_val = args
                    .min
                    .as_ref()
                    .map_or(f64::NEG_INFINITY, |s| s.to_px(rel_type, rel_ctx, abs_ctx));
                let val_val = args.val.to_px(rel_type, rel_ctx, abs_ctx);
                let max_val = args
                    .max
                    .as_ref()
                    .map_or(f64::INFINITY, |s| s.to_px(rel_type, rel_ctx, abs_ctx));
                val_val.clamp(min_val, max_val)
            }
        }
    }
}

impl PixelRepr for CalcProduct {
    fn to_px(
        &self,
        rel_type: Option<RelativeType>,
        rel_ctx: Option<&RelativeContext>,
        abs_ctx: &AbsoluteContext,
    ) -> f64 {
        match self {
            Self::Value(v) => v.to_px(rel_type, rel_ctx, abs_ctx),
            Self::Multiply(left, right) => {
                left.to_px(rel_type, rel_ctx, abs_ctx) * right.to_px(rel_type, rel_ctx, abs_ctx)
            }
            Self::Divide(left, right) => {
                let divisor = right.to_px(rel_type, rel_ctx, abs_ctx);
                if divisor == 0.0 {
                    f64::NAN
                } else {
                    left.to_px(rel_type, rel_ctx, abs_ctx) / divisor
                }
            }
        }
    }
}

impl PixelRepr for CalcSum {
    fn to_px(
        &self,
        rel_type: Option<RelativeType>,
        rel_ctx: Option<&RelativeContext>,
        abs_ctx: &AbsoluteContext,
    ) -> f64 {
        match self {
            Self::Product(p) => p.to_px(rel_type, rel_ctx, abs_ctx),
            Self::Add(left, right) => left.to_px(rel_type, rel_ctx, abs_ctx) + right.to_px(rel_type, rel_ctx, abs_ctx),
            Self::Subtract(left, right) => {
                left.to_px(rel_type, rel_ctx, abs_ctx) - right.to_px(rel_type, rel_ctx, abs_ctx)
            }
        }
    }
}

impl PixelRepr for CalcExpression {
    fn to_px(
        &self,
        rel_type: Option<RelativeType>,
        rel_ctx: Option<&RelativeContext>,
        abs_ctx: &AbsoluteContext,
    ) -> f64 {
        self.sum.to_px(rel_type, rel_ctx, abs_ctx)
    }
}

#[cfg(test)]
mod tests {
    use std::net::Ipv4Addr;

    use crate::ComputedStyle;

    use super::*;
    use css_cssom::{ComponentValue, CssToken, CssTokenKind, Function, NumericValue};
    use css_values::error::CssValueError;
    use url::Url;

    /// Helper function to create test contexts
    fn create_test_contexts() -> (RelativeContext, AbsoluteContext<'static>) {
        let url = Box::leak(Box::new(Url::parse(&format!("http://{}", Ipv4Addr::LOCALHOST)).unwrap()));
        let rel_ctx = RelativeContext {
            parent: ComputedStyle {
                font_size: 16.0,
                intrinsic_width: 800.0,
                intrinsic_height: 600.0,
                ..Default::default()
            }
            .into(),
            font_size: 16.0,
        };
        let abs_ctx = AbsoluteContext {
            root_font_size: 16.0,
            viewport_width: 1024.0,
            viewport_height: 768.0,
            ..AbsoluteContext::default_url(url)
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
        let result = expr.to_px(None, Some(&rel_ctx), &abs_ctx);
        assert_eq!(result, 42.0);
    }

    #[test]
    fn test_simple_negative_number() {
        let input = vec![number_token(-42.0)];
        let expr = CalcExpression::parse(&input).unwrap();
        let (rel_ctx, abs_ctx) = create_test_contexts();
        let result = expr.to_px(None, Some(&rel_ctx), &abs_ctx);
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
        let result = expr.to_px(None, Some(&rel_ctx), &abs_ctx);
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
        let result = expr.to_px(None, Some(&rel_ctx), &abs_ctx);
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
        let result = expr.to_px(None, Some(&rel_ctx), &abs_ctx);
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
        let result = expr.to_px(None, Some(&rel_ctx), &abs_ctx);
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
        let result = expr.to_px(None, Some(&rel_ctx), &abs_ctx);
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
        let result = expr.to_px(None, Some(&rel_ctx), &abs_ctx);
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
        let result = expr.to_px(None, Some(&rel_ctx), &abs_ctx);
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
        let result = expr.to_px(None, Some(&rel_ctx), &abs_ctx);
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
        let result = expr.to_px(None, Some(&rel_ctx), &abs_ctx);
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
        let result = expr.to_px(None, Some(&rel_ctx), &abs_ctx);
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
        let result = expr.to_px(None, Some(&rel_ctx), &abs_ctx);
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
        let result = expr.to_px(None, Some(&rel_ctx), &abs_ctx);
        assert!((result - (std::f64::consts::PI * 2.0)).abs() < 0.001);
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
        let result = expr.to_px(None, Some(&rel_ctx), &abs_ctx);
        assert!((result - (std::f64::consts::E * 2.0)).abs() < 0.001);
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
        let result = expr.to_px(None, Some(&rel_ctx), &abs_ctx);
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
        let result = expr.to_px(None, Some(&rel_ctx), &abs_ctx);
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
        let result = expr.to_px(None, Some(&rel_ctx), &abs_ctx);
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
        let result = expr.to_px(None, Some(&rel_ctx), &abs_ctx);
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
        let result = expr.to_px(None, Some(&rel_ctx), &abs_ctx);
        assert_eq!(result, 4.0);
    }

    #[test]
    fn test_whitespace_required_around_plus() {
        let input = vec![number_token(10.0), delim_token('+'), number_token(5.0)];
        let result = CalcExpression::parse(&input);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            CssValueError::InvalidValue("Whitespace is required before '+' or '-' operator in calc()".into())
        );
    }

    #[test]
    fn test_whitespace_required_around_minus() {
        let input = vec![number_token(10.0), delim_token('-'), number_token(5.0)];
        let result = CalcExpression::parse(&input);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            CssValueError::InvalidValue("Whitespace is required before '+' or '-' operator in calc()".into())
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
        assert_eq!(
            result.unwrap_err(),
            CssValueError::InvalidValue("Whitespace is required after '+' or '-' operator in calc()".into())
        );
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
        assert_eq!(
            result.unwrap_err(),
            CssValueError::InvalidValue("Whitespace is required after '+' or '-' operator in calc()".into())
        );
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
        let result = expr.to_px(None, Some(&rel_ctx), &abs_ctx);
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
        let result = expr.to_px(None, Some(&rel_ctx), &abs_ctx);
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
        let result = expr.to_px(None, Some(&rel_ctx), &abs_ctx);
        assert_eq!(result, -50.0);
    }

    #[test]
    fn test_no_whitespace_required_for_multiply() {
        let input = vec![number_token(10.0), delim_token('*'), number_token(5.0)];
        let expr = CalcExpression::parse(&input).unwrap();
        let (rel_ctx, abs_ctx) = create_test_contexts();
        let result = expr.to_px(None, Some(&rel_ctx), &abs_ctx);
        assert_eq!(result, 50.0);
    }

    #[test]
    fn test_no_whitespace_required_for_divide() {
        let input = vec![number_token(10.0), delim_token('/'), number_token(5.0)];
        let expr = CalcExpression::parse(&input).unwrap();
        let (rel_ctx, abs_ctx) = create_test_contexts();
        let result = expr.to_px(None, Some(&rel_ctx), &abs_ctx);
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
        let result = expr.to_px(None, Some(&rel_ctx), &abs_ctx);
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
        assert_eq!(
            result.unwrap_err(),
            CssValueError::InvalidValue("Whitespace is required before '+' or '-' operator in calc()".into())
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
        let result = expr.to_px(None, Some(&rel_ctx), &abs_ctx);
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
        let result = expr.to_px(Some(RelativeType::FontSize), Some(&rel_ctx), &abs_ctx);
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
        let result = expr.to_px(Some(RelativeType::RootFontSize), Some(&rel_ctx), &abs_ctx);
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
        let result = expr.to_px(Some(RelativeType::ParentWidth), Some(&rel_ctx), &abs_ctx);
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
        let result = expr.to_px(Some(RelativeType::ViewportWidth), Some(&rel_ctx), &abs_ctx);
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
        let result = expr.to_px(None, Some(&rel_ctx), &abs_ctx);
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
        let result = expr.to_px(None, Some(&rel_ctx), &abs_ctx);
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
        let result = expr.to_px(None, Some(&rel_ctx), &abs_ctx);
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
        let result = expr.to_px(None, Some(&rel_ctx), &abs_ctx);
        assert_eq!(result, 100.0);
    }

    #[test]
    fn test_min_expression_standalone() {
        let input = vec![number_token(100.0), comma_token(), number_token(200.0)];
        let expr = CalcExpression::parse_math_function("min", &input).unwrap();
        let (rel_ctx, abs_ctx) = create_test_contexts();
        let result = expr.to_px(None, Some(&rel_ctx), &abs_ctx);
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
        let result = expr.to_px(None, Some(&rel_ctx), &abs_ctx);
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
        let result = expr.to_px(None, Some(&rel_ctx), &abs_ctx);
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
        let result = expr.to_px(None, Some(&rel_ctx), &abs_ctx);
        assert_eq!(result, 100.0);
    }

    #[test]
    fn test_max_expression_standalone() {
        let input = vec![number_token(100.0), comma_token(), number_token(200.0)];
        let expr = CalcExpression::parse_math_function("max", &input).unwrap();
        let (rel_ctx, abs_ctx) = create_test_contexts();
        let result = expr.to_px(None, Some(&rel_ctx), &abs_ctx);
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
        let result = expr.to_px(None, Some(&rel_ctx), &abs_ctx);
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
        let result = expr.to_px(None, Some(&rel_ctx), &abs_ctx);
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
        let result = expr.to_px(None, Some(&rel_ctx), &abs_ctx);
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
        let result = expr.to_px(None, Some(&rel_ctx), &abs_ctx);
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
        let result = expr.to_px(None, Some(&rel_ctx), &abs_ctx);
        assert_eq!(result, 150.0);
    }

    #[test]
    fn test_clamp_wrong_arg_count_two() {
        let input = vec![number_token(100.0), comma_token(), number_token(200.0)];
        let result = CalcExpression::parse_math_function("clamp", &input);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            CssValueError::InvalidValue("clamp() requires exactly 3 arguments separated by commas, got 2".into())
        );
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
        assert_eq!(
            result.unwrap_err(),
            CssValueError::InvalidValue("clamp() requires exactly 3 arguments separated by commas, got 4".into())
        );
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
        let result = expr.to_px(None, Some(&rel_ctx), &abs_ctx);
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
        let result = expr.to_px(None, Some(&rel_ctx), &abs_ctx);
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
        let result = expr.to_px(None, Some(&rel_ctx), &abs_ctx);
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
        let result = expr.to_px(None, Some(&rel_ctx), &abs_ctx);
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
        let result = expr.to_px(None, Some(&rel_ctx), &abs_ctx);
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
        let result = expr.to_px(None, Some(&rel_ctx), &abs_ctx);
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
        let result = expr.to_px(None, Some(&rel_ctx), &abs_ctx);
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
        let result = expr.to_px(None, Some(&rel_ctx), &abs_ctx);
        assert_eq!(result, 150.0);
    }
}

use css_cssom::{ComponentValue, ComponentValueStream, CssTokenKind};

use crate::{
    CSSParsable,
    calc::{CalcDomain, CalcExpression, is_math_function},
    error::CssValueError,
    numeric::Percentage,
    quantity::Length,
};

#[derive(Debug, Clone, Default, PartialEq)]
pub enum Gap {
    #[default]
    Normal,
    Length(Length),
    Percentage(Percentage),
    Calc(CalcExpression),
}

impl CSSParsable for Gap {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, crate::error::CssValueError> {
        if let Some(cv) = stream.next_non_whitespace() {
            match cv {
                ComponentValue::Function(func) => {
                    if is_math_function(&func.name) {
                        let expr = CalcExpression::parse_math_function(&func.name, &func.value)?;
                        let domain = expr.resolve_domain()?;

                        if !matches!(domain, CalcDomain::Length | CalcDomain::Percentage) {
                            return Err(CssValueError::InvalidCalcDomain {
                                expected: vec![CalcDomain::Length, CalcDomain::Percentage],
                                found: domain,
                            });
                        }

                        Ok(Self::Calc(expr))
                    } else {
                        Err(CssValueError::InvalidFunction(func.name.clone()))
                    }
                }
                ComponentValue::Token(token) => {
                    if let Ok(length) = Length::try_from(token) {
                        Ok(Self::Length(length))
                    } else if let Ok(percentage) = Percentage::try_from(token) {
                        Ok(Self::Percentage(percentage))
                    } else if let CssTokenKind::Ident(ident) = &token.kind
                        && ident.eq_ignore_ascii_case("normal")
                    {
                        Ok(Self::Normal)
                    } else {
                        Err(CssValueError::InvalidToken(token.kind.clone()))
                    }
                }
                cvs @ ComponentValue::SimpleBlock(_) => Err(CssValueError::InvalidComponentValue(cvs.clone())),
            }
        } else {
            Err(CssValueError::UnexpectedEndOfInput)
        }
    }
}

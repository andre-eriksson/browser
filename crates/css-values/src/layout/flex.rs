use css_cssom::{ComponentValue, ComponentValueStream, CssTokenKind};
use strum::EnumString;

use crate::{CSSParsable, dimension::Size, error::CssValueError};

#[derive(Debug, Clone, PartialEq)]
pub enum FlexBasis {
    Content,
    Size(Size),
}

impl Default for FlexBasis {
    fn default() -> Self {
        Self::Size(Size::Auto)
    }
}

impl CSSParsable for FlexBasis {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, CssValueError> {
        let checkpoint = stream.checkpoint();

        if let Ok(size) = Size::parse(stream) {
            return Ok(Self::Size(size));
        }

        stream.restore(checkpoint);

        if let Some(cv) = stream.next_non_whitespace()
            && let Some(token) = cv.as_token()
            && let CssTokenKind::Ident(ident) = &token.kind
            && ident.eq_ignore_ascii_case("content")
        {
            return Ok(Self::Content);
        }

        Err(CssValueError::InvalidValue(format!("Invalid flex-basis value: {:?}", stream.next_non_whitespace())))
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PartialOrd, Ord, EnumString)]
#[strum(serialize_all = "kebab_case", ascii_case_insensitive)]
pub enum FlexDirection {
    #[default]
    Row,
    RowReverse,
    Column,
    ColumnReverse,
}

impl CSSParsable for FlexDirection {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, CssValueError> {
        stream
            .next_non_whitespace()
            .ok_or(CssValueError::UnexpectedEndOfInput)
            .and_then(|cv| match cv {
                ComponentValue::Token(token) => match &token.kind {
                    CssTokenKind::Ident(ident) => ident
                        .parse()
                        .map_err(|_| CssValueError::InvalidValue(format!("Invalid flex-direction value: {}", ident))),
                    _ => Err(CssValueError::InvalidToken(token.kind.clone())),
                },
                cvs => Err(CssValueError::InvalidComponentValue(cvs.clone())),
            })
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PartialOrd, Ord, EnumString)]
#[strum(serialize_all = "kebab_case", ascii_case_insensitive)]
pub enum FlexWrap {
    #[default]
    Nowrap,
    Wrap,
    WrapReverse,
}

impl CSSParsable for FlexWrap {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, CssValueError> {
        stream
            .next_non_whitespace()
            .ok_or(CssValueError::UnexpectedEndOfInput)
            .and_then(|cv| match cv {
                ComponentValue::Token(token) => match &token.kind {
                    CssTokenKind::Ident(ident) => ident
                        .parse()
                        .map_err(|_| CssValueError::InvalidValue(format!("Invalid flex-wrap value: {}", ident))),
                    _ => Err(CssValueError::InvalidToken(token.kind.clone())),
                },
                cvs => Err(CssValueError::InvalidComponentValue(cvs.clone())),
            })
    }
}

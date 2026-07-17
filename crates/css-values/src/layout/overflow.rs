use css_cssom::{ComponentValue, ComponentValueStream, CssTokenKind};
use strum::EnumString;

use crate::{CSSParsable, error::CssValueError};

#[derive(Debug, Clone, Copy, Default, PartialEq, EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum OverflowBlock {
    #[default]
    Visible,
    Hidden,
    Clip,
    Scroll,
    Auto,
}

impl CSSParsable for OverflowBlock {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, CssValueError> {
        stream
            .next_non_whitespace()
            .ok_or(CssValueError::UnexpectedEndOfInput)
            .and_then(|cv| match cv {
                ComponentValue::Token(token) => match &token.kind {
                    CssTokenKind::Ident(ident) => ident
                        .parse()
                        .map_err(|_| CssValueError::InvalidValue(format!("Invalid overflow-block value: {}", ident))),
                    _ => Err(CssValueError::InvalidToken(token.kind.clone())),
                },
                cvs => Err(CssValueError::InvalidComponentValue(cvs.clone())),
            })
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum OverflowWrap {
    #[default]
    Normal,
    BreakWord,
    Anywhere,
}

impl CSSParsable for OverflowWrap {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, CssValueError> {
        stream
            .next_non_whitespace()
            .ok_or(CssValueError::UnexpectedEndOfInput)
            .and_then(|cv| match cv {
                ComponentValue::Token(token) => match &token.kind {
                    CssTokenKind::Ident(ident) => ident
                        .parse()
                        .map_err(|_| CssValueError::InvalidValue(format!("Invalid overflow-wrap value: {}", ident))),
                    _ => Err(CssValueError::InvalidToken(token.kind.clone())),
                },
                cvs => Err(CssValueError::InvalidComponentValue(cvs.clone())),
            })
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum OverflowAnchor {
    #[default]
    Auto,
    None,
}

impl CSSParsable for OverflowAnchor {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, CssValueError> {
        stream
            .next_non_whitespace()
            .ok_or(CssValueError::UnexpectedEndOfInput)
            .and_then(|cv| match cv {
                ComponentValue::Token(token) => match &token.kind {
                    CssTokenKind::Ident(ident) => ident
                        .parse()
                        .map_err(|_| CssValueError::InvalidValue(format!("Invalid overflow-anchor value: {}", ident))),
                    _ => Err(CssValueError::InvalidToken(token.kind.clone())),
                },
                cvs => Err(CssValueError::InvalidComponentValue(cvs.clone())),
            })
    }
}

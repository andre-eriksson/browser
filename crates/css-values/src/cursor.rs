use css_cssom::{ComponentValue, ComponentValueStream, CssTokenKind};
use strum::EnumString;

use crate::{CSSParsable, error::CssValueError};

/// Represents the CSS `cursor` property, which specifies the type of cursor to be displayed when pointing over an element.
///
/// <https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/cursor>
#[derive(Debug, Clone, Default, Copy, PartialEq, Eq, EnumString)]
#[strum(serialize_all = "kebab_case", ascii_case_insensitive)]
pub enum Cursor {
    Alias,
    AllScroll,
    #[default]
    Auto,
    Cell,
    ColResize,
    ContextMenu,
    Copy,
    Crosshair,
    Default,
    EResize,
    EwResize,
    Grab,
    Grabbing,
    Help,
    Move,
    NResize,
    NeResize,
    NeswResize,
    NoDrop,
    None,
    NotAllowed,
    NsResize,
    NwResize,
    NwseResize,
    Pointer,
    Progress,
    RowResize,
    SResize,
    SeResize,
    SwResize,
    Text,
    // TODO: Url
    VerticalText,
    WResize,
    Wait,
    ZoomIn,
    ZoomOut,
    // TODO: x y
}

impl CSSParsable for Cursor {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, CssValueError> {
        if let Some(cv) = stream.next_non_whitespace() {
            match cv {
                ComponentValue::Token(token) => match &token.kind {
                    CssTokenKind::Ident(ident) => ident
                        .parse()
                        .map_err(|e| CssValueError::InvalidValue(format!("Failed to parse cursor value: {}", e))),
                    _ => Err(CssValueError::InvalidToken(token.kind.clone())),
                },
                cvs => Err(CssValueError::InvalidComponentValue(cvs.clone())),
            }
        } else {
            Err(CssValueError::UnexpectedEndOfInput)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use css_cssom::{ComponentValue, CssToken};

    #[test]
    fn test_parse_cursor_pointer() {
        let input = vec![ComponentValue::Token(CssToken {
            kind: CssTokenKind::Ident("pointer".to_string()),
            position: Default::default(),
        })];
        let mut stream = ComponentValueStream::new(&input);
        let cursor = Cursor::parse(&mut stream).unwrap();
        assert_eq!(cursor, Cursor::Pointer);
    }

    #[test]
    fn test_parse_cursor_n_resize() {
        let input = vec![ComponentValue::Token(CssToken {
            kind: CssTokenKind::Ident("n-resize".to_string()),
            position: Default::default(),
        })];
        let mut stream = ComponentValueStream::new(&input);
        let cursor = Cursor::parse(&mut stream).unwrap();
        assert_eq!(cursor, Cursor::NResize);
    }
}

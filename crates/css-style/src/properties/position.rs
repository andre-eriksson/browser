//! The `position` property specifies how an element is positioned in a document. It has five possible values: `static`, `relative`, `absolute`, `fixed`, and `sticky`.
//! Each value determines how the element is positioned in relation to its normal flow, its containing block, and the viewport.

use css_cssom::{ComponentValue, ComponentValueStream, CssTokenKind};
use css_values::error::CssValueError;
use strum::EnumString;

use crate::properties::CSSParsable;

/// The `position` property specifies how an element is positioned in a document. It has five possible values: `static`, `relative`, `absolute`, `fixed`, and `sticky`.
/// Each value determines how the element is positioned in relation to its normal flow, its containing block, and the viewport.
///
/// <https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/position>
#[derive(Debug, Clone, Default, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, EnumString)]
#[strum(serialize_all = "kebab_case", ascii_case_insensitive)]
pub enum Position {
    /// The element is positioned according to the Normal Flow of the document. The `top`, `right`, `bottom`, `left`, and `z-index` properties have no effect.
    #[default]
    Static,
    /// The element is positioned according to the normal flow of the document, and then offset relative to itself based on the values of `top`, `right`, `bottom`, and `left`.
    /// The offset does not affect the position of any other elements; thus, the space given for the element in the page layout is the same as if position were `static`.
    Relative,

    /// The element is removed from the normal document flow, and no space is created for the element in the page layout. The element is positioned relative to its closest positioned
    /// ancestor (if any) or to the initial containing block. Its final position is determined by the values of `top`, `right`, `bottom`, and `left`.
    Absolute,

    /// The element is removed from the normal document flow, and no space is created for the element in the page layout. The element is positioned relative to its initial containing block,
    /// which is the viewport in the case of visual media. Its final position is determined by the values of `top`, `right`, `bottom`, and `left`.
    Fixed,

    /// The element is positioned according to the normal flow of the document, and then offset relative to its nearest scrolling ancestor and containing block (nearest block-level ancestor),
    /// including table-related elements, based on the values of `top`, `right`, `bottom`, and `left`. The offset does not affect the position of any other elements.
    Sticky,
}

impl Position {
    pub fn is_out_of_flow(&self) -> bool {
        matches!(self, Position::Absolute | Position::Fixed)
    }

    pub fn affects_normal_flow(&self) -> bool {
        matches!(self, Position::Sticky | Position::Relative | Position::Static)
    }
}

impl CSSParsable for Position {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, CssValueError> {
        stream.skip_whitespace();

        if let Some(cv) = stream.peek() {
            match cv {
                ComponentValue::Token(token) => match &token.kind {
                    CssTokenKind::Ident(ident) => ident
                        .parse()
                        .map_err(|_| CssValueError::InvalidValue(format!("Invalid position value: {}", ident))),
                    _ => Err(CssValueError::InvalidToken(token.kind.clone())),
                },
                _ => Err(CssValueError::InvalidComponentValue(cv.clone())),
            }
        } else {
            Err(CssValueError::UnexpectedEndOfInput)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_position() {
        assert_eq!("static".parse(), Ok(Position::Static));
        assert_eq!("relative".parse(), Ok(Position::Relative));
        assert_eq!("absolute".parse(), Ok(Position::Absolute));
        assert_eq!("fixed".parse(), Ok(Position::Fixed));
        assert_eq!("sticky".parse(), Ok(Position::Sticky));
        assert!("unknown".parse::<Position>().is_err());
    }
}

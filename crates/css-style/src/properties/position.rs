//! The `position` property specifies how an element is positioned in a document. It has five possible values: `static`, `relative`, `absolute`, `fixed`, and `sticky`.
//! Each value determines how the element is positioned in relation to its normal flow, its containing block, and the viewport.

use css_cssom::{ComponentValue, CssTokenKind};
use strum::EnumString;

/// The `position` property specifies how an element is positioned in a document. It has five possible values: `static`, `relative`, `absolute`, `fixed`, and `sticky`.
/// Each value determines how the element is positioned in relation to its normal flow, its containing block, and the viewport.
///
/// <https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/position>
#[derive(Debug, Clone, Default, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, EnumString)]
#[strum(serialize_all = "kebab_case", ascii_case_insensitive, parse_err_ty = String, parse_err_fn = String::from)]
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

impl TryFrom<&[ComponentValue]> for Position {
    type Error = String;

    fn try_from(value: &[ComponentValue]) -> Result<Self, Self::Error> {
        for cv in value {
            match cv {
                ComponentValue::Token(token) => {
                    if let CssTokenKind::Ident(ident) = &token.kind
                        && let Ok(pos) = ident.parse()
                    {
                        return Ok(pos);
                    }
                }
                _ => continue,
            }
        }
        Err("No valid position value found".to_string())
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

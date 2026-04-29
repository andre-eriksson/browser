//! This module defines the `align-content`, `align-items`, and `align-self` properties and their associated types.

use css_cssom::{ComponentValue, ComponentValueStream, CssTokenKind};

use crate::{
    AlignSelfAlignment, BaselinePosition, CSSParsable, ContentDistribution, ContentPosition, ItemsAlignment,
    error::CssValueError,
};

/// # Syntax
/// ```text
/// align-content =
///  normal                                   |
///  <baseline-position>                      |
///  <content-distribution>                   |
///  <overflow-position>? <content-position>
/// ```
///
/// <https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/align-content>
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum AlignContent {
    #[default]
    Normal,
    BaselinePosition(Option<BaselinePosition>),
    ContentDistribution(ContentDistribution),
    Alignment {
        safe: bool,
        position: ContentPosition,
    },
}

impl CSSParsable for AlignContent {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, CssValueError> {
        fn ensure_free(idk: u8, align_content: Option<AlignContent>) -> bool {
            if align_content.is_some() {
                return false;
            }

            if idk > 0 {
                return false;
            }

            true
        }

        let mut align_content = None;
        let mut baseline_position = None;
        let mut safe = None;
        let mut idx = 0;

        while let Some(cv) = stream.next_non_whitespace() {
            match cv {
                ComponentValue::Token(token) => match &token.kind {
                    CssTokenKind::Ident(ident) => {
                        if ident.eq_ignore_ascii_case("normal") {
                            if !ensure_free(idx, align_content) {
                                return Err(CssValueError::InvalidValue(
                                    "Unexpected 'normal' after other values".to_string(),
                                ));
                            }

                            align_content = Some(Self::Normal);
                        } else if ident.eq_ignore_ascii_case("baseline") && safe.is_none() {
                            if align_content.is_some() {
                                return Err(CssValueError::InvalidValue(
                                    "Unexpected 'baseline' after other values".to_string(),
                                ));
                            }

                            align_content = Some(Self::BaselinePosition(baseline_position));
                        } else if ident.eq_ignore_ascii_case("safe") {
                            if align_content.is_some() {
                                return Err(CssValueError::InvalidValue(
                                    "Unexpected 'safe' after other values".to_string(),
                                ));
                            }

                            if safe.is_some() {
                                return Err(CssValueError::InvalidValue(
                                    "Multiple overflow positions specified".to_string(),
                                ));
                            }

                            safe = Some(true);
                        } else if ident.eq_ignore_ascii_case("unsafe") {
                            if align_content.is_some() {
                                return Err(CssValueError::InvalidValue(
                                    "Unexpected 'unsafe' after other values".to_string(),
                                ));
                            }

                            if safe.is_some() {
                                return Err(CssValueError::InvalidValue(
                                    "Multiple overflow positions specified".to_string(),
                                ));
                            }

                            safe = Some(false);
                        } else if let Ok(baseline) = ident.parse::<BaselinePosition>() {
                            if !ensure_free(idx, align_content) {
                                return Err(CssValueError::InvalidValue(
                                    "Unexpected baseline position after other values".to_string(),
                                ));
                            }

                            if baseline_position.is_none() {
                                baseline_position = Some(baseline);
                            } else {
                                return Err(CssValueError::InvalidValue(format!(
                                    "Multiple baseline positions specified: {}",
                                    ident
                                )));
                            }
                        } else if let Ok(distribution) = ident.parse::<ContentDistribution>() {
                            if !ensure_free(idx, align_content) {
                                return Err(CssValueError::InvalidValue(
                                    "Unexpected content distribution after other values".to_string(),
                                ));
                            }

                            align_content = Some(Self::ContentDistribution(distribution));
                        } else if let Ok(position) = ident.parse::<ContentPosition>() {
                            if align_content.is_some() {
                                return Err(CssValueError::InvalidValue(
                                    "Multiple alignment positions specified".to_string(),
                                ));
                            }

                            align_content = Some(Self::Alignment {
                                safe: safe.unwrap_or(false),
                                position,
                            });
                        } else {
                            return Err(CssValueError::InvalidValue(format!("Unrecognized identifier: {}", ident)));
                        }

                        idx += 1;
                    }
                    _ => return Err(CssValueError::InvalidToken(token.kind.clone())),
                },
                cvs @ ComponentValue::Function(_) | cvs @ ComponentValue::SimpleBlock(_) => {
                    return Err(CssValueError::InvalidComponentValue(cvs.clone()));
                }
            }
        }

        stream.skip_whitespace();
        if align_content.is_some() && stream.peek().is_some() {
            Err(CssValueError::UnexpectedRemainingInput)
        } else if let Some(value) = align_content {
            Ok(value)
        } else {
            Err(CssValueError::UnexpectedEndOfInput)
        }
    }
}

/// # Syntax
/// ```text
/// align-items =
///  normal                                |
///  stretch                               |
///  <baseline-position>                   |
///  <overflow-position>? <self-position>
/// ```
///
/// <https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/align-items>
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum AlignItems {
    #[default]
    Normal,
    Stretch,
    BaselinePosition(Option<BaselinePosition>),
    Alignment {
        safe: bool,
        position: ItemsAlignment,
    },
}

impl CSSParsable for AlignItems {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, CssValueError> {
        fn ensure_free(idk: u8, align_items: Option<AlignItems>) -> bool {
            if align_items.is_some() {
                return false;
            }

            if idk > 0 {
                return false;
            }

            true
        }

        let mut align_items = None;
        let mut safe = None;
        let mut baseline_position = None;
        let mut idx = 0;

        while let Some(cv) = stream.next_non_whitespace() {
            match cv {
                ComponentValue::Token(token) => match &token.kind {
                    CssTokenKind::Ident(ident) => {
                        if ident.eq_ignore_ascii_case("normal") {
                            if !ensure_free(idx, align_items) {
                                return Err(CssValueError::InvalidValue(
                                    "Unexpected 'normal' after other values".to_string(),
                                ));
                            }

                            align_items = Some(Self::Normal);
                        } else if ident.eq_ignore_ascii_case("stretch") {
                            if !ensure_free(idx, align_items) {
                                return Err(CssValueError::InvalidValue(
                                    "Unexpected 'stretch' after other values".to_string(),
                                ));
                            }

                            align_items = Some(Self::Stretch);
                        } else if ident.eq_ignore_ascii_case("safe") {
                            if !ensure_free(idx, align_items) {
                                return Err(CssValueError::InvalidValue(
                                    "Unexpected 'safe' after other values".to_string(),
                                ));
                            }

                            if safe.is_some() {
                                return Err(CssValueError::InvalidValue(
                                    "Multiple overflow positions specified".to_string(),
                                ));
                            }

                            safe = Some(true);
                        } else if ident.eq_ignore_ascii_case("unsafe") {
                            if !ensure_free(idx, align_items) {
                                return Err(CssValueError::InvalidValue(
                                    "Unexpected 'unsafe' after other values".to_string(),
                                ));
                            }

                            if safe.is_some() {
                                return Err(CssValueError::InvalidValue(
                                    "Multiple overflow positions specified".to_string(),
                                ));
                            }

                            safe = Some(false);
                        } else if ident.eq_ignore_ascii_case("baseline") && safe.is_none() {
                            if align_items.is_some() {
                                return Err(CssValueError::InvalidValue(
                                    "Unexpected 'baseline' after other values".to_string(),
                                ));
                            }

                            align_items = Some(Self::BaselinePosition(baseline_position));
                        } else if let Ok(baseline) = ident.parse::<BaselinePosition>() {
                            if idx > 0 {
                                return Err(CssValueError::InvalidValue(
                                    "Unexpected baseline position after other values".to_string(),
                                ));
                            }

                            if baseline_position.is_none() {
                                baseline_position = Some(baseline);
                            } else {
                                return Err(CssValueError::InvalidValue(format!(
                                    "Multiple baseline positions specified: {}",
                                    ident
                                )));
                            }
                        } else if let Ok(position) = ident.parse::<ItemsAlignment>() {
                            if align_items.is_some() {
                                return Err(CssValueError::InvalidValue(
                                    "Multiple alignment positions specified".to_string(),
                                ));
                            }

                            align_items = Some(Self::Alignment {
                                safe: safe.unwrap_or(false),
                                position,
                            });
                        } else {
                            return Err(CssValueError::InvalidValue(format!("Unrecognized identifier: {}", ident)));
                        }

                        idx += 1;
                    }
                    _ => return Err(CssValueError::InvalidToken(token.kind.clone())),
                },
                cvs @ ComponentValue::Function(_) | cvs @ ComponentValue::SimpleBlock(_) => {
                    return Err(CssValueError::InvalidComponentValue(cvs.clone()));
                }
            }
        }

        stream.skip_whitespace();
        if align_items.is_some() && stream.peek().is_some() {
            Err(CssValueError::UnexpectedRemainingInput)
        } else if let Some(value) = align_items {
            Ok(value)
        } else {
            Err(CssValueError::UnexpectedEndOfInput)
        }
    }
}

/// # Syntax
/// ```text
/// align-self =
///  auto                                               |
///  <overflow-position>? [ normal | <self-position> ]  |
///  stretch                                            |
///  <baseline-position>                                |
///  anchor-center
/// ```
///
/// <https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/align-self>
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum AlignSelf {
    #[default]
    Auto,
    Alignment {
        safe: bool,
        position: AlignSelfAlignment,
    },
    Stretch,
    BaselinePosition(Option<BaselinePosition>),
    AnchorCenter,
}

impl CSSParsable for AlignSelf {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, CssValueError> {
        fn ensure_free(idk: u8, align_self: Option<AlignSelf>) -> bool {
            if align_self.is_some() {
                return false;
            }

            if idk > 0 {
                return false;
            }

            true
        }

        let mut align_self = None;
        let mut safe = None;
        let mut baseline_position = None;
        let mut idx = 0;

        while let Some(cv) = stream.next_non_whitespace() {
            match cv {
                ComponentValue::Token(token) => match &token.kind {
                    CssTokenKind::Ident(ident) => {
                        if ident.eq_ignore_ascii_case("auto") {
                            if !ensure_free(idx, align_self) {
                                return Err(CssValueError::InvalidValue(
                                    "Unexpected 'auto' after other values".to_string(),
                                ));
                            }

                            align_self = Some(Self::Auto);
                        } else if ident.eq_ignore_ascii_case("stretch") {
                            if !ensure_free(idx, align_self) {
                                return Err(CssValueError::InvalidValue(
                                    "Unexpected 'stretch' after other values".to_string(),
                                ));
                            }

                            align_self = Some(Self::Stretch);
                        } else if ident.eq_ignore_ascii_case("anchor-center") {
                            if !ensure_free(idx, align_self) {
                                return Err(CssValueError::InvalidValue(
                                    "Unexpected 'anchor-center' after other values".to_string(),
                                ));
                            }

                            align_self = Some(Self::AnchorCenter);
                        } else if ident.eq_ignore_ascii_case("safe") {
                            if !ensure_free(idx, align_self) {
                                return Err(CssValueError::InvalidValue(
                                    "Unexpected 'safe' after other values".to_string(),
                                ));
                            }

                            if safe.is_some() {
                                return Err(CssValueError::InvalidValue(
                                    "Multiple overflow positions specified".to_string(),
                                ));
                            }

                            safe = Some(true);
                        } else if ident.eq_ignore_ascii_case("unsafe") {
                            if !ensure_free(idx, align_self) {
                                return Err(CssValueError::InvalidValue(
                                    "Unexpected 'unsafe' after other values".to_string(),
                                ));
                            }

                            if safe.is_some() {
                                return Err(CssValueError::InvalidValue(
                                    "Multiple overflow positions specified".to_string(),
                                ));
                            }

                            safe = Some(false);
                        } else if ident.eq_ignore_ascii_case("baseline") && safe.is_none() {
                            if align_self.is_some() {
                                return Err(CssValueError::InvalidValue(
                                    "Unexpected 'baseline' after other values".to_string(),
                                ));
                            }

                            align_self = Some(Self::BaselinePosition(baseline_position));
                        } else if let Ok(baseline) = ident.parse::<BaselinePosition>() {
                            if idx > 0 {
                                return Err(CssValueError::InvalidValue(
                                    "Unexpected baseline position after other values".to_string(),
                                ));
                            }

                            if baseline_position.is_none() {
                                baseline_position = Some(baseline);
                            } else {
                                return Err(CssValueError::InvalidValue(format!(
                                    "Multiple baseline positions specified: {}",
                                    ident
                                )));
                            }
                        } else if let Ok(position) = ident.parse::<AlignSelfAlignment>() {
                            if align_self.is_some() {
                                return Err(CssValueError::InvalidValue(
                                    "Multiple alignment positions specified".to_string(),
                                ));
                            }

                            align_self = Some(Self::Alignment {
                                safe: safe.unwrap_or(false),
                                position,
                            });
                        } else {
                            return Err(CssValueError::InvalidValue(format!("Unrecognized identifier: {}", ident)));
                        }

                        idx += 1;
                    }
                    _ => return Err(CssValueError::InvalidToken(token.kind.clone())),
                },
                cvs @ ComponentValue::Function(_) | cvs @ ComponentValue::SimpleBlock(_) => {
                    return Err(CssValueError::InvalidComponentValue(cvs.clone()));
                }
            }
        }

        stream.skip_whitespace();

        if align_self.is_some() && stream.peek().is_some() {
            Err(CssValueError::UnexpectedRemainingInput)
        } else if let Some(value) = align_self {
            Ok(value)
        } else {
            Err(CssValueError::UnexpectedEndOfInput)
        }
    }
}

#[cfg(test)]
mod tests {
    use css_cssom::CssToken;

    use crate::{SelfPosition, position::HorizontalSide};

    use super::*;

    #[test]
    fn test_parse_align_content() {
        let green_cases = vec![
            ("normal", AlignContent::Normal),
            ("baseline", AlignContent::BaselinePosition(None)),
            ("first baseline", AlignContent::BaselinePosition(Some(BaselinePosition::First))),
            ("last baseline", AlignContent::BaselinePosition(Some(BaselinePosition::Last))),
            ("space-between", AlignContent::ContentDistribution(ContentDistribution::SpaceBetween)),
            ("space-around", AlignContent::ContentDistribution(ContentDistribution::SpaceAround)),
            ("space-evenly", AlignContent::ContentDistribution(ContentDistribution::SpaceEvenly)),
            ("stretch", AlignContent::ContentDistribution(ContentDistribution::Stretch)),
            (
                "safe center",
                AlignContent::Alignment {
                    safe: true,
                    position: ContentPosition::Center,
                },
            ),
            (
                "unsafe start",
                AlignContent::Alignment {
                    safe: false,
                    position: ContentPosition::Start,
                },
            ),
            (
                "end",
                AlignContent::Alignment {
                    safe: false,
                    position: ContentPosition::End,
                },
            ),
            (
                "flex-start",
                AlignContent::Alignment {
                    safe: false,
                    position: ContentPosition::FlexStart,
                },
            ),
            (
                "flex-end",
                AlignContent::Alignment {
                    safe: false,
                    position: ContentPosition::FlexEnd,
                },
            ),
            (
                "start",
                AlignContent::Alignment {
                    safe: false,
                    position: ContentPosition::Start,
                },
            ),
        ];

        let red_cases = vec![
            "safe center start",
            "safe unsafe normal",
            "normal safe",
            "baseline safe",
            "safe baseline",
            "first",
            "unknown-value",
        ];

        for (id, (input, expected)) in green_cases.into_iter().enumerate() {
            let split = input.split_whitespace().collect::<Vec<_>>();

            let mut cvs = Vec::new();

            for part in &split {
                cvs.push(ComponentValue::Token(CssToken {
                    kind: CssTokenKind::Ident(part.to_string()),
                    position: None,
                }));
                cvs.push(ComponentValue::Token(CssToken {
                    kind: CssTokenKind::Whitespace,
                    position: None,
                }));
            }

            let mut stream = ComponentValueStream::new(&cvs);
            let result = match AlignContent::parse(&mut stream) {
                Ok(value) => value,
                Err(e) => panic!("Failed to parse '{}': {:?}", input, e),
            };
            assert_eq!(result, expected, "Test case {} failed: expected {:?}, got {:?}", id, expected, result);
        }

        for input in red_cases {
            let split = input.split_whitespace().collect::<Vec<_>>();

            let mut cvs = Vec::new();

            for part in &split {
                cvs.push(ComponentValue::Token(CssToken {
                    kind: CssTokenKind::Ident(part.to_string()),
                    position: None,
                }));
                cvs.push(ComponentValue::Token(CssToken {
                    kind: CssTokenKind::Whitespace,
                    position: None,
                }));
            }

            let mut stream = ComponentValueStream::new(&cvs);
            let result = AlignContent::parse(&mut stream);
            assert!(
                result.is_err(),
                "Test case with invalid input '{}' should have failed but got {:?}",
                input,
                result
            );
        }
    }

    #[test]
    fn test_parse_align_items() {
        let green_cases = vec![
            ("normal", AlignItems::Normal),
            ("stretch", AlignItems::Stretch),
            ("baseline", AlignItems::BaselinePosition(None)),
            ("first baseline", AlignItems::BaselinePosition(Some(BaselinePosition::First))),
            ("last baseline", AlignItems::BaselinePosition(Some(BaselinePosition::Last))),
            (
                "safe center",
                AlignItems::Alignment {
                    safe: true,
                    position: ItemsAlignment::SelfPosition(SelfPosition::Center),
                },
            ),
            (
                "unsafe left",
                AlignItems::Alignment {
                    safe: false,
                    position: ItemsAlignment::HorizontalSide(HorizontalSide::Left),
                },
            ),
            (
                "self-start",
                AlignItems::Alignment {
                    safe: false,
                    position: ItemsAlignment::SelfPosition(SelfPosition::SelfStart),
                },
            ),
            (
                "self-end",
                AlignItems::Alignment {
                    safe: false,
                    position: ItemsAlignment::SelfPosition(SelfPosition::SelfEnd),
                },
            ),
        ];

        let red_cases = vec![
            "safe unsafe normal",
            "normal safe",
            "baseline safe",
            "safe baseline",
            "first",
            "unknown-value",
        ];

        for (id, (input, expected)) in green_cases.into_iter().enumerate() {
            let split = input.split_whitespace().collect::<Vec<_>>();

            let mut cvs = Vec::new();

            for part in &split {
                cvs.push(ComponentValue::Token(CssToken {
                    kind: CssTokenKind::Ident(part.to_string()),
                    position: None,
                }));
                cvs.push(ComponentValue::Token(CssToken {
                    kind: CssTokenKind::Whitespace,
                    position: None,
                }));
            }

            let mut stream = ComponentValueStream::new(&cvs);
            let result = match AlignItems::parse(&mut stream) {
                Ok(value) => value,
                Err(e) => panic!("Failed to parse '{}': {:?}", input, e),
            };
            assert_eq!(result, expected, "Test case {} failed: expected {:?}, got {:?}", id, expected, result);
        }

        for input in red_cases {
            let split = input.split_whitespace().collect::<Vec<_>>();

            let mut cvs = Vec::new();

            for part in &split {
                cvs.push(ComponentValue::Token(CssToken {
                    kind: CssTokenKind::Ident(part.to_string()),
                    position: None,
                }));
                cvs.push(ComponentValue::Token(CssToken {
                    kind: CssTokenKind::Whitespace,
                    position: None,
                }));
            }

            let mut stream = ComponentValueStream::new(&cvs);
            let result = AlignItems::parse(&mut stream);
            assert!(
                result.is_err(),
                "Test case with invalid input '{}' should have failed but got {:?}",
                input,
                result
            );
        }
    }

    #[test]
    fn test_parse_align_self() {
        let green_cases = vec![
            ("auto", AlignSelf::Auto),
            ("stretch", AlignSelf::Stretch),
            ("anchor-stretch", AlignSelf::AnchorCenter),
            (
                "safe normal",
                AlignSelf::Alignment {
                    safe: true,
                    position: AlignSelfAlignment::Normal,
                },
            ),
            (
                "unsafe end",
                AlignSelf::Alignment {
                    safe: false,
                    position: AlignSelfAlignment::SelfPosition(SelfPosition::End),
                },
            ),
            (
                "self-start",
                AlignSelf::Alignment {
                    safe: false,
                    position: AlignSelfAlignment::SelfPosition(SelfPosition::SelfStart),
                },
            ),
            ("baseline", AlignSelf::BaselinePosition(None)),
            ("first baseline", AlignSelf::BaselinePosition(Some(BaselinePosition::First))),
            ("last baseline", AlignSelf::BaselinePosition(Some(BaselinePosition::Last))),
        ];

        let red_cases = vec![
            "safe unsafe normal",
            "normal safe",
            "stretch auto",
            "anchor-stretch left",
            "baseline safe",
            "safe baseline",
            "first safe",
            "first",
            "unknown-value",
        ];

        for (id, (input, expected)) in green_cases.into_iter().enumerate() {
            let split = input.split_whitespace().collect::<Vec<_>>();

            let mut cvs = Vec::new();

            for part in &split {
                cvs.push(ComponentValue::Token(CssToken {
                    kind: CssTokenKind::Ident(part.to_string()),
                    position: None,
                }));
                cvs.push(ComponentValue::Token(CssToken {
                    kind: CssTokenKind::Whitespace,
                    position: None,
                }));
            }

            let mut stream = ComponentValueStream::new(&cvs);
            let result = match AlignSelf::parse(&mut stream) {
                Ok(value) => value,
                Err(e) => panic!("Failed to parse '{}': {:?}", input, e),
            };
            assert_eq!(result, expected, "Test case {} failed: expected {:?}, got {:?}", id, expected, result);
        }

        for input in red_cases {
            let split = input.split_whitespace().collect::<Vec<_>>();

            let mut cvs = Vec::new();

            for part in &split {
                cvs.push(ComponentValue::Token(CssToken {
                    kind: CssTokenKind::Ident(part.to_string()),
                    position: None,
                }));
                cvs.push(ComponentValue::Token(CssToken {
                    kind: CssTokenKind::Whitespace,
                    position: None,
                }));
            }

            let mut stream = ComponentValueStream::new(&cvs);
            let result = AlignSelf::parse(&mut stream);
            assert!(
                result.is_err(),
                "Test case with invalid input '{}' should have failed but got {:?}",
                input,
                result
            );
        }
    }
}

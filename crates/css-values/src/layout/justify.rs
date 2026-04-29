//! This module defines the `justify-content`, `justify-items`, and `justify-self` properties and their associated types.

use css_cssom::{ComponentValue, ComponentValueStream, CssTokenKind};

use crate::{
    BaselinePosition, CSSParsable, ContentAlignment, ContentDistribution, ItemsAlignment, JustifySelfAlignment,
    error::CssValueError, position::RelativeHorizontalSide,
};

/// # Syntax
/// ```text
/// justify-content =
///  normal                                                     |
///  <content-distribution>                                     |
///  <overflow-position>? [ <content-position> | left | right ]
/// ```
///
/// <https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/justify-content>
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum JustifyContent {
    #[default]
    Normal,
    ContentDistribution(ContentDistribution),
    Alignment {
        safe: bool,
        position: ContentAlignment,
    },
}

impl CSSParsable for JustifyContent {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, CssValueError> {
        fn ensure_free(idk: u8, justify_content: Option<JustifyContent>) -> bool {
            if justify_content.is_some() {
                return false;
            }

            if idk > 0 {
                return false;
            }

            true
        }

        let mut justify_content = None;
        let mut safe = None;
        let mut idx = 0;

        while let Some(cv) = stream.next_non_whitespace() {
            match cv {
                ComponentValue::Token(token) => match &token.kind {
                    CssTokenKind::Ident(ident) => {
                        if ident.eq_ignore_ascii_case("normal") {
                            if !ensure_free(idx, justify_content) {
                                return Err(CssValueError::InvalidValue(
                                    "Unexpected 'normal' after other values".to_string(),
                                ));
                            }

                            justify_content = Some(Self::Normal);
                        } else if ident.eq_ignore_ascii_case("safe") {
                            if !ensure_free(idx, justify_content) {
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
                            if !ensure_free(idx, justify_content) {
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
                        } else if let Ok(distribution) = ident.parse::<ContentDistribution>() {
                            if !ensure_free(idx, justify_content) {
                                return Err(CssValueError::InvalidValue(
                                    "Unexpected content distribution after other values".to_string(),
                                ));
                            }

                            justify_content = Some(Self::ContentDistribution(distribution));
                        } else if let Ok(position) = ident.parse::<ContentAlignment>() {
                            if justify_content.is_some() {
                                return Err(CssValueError::InvalidValue(
                                    "Multiple alignment positions specified".to_string(),
                                ));
                            }

                            justify_content = Some(Self::Alignment {
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
        if justify_content.is_some() && stream.peek().is_some() {
            Err(CssValueError::UnexpectedRemainingInput)
        } else if let Some(value) = justify_content {
            Ok(value)
        } else {
            Err(CssValueError::UnexpectedEndOfInput)
        }
    }
}

/// # Syntax
/// ```text
/// justify-items =
///  normal                                                   |
///  stretch                                                  |
///  <baseline-position>                                      |
///  <overflow-position>? [ <self-position> | left | right ]  |
///  legacy                                                   |
///  legacy && [ left | right | center ]
///
/// <https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/justify-items>
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum JustifyItems {
    #[default]
    Normal,
    Stretch,
    BaselinePosition(Option<BaselinePosition>),
    Alignment {
        safe: bool,
        position: ItemsAlignment,
    },
    Legacy(Option<RelativeHorizontalSide>),
}

impl CSSParsable for JustifyItems {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, CssValueError> {
        fn ensure_free(idk: u8, justify_items: Option<JustifyItems>) -> bool {
            if justify_items.is_some() {
                return false;
            }

            if idk > 0 {
                return false;
            }

            true
        }

        let mut justify_items = None;
        let mut safe = None;
        let mut baseline_position = None;
        let mut is_legacy = false;
        let mut legacy_side = None;
        let mut idx = 0;

        while let Some(cv) = stream.next_non_whitespace() {
            match cv {
                ComponentValue::Token(token) => match &token.kind {
                    CssTokenKind::Ident(ident) => {
                        if ident.eq_ignore_ascii_case("legacy") && safe.is_none() && baseline_position.is_none() {
                            if justify_items.is_some() || is_legacy {
                                return Err(CssValueError::InvalidValue(
                                    "Unexpected 'legacy' after other values".to_string(),
                                ));
                            }

                            is_legacy = true;
                        } else if ident.eq_ignore_ascii_case("normal") {
                            if !ensure_free(idx, justify_items) {
                                return Err(CssValueError::InvalidValue(
                                    "Unexpected 'normal' after other values".to_string(),
                                ));
                            }

                            justify_items = Some(Self::Normal);
                        } else if ident.eq_ignore_ascii_case("stretch") {
                            if !ensure_free(idx, justify_items) {
                                return Err(CssValueError::InvalidValue(
                                    "Unexpected 'stretch' after other values".to_string(),
                                ));
                            }

                            justify_items = Some(Self::Stretch);
                        } else if ident.eq_ignore_ascii_case("safe") {
                            if !ensure_free(idx, justify_items) {
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
                            if !ensure_free(idx, justify_items) {
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
                        } else if ident.eq_ignore_ascii_case("baseline") {
                            if justify_items.is_some() {
                                return Err(CssValueError::InvalidValue(
                                    "Unexpected 'baseline' after other values".to_string(),
                                ));
                            }

                            justify_items = Some(Self::BaselinePosition(baseline_position));
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
                        } else if let Ok(position) = ident.parse::<ItemsAlignment>()
                            && !is_legacy
                        {
                            if justify_items.is_some() {
                                return Err(CssValueError::InvalidValue(
                                    "Multiple alignment positions specified".to_string(),
                                ));
                            }

                            justify_items = Some(Self::Alignment {
                                safe: safe.unwrap_or(false),
                                position,
                            });
                        } else if let Ok(side) = ident.parse::<RelativeHorizontalSide>() {
                            if justify_items.is_some() && !is_legacy {
                                return Err(CssValueError::InvalidValue(
                                    "Unexpected horizontal side after other values".to_string(),
                                ));
                            }

                            if legacy_side.is_none() {
                                legacy_side = Some(side);
                            } else {
                                return Err(CssValueError::InvalidValue(format!(
                                    "Multiple horizontal sides specified: {}",
                                    ident
                                )));
                            }
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

        if is_legacy {
            if justify_items.is_some() {
                return Err(CssValueError::InvalidValue("(end) Unexpected 'legacy' with other values".to_string()));
            }

            Ok(Self::Legacy(legacy_side))
        } else if justify_items.is_some() && stream.peek().is_some() {
            Err(CssValueError::UnexpectedRemainingInput)
        } else if let Some(value) = justify_items {
            Ok(value)
        } else {
            Err(CssValueError::UnexpectedEndOfInput)
        }
    }
}

/// # Syntax
/// ```text
/// justify-self =
///  auto                                                              |
///  <overflow-position>? [ normal | <self-position> | left | right ]  |
///  stretch                                                           |
///  <baseline-position>                                               |
///  anchor-center
/// ```
///
/// <https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/justify-self>
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum JustifySelf {
    #[default]
    Auto,
    Alignment {
        safe: bool,
        position: JustifySelfAlignment,
    },
    Stretch,
    BaselinePosition(Option<BaselinePosition>),
    AnchorCenter,
}

impl CSSParsable for JustifySelf {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, CssValueError> {
        fn can_assign(idk: u8, justify_self: Option<JustifySelf>) -> bool {
            if justify_self.is_some() {
                return false;
            }

            if idk > 0 {
                return false;
            }

            true
        }

        let mut justify_self = None;
        let mut safe = None;
        let mut baseline_position = None;
        let mut idx = 0;

        while let Some(cv) = stream.next_non_whitespace() {
            match cv {
                ComponentValue::Token(token) => match &token.kind {
                    CssTokenKind::Ident(ident) => {
                        if ident.eq_ignore_ascii_case("auto") {
                            if !can_assign(idx, justify_self) {
                                return Err(CssValueError::InvalidValue(
                                    "Unexpected 'auto' after other values".to_string(),
                                ));
                            }

                            justify_self = Some(Self::Auto);
                        } else if ident.eq_ignore_ascii_case("stretch") {
                            if !can_assign(idx, justify_self) {
                                return Err(CssValueError::InvalidValue(
                                    "Unexpected 'stretch' after other values".to_string(),
                                ));
                            }

                            justify_self = Some(Self::Stretch);
                        } else if ident.eq_ignore_ascii_case("anchor-center") {
                            if !can_assign(idx, justify_self) {
                                return Err(CssValueError::InvalidValue(
                                    "Unexpected 'anchor-center' after other values".to_string(),
                                ));
                            }

                            justify_self = Some(Self::AnchorCenter);
                        } else if ident.eq_ignore_ascii_case("safe") {
                            if !can_assign(idx, justify_self) {
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
                            if !can_assign(idx, justify_self) {
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
                            if justify_self.is_some() {
                                return Err(CssValueError::InvalidValue(
                                    "Unexpected 'baseline' after other values".to_string(),
                                ));
                            }

                            justify_self = Some(Self::BaselinePosition(baseline_position));
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
                        } else if let Ok(position) = ident.parse::<JustifySelfAlignment>() {
                            if justify_self.is_some() {
                                return Err(CssValueError::InvalidValue(
                                    "Multiple alignment positions specified".to_string(),
                                ));
                            }

                            justify_self = Some(Self::Alignment {
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
        if justify_self.is_some() && stream.peek().is_some() {
            Err(CssValueError::UnexpectedRemainingInput)
        } else if let Some(value) = justify_self {
            Ok(value)
        } else {
            Err(CssValueError::UnexpectedEndOfInput)
        }
    }
}

#[cfg(test)]
mod tests {
    use css_cssom::CssToken;

    use crate::{ContentPosition, SelfPosition, position::HorizontalSide};

    use super::*;

    #[test]
    fn test_parse_justify_content() {
        let green_cases = vec![
            ("normal", JustifyContent::Normal),
            ("space-between", JustifyContent::ContentDistribution(ContentDistribution::SpaceBetween)),
            ("space-around", JustifyContent::ContentDistribution(ContentDistribution::SpaceAround)),
            ("space-evenly", JustifyContent::ContentDistribution(ContentDistribution::SpaceEvenly)),
            ("stretch", JustifyContent::ContentDistribution(ContentDistribution::Stretch)),
            (
                "safe center",
                JustifyContent::Alignment {
                    safe: true,
                    position: ContentAlignment::ContentPosition(ContentPosition::Center),
                },
            ),
            (
                "unsafe left",
                JustifyContent::Alignment {
                    safe: false,
                    position: ContentAlignment::HorizontalSide(HorizontalSide::Left),
                },
            ),
            (
                "right",
                JustifyContent::Alignment {
                    safe: false,
                    position: ContentAlignment::HorizontalSide(HorizontalSide::Right),
                },
            ),
            (
                "safe end",
                JustifyContent::Alignment {
                    safe: true,
                    position: ContentAlignment::ContentPosition(ContentPosition::End),
                },
            ),
        ];

        let red_cases = vec![
            "safe unsafe normal",
            "normal safe",
            "center safe",
            "safe stretch",
            "safe space-between",
            "left right",
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
            let result = match JustifyContent::parse(&mut stream) {
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
            let result = JustifyContent::parse(&mut stream);
            assert!(
                result.is_err(),
                "Test case with invalid input '{}' should have failed but got {:?}",
                input,
                result
            );
        }
    }

    #[test]
    fn test_parse_justify_items() {
        let green_cases = vec![
            ("normal", JustifyItems::Normal),
            ("stretch", JustifyItems::Stretch),
            ("baseline", JustifyItems::BaselinePosition(None)),
            ("first baseline", JustifyItems::BaselinePosition(Some(BaselinePosition::First))),
            ("last baseline", JustifyItems::BaselinePosition(Some(BaselinePosition::Last))),
            (
                "safe center",
                JustifyItems::Alignment {
                    safe: true,
                    position: ItemsAlignment::SelfPosition(SelfPosition::Center),
                },
            ),
            (
                "unsafe left",
                JustifyItems::Alignment {
                    safe: false,
                    position: ItemsAlignment::HorizontalSide(HorizontalSide::Left),
                },
            ),
            (
                "safe self-end",
                JustifyItems::Alignment {
                    safe: true,
                    position: ItemsAlignment::SelfPosition(SelfPosition::SelfEnd),
                },
            ),
            ("legacy", JustifyItems::Legacy(None)),
            ("legacy left", JustifyItems::Legacy(Some(RelativeHorizontalSide::Horizontal(HorizontalSide::Left)))),
            (
                "legacy right",
                JustifyItems::Legacy(Some(RelativeHorizontalSide::Horizontal(HorizontalSide::Right))),
            ),
        ];

        let red_cases = vec![
            "safe unsafe normal",
            "normal safe",
            "center safe",
            "left legacy",
            "safe legacy",
            "legacy legacy",
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
            let result = match JustifyItems::parse(&mut stream) {
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
            let result = JustifyItems::parse(&mut stream);
            assert!(
                result.is_err(),
                "Test case with invalid input '{}' should have failed but got {:?}",
                input,
                result
            );
        }
    }

    #[test]
    fn test_parse_justify_self() {
        let green_cases = vec![
            ("auto", JustifySelf::Auto),
            ("stretch", JustifySelf::Stretch),
            ("anchor-stretch", JustifySelf::AnchorCenter),
            (
                "safe normal",
                JustifySelf::Alignment {
                    safe: true,
                    position: JustifySelfAlignment::Normal,
                },
            ),
            (
                "unsafe left",
                JustifySelf::Alignment {
                    safe: false,
                    position: JustifySelfAlignment::HorizontalSide(HorizontalSide::Left),
                },
            ),
            (
                "safe self-start",
                JustifySelf::Alignment {
                    safe: true,
                    position: JustifySelfAlignment::SelfPosition(SelfPosition::SelfStart),
                },
            ),
            (
                "right",
                JustifySelf::Alignment {
                    safe: false,
                    position: JustifySelfAlignment::HorizontalSide(HorizontalSide::Right),
                },
            ),
            ("first baseline", JustifySelf::BaselinePosition(Some(BaselinePosition::First))),
            ("last baseline", JustifySelf::BaselinePosition(Some(BaselinePosition::Last))),
        ];

        let red_cases = vec![
            "safe unsafe normal",
            "normal safe",
            "stretch auto",
            "anchor-stretch left",
            "baseline safe",
            "first safe",
            "safe baseline",
            "safe anchor-stretch",
            "left right",
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
            let result = match JustifySelf::parse(&mut stream) {
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
            let result = JustifySelf::parse(&mut stream);
            assert!(
                result.is_err(),
                "Test case with invalid input '{}' should have failed but got {:?}",
                input,
                result
            );
        }
    }
}

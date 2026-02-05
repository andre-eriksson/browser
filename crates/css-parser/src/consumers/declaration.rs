use crate::errors::CssParsingError;
use css_tokenizer::{CssToken, CssTokenKind, SourcePosition};

use crate::{
    ComponentValue, CssParser, Declaration, DeclarationOrAtRule,
    consumers::{component::consume_component_value, rule::consume_at_rule},
};

/// Consume a list of declarations
///
/// <https://www.w3.org/TR/css-syntax-3/#consume-a-list-of-declarations>
pub(crate) fn consume_list_of_declarations(css_parser: &mut CssParser) -> Vec<DeclarationOrAtRule> {
    let mut declarations = Vec::new();

    while let Some(token) = css_parser.peek() {
        match &token.kind {
            CssTokenKind::Eof => break,
            CssTokenKind::Whitespace | CssTokenKind::Semicolon => {
                css_parser.consume();
            }
            CssTokenKind::AtKeyword(_) => {
                declarations.push(DeclarationOrAtRule::AtRule(consume_at_rule(css_parser)));
            }
            CssTokenKind::Ident(_) => {
                let mut temp_tokens: Vec<CssToken> = Vec::new();

                temp_tokens.push(css_parser.consume().unwrap());
                while let Some(token) = css_parser.peek() {
                    if matches!(token.kind, CssTokenKind::Eof | CssTokenKind::Semicolon) {
                        break;
                    }
                    let cv = consume_component_value(css_parser);
                    CssParser::append_component_value_tokens(&cv, &mut temp_tokens);
                }

                if let Some(decl) = consume_declaration_from_tokens(&temp_tokens) {
                    declarations.push(DeclarationOrAtRule::Declaration(decl));
                }
            }
            _ => {
                let pos = token.position.unwrap_or_default();
                css_parser.record_error(CssParsingError::InvalidDeclarationStart(pos));

                while let Some(token) = css_parser.peek() {
                    if matches!(token.kind, CssTokenKind::Eof | CssTokenKind::Semicolon) {
                        break;
                    }
                    consume_component_value(css_parser);
                }
            }
        }
    }

    declarations
}

/// Consume a declaration from a list of tokens
fn consume_declaration_from_tokens(tokens: &[CssToken]) -> Option<Declaration> {
    if tokens.is_empty() {
        return None;
    }

    let mut sub_parser = CssParser::new(Some(tokens.to_vec()));

    let name = match sub_parser.consume() {
        Some(token) => match token.kind {
            CssTokenKind::Ident(ref ident) => ident.clone(),
            _ => {
                sub_parser.record_error(CssParsingError::InvalidDeclarationName(
                    token.position.unwrap_or_default(),
                ));
                return None;
            }
        },
        _ => {
            sub_parser.record_error(CssParsingError::EofInDeclaration(SourcePosition::default()));
            return None;
        }
    };

    let mut declaration = Declaration::new(name);

    sub_parser.skip_whitespace();

    if !matches!(
        sub_parser.peek().map(|t| &t.kind),
        Some(CssTokenKind::Colon)
    ) {
        sub_parser.record_error(CssParsingError::MissingColonInDeclaration(
            sub_parser
                .peek()
                .and_then(|t| t.position)
                .unwrap_or_default(),
        ));
        return None;
    }

    sub_parser.consume();
    sub_parser.skip_whitespace();

    while !sub_parser.is_eof() {
        declaration
            .value
            .push(consume_component_value(&mut sub_parser));
    }

    sub_parser.check_important(&mut declaration);

    while matches!(
        declaration.value.last(),
        Some(ComponentValue::Token(CssToken {
            kind: CssTokenKind::Whitespace,
            ..
        }))
    ) {
        declaration.value.pop();
    }

    Some(declaration)
}

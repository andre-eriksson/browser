use css_tokenizer::CssToken;

use crate::{
    ComponentValue, CssParser, Declaration, DeclarationOrAtRule,
    consumers::{component::consume_component_value, rule::consume_at_rule},
};

/// Consume a list of declarations
///
/// <https://www.w3.org/TR/css-syntax-3/#consume-a-list-of-declarations>
pub(crate) fn consume_list_of_declarations(css_parser: &mut CssParser) -> Vec<DeclarationOrAtRule> {
    let mut declarations = Vec::new();

    loop {
        match css_parser.peek() {
            None | Some(CssToken::Eof) => {
                break;
            }
            Some(CssToken::Whitespace) | Some(CssToken::Semicolon) => {
                css_parser.consume();
            }
            Some(CssToken::AtKeyword(_)) => {
                declarations.push(DeclarationOrAtRule::AtRule(consume_at_rule(css_parser)));
            }
            Some(CssToken::Ident(_)) => {
                // Initialize a temporary list with the current token
                let mut temp_tokens: Vec<CssToken> = Vec::new();
                temp_tokens.push(css_parser.consume().unwrap());

                // Consume until semicolon or EOF
                while !matches!(
                    css_parser.peek(),
                    None | Some(CssToken::Eof) | Some(CssToken::Semicolon)
                ) {
                    let cv = consume_component_value(css_parser);
                    CssParser::append_component_value_tokens(&cv, &mut temp_tokens);
                }

                // Try to consume a declaration from the temporary list
                if let Some(decl) = consume_declaration_from_tokens(&temp_tokens) {
                    declarations.push(DeclarationOrAtRule::Declaration(decl));
                }
            }
            _ => {
                // Parse error, consume until semicolon or EOF
                while !matches!(
                    css_parser.peek(),
                    None | Some(CssToken::Eof) | Some(CssToken::Semicolon)
                ) {
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
        Some(CssToken::Ident(name)) => name,
        _ => return None,
    };

    let mut declaration = Declaration::new(name);

    sub_parser.skip_whitespace();

    if !matches!(sub_parser.peek(), Some(CssToken::Colon)) {
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
        Some(ComponentValue::Token(CssToken::Whitespace))
    ) {
        declaration.value.pop();
    }

    Some(declaration)
}

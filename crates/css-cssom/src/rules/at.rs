use css_parser::{
    AssociatedToken, AtRule, ComponentValue, CssTokenKind, Property, QualifiedRule, SimpleBlock,
};
use serde::{Deserialize, Serialize};

use crate::{
    declaration::CSSDeclaration,
    rules::{css::CSSRule, style::CSSStyleRule},
    string::prelude_to_string,
};

/// A CSS at-rule (@media, @import, @font-face, etc.)
///
/// <https://www.w3.org/TR/css-syntax-3/#at-rule>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CSSAtRule {
    /// The name of the at-rule (without the @)
    name: String,

    /// The prelude (everything between the name and the block/semicolon)
    prelude: String,

    /// The raw prelude as component values
    prelude_values: Vec<ComponentValue>,

    /// Child rules (for block at-rules like @media)
    pub rules: Vec<CSSRule>,

    /// Declarations (for at-rules that contain declarations, like @font-face)
    declarations: Vec<CSSDeclaration>,

    /// Whether this at-rule has a block (vs ending with semicolon)
    has_block: bool,
}

impl CSSAtRule {
    /// Create a new at-rule with the given name
    pub fn new(name: String) -> Self {
        CSSAtRule {
            name,
            prelude: String::new(),
            prelude_values: Vec::new(),
            rules: Vec::new(),
            declarations: Vec::new(),
            has_block: false,
        }
    }

    /// Create a CSSAtRule from a parsed AtRule
    pub fn from_parsed(ar: AtRule, collect_positions: bool) -> Self {
        let prelude = prelude_to_string(&ar.prelude);
        let has_block = ar.block.is_some();

        let mut css_at_rule = CSSAtRule {
            name: ar.name.clone(),
            prelude,
            prelude_values: ar.prelude,
            rules: Vec::new(),
            declarations: Vec::new(),
            has_block,
        };

        if let Some(block) = ar.block {
            css_at_rule.parse_block_contents(&ar.name, &block, collect_positions);
        }

        css_at_rule
    }

    /// Parse block contents based on the at-rule type
    fn parse_block_contents(&mut self, name: &str, block: &SimpleBlock, collect_positions: bool) {
        match name.to_lowercase().as_str() {
            "media" | "supports" | "document" | "layer" | "scope" | "container" => {
                self.parse_nested_rules(block, collect_positions)
            }
            "font-face"
            | "page"
            | "counter-style"
            | "font-feature-values"
            | "font-palette-values"
            | "property" => self.parse_declarations(block),
            "keyframes" => self.parse_keyframe_rules(block, collect_positions),
            _ => self.parse_nested_rules(block, collect_positions),
        }
    }

    /// Parse block as nested rules
    fn parse_nested_rules(&mut self, block: &SimpleBlock, collect_positions: bool) {
        let mut current_prelude: Vec<ComponentValue> = Vec::new();
        let mut in_block = false;
        let mut block_depth = 0;
        let mut current_block_value: Vec<ComponentValue> = Vec::new();

        for cv in &block.value {
            if in_block {
                match cv {
                    ComponentValue::Token(token) => {
                        match token.kind {
                            CssTokenKind::OpenCurly => {
                                block_depth += 1;
                                current_block_value.push(cv.clone());
                            }
                            CssTokenKind::CloseCurly => {
                                if block_depth > 0 {
                                    block_depth -= 1;
                                    current_block_value.push(cv.clone());
                                } else {
                                    // End of this rule's block
                                    let qr = QualifiedRule {
                                        prelude: current_prelude.clone(),
                                        block: SimpleBlock {
                                            associated_token: AssociatedToken::CurlyBracket,
                                            value: current_block_value.clone(),
                                        },
                                    };
                                    if let Some(style_rule) =
                                        CSSStyleRule::from_parsed(qr, collect_positions)
                                    {
                                        self.rules.push(CSSRule::Style(style_rule));
                                    }
                                    current_prelude.clear();
                                    current_block_value.clear();
                                    in_block = false;
                                }
                            }
                            _ => {
                                current_block_value.push(cv.clone());
                            }
                        }
                    }
                    ComponentValue::SimpleBlock(sb)
                        if sb.associated_token == AssociatedToken::CurlyBracket =>
                    {
                        current_block_value.push(cv.clone());
                        block_depth += 1;
                    }
                    _ => {
                        current_block_value.push(cv.clone());
                    }
                }
            } else {
                match cv {
                    ComponentValue::Token(token) => match token.kind {
                        CssTokenKind::OpenCurly => {
                            in_block = true;
                        }
                        CssTokenKind::Whitespace if current_prelude.is_empty() => {}
                        _ => {
                            current_prelude.push(cv.clone());
                        }
                    },
                    ComponentValue::SimpleBlock(sb)
                        if sb.associated_token == AssociatedToken::CurlyBracket =>
                    {
                        let qr = QualifiedRule {
                            prelude: current_prelude.clone(),
                            block: sb.clone(),
                        };
                        if let Some(style_rule) = CSSStyleRule::from_parsed(qr, collect_positions) {
                            self.rules.push(CSSRule::Style(style_rule));
                        }
                        current_prelude.clear();
                    }
                    _ => {
                        current_prelude.push(cv.clone());
                    }
                }
            }
        }
    }

    /// Parse block as declarations
    fn parse_declarations(&mut self, block: &SimpleBlock) {
        let mut temp_ident: Option<String> = None;
        let mut temp_values: Vec<ComponentValue> = Vec::new();
        let mut in_declaration = false;

        for cv in &block.value {
            match cv {
                ComponentValue::Token(token) => match &token.kind {
                    CssTokenKind::Ident(name) if !in_declaration && temp_ident.is_none() => {
                        temp_ident = Some(name.clone());
                    }
                    CssTokenKind::Colon if temp_ident.is_some() && !in_declaration => {
                        in_declaration = true;
                    }
                    CssTokenKind::Semicolon if in_declaration => {
                        if let Some(name) = temp_ident.take() {
                            let property = Property::from(name);
                            let decl = CSSDeclaration::from_values(property, temp_values.clone());
                            self.declarations.push(decl);
                        }
                        temp_values.clear();
                        in_declaration = false;
                    }
                    CssTokenKind::Whitespace => {
                        if in_declaration && !temp_values.is_empty() {
                            temp_values.push(cv.clone());
                        }
                    }
                    _ => {
                        if in_declaration {
                            temp_values.push(cv.clone());
                        } else if temp_ident.is_some() {
                            temp_ident = None;
                        }
                    }
                },
                ComponentValue::Function(_) | ComponentValue::SimpleBlock(_) => {
                    if in_declaration {
                        temp_values.push(cv.clone());
                    }
                }
            }
        }

        if in_declaration && let Some(name) = temp_ident {
            let property = Property::from(name);
            let decl = CSSDeclaration::from_values(property, temp_values);
            self.declarations.push(decl);
        }
    }

    /// Parse block as keyframe rules
    fn parse_keyframe_rules(&mut self, block: &SimpleBlock, collect_positions: bool) {
        // TODO: Implement a dedicated CSSKeyframeRule struct someday
        // For now, we treat them as nested style rules
        self.parse_nested_rules(block, collect_positions);
    }

    /// Get the at-rule name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the prelude text
    pub fn prelude(&self) -> &str {
        &self.prelude
    }

    /// Get the declarations
    pub fn declarations(&self) -> &[CSSDeclaration] {
        &self.declarations
    }

    /// Check if this at-rule has a block
    pub fn has_block(&self) -> bool {
        self.has_block
    }

    /// Serialize this at-rule to CSS text
    pub fn to_css_string(&self) -> String {
        let mut result = format!("@{}", self.name);

        if !self.prelude.is_empty() {
            result.push(' ');
            result.push_str(&self.prelude);
        }

        if self.has_block {
            result.push_str(" {\n");

            for decl in &self.declarations {
                result.push_str("  ");
                result.push_str(&decl.to_css_string());
                result.push_str(";\n");
            }

            for rule in &self.rules {
                result.push_str("  ");
                result.push_str(&rule.to_css_string().replace('\n', "\n  "));
                result.push('\n');
            }

            result.push('}');
        } else {
            result.push(';');
        }

        result
    }
}

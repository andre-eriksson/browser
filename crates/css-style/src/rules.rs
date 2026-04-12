use std::collections::HashSet;

use browser_preferences::theme::ThemeCategory;
use css_cssom::{
    CSSAtRule, CSSDeclaration, CSSRule, CSSStyleRule, CSSStyleSheet, ComponentValue, ComponentValueStream,
    CssTokenKind, Property, SimpleBlock, StylesheetOrigin,
};
use css_selectors::{CompoundSelectorSequence, SelectorSpecificity, SpecificityCalculable, generate_selector_list};
use css_values::{
    media::{MediaCondition, MediaFeature, MediaType, RangeOperator},
    property::{PropertyDescriptor, PropertySyntax, SyntaxComponent},
    quantity::{Length, LengthUnit},
};

use crate::{
    AbsoluteContext,
    cascade::{CascadeSpecificity, CascadedDeclaration},
    properties::PixelRepr,
    specified::SpecifiedStyle,
    tree::PropertyRegistry,
};

/// A rule that has been generated from the stylesheets, containing the selector sequences, declarations, origin, and specificity for cascade resolution.
#[derive(Debug)]
pub struct GeneratedRule<'css> {
    pub selector_sequences: Vec<CompoundSelectorSequence>,
    pub declarations: &'css [CSSDeclaration],
    pub origin: StylesheetOrigin,
    pub specificity: SelectorSpecificity,
}

impl<'css> GeneratedRule<'css> {
    /// Build a list of generated rules from the provided stylesheets, filtering out any rules that are not
    /// applicable based on the absolute context (e.g. media queries that don't match the current environment).
    pub fn build(
        stylesheets: &'css [CSSStyleSheet],
        property_registry: &mut PropertyRegistry,
        absolute_ctx: &AbsoluteContext,
    ) -> Vec<Self> {
        let mut generated_rules = Vec::new();

        for stylesheet in stylesheets {
            for rule in stylesheet.css_rules() {
                match rule {
                    CSSRule::Style(style) => {
                        Self::push_rule(&mut generated_rules, stylesheet, style);
                    }
                    CSSRule::AtRule(at_rule) => {
                        if !Self::allows_at_rule(at_rule, property_registry, absolute_ctx) {
                            continue;
                        }

                        for rule in &at_rule.rules {
                            match rule {
                                CSSRule::Style(style) => Self::push_rule(&mut generated_rules, stylesheet, style),
                                CSSRule::AtRule(nested_at_rule) => {
                                    Self::handle_nested_at_rule(
                                        &mut generated_rules,
                                        stylesheet,
                                        nested_at_rule,
                                        property_registry,
                                        absolute_ctx,
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        generated_rules
    }

    /// Recursively handle nested at-rules, filtering out any that are not applicable based on the absolute context.
    fn handle_nested_at_rule(
        generated_rules: &mut Vec<Self>,
        stylesheet: &CSSStyleSheet,
        at_rule: &'css CSSAtRule,
        property_registry: &mut PropertyRegistry,
        absolute_ctx: &AbsoluteContext,
    ) {
        if !at_rule.can_be_nested() || !Self::allows_at_rule(at_rule, property_registry, absolute_ctx) {
            return;
        }

        for rule in &at_rule.rules {
            match rule {
                CSSRule::Style(style) => Self::push_rule(generated_rules, stylesheet, style),
                CSSRule::AtRule(nested_at_rule) => Self::handle_nested_at_rule(
                    generated_rules,
                    stylesheet,
                    nested_at_rule,
                    property_registry,
                    absolute_ctx,
                ),
            }
        }
    }

    /// Check if an at-rule is allowed based on the absolute context (e.g. media queries that don't match the current environment should be disallowed).
    fn allows_at_rule(
        at_rule: &CSSAtRule,
        property_registry: &mut PropertyRegistry,
        absolute_ctx: &AbsoluteContext,
    ) -> bool {
        if at_rule.name().eq_ignore_ascii_case("import")
            || at_rule.name().eq_ignore_ascii_case("scope")
            || at_rule.name().eq_ignore_ascii_case("font-face")
            || at_rule.name().eq_ignore_ascii_case("keyframes")
        {
            // TODO: Handle these at-rules properly
            //       (e.g. @import should be processed as if its rules were inlined here,
            //       @supports should conditionally include rules based on support, etc.)
            return false;
        }

        if at_rule.name().eq_ignore_ascii_case("media") {
            let stream = ComponentValueStream::new(at_rule.prelude_values());

            Self::eval_or_logic(stream, |and_stream| {
                Self::eval_and_logic(and_stream, HashSet::new, |query, media_types| {
                    Self::handle_media_query(query, absolute_ctx, media_types)
                })
            })
        } else if at_rule.name().eq_ignore_ascii_case("supports") {
            let stream = ComponentValueStream::new(at_rule.prelude_values());

            Self::eval_or_logic(stream, |and_stream| {
                Self::eval_and_logic(
                    and_stream,
                    || (),
                    |query, _| Self::handle_supports_condition(query, property_registry, absolute_ctx),
                )
            })
        } else if at_rule.name().eq_ignore_ascii_case("layer") {
            // TODO: Handle @layer properly by respecting layer order and allowing layers to be enabled/disabled via media queries or other conditions.
            //       For now, we will simply ignore all rules inside @layer blocks to avoid complications with layer ordering and conditional enabling.
            true
        } else if at_rule.name().eq_ignore_ascii_case("property") {
            let mut name_stream = ComponentValueStream::new(at_rule.prelude_values());
            let mut syntax = None;
            let mut inherits = false;
            let mut initial_value = None;

            let name = match name_stream.next_non_whitespace() {
                Some(ComponentValue::Token(token)) => {
                    if let CssTokenKind::Ident(ident) = &token.kind {
                        ident.clone()
                    } else {
                        return false;
                    }
                }
                _ => return false,
            };

            for decl in at_rule.declarations() {
                if let Some(custom) = decl.property().as_custom() {
                    if custom.eq_ignore_ascii_case("syntax") {
                        let mut value_stream = ComponentValueStream::new(decl.original_values.as_slice());

                        if let Some(ComponentValue::Token(token)) = value_stream.next_non_whitespace() {
                            if let CssTokenKind::String(syntax_str) = &token.kind {
                                syntax = Some(Self::parse_syntax_string(syntax_str));
                            } else {
                                return false;
                            }
                        } else {
                            return false;
                        }
                    } else if custom.eq_ignore_ascii_case("inherits") {
                        let mut value_stream = ComponentValueStream::new(decl.original_values.as_slice());
                        if let Some(ComponentValue::Token(token)) = value_stream.next_non_whitespace()
                            && let CssTokenKind::Ident(ident) = &token.kind
                        {
                            if ident.eq_ignore_ascii_case("true") {
                                inherits = true;
                            } else if ident.eq_ignore_ascii_case("false") {
                                inherits = false;
                            } else {
                                return false;
                            }
                        } else {
                            return false;
                        }
                    } else if custom.eq_ignore_ascii_case("initial-value") {
                        if let Some(syntax) = &syntax
                            && !syntax.validate(&decl.original_values)
                        {
                            return false;
                        }

                        initial_value = Some(decl.original_values.clone());
                    }
                }
            }

            if !name.starts_with("--") {
                return false;
            }

            let syntax = match syntax {
                Some(s) => s,
                None => return false,
            };

            if initial_value.is_none() && syntax != PropertySyntax::Universal {
                return false;
            }

            let descriptor = PropertyDescriptor {
                name: name.clone(),
                syntax,
                inherits,
                initial_value,
            };

            property_registry.descriptors.insert(name, descriptor);

            false
        } else {
            false
        }
    }

    /// Parses a syntax string from @property (e.g., "<length>", "<color> | <length>", "*")
    fn parse_syntax_string(syntax_str: &str) -> PropertySyntax {
        let trimmed = syntax_str.trim();

        if trimmed == "*" {
            return PropertySyntax::Universal;
        }

        let mut components = Vec::new();

        for part in trimmed.split('|') {
            let part = part.trim();

            if part.starts_with('<') && part.ends_with('>') {
                let type_name = &part[1..part.len() - 1];
                if let Ok(component) = type_name.parse::<SyntaxComponent>() {
                    components.push(component);
                }
            } else if !part.is_empty() {
                components.push(SyntaxComponent::Ident(part.to_string()));
            }
        }

        if components.is_empty() {
            PropertySyntax::Universal
        } else {
            PropertySyntax::Typed(components)
        }
    }

    fn eval_or_logic<F>(stream: ComponentValueStream, check: F) -> bool
    where
        F: FnMut(ComponentValueStream) -> bool,
    {
        stream
            .split_by(|cv| Self::is_logic_split(cv, "or", true))
            .any(check)
    }

    fn eval_and_logic<S, I, F>(stream: ComponentValueStream, init_state: I, mut check: F) -> bool
    where
        I: FnOnce() -> S,
        F: FnMut(ComponentValueStream, &mut S) -> bool,
    {
        let mut state = init_state();
        stream
            .split_by(|cv| Self::is_logic_split(cv, "and", false))
            .all(|sub_query| check(sub_query, &mut state))
    }

    fn is_logic_split(cv: &ComponentValue, ident: &str, include_comma: bool) -> bool {
        matches!(cv, ComponentValue::Token(token) if
            (include_comma && matches!(&token.kind, CssTokenKind::Comma)) ||
            matches!(&token.kind, CssTokenKind::Ident(i) if i.eq_ignore_ascii_case(ident))
        )
    }

    fn handle_media_query(
        mut stream: ComponentValueStream,
        absolute_ctx: &AbsoluteContext,
        media_types: &mut HashSet<MediaType>,
    ) -> bool {
        let mut is_not = false;

        while let Some(cv) = stream.next_non_whitespace() {
            match cv {
                ComponentValue::Token(token) => match &token.kind {
                    CssTokenKind::Ident(ident) => {
                        if ident.eq_ignore_ascii_case("not") {
                            is_not = true;
                            continue;
                        }

                        let media_type = match ident.parse::<MediaType>() {
                            Ok(mt) => mt,
                            Err(_) => return false,
                        };

                        if !matches!(media_type, MediaType::All | MediaType::Screen | MediaType::Print) {
                            // <https://drafts.csswg.org/mediaqueries/#media-types>
                            continue;
                        }

                        media_types.insert(media_type);
                    }
                    _ => continue,
                },

                ComponentValue::SimpleBlock(block) => {
                    if media_types.contains(&MediaType::Print) && media_types.len() == 1 {
                        // NOTE: We don't support print media queries or the print format.
                        return false;
                    }

                    return Self::handle_media_block(block, absolute_ctx) ^ is_not;
                }
                _ => return false,
            }
        }

        false
    }

    fn handle_media_block(block: &SimpleBlock, absolute_ctx: &AbsoluteContext) -> bool {
        let mut block_stream = ComponentValueStream::new(&block.value);

        if let Some(ComponentValue::Token(token)) = block_stream.next_non_whitespace() {
            match &token.kind {
                CssTokenKind::Ident(ident) => {
                    let bytes = ident.as_bytes();

                    if bytes.len() < 4 {
                        return false;
                    }

                    let mut buf = [0u8; 4];
                    buf.copy_from_slice(bytes[..4].try_into().unwrap());
                    buf.make_ascii_lowercase();

                    match &buf {
                        b"max-" => {
                            let remaining = match str::from_utf8(&bytes[4..]) {
                                Ok(s) => s,
                                Err(_) => return false,
                            };

                            let media_condition = match remaining.parse::<MediaCondition>() {
                                Ok(media_condition) => media_condition,
                                Err(_) => return false,
                            };

                            return Self::handle_media_range_max(media_condition, &mut block_stream, absolute_ctx);
                        }
                        b"min-" => {
                            let remaining = match str::from_utf8(&bytes[4..]) {
                                Ok(s) => s,
                                Err(_) => return false,
                            };

                            let media_condition = match remaining.parse::<MediaCondition>() {
                                Ok(media_condition) => media_condition,
                                Err(_) => return false,
                            };

                            return Self::handle_media_range_min(media_condition, &mut block_stream, absolute_ctx);
                        }
                        _ => {
                            let media_feature = match ident.parse::<MediaFeature>() {
                                Ok(feature) => feature,
                                Err(_) => return false,
                            };

                            match media_feature {
                                MediaFeature::PrefersColorScheme => {
                                    block_stream.skip_whitespace();
                                    block_stream.next_cv(); // Skip ':'
                                    if let Some(ComponentValue::Token(value_token)) = block_stream.next_non_whitespace()
                                        && let CssTokenKind::Ident(value_ident) = &value_token.kind
                                    {
                                        return match absolute_ctx.theme_category {
                                            ThemeCategory::Dark => value_ident.eq_ignore_ascii_case("dark"),
                                            ThemeCategory::Light => value_ident.eq_ignore_ascii_case("light"),
                                        };
                                    }
                                }
                            }
                        }
                    }
                }
                CssTokenKind::Dimension { value, unit } => {
                    // Range: 100px <= width <= 500px OR 500px >= width >= 100px
                    // Range Values (ident): aspect-ratio, color, color-index, device-aspect-ratio,
                    //               device-height, device-width, height, resolution, width

                    // TODO: Resolution
                    let first_length_unit = match unit.parse::<LengthUnit>() {
                        Ok(unit) => unit,
                        Err(_) => return false,
                    };
                    let first_length = Length::new(value.to_f64() as f32, first_length_unit);

                    let first_comparison_token = match block_stream.next_non_whitespace() {
                        Some(ComponentValue::Token(token)) => {
                            if let CssTokenKind::Delim(delim) = token.kind {
                                delim
                            } else {
                                return false;
                            }
                        }
                        _ => return false,
                    };

                    let second_comparison_token = match block_stream.peek() {
                        Some(ComponentValue::Token(token)) => {
                            if let CssTokenKind::Delim(delim) = token.kind {
                                block_stream.next_cv();
                                Some(delim)
                            } else {
                                None
                            }
                        }
                        _ => None,
                    };

                    let first_comparison = match (first_comparison_token, second_comparison_token) {
                        ('<', Some('=')) => RangeOperator::LessThanOrEqual,
                        ('>', Some('=')) => RangeOperator::GreaterThanOrEqual,
                        ('<', None) => RangeOperator::LessThan,
                        ('>', None) => RangeOperator::GreaterThan,
                        _ => return false,
                    };

                    let value = match block_stream.next_non_whitespace() {
                        Some(ComponentValue::Token(token)) => {
                            if let CssTokenKind::Ident(ident) = &token.kind {
                                match ident.parse::<MediaCondition>() {
                                    Ok(cond) => cond,
                                    Err(_) => return false,
                                }
                            } else {
                                return false;
                            }
                        }
                        _ => return false,
                    };

                    let first_comparison_token = match block_stream.next_non_whitespace() {
                        Some(ComponentValue::Token(token)) => {
                            if let CssTokenKind::Delim(delim) = token.kind {
                                delim
                            } else {
                                return false;
                            }
                        }
                        _ => return false,
                    };

                    let second_comparison_token = match block_stream.peek() {
                        Some(ComponentValue::Token(token)) => {
                            if let CssTokenKind::Delim(delim) = token.kind {
                                block_stream.next_cv();
                                Some(delim)
                            } else {
                                None
                            }
                        }
                        _ => None,
                    };

                    let second_comparison = match (first_comparison_token, second_comparison_token) {
                        ('<', Some('=')) => RangeOperator::LessThanOrEqual,
                        ('>', Some('=')) => RangeOperator::GreaterThanOrEqual,
                        ('<', None) => RangeOperator::LessThan,
                        ('>', None) => RangeOperator::GreaterThan,
                        _ => return false,
                    };

                    let second_length = match block_stream.next_non_whitespace() {
                        Some(ComponentValue::Token(token)) => {
                            if let CssTokenKind::Dimension { value, unit } = &token.kind {
                                let length_unit = match unit.parse::<LengthUnit>() {
                                    Ok(unit) => unit,
                                    Err(_) => return false,
                                };
                                Length::new(value.to_f64() as f32, length_unit)
                            } else {
                                return false;
                            }
                        }
                        _ => return false,
                    };

                    match value {
                        MediaCondition::Width | MediaCondition::DeviceWidth => {
                            let viewport_width = absolute_ctx.viewport_width;
                            return Self::resolve_range(
                                absolute_ctx,
                                first_length,
                                first_comparison,
                                second_comparison,
                                second_length,
                                viewport_width,
                            );
                        }
                        MediaCondition::Height | MediaCondition::DeviceHeight => {
                            let viewport_height = absolute_ctx.viewport_height;
                            return Self::resolve_range(
                                absolute_ctx,
                                first_length,
                                first_comparison,
                                second_comparison,
                                second_length,
                                viewport_height,
                            );
                        }
                        _ => return false,
                    }
                }
                _ => return false,
            }
        }

        false
    }

    fn handle_media_range_max(
        media_condition: MediaCondition,
        block_stream: &mut ComponentValueStream,
        absolute_ctx: &AbsoluteContext,
    ) -> bool {
        block_stream.skip_whitespace();
        block_stream.next_cv(); // Skip ':'

        if let Some(ComponentValue::Token(token)) = block_stream.next_non_whitespace() {
            match &token.kind {
                CssTokenKind::Dimension { value, unit } => {
                    let length_unit = match unit.parse::<LengthUnit>() {
                        Ok(unit) => unit,
                        Err(_) => return false,
                    };
                    let length = Length::new(value.to_f64() as f32, length_unit);

                    match media_condition {
                        MediaCondition::Width | MediaCondition::DeviceWidth => {
                            let viewport_width = absolute_ctx.viewport_width;
                            return viewport_width <= length.to_px(None, None, absolute_ctx);
                        }
                        MediaCondition::Height | MediaCondition::DeviceHeight => {
                            let viewport_height = absolute_ctx.viewport_height;
                            return viewport_height <= length.to_px(None, None, absolute_ctx);
                        }
                        _ => return false,
                    }
                }
                _ => return false,
            }
        }

        false
    }

    fn handle_media_range_min(
        media_condition: MediaCondition,
        block_stream: &mut ComponentValueStream,
        absolute_ctx: &AbsoluteContext,
    ) -> bool {
        block_stream.skip_whitespace();
        block_stream.next_cv(); // Skip ':'
        block_stream.skip_whitespace();

        if let Some(ComponentValue::Token(token)) = block_stream.next_non_whitespace() {
            match &token.kind {
                CssTokenKind::Dimension { value, unit } => {
                    let length_unit = match unit.parse::<LengthUnit>() {
                        Ok(unit) => unit,
                        Err(_) => return false,
                    };
                    let length = Length::new(value.to_f64() as f32, length_unit);

                    match media_condition {
                        MediaCondition::Width | MediaCondition::DeviceWidth => {
                            let viewport_width = absolute_ctx.viewport_width;
                            return viewport_width >= length.to_px(None, None, absolute_ctx);
                        }
                        MediaCondition::Height | MediaCondition::DeviceHeight => {
                            let viewport_height = absolute_ctx.viewport_height;
                            return viewport_height >= length.to_px(None, None, absolute_ctx);
                        }
                        _ => return false,
                    }
                }
                _ => return false,
            }
        }

        false
    }

    fn resolve_range(
        absolute_ctx: &AbsoluteContext<'_>,
        first_length: Length,
        first_comparison: RangeOperator,
        second_comparison: RangeOperator,
        second_length: Length,
        value: f32,
    ) -> bool {
        if matches!(first_comparison, RangeOperator::LessThan | RangeOperator::LessThanOrEqual)
            && matches!(second_comparison, RangeOperator::GreaterThan | RangeOperator::GreaterThanOrEqual)
        {
            return false;
        }

        let first_condition = match first_comparison {
            RangeOperator::LessThan => value > first_length.to_px(None, None, absolute_ctx),
            RangeOperator::LessThanOrEqual => value >= first_length.to_px(None, None, absolute_ctx),
            RangeOperator::GreaterThan => value < first_length.to_px(None, None, absolute_ctx),
            RangeOperator::GreaterThanOrEqual => value <= first_length.to_px(None, None, absolute_ctx),
            _ => return false,
        };
        let second_condition = match second_comparison {
            RangeOperator::LessThan => value < second_length.to_px(None, None, absolute_ctx),
            RangeOperator::LessThanOrEqual => value <= second_length.to_px(None, None, absolute_ctx),
            RangeOperator::GreaterThan => value > second_length.to_px(None, None, absolute_ctx),
            RangeOperator::GreaterThanOrEqual => value >= second_length.to_px(None, None, absolute_ctx),
            _ => return false,
        };
        first_condition && second_condition
    }

    fn handle_supports_condition(
        mut stream: ComponentValueStream,
        property_registry: &PropertyRegistry,
        absolute_ctx: &AbsoluteContext,
    ) -> bool {
        let mut is_not = false;

        while let Some(cv) = stream.next_non_whitespace() {
            match cv {
                ComponentValue::Token(token) => match &token.kind {
                    CssTokenKind::Ident(ident) if ident.eq_ignore_ascii_case("not") => {
                        is_not = true;
                        continue;
                    }
                    _ => return false,
                },
                ComponentValue::SimpleBlock(block) => {
                    return Self::handle_supports_block(block, property_registry, absolute_ctx) ^ is_not;
                }
                _ => return false,
            }
        }

        false
    }

    fn handle_supports_block(
        block: &SimpleBlock,
        property_registry: &PropertyRegistry,
        absolute_ctx: &AbsoluteContext,
    ) -> bool {
        let block_stream = ComponentValueStream::new(&block.value);

        let prop_value_streams = block_stream
            .split_by(|cv| matches!(cv, ComponentValue::Token(token) if matches!(&token.kind, CssTokenKind::Colon)));

        let mut property = None;
        let mut value = None;

        for mut stream in prop_value_streams {
            if property.is_none()
                && let Some(ComponentValue::Token(token)) = stream.next_non_whitespace()
                && let CssTokenKind::Ident(ident) = &token.kind
            {
                property = Some(Property::from(ident.clone()));
            } else if let Some(p) = std::mem::take(&mut property) {
                stream.skip_whitespace();
                value = Some(CSSDeclaration::from_values(p, stream.remaining().to_vec()));
                break;
            }
        }

        let declaration = match value {
            Some(decl) => decl,
            None => return false,
        };

        let decl = CascadedDeclaration {
            important: false,
            origin: StylesheetOrigin::Author,
            property: &declaration.property,
            source_order: 0,
            specificity: CascadeSpecificity::inline(),
            values: &declaration.original_values,
        };

        SpecifiedStyle::supports(decl, property_registry, absolute_ctx)
    }

    /// Push a style rule into the generated rules list, extracting its selector sequences, declarations, origin, and specificity for cascade resolution.
    fn push_rule(generated_rules: &mut Vec<Self>, stylesheet: &CSSStyleSheet, style_rule: &'css CSSStyleRule) {
        let selector_list = generate_selector_list(&style_rule.prelude);

        for selector_sequence in selector_list {
            let specificity = selector_sequence
                .iter()
                .map(|seq| seq.specificity())
                .max()
                .unwrap_or_default();

            generated_rules.push(GeneratedRule {
                selector_sequences: selector_sequence,
                declarations: style_rule.declarations(),
                origin: stylesheet.origin(),
                specificity,
            });
        }
    }
}

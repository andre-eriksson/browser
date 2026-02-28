use std::{collections::HashMap, sync::Arc};

use css_cssom::{CSSStyleSheet, ComponentValue, KnownProperty, Property};
use html_dom::{DocumentRoot, NodeId};

use crate::{
    ComputedStyle, RelativeContext,
    cascade::{CascadedDeclaration, GeneratedRule, RuleIndex, cascade, cascade_variables},
    functions::variables::resolve_css_variables,
    global::Global,
    handler::{
        PropertyUpdateContext, handle_background_color, handle_border, handle_border_bottom_color,
        handle_border_bottom_style, handle_border_bottom_width, handle_border_left_color, handle_border_left_style,
        handle_border_left_width, handle_border_right_color, handle_border_right_style, handle_border_right_width,
        handle_border_top_color, handle_border_top_style, handle_border_top_width, handle_color, handle_display,
        handle_font_family, handle_font_size, handle_font_weight, handle_height, handle_line_height, handle_margin,
        handle_margin_block, handle_margin_block_end, handle_margin_block_start, handle_margin_bottom,
        handle_margin_inline, handle_margin_inline_end, handle_margin_inline_start, handle_margin_left,
        handle_margin_right, handle_margin_top, handle_max_height, handle_max_width, handle_padding,
        handle_padding_block, handle_padding_block_end, handle_padding_block_start, handle_padding_bottom,
        handle_padding_inline, handle_padding_inline_end, handle_padding_inline_start, handle_padding_left,
        handle_padding_right, handle_padding_top, handle_position, handle_text_align, handle_whitespace, handle_width,
        handle_writing_mode,
    },
    properties::{
        AbsoluteContext, BorderStyleValueProperty, BorderWidthValueProperty, CSSProperty, ColorProperty,
        DisplayProperty, FontFamilyProperty, FontSizeProperty, FontWeightProperty, HeightProperty, LineHeightProperty,
        MaxHeightProperty, MaxWidthProperty, OffsetValueProperty, PositionProperty, TextAlignProperty,
        WhitespaceProperty, WidthProperty, WritingModeProperty,
    },
};

/// Represents the specified style of an element after applying the cascade and resolving variables.
#[derive(Debug, Clone, PartialEq)]
pub struct SpecifiedStyle {
    pub background_color: ColorProperty,
    pub border_top_color: ColorProperty,
    pub border_right_color: ColorProperty,
    pub border_bottom_color: ColorProperty,
    pub border_left_color: ColorProperty,
    pub border_top_style: BorderStyleValueProperty,
    pub border_right_style: BorderStyleValueProperty,
    pub border_bottom_style: BorderStyleValueProperty,
    pub border_left_style: BorderStyleValueProperty,
    pub border_top_width: BorderWidthValueProperty,
    pub border_right_width: BorderWidthValueProperty,
    pub border_bottom_width: BorderWidthValueProperty,
    pub border_left_width: BorderWidthValueProperty,
    pub color: ColorProperty,
    pub display: DisplayProperty,
    pub font_family: FontFamilyProperty,
    pub font_size: FontSizeProperty,
    pub font_weight: FontWeightProperty,
    pub height: HeightProperty,
    pub max_height: MaxHeightProperty,
    pub line_height: LineHeightProperty,
    pub margin_top: OffsetValueProperty,
    pub margin_right: OffsetValueProperty,
    pub margin_bottom: OffsetValueProperty,
    pub margin_left: OffsetValueProperty,
    pub padding_top: OffsetValueProperty,
    pub padding_right: OffsetValueProperty,
    pub padding_bottom: OffsetValueProperty,
    pub padding_left: OffsetValueProperty,
    pub position: PositionProperty,
    pub text_align: TextAlignProperty,
    pub whitespace: WhitespaceProperty,
    pub width: WidthProperty,
    pub max_width: MaxWidthProperty,
    pub writing_mode: WritingModeProperty,

    // === Non-CSS properties ===
    pub computed_font_size_px: f32,
    pub variables: Arc<Vec<(Property, Vec<ComponentValue>)>>,
}

impl SpecifiedStyle {
    /// Computes the ComputedStyle for a given node in the DOM.
    pub fn from_node(
        absolute_ctx: &AbsoluteContext,
        relative_ctx: &RelativeContext,
        node_id: &NodeId,
        dom: &DocumentRoot,
        rules: &[GeneratedRule],
        rule_index: &RuleIndex,
        parent_style: Option<&ComputedStyle>,
    ) -> Self {
        let mut specified_style = SpecifiedStyle::default();

        let parent_variables = parent_style
            .as_ref()
            .map(|p| Arc::clone(&p.variables))
            .unwrap_or_default();

        let node = match dom.get_node(node_id) {
            Some(n) => n,
            None => return specified_style,
        };

        let inline_declarations = node
            .data
            .as_element()
            .and_then(|e| e.get_attribute("style"))
            .map(CSSStyleSheet::from_inline)
            .unwrap_or_default();

        let (declarations, variables) =
            &mut CascadedDeclaration::collect(node, dom, rules, rule_index, &inline_declarations);

        let properties = cascade(declarations);

        let new_vars = cascade_variables(variables);

        if !new_vars.is_empty() {
            let mut merged: HashMap<Property, Vec<ComponentValue>> = if parent_variables.is_empty() {
                HashMap::with_capacity(new_vars.len())
            } else {
                (*parent_variables)
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect()
            };

            for (name, value) in new_vars {
                merged.insert(name.clone(), value.to_vec());
            }

            specified_style.variables = Arc::new(merged.into_iter().collect());
        } else if !parent_variables.is_empty() {
            specified_style.variables = parent_variables;
        }

        let mut ctx = PropertyUpdateContext::new(absolute_ctx, &mut specified_style, relative_ctx);

        for (key, value) in properties {
            let val = resolve_css_variables(&ctx.specified_style.variables, value.as_slice());
            let v = val.as_slice();

            match key {
                Property::Known(prop) => match prop {
                    KnownProperty::Background => handle_background_color(&mut ctx, v), // TODO: handle other background properties
                    KnownProperty::BackgroundColor => handle_background_color(&mut ctx, v),
                    KnownProperty::Border => handle_border(&mut ctx, v),
                    KnownProperty::BorderBottomColor => handle_border_bottom_color(&mut ctx, v),
                    KnownProperty::BorderBottomStyle => handle_border_bottom_style(&mut ctx, v),
                    KnownProperty::BorderBottomWidth => handle_border_bottom_width(&mut ctx, v),
                    KnownProperty::BorderLeftColor => handle_border_left_color(&mut ctx, v),
                    KnownProperty::BorderLeftStyle => handle_border_left_style(&mut ctx, v),
                    KnownProperty::BorderLeftWidth => handle_border_left_width(&mut ctx, v),
                    KnownProperty::BorderRightColor => handle_border_right_color(&mut ctx, v),
                    KnownProperty::BorderRightStyle => handle_border_right_style(&mut ctx, v),
                    KnownProperty::BorderRightWidth => handle_border_right_width(&mut ctx, v),
                    KnownProperty::BorderTopColor => handle_border_top_color(&mut ctx, v),
                    KnownProperty::BorderTopStyle => handle_border_top_style(&mut ctx, v),
                    KnownProperty::BorderTopWidth => handle_border_top_width(&mut ctx, v),
                    KnownProperty::Color => handle_color(&mut ctx, v),
                    KnownProperty::Display => handle_display(&mut ctx, v),
                    KnownProperty::FontFamily => handle_font_family(&mut ctx, v),
                    KnownProperty::FontSize => handle_font_size(&mut ctx, v),
                    KnownProperty::FontWeight => handle_font_weight(&mut ctx, v),
                    KnownProperty::Height => handle_height(&mut ctx, v),
                    KnownProperty::LineHeight => handle_line_height(&mut ctx, v),
                    KnownProperty::Margin => handle_margin(&mut ctx, v),
                    KnownProperty::MarginBlock => handle_margin_block(&mut ctx, v),
                    KnownProperty::MarginBlockEnd => handle_margin_block_end(&mut ctx, v),
                    KnownProperty::MarginBlockStart => handle_margin_block_start(&mut ctx, v),
                    KnownProperty::MarginBottom => handle_margin_bottom(&mut ctx, v),
                    KnownProperty::MarginLeft => handle_margin_left(&mut ctx, v),
                    KnownProperty::MarginRight => handle_margin_right(&mut ctx, v),
                    KnownProperty::MarginTop => handle_margin_top(&mut ctx, v),
                    KnownProperty::MarginInline => handle_margin_inline(&mut ctx, v),
                    KnownProperty::MarginInlineStart => handle_margin_inline_start(&mut ctx, v),
                    KnownProperty::MarginInlineEnd => handle_margin_inline_end(&mut ctx, v),
                    KnownProperty::MaxHeight => handle_max_height(&mut ctx, v),
                    KnownProperty::MaxWidth => handle_max_width(&mut ctx, v),
                    KnownProperty::Padding => handle_padding(&mut ctx, v),
                    KnownProperty::PaddingBlock => handle_padding_block(&mut ctx, v),
                    KnownProperty::PaddingBlockEnd => handle_padding_block_end(&mut ctx, v),
                    KnownProperty::PaddingBlockStart => handle_padding_block_start(&mut ctx, v),
                    KnownProperty::PaddingBottom => handle_padding_bottom(&mut ctx, v),
                    KnownProperty::PaddingLeft => handle_padding_left(&mut ctx, v),
                    KnownProperty::PaddingRight => handle_padding_right(&mut ctx, v),
                    KnownProperty::PaddingTop => handle_padding_top(&mut ctx, v),
                    KnownProperty::PaddingInline => handle_padding_inline(&mut ctx, v),
                    KnownProperty::PaddingInlineStart => handle_padding_inline_start(&mut ctx, v),
                    KnownProperty::PaddingInlineEnd => handle_padding_inline_end(&mut ctx, v),
                    KnownProperty::Position => handle_position(&mut ctx, v),
                    KnownProperty::TextAlign => handle_text_align(&mut ctx, v),
                    KnownProperty::WhiteSpace => handle_whitespace(&mut ctx, v),
                    KnownProperty::Width => handle_width(&mut ctx, v),
                    KnownProperty::WritingMode => handle_writing_mode(&mut ctx, v),
                    _ => {}
                },
                Property::Custom(_) => { /* Ignore custom properties here since they are already resolved */ }
            }
        }

        ctx.log_errors();

        specified_style
    }
}

impl Default for SpecifiedStyle {
    fn default() -> Self {
        SpecifiedStyle {
            variables: Arc::new(vec![]),
            computed_font_size_px: 16.0,

            // Non-inherited properties
            background_color: CSSProperty::Global(Global::Initial),
            border_top_color: CSSProperty::Global(Global::Initial),
            border_right_color: CSSProperty::Global(Global::Initial),
            border_bottom_color: CSSProperty::Global(Global::Initial),
            border_left_color: CSSProperty::Global(Global::Initial),
            border_top_style: CSSProperty::Global(Global::Initial),
            border_right_style: CSSProperty::Global(Global::Initial),
            border_bottom_style: CSSProperty::Global(Global::Initial),
            border_left_style: CSSProperty::Global(Global::Initial),
            border_top_width: CSSProperty::Global(Global::Initial),
            border_right_width: CSSProperty::Global(Global::Initial),
            border_bottom_width: CSSProperty::Global(Global::Initial),
            border_left_width: CSSProperty::Global(Global::Initial),
            display: CSSProperty::Global(Global::Initial),
            height: CSSProperty::Global(Global::Initial),
            max_height: CSSProperty::Global(Global::Initial),
            margin_top: CSSProperty::Global(Global::Initial),
            margin_right: CSSProperty::Global(Global::Initial),
            margin_bottom: CSSProperty::Global(Global::Initial),
            margin_left: CSSProperty::Global(Global::Initial),
            padding_top: CSSProperty::Global(Global::Initial),
            padding_right: CSSProperty::Global(Global::Initial),
            padding_bottom: CSSProperty::Global(Global::Initial),
            padding_left: CSSProperty::Global(Global::Initial),
            position: CSSProperty::Global(Global::Initial),
            width: CSSProperty::Global(Global::Initial),
            max_width: CSSProperty::Global(Global::Initial),

            // Inherited properties
            color: CSSProperty::Global(Global::Inherit),
            font_family: CSSProperty::Global(Global::Inherit),
            font_size: CSSProperty::Global(Global::Inherit),
            font_weight: CSSProperty::Global(Global::Inherit),
            line_height: CSSProperty::Global(Global::Inherit),
            text_align: CSSProperty::Global(Global::Inherit),
            whitespace: CSSProperty::Global(Global::Inherit),
            writing_mode: CSSProperty::Global(Global::Inherit),
        }
    }
}

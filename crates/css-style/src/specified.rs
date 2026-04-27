use std::{collections::HashMap, sync::Arc};

use css_cssom::{CSSStyleSheet, ComponentValue, ComponentValueStream, KnownProperty, Property};
use css_values::global::Global;
use html_dom::{DocumentRoot, NodeId};
use tracing::debug;

use crate::{
    RelativeContext,
    cascade::{CascadedDeclaration, cascade, cascade_variables},
    functions::variables::resolve_css_variables,
    handler::*,
    properties::*,
    rules::Rules,
    tree::PropertyRegistry,
};

/// Represents the specified style of an element after applying the cascade and resolving variables.
#[derive(Debug, Clone, PartialEq)]
pub struct SpecifiedStyle {
    pub align_content: AlignContentProperty,
    pub align_items: AlignItemsProperty,
    pub align_self: AlignSelfProperty,
    pub background_attachment: BackgroundAttachmentProperty,
    pub background_blend_mode: BackgroundBlendModeProperty,
    pub background_clip: BackgroundClipProperty,
    pub background_color: ColorProperty,
    pub background_image: BackgroundImageProperty,
    pub background_origin: BackgroundOriginProperty,
    pub background_position_x: BackgroundPositionXProperty,
    pub background_position_y: BackgroundPositionYProperty,
    pub background_repeat: BackgroundRepeatProperty,
    pub background_size: BackgroundSizeProperty,
    pub border_bottom_color: ColorProperty,
    pub border_bottom_style: BorderStyleValueProperty,
    pub border_bottom_width: BorderWidthValueProperty,
    pub border_left_color: ColorProperty,
    pub border_left_style: BorderStyleValueProperty,
    pub border_left_width: BorderWidthValueProperty,
    pub border_right_color: ColorProperty,
    pub border_right_style: BorderStyleValueProperty,
    pub border_right_width: BorderWidthValueProperty,
    pub border_top_color: ColorProperty,
    pub border_top_style: BorderStyleValueProperty,
    pub border_top_width: BorderWidthValueProperty,
    pub bottom: MarginProperty,
    pub clear: ClearProperty,
    pub color: ColorProperty,
    pub column_gap: GapProperty,
    pub cursor: CursorProperty,
    pub display: DisplayProperty,
    pub flex_basis: FlexBasisProperty,
    pub flex_direction: FlexDirectionProperty,
    pub flex_grow: FlexValueProperty,
    pub flex_shrink: FlexValueProperty,
    pub flex_wrap: FlexWrapProperty,
    pub float: FloatProperty,
    pub font_family: FontFamilyProperty,
    pub font_size: FontSizeProperty,
    pub font_weight: FontWeightProperty,
    pub height: SizeProperty,
    pub justify_content: JustifyContentProperty,
    pub justify_items: JustifyItemsProperty,
    pub justify_self: JustifySelfProperty,
    pub left: MarginProperty,
    pub line_height: LineHeightProperty,
    pub margin_bottom: MarginProperty,
    pub margin_left: MarginProperty,
    pub margin_right: MarginProperty,
    pub margin_top: MarginProperty,
    pub max_height: MaxSizeProperty,
    pub max_width: MaxSizeProperty,
    pub order: OrderProperty,
    pub padding_bottom: OffsetProperty,
    pub padding_left: OffsetProperty,
    pub padding_right: OffsetProperty,
    pub padding_top: OffsetProperty,
    pub position: PositionProperty,
    pub right: MarginProperty,
    pub row_gap: GapProperty,
    pub text_align: TextAlignProperty,
    pub top: MarginProperty,
    pub whitespace: WhitespaceProperty,
    pub width: SizeProperty,
    pub writing_mode: WritingModeProperty,

    // === Non-CSS properties ===
    pub computed_font_size_px: f64,
    pub variables: Arc<Vec<(Property, Vec<ComponentValue>)>>,
}

impl SpecifiedStyle {
    /// Computes the `ComputedStyle` for a given node in the DOM.
    pub fn from_node(
        absolute_ctx: &AbsoluteContext,
        relative_ctx: &RelativeContext,
        node_id: NodeId,
        dom: &DocumentRoot,
        rules: &Rules,
        property_registry: &PropertyRegistry,
    ) -> Self {
        let mut specified_style = Self::default();

        let parent_variables = Arc::clone(&relative_ctx.parent.variables);

        let Some(node) = dom.get_node(&node_id) else {
            return specified_style;
        };

        let inline_declarations = node
            .data
            .as_element()
            .and_then(|e| e.get_attribute("style"))
            .map(CSSStyleSheet::from_inline)
            .unwrap_or_default();

        let (declarations, variables) =
            &mut CascadedDeclaration::collect(node, dom, rules.generated, rules.index, &inline_declarations);

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
                merged.insert(name.clone(), value.clone());
            }

            specified_style.variables = Arc::new(merged.into_iter().collect());
        } else if !parent_variables.is_empty() {
            specified_style.variables = parent_variables;
        }

        let mut ctx = PropertyUpdateContext::new(absolute_ctx, &mut specified_style, relative_ctx);

        for (property, value) in properties {
            Self::resolve_property(property, value, property_registry, &mut ctx);
        }

        ctx.log_errors();

        specified_style
    }

    /// Checks if a given declaration is supported by the specified style system.
    pub fn supports(
        declaration: CascadedDeclaration,
        property_registry: &PropertyRegistry,
        absolute_ctx: &AbsoluteContext,
    ) -> bool {
        let mut declarations = vec![declaration];

        let mut default_style = Self::default();
        let default_relative_ctx = RelativeContext::default();

        let mut ctx = PropertyUpdateContext::new(absolute_ctx, &mut default_style, &default_relative_ctx);

        let properties = cascade(&mut declarations);

        for (property, value) in properties {
            if Self::resolve_property(property, value, property_registry, &mut ctx) && !ctx.has_errors() {
                return true;
            }
        }

        false
    }

    fn resolve_property(
        property: &Property,
        value: &Vec<ComponentValue>,
        property_registry: &PropertyRegistry,
        ctx: &mut PropertyUpdateContext<'_>,
    ) -> bool {
        let Some(val) = resolve_css_variables(&ctx.specified_style.variables, property_registry, value.as_slice())
        else {
            debug!(
                "Failed to resolve variables for property {:?} with value {}",
                property,
                value
                    .iter()
                    .map(std::string::ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(" ")
            );
            return false;
        };

        let mut stream = ComponentValueStream::new(&val);

        match property {
            Property::Known(prop) => match prop {
                KnownProperty::AlignContent => handle_align_content(ctx, &mut stream),
                KnownProperty::AlignItems => handle_align_items(ctx, &mut stream),
                KnownProperty::AlignSelf => handle_align_self(ctx, &mut stream),
                KnownProperty::Background => handle_background(ctx, &mut stream),
                KnownProperty::BackgroundAttachment => handle_background_attachment(ctx, &mut stream),
                KnownProperty::BackgroundBlendMode => handle_background_blend_mode(ctx, &mut stream),
                KnownProperty::BackgroundClip => handle_background_clip(ctx, &mut stream),
                KnownProperty::BackgroundColor => handle_background_color(ctx, &mut stream),
                KnownProperty::BackgroundImage => handle_background_image(ctx, &mut stream),
                KnownProperty::BackgroundOrigin => handle_background_origin(ctx, &mut stream),
                KnownProperty::BackgroundPosition => handle_background_position(ctx, &mut stream),
                KnownProperty::BackgroundPositionX => handle_background_position_x(ctx, &mut stream),
                KnownProperty::BackgroundPositionY => handle_background_position_y(ctx, &mut stream),
                KnownProperty::BackgroundRepeat => handle_background_repeat(ctx, &mut stream),
                KnownProperty::BackgroundSize => handle_background_size(ctx, &mut stream),
                KnownProperty::Border => handle_border(ctx, &mut stream),
                KnownProperty::BorderBottomColor => handle_border_bottom_color(ctx, &mut stream),
                KnownProperty::BorderBottomStyle => handle_border_bottom_style(ctx, &mut stream),
                KnownProperty::BorderBottomWidth => handle_border_bottom_width(ctx, &mut stream),
                KnownProperty::BorderColor => handle_border_color(ctx, &mut stream),
                KnownProperty::BorderLeftColor => handle_border_left_color(ctx, &mut stream),
                KnownProperty::BorderLeftStyle => handle_border_left_style(ctx, &mut stream),
                KnownProperty::BorderLeftWidth => handle_border_left_width(ctx, &mut stream),
                KnownProperty::BorderRightColor => handle_border_right_color(ctx, &mut stream),
                KnownProperty::BorderRightStyle => handle_border_right_style(ctx, &mut stream),
                KnownProperty::BorderRightWidth => handle_border_right_width(ctx, &mut stream),
                KnownProperty::BorderStyle => handle_border_style(ctx, &mut stream),
                KnownProperty::BorderTopColor => handle_border_top_color(ctx, &mut stream),
                KnownProperty::BorderTopStyle => handle_border_top_style(ctx, &mut stream),
                KnownProperty::BorderTopWidth => handle_border_top_width(ctx, &mut stream),
                KnownProperty::BorderWidth => handle_border_width(ctx, &mut stream),
                KnownProperty::Bottom => handle_bottom(ctx, &mut stream),
                KnownProperty::Clear => handle_clear(ctx, &mut stream),
                KnownProperty::Color => handle_color(ctx, &mut stream),
                KnownProperty::ColumnGap => handle_column_gap(ctx, &mut stream),
                KnownProperty::Cursor => handle_cursor(ctx, &mut stream),
                KnownProperty::Display => handle_display(ctx, &mut stream),
                KnownProperty::FlexBasis => handle_flex_basis(ctx, &mut stream),
                KnownProperty::FlexDirection => handle_flex_direction(ctx, &mut stream),
                KnownProperty::FlexFlow => handle_flex_flow(ctx, &mut stream),
                KnownProperty::FlexGrow => handle_flex_grow(ctx, &mut stream),
                KnownProperty::FlexShrink => handle_flex_shrink(ctx, &mut stream),
                KnownProperty::FlexWrap => handle_flex_wrap(ctx, &mut stream),
                KnownProperty::Float => handle_float(ctx, &mut stream),
                KnownProperty::FontFamily => handle_font_family(ctx, &mut stream),
                KnownProperty::FontSize => handle_font_size(ctx, &mut stream),
                KnownProperty::FontWeight => handle_font_weight(ctx, &mut stream),
                KnownProperty::Gap => handle_gap(ctx, &mut stream),
                KnownProperty::Height => handle_height(ctx, &mut stream),
                KnownProperty::JustifyContent => handle_justify_content(ctx, &mut stream),
                KnownProperty::JustifyItems => handle_justify_items(ctx, &mut stream),
                KnownProperty::JustifySelf => handle_justify_self(ctx, &mut stream),
                KnownProperty::Left => handle_left(ctx, &mut stream),
                KnownProperty::LineHeight => handle_line_height(ctx, &mut stream),
                KnownProperty::Margin => handle_margin(ctx, &mut stream),
                KnownProperty::MarginBlock => handle_margin_block(ctx, &mut stream),
                KnownProperty::MarginBlockEnd => handle_margin_block_end(ctx, &mut stream),
                KnownProperty::MarginBlockStart => handle_margin_block_start(ctx, &mut stream),
                KnownProperty::MarginBottom => handle_margin_bottom(ctx, &mut stream),
                KnownProperty::MarginInline => handle_margin_inline(ctx, &mut stream),
                KnownProperty::MarginInlineEnd => handle_margin_inline_end(ctx, &mut stream),
                KnownProperty::MarginInlineStart => handle_margin_inline_start(ctx, &mut stream),
                KnownProperty::MarginLeft => handle_margin_left(ctx, &mut stream),
                KnownProperty::MarginRight => handle_margin_right(ctx, &mut stream),
                KnownProperty::MarginTop => handle_margin_top(ctx, &mut stream),
                KnownProperty::MaxHeight => handle_max_height(ctx, &mut stream),
                KnownProperty::MaxWidth => handle_max_width(ctx, &mut stream),
                KnownProperty::Order => handle_order(ctx, &mut stream),
                KnownProperty::Padding => handle_padding(ctx, &mut stream),
                KnownProperty::PaddingBlock => handle_padding_block(ctx, &mut stream),
                KnownProperty::PaddingBlockEnd => handle_padding_block_end(ctx, &mut stream),
                KnownProperty::PaddingBlockStart => handle_padding_block_start(ctx, &mut stream),
                KnownProperty::PaddingBottom => handle_padding_bottom(ctx, &mut stream),
                KnownProperty::PaddingInline => handle_padding_inline(ctx, &mut stream),
                KnownProperty::PaddingInlineEnd => handle_padding_inline_end(ctx, &mut stream),
                KnownProperty::PaddingInlineStart => handle_padding_inline_start(ctx, &mut stream),
                KnownProperty::PaddingLeft => handle_padding_left(ctx, &mut stream),
                KnownProperty::PaddingRight => handle_padding_right(ctx, &mut stream),
                KnownProperty::PaddingTop => handle_padding_top(ctx, &mut stream),
                KnownProperty::Position => handle_position(ctx, &mut stream),
                KnownProperty::Right => handle_right(ctx, &mut stream),
                KnownProperty::RowGap => handle_row_gap(ctx, &mut stream),
                KnownProperty::TextAlign => handle_text_align(ctx, &mut stream),
                KnownProperty::Top => handle_top(ctx, &mut stream),
                KnownProperty::WhiteSpace => handle_whitespace(ctx, &mut stream),
                KnownProperty::Width => handle_width(ctx, &mut stream),
                KnownProperty::WritingMode => handle_writing_mode(ctx, &mut stream),
                _ => {
                    return false;
                }
            },
            Property::Custom(_) => { /* Ignore custom properties here since they are already resolved */ }
        }

        true
    }
}

impl Default for SpecifiedStyle {
    fn default() -> Self {
        Self {
            variables: Arc::new(vec![]),
            computed_font_size_px: 16.0,

            // Non-inherited properties
            align_content: CSSProperty::Global(Global::Initial),
            align_items: CSSProperty::Global(Global::Initial),
            align_self: CSSProperty::Global(Global::Initial),
            background_attachment: CSSProperty::Global(Global::Initial),
            background_blend_mode: CSSProperty::Global(Global::Initial),
            background_clip: CSSProperty::Global(Global::Initial),
            background_color: CSSProperty::Global(Global::Initial),
            background_image: CSSProperty::Global(Global::Initial),
            background_origin: CSSProperty::Global(Global::Initial),
            background_position_x: CSSProperty::Global(Global::Initial),
            background_position_y: CSSProperty::Global(Global::Initial),
            background_repeat: CSSProperty::Global(Global::Initial),
            background_size: CSSProperty::Global(Global::Initial),
            border_bottom_color: CSSProperty::Global(Global::Initial),
            border_bottom_style: CSSProperty::Global(Global::Initial),
            border_bottom_width: CSSProperty::Global(Global::Initial),
            border_left_color: CSSProperty::Global(Global::Initial),
            border_left_style: CSSProperty::Global(Global::Initial),
            border_left_width: CSSProperty::Global(Global::Initial),
            border_right_color: CSSProperty::Global(Global::Initial),
            border_right_style: CSSProperty::Global(Global::Initial),
            border_right_width: CSSProperty::Global(Global::Initial),
            border_top_color: CSSProperty::Global(Global::Initial),
            border_top_style: CSSProperty::Global(Global::Initial),
            border_top_width: CSSProperty::Global(Global::Initial),
            bottom: CSSProperty::Global(Global::Initial),
            clear: CSSProperty::Global(Global::Initial),
            column_gap: CSSProperty::Global(Global::Initial),
            display: CSSProperty::Global(Global::Initial),
            flex_basis: CSSProperty::Global(Global::Initial),
            flex_direction: CSSProperty::Global(Global::Initial),
            flex_grow: CSSProperty::Global(Global::Initial),
            flex_shrink: CSSProperty::Global(Global::Initial),
            flex_wrap: CSSProperty::Global(Global::Initial),
            float: CSSProperty::Global(Global::Initial),
            height: CSSProperty::Global(Global::Initial),
            justify_content: CSSProperty::Global(Global::Initial),
            justify_items: CSSProperty::Global(Global::Initial),
            justify_self: CSSProperty::Global(Global::Initial),
            left: CSSProperty::Global(Global::Initial),
            margin_bottom: CSSProperty::Global(Global::Initial),
            margin_left: CSSProperty::Global(Global::Initial),
            margin_right: CSSProperty::Global(Global::Initial),
            margin_top: CSSProperty::Global(Global::Initial),
            max_height: CSSProperty::Global(Global::Initial),
            max_width: CSSProperty::Global(Global::Initial),
            order: CSSProperty::Global(Global::Initial),
            padding_bottom: CSSProperty::Global(Global::Initial),
            padding_left: CSSProperty::Global(Global::Initial),
            padding_right: CSSProperty::Global(Global::Initial),
            padding_top: CSSProperty::Global(Global::Initial),
            position: CSSProperty::Global(Global::Initial),
            right: CSSProperty::Global(Global::Initial),
            row_gap: CSSProperty::Global(Global::Initial),
            top: CSSProperty::Global(Global::Initial),
            width: CSSProperty::Global(Global::Initial),

            // Inherited properties
            color: CSSProperty::Global(Global::Inherit),
            cursor: CSSProperty::Global(Global::Inherit),
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

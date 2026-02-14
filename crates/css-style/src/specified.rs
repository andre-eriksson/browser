use css_cssom::{ComponentValue, KnownProperty, Property};
use html_dom::{DocumentRoot, NodeId};

use crate::{
    BorderStyle, BorderWidth, ComputedStyle, OffsetValue, RelativeContext,
    cascade::{CascadedDeclaration, GeneratedRule, cascade, cascade_variables},
    color::named::NamedColor,
    handler::{
        PropertyUpdateContext, handle_background_color, handle_border, handle_border_bottom_color,
        handle_border_bottom_style, handle_border_bottom_width, handle_border_left_color,
        handle_border_left_style, handle_border_left_width, handle_border_right_color,
        handle_border_right_style, handle_border_right_width, handle_border_top_color,
        handle_border_top_style, handle_border_top_width, handle_color, handle_display,
        handle_font_family, handle_font_size, handle_font_weight, handle_height,
        handle_line_height, handle_margin, handle_margin_block, handle_margin_block_end,
        handle_margin_block_start, handle_margin_bottom, handle_margin_left, handle_margin_right,
        handle_margin_top, handle_max_height, handle_max_width, handle_padding,
        handle_padding_block, handle_padding_block_end, handle_padding_block_start,
        handle_padding_bottom, handle_padding_left, handle_padding_right, handle_padding_top,
        handle_position, handle_text_align, handle_whitespace, handle_width, handle_writing_mode,
        resolve_css_variables,
    },
    length::Length,
    primitives::{
        display::{InsideDisplay, OutsideDisplay},
        font::{AbsoluteSize, GenericName},
    },
    properties::{
        AbsoluteContext, BorderStyleValueProperty, BorderWidthValueProperty, CSSProperty,
        ColorProperty, DisplayProperty, FontFamilyProperty, FontSizeProperty, FontWeightProperty,
        HeightProperty, LineHeightProperty, MaxHeightProperty, MaxWidthProperty,
        OffsetValueProperty, PositionProperty, TextAlignProperty, WhitespaceProperty,
        WidthProperty, WritingModeProperty,
        color::Color,
        dimension::{Dimension, MaxDimension},
        display::Display,
        font::{FontFamily, FontFamilyName, FontSize, FontWeight},
        position::Position,
        text::{LineHeight, TextAlign, Whitespace, WritingMode},
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
    pub variables: Vec<(Property, Vec<ComponentValue>)>,
}

impl SpecifiedStyle {
    /// Computes the ComputedStyle for a given node in the DOM.
    pub fn from_node(
        absolute_ctx: &AbsoluteContext,
        relative_ctx: &RelativeContext,
        node_id: &NodeId,
        dom: &DocumentRoot,
        rules: &[GeneratedRule],
        parent_style: Option<&ComputedStyle>,
    ) -> Self {
        let mut specified_style = match parent_style {
            Some(style) => SpecifiedStyle::from(style.inherited_subset()),
            None => SpecifiedStyle::default(),
        };

        let node = match dom.get_node(node_id) {
            Some(n) => n,
            None => return specified_style,
        };

        let (declarations, variables) = &mut CascadedDeclaration::collect(node, dom, rules);

        let properties = cascade(declarations);
        let mut merged_variables = specified_style.variables.clone();
        for (name, value) in cascade_variables(variables) {
            if let Some(existing) = merged_variables.iter_mut().find(|(n, _)| n == &name) {
                existing.1 = value;
            } else {
                merged_variables.push((name, value));
            }
        }
        specified_style.variables = merged_variables;

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
                    KnownProperty::Position => handle_position(&mut ctx, v),
                    KnownProperty::TextAlign => handle_text_align(&mut ctx, v),
                    KnownProperty::WhiteSpace => handle_whitespace(&mut ctx, v),
                    KnownProperty::Width => handle_width(&mut ctx, v),
                    KnownProperty::WritingMode => handle_writing_mode(&mut ctx, v),
                    _ => {}
                },
                Property::Custom(_) => { /* Ignore custom properties here since they are already resolved */
                }
            }
        }

        ctx.log_errors();

        specified_style
    }
}

impl Default for SpecifiedStyle {
    fn default() -> Self {
        let black = Color::Named(NamedColor::Black);

        SpecifiedStyle {
            variables: Vec::with_capacity(32),
            background_color: CSSProperty::from(Color::Transparent),
            border_bottom_color: CSSProperty::from(black),
            border_left_color: CSSProperty::from(black),
            border_right_color: CSSProperty::from(black),
            border_top_color: CSSProperty::from(black),
            border_top_style: CSSProperty::from(BorderStyle::None),
            border_right_style: CSSProperty::from(BorderStyle::None),
            border_bottom_style: CSSProperty::from(BorderStyle::None),
            border_left_style: CSSProperty::from(BorderStyle::None),
            border_top_width: CSSProperty::from(BorderWidth::Length(Length::zero())),
            border_right_width: CSSProperty::from(BorderWidth::Length(Length::zero())),
            border_bottom_width: CSSProperty::from(BorderWidth::Length(Length::zero())),
            border_left_width: CSSProperty::from(BorderWidth::Length(Length::zero())),
            color: CSSProperty::from(black),
            display: CSSProperty::from(Display::new(
                Some(OutsideDisplay::Inline),
                Some(InsideDisplay::Flow),
                None,
                None,
                None,
            )),
            font_family: CSSProperty::from(FontFamily::new(&[FontFamilyName::Generic(
                GenericName::Serif,
            )])),
            font_size: CSSProperty::from(FontSize::Absolute(AbsoluteSize::Medium)),
            font_weight: CSSProperty::from(FontWeight::Normal),
            computed_font_size_px: AbsoluteSize::Medium.to_px(),
            height: CSSProperty::from(Dimension::Auto),
            max_height: CSSProperty::from(MaxDimension::None),
            line_height: CSSProperty::from(LineHeight::Normal),
            margin_top: CSSProperty::from(OffsetValue::zero()),
            margin_right: CSSProperty::from(OffsetValue::zero()),
            margin_bottom: CSSProperty::from(OffsetValue::zero()),
            margin_left: CSSProperty::from(OffsetValue::zero()),
            padding_top: CSSProperty::from(OffsetValue::zero()),
            padding_right: CSSProperty::from(OffsetValue::zero()),
            padding_bottom: CSSProperty::from(OffsetValue::zero()),
            padding_left: CSSProperty::from(OffsetValue::zero()),
            position: CSSProperty::from(Position::Static),
            text_align: CSSProperty::from(TextAlign::Left),
            whitespace: CSSProperty::from(Whitespace::Normal),
            width: CSSProperty::from(Dimension::Auto),
            max_width: CSSProperty::from(MaxDimension::None),
            writing_mode: CSSProperty::from(WritingMode::HorizontalTb),
        }
    }
}

impl From<ComputedStyle> for SpecifiedStyle {
    fn from(value: ComputedStyle) -> Self {
        Self {
            background_color: CSSProperty::from(Color::from(value.background_color)),
            border_top_color: CSSProperty::from(Color::from(value.border_top_color)),
            border_right_color: CSSProperty::from(Color::from(value.border_right_color)),
            border_bottom_color: CSSProperty::from(Color::from(value.border_bottom_color)),
            border_left_color: CSSProperty::from(Color::from(value.border_left_color)),
            border_top_style: CSSProperty::from(value.border_top_style),
            border_right_style: CSSProperty::from(value.border_right_style),
            border_bottom_style: CSSProperty::from(value.border_bottom_style),
            border_left_style: CSSProperty::from(value.border_left_style),
            border_top_width: CSSProperty::from(BorderWidth::px(value.border_top_width)),
            border_right_width: CSSProperty::from(BorderWidth::px(value.border_right_width)),
            border_bottom_width: CSSProperty::from(BorderWidth::px(value.border_bottom_width)),
            border_left_width: CSSProperty::from(BorderWidth::px(value.border_left_width)),
            color: CSSProperty::from(Color::from(value.color)),
            display: CSSProperty::from(value.display),
            font_family: CSSProperty::from(value.font_family),
            font_size: CSSProperty::from(FontSize::px(value.font_size)),
            font_weight: CSSProperty::from(
                FontWeight::try_from(value.font_weight).unwrap_or(FontWeight::Normal),
            ),
            height: CSSProperty::from(value.height),
            max_height: CSSProperty::from(value.max_height),
            line_height: CSSProperty::from(value.line_height),
            margin_top: CSSProperty::from(value.margin_top),
            margin_right: CSSProperty::from(value.margin_right),
            margin_bottom: CSSProperty::from(value.margin_bottom),
            margin_left: CSSProperty::from(value.margin_left),
            padding_top: CSSProperty::from(value.padding_top),
            padding_right: CSSProperty::from(value.padding_right),
            padding_bottom: CSSProperty::from(value.padding_bottom),
            padding_left: CSSProperty::from(value.padding_left),
            position: CSSProperty::from(value.position),
            text_align: CSSProperty::from(value.text_align),
            whitespace: CSSProperty::from(value.whitespace),
            width: CSSProperty::from(value.width),
            max_width: CSSProperty::from(value.max_width),
            writing_mode: CSSProperty::from(value.writing_mode),
            computed_font_size_px: 16.0, // TODO: Solve
            variables: value.variables.clone(),
        }
    }
}

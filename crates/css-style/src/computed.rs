use css_cssom::{KnownProperty, Property};
use html_dom::{DocumentRoot, NodeId};

use crate::{
    BorderStyleValue, BorderWidthValue, OffsetValue,
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
        resolve_css_variable,
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

#[derive(Debug, Clone)]
pub struct ComputedStyle {
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
    pub variables: Vec<(Property, String)>,
}

impl ComputedStyle {
    /// Computes the ComputedStyle for a given node in the DOM.
    ///
    /// # Arguments
    /// * `node_id` - The NodeId of the DOM node to compute the style for.
    /// * `dom` - The DocumentRoot representing the DOM tree.
    /// * `rules` - A slice of GeneratedRule representing the CSS rules to apply.
    /// * `parent_style` - An optional reference to the ComputedStyle of the parent node for inheritance.
    pub fn from_node(
        absolute_ctx: &AbsoluteContext,
        node_id: &NodeId,
        dom: &DocumentRoot,
        rules: &[GeneratedRule],
        parent_style: Option<&ComputedStyle>,
    ) -> Self {
        let mut computed_style = match parent_style {
            Some(style) => style.inherited_subset(),
            None => ComputedStyle::default(),
        };

        let node = match dom.get_node(node_id) {
            Some(n) => n,
            None => return computed_style,
        };

        let (declarations, variables) = &mut CascadedDeclaration::collect(node, dom, rules);

        let properties = cascade(declarations);
        let mut merged_variables = computed_style.variables.clone();
        for (name, value) in cascade_variables(variables) {
            if let Some(existing) = merged_variables.iter_mut().find(|(n, _)| n == &name) {
                existing.1 = value;
            } else {
                merged_variables.push((name, value));
            }
        }
        computed_style.variables = merged_variables;

        let mut ctx = PropertyUpdateContext::new(absolute_ctx, &mut computed_style, parent_style);

        for (key, value) in properties {
            let val = resolve_css_variable(&ctx.computed_style.variables, value.clone(), value);
            let v = val.as_str();

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

        computed_style
    }

    /// Returns a subset of the ComputedStyle containing only inherited properties.
    pub fn inherited_subset(&self) -> Self {
        ComputedStyle {
            color: self.color,
            font_family: self.font_family.clone(),
            font_size: self.font_size.clone(),
            computed_font_size_px: self.computed_font_size_px,
            line_height: self.line_height.clone(),
            text_align: self.text_align,
            font_weight: self.font_weight,
            whitespace: self.whitespace,
            writing_mode: self.writing_mode,
            variables: self.variables.clone(),
            ..ComputedStyle::default()
        }
    }

    pub fn set_margin_all(&mut self, value: OffsetValue) {
        self.margin_top = value.clone().into();
        self.margin_right = value.clone().into();
        self.margin_bottom = value.clone().into();
        self.margin_left = value.into();
    }

    pub fn set_padding_all(&mut self, value: OffsetValue) {
        self.padding_top = value.clone().into();
        self.padding_right = value.clone().into();
        self.padding_bottom = value.clone().into();
        self.padding_left = value.into();
    }
}

impl Default for ComputedStyle {
    fn default() -> Self {
        let black = Color::Named(NamedColor::Black);

        ComputedStyle {
            variables: Vec::with_capacity(32),
            background_color: CSSProperty::from(Color::Transparent),
            border_bottom_color: CSSProperty::from(black),
            border_left_color: CSSProperty::from(black),
            border_right_color: CSSProperty::from(black),
            border_top_color: CSSProperty::from(black),
            border_top_style: CSSProperty::from(BorderStyleValue::None),
            border_right_style: CSSProperty::from(BorderStyleValue::None),
            border_bottom_style: CSSProperty::from(BorderStyleValue::None),
            border_left_style: CSSProperty::from(BorderStyleValue::None),
            border_top_width: CSSProperty::from(BorderWidthValue::Length(Length::zero())),
            border_right_width: CSSProperty::from(BorderWidthValue::Length(Length::zero())),
            border_bottom_width: CSSProperty::from(BorderWidthValue::Length(Length::zero())),
            border_left_width: CSSProperty::from(BorderWidthValue::Length(Length::zero())),
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

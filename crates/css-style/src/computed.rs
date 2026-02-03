use html_dom::{DocumentRoot, NodeId};

use crate::{
    cascade::{CascadedDeclaration, GeneratedRule, cascade, cascade_variables},
    handler::{
        PropertyUpdateContext, handle_background_color, handle_border, handle_border_color,
        handle_border_style, handle_border_width, handle_color, handle_display, handle_font_family,
        handle_font_size, handle_font_weight, handle_height, handle_line_height, handle_margin,
        handle_margin_block, handle_margin_block_end, handle_margin_block_start,
        handle_margin_bottom, handle_margin_left, handle_margin_right, handle_margin_top,
        handle_max_height, handle_max_width, handle_padding, handle_padding_block,
        handle_padding_block_end, handle_padding_block_start, handle_padding_bottom,
        handle_padding_left, handle_padding_right, handle_padding_top, handle_position,
        handle_text_align, handle_whitespace, handle_width, handle_writing_mode,
        resolve_css_variable,
    },
    primitives::{
        color::NamedColor,
        display::{InsideDisplay, OutsideDisplay},
        font::{AbsoluteSize, GenericName},
    },
    properties::{
        BorderColorProperty, BorderProperty, BorderStyleProperty, BorderWidthProperty,
        ColorProperty, DisplayProperty, FontFamilyProperty, FontSizeProperty, FontWeightProperty,
        HeightProperty, LineHeightProperty, MaxHeightProperty, MaxWidthProperty, OffsetProperty,
        PositionProperty, Property, TextAlignProperty, WhitespaceProperty, WidthProperty,
        WritingModeProperty,
        border::{Border, BorderColor, BorderStyle, BorderWidth},
        color::Color,
        dimension::{Dimension, MaxDimension},
        display::Display,
        font::{FontFamily, FontFamilyName, FontSize, FontWeight},
        offset::Offset,
        position::Position,
        text::{LineHeight, TextAlign, Whitespace, WritingMode},
    },
};

#[derive(Debug, Clone)]
pub struct ComputedStyle {
    pub background_color: ColorProperty,
    pub border: BorderProperty,
    pub border_color: BorderColorProperty,
    pub border_style: BorderStyleProperty,
    pub border_width: BorderWidthProperty,
    pub color: ColorProperty,
    pub display: DisplayProperty,
    pub font_family: FontFamilyProperty,
    pub font_size: FontSizeProperty,
    pub font_weight: FontWeightProperty,
    pub height: HeightProperty,
    pub max_height: MaxHeightProperty,
    pub line_height: LineHeightProperty,
    pub margin: OffsetProperty,
    pub padding: OffsetProperty,
    pub position: PositionProperty,
    pub text_align: TextAlignProperty,
    pub whitespace: WhitespaceProperty,
    pub width: WidthProperty,
    pub max_width: MaxWidthProperty,
    pub writing_mode: WritingModeProperty,

    // === Non-CSS properties ===
    pub computed_font_size_px: f32,
    pub variables: Vec<(String, String)>,
}

impl ComputedStyle {
    /// Computes the ComputedStyle for a given node in the DOM.
    ///
    /// # Arguments
    /// * `node_id` - The NodeId of the DOM node to compute the style for.
    /// * `dom` - The DocumentRoot representing the DOM tree.
    /// * `rules` - A slice of GeneratedRule representing the CSS rules to apply.
    /// * `parent_style` - An optional reference to the ComputedStyle of the parent node for inheritance.
    #[allow(clippy::single_match)]
    pub fn from_node(
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

        let mut ctx = PropertyUpdateContext::new(&mut computed_style, parent_style);

        for (key, value) in properties {
            let val = resolve_css_variable(&ctx.computed_style.variables, value);
            let v = val.as_str();

            match key.as_str() {
                "background-color" => handle_background_color(&mut ctx, v),
                "border" => handle_border(&mut ctx, v),
                "border-color" => handle_border_color(&mut ctx, v),
                "border-style" => handle_border_style(&mut ctx, v),
                "border-width" => handle_border_width(&mut ctx, v),
                "color" => handle_color(&mut ctx, v),
                "display" => handle_display(&mut ctx, v),
                "font-family" => handle_font_family(&mut ctx, v),
                "font-size" => handle_font_size(&mut ctx, v),
                "font-weight" => handle_font_weight(&mut ctx, v),
                "height" => handle_height(&mut ctx, v),
                "max-height" => handle_max_height(&mut ctx, v),
                "line-height" => handle_line_height(&mut ctx, v),
                "margin" => handle_margin(&mut ctx, v),
                "margin-top" => handle_margin_top(&mut ctx, v),
                "margin-right" => handle_margin_right(&mut ctx, v),
                "margin-bottom" => handle_margin_bottom(&mut ctx, v),
                "margin-left" => handle_margin_left(&mut ctx, v),
                "margin-block" => handle_margin_block(&mut ctx, v),
                "margin-block-start" => handle_margin_block_start(&mut ctx, v),
                "margin-block-end" => handle_margin_block_end(&mut ctx, v),
                "padding" => handle_padding(&mut ctx, v),
                "padding-top" => handle_padding_top(&mut ctx, v),
                "padding-right" => handle_padding_right(&mut ctx, v),
                "padding-bottom" => handle_padding_bottom(&mut ctx, v),
                "padding-left" => handle_padding_left(&mut ctx, v),
                "padding-block" => handle_padding_block(&mut ctx, v),
                "padding-block-start" => handle_padding_block_start(&mut ctx, v),
                "padding-block-end" => handle_padding_block_end(&mut ctx, v),
                "position" => handle_position(&mut ctx, v),
                "text-align" => handle_text_align(&mut ctx, v),
                "white-space" => handle_whitespace(&mut ctx, v),
                "width" => handle_width(&mut ctx, v),
                "max-width" => handle_max_width(&mut ctx, v),
                "writing-mode" => handle_writing_mode(&mut ctx, v),
                _ => {}
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
            font_size: self.font_size,
            computed_font_size_px: self.computed_font_size_px,
            line_height: self.line_height,
            text_align: self.text_align,
            font_weight: self.font_weight,
            whitespace: self.whitespace,
            writing_mode: self.writing_mode,
            variables: self.variables.clone(),
            ..ComputedStyle::default()
        }
    }
}

impl Default for ComputedStyle {
    fn default() -> Self {
        let black = Color::Named(NamedColor::Black);

        ComputedStyle {
            variables: Vec::with_capacity(32),
            background_color: Property::from(Color::Transparent),
            border: Property::from(Border::none()),
            border_color: Property::from(BorderColor::all(black)),
            border_style: Property::from(BorderStyle::none()),
            border_width: Property::from(BorderWidth::zero()),
            color: Property::from(black),
            display: Property::from(Display::new(
                Some(OutsideDisplay::Inline),
                Some(InsideDisplay::Flow),
                None,
                None,
            )),
            font_family: Property::from(FontFamily::new(&[FontFamilyName::Generic(
                GenericName::Serif,
            )])),
            font_size: Property::from(FontSize::Absolute(AbsoluteSize::Medium)),
            font_weight: Property::from(FontWeight::Normal),
            computed_font_size_px: AbsoluteSize::Medium.to_px(),
            height: Property::from(Dimension::Auto),
            max_height: Property::from(MaxDimension::None),
            line_height: Property::from(LineHeight::Normal),
            margin: Property::from(Offset::zero()),
            padding: Property::from(Offset::zero()),
            position: Property::from(Position::Static),
            text_align: Property::from(TextAlign::Left),
            whitespace: Property::from(Whitespace::Normal),
            width: Property::from(Dimension::Auto),
            max_width: Property::from(MaxDimension::None),
            writing_mode: Property::from(WritingMode::HorizontalTb),
        }
    }
}

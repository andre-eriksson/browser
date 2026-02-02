use html_dom::{DocumentRoot, NodeId};

use crate::{
    cascade::{CascadedDeclaration, GeneratedRule, cascade, cascade_variables},
    types::{
        Parseable,
        border::Border,
        color::{Color, NamedColor},
        display::{Display, InsideDisplay, OutsideDisplay},
        font::{AbsoluteSize, FontFamily, FontFamilyName, FontSize, FontWeight, GenericName},
        height::Height,
        line_height::LineHeight,
        margin::{Margin, MarginValue},
        padding::{Padding, PaddingValue},
        position::Position,
        text_align::TextAlign,
        whitespace::Whitespace,
        width::{MaxWidth, Width},
        writing_mode::WritingMode,
    },
};

#[derive(Debug, Clone)]
pub struct ComputedStyle {
    pub background_color: Color,
    pub border: Border,
    pub color: Color,
    pub display: Display,
    pub font_family: FontFamily,
    pub font_size: FontSize,
    pub font_weight: FontWeight,
    pub height: Height,
    pub line_height: LineHeight,
    pub margin: Margin,
    pub padding: Padding,
    pub position: Position,
    pub text_align: TextAlign,
    pub whitespace: Whitespace,
    pub width: Width,
    pub max_width: MaxWidth,
    pub writing_mode: WritingMode,

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

        for (key, value) in properties {
            let mut v = value.as_str();

            if v.starts_with("var")
                && let Some(start) = v.find('(')
                && let Some(end) = v.rfind(')')
            {
                let var_name = v[start + 1..end].trim();
                if let Some((_, var_value)) = computed_style
                    .variables
                    .iter()
                    .find(|(name, _)| name == var_name)
                {
                    v = var_value.as_str();
                }
            }

            match key.as_str() {
                "background" | "background-color" => {
                    if let Some(color) = Color::parse(v) {
                        computed_style.background_color = color;
                    }
                }
                "border" => {
                    if let Some(border) = Border::parse(v) {
                        computed_style.border = border;
                    }
                }
                "color" => {
                    if let Some(color) = Color::parse(v) {
                        computed_style.color = color;
                    }
                }
                "display" => {
                    if let Some(display) = Display::parse(v) {
                        computed_style.display = display;
                    }
                }
                "font-family" => {
                    if let Some(font_family) = FontFamily::parse(v) {
                        computed_style.font_family = font_family;
                    }
                }
                "font-size" => {
                    if let Some(font_size) = FontSize::parse(v) {
                        let parent_px = parent_style
                            .map(|p| p.computed_font_size_px)
                            .unwrap_or(AbsoluteSize::Medium.to_px());
                        computed_style.computed_font_size_px = font_size.to_px(parent_px);
                        computed_style.font_size = font_size;
                    }
                }
                "font-weight" => {
                    if let Some(font_weight) = FontWeight::parse(v) {
                        computed_style.font_weight = font_weight;
                    }
                }
                "height" => {
                    if let Some(height) = Height::parse(v) {
                        computed_style.height = height;
                    }
                }
                "line-height" => {
                    if let Some(line_height) = LineHeight::parse(v) {
                        computed_style.line_height = line_height;
                    }
                }
                "margin" => {
                    if let Some(margin) = Margin::parse(v) {
                        computed_style.margin = margin;
                    }
                }
                "margin-top" => {
                    if let Some(margin_value) = MarginValue::parse(v) {
                        computed_style.margin.top = margin_value;
                    }
                }
                "margin-right" => {
                    if let Some(margin_value) = MarginValue::parse(v) {
                        computed_style.margin.right = margin_value;
                    }
                }
                "margin-bottom" => {
                    if let Some(margin_value) = MarginValue::parse(v) {
                        computed_style.margin.bottom = margin_value;
                    }
                }
                "margin-left" => {
                    if let Some(margin_value) = MarginValue::parse(v) {
                        computed_style.margin.left = margin_value;
                    }
                }
                "margin-block" => {
                    if let Some(margin) = Margin::parse(v) {
                        match computed_style.writing_mode {
                            WritingMode::HorizontalTb => {
                                computed_style.margin.top = margin.top;
                                computed_style.margin.bottom = margin.bottom;
                            }
                            WritingMode::VerticalRl | WritingMode::VerticalLr => {
                                computed_style.margin.left = margin.left;
                                computed_style.margin.right = margin.right;
                            }
                            _ => {}
                        }
                    }
                }
                "margin-block-start" => {
                    if let Some(margin_value) = MarginValue::parse(v) {
                        match computed_style.writing_mode {
                            WritingMode::HorizontalTb => {
                                computed_style.margin.top = margin_value;
                            }
                            WritingMode::VerticalRl | WritingMode::VerticalLr => {
                                computed_style.margin.right = margin_value;
                            }
                            _ => {}
                        }
                    }
                }
                "margin-block-end" => {
                    if let Some(margin_value) = MarginValue::parse(v) {
                        match computed_style.writing_mode {
                            WritingMode::HorizontalTb => {
                                computed_style.margin.bottom = margin_value;
                            }
                            WritingMode::VerticalRl | WritingMode::VerticalLr => {
                                computed_style.margin.left = margin_value;
                            }
                            _ => {}
                        }
                    }
                }

                "padding" => {
                    if let Some(padding) = Padding::parse(v) {
                        computed_style.padding = padding;
                    }
                }
                "padding-top" => {
                    if let Some(padding_value) = PaddingValue::parse(v) {
                        computed_style.padding.top = padding_value;
                    }
                }
                "padding-right" => {
                    if let Some(padding_value) = PaddingValue::parse(v) {
                        computed_style.padding.right = padding_value;
                    }
                }
                "padding-bottom" => {
                    if let Some(padding_value) = PaddingValue::parse(v) {
                        computed_style.padding.bottom = padding_value;
                    }
                }
                "padding-left" => {
                    if let Some(padding_value) = PaddingValue::parse(v) {
                        computed_style.padding.left = padding_value;
                    }
                }
                "padding-block" => {
                    if let Some(padding) = Padding::parse(v) {
                        match computed_style.writing_mode {
                            WritingMode::HorizontalTb => {
                                computed_style.padding.top = padding.top;
                                computed_style.padding.bottom = padding.bottom;
                            }
                            WritingMode::VerticalRl | WritingMode::VerticalLr => {
                                computed_style.padding.left = padding.left;
                                computed_style.padding.right = padding.right;
                            }
                            _ => {}
                        }
                    }
                }
                "padding-block-start" => {
                    if let Some(padding_value) = PaddingValue::parse(v) {
                        match computed_style.writing_mode {
                            WritingMode::HorizontalTb => {
                                computed_style.padding.top = padding_value;
                            }
                            WritingMode::VerticalRl | WritingMode::VerticalLr => {
                                computed_style.padding.right = padding_value;
                            }
                            _ => {}
                        }
                    }
                }
                "padding-block-end" => {
                    if let Some(padding_value) = PaddingValue::parse(v) {
                        match computed_style.writing_mode {
                            WritingMode::HorizontalTb => {
                                computed_style.padding.bottom = padding_value;
                            }
                            WritingMode::VerticalRl | WritingMode::VerticalLr => {
                                computed_style.padding.left = padding_value;
                            }
                            _ => {}
                        }
                    }
                }

                "position" => {
                    if let Some(position) = Position::parse(v) {
                        computed_style.position = position;
                    }
                }
                "text-align" => {
                    if let Some(text_align) = TextAlign::parse(v) {
                        computed_style.text_align = text_align;
                    }
                }
                "white-space" => {
                    if let Some(whitespace) = Whitespace::parse(v) {
                        computed_style.whitespace = whitespace;
                    }
                }
                "width" => {
                    if let Some(width) = Width::parse(v) {
                        computed_style.width = width;
                    }
                }
                "max-width" => {
                    if let Some(max_width) = MaxWidth::parse(v) {
                        computed_style.max_width = max_width;
                    }
                }
                "writing-mode" => {
                    if let Some(writing_mode) = WritingMode::parse(v) {
                        computed_style.writing_mode = writing_mode;
                    }
                }
                _ => {}
            }
        }

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
        ComputedStyle {
            variables: Vec::with_capacity(32),
            background_color: Color::Transparent,
            border: Border::none(),
            color: Color::Named(NamedColor::Black),
            display: Display {
                outside: Some(OutsideDisplay::Inline),
                inside: Some(InsideDisplay::Flow),
                internal: None,
                box_display: None,
                global: None,
            },
            font_family: FontFamily {
                names: vec![FontFamilyName::Generic(GenericName::Serif)],
            },
            font_size: FontSize::Absolute(AbsoluteSize::Medium),
            font_weight: FontWeight::Normal,
            computed_font_size_px: AbsoluteSize::Medium.to_px(),
            height: Height::Auto,
            line_height: LineHeight::Normal,
            margin: Margin::zero(),
            padding: Padding::zero(),
            position: Position::Static,
            text_align: TextAlign::Left,
            whitespace: Whitespace::Normal,
            width: Width::Auto,
            max_width: MaxWidth::None,
            writing_mode: WritingMode::HorizontalTb,
        }
    }
}

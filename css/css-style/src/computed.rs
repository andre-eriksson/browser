use css_cssom::CSSStyleSheet;
use html_dom::{DocumentRoot, NodeId};

use crate::{
    cascade::{cascade, collect_declarations},
    resolver::PropertyResolver,
    types::{
        border::Border,
        color::{Color, NamedColor},
        display::{Display, InsideDisplay, OutsideDisplay},
        font::{AbsoluteSize, FontFamily, FontFamilyName, FontSize, GenericName},
        height::Height,
        line_height::LineHeight,
        margin::Margin,
        padding::Padding,
        position::Position,
        text_align::TextAlign,
        width::Width,
    },
};

#[derive(Clone, Debug)]
pub struct ComputedStyle {
    pub background_color: Color,
    pub border: Border,
    pub color: Color,
    pub display: Display,
    pub font_family: FontFamily,
    pub font_size: FontSize,
    pub computed_font_size_px: f32,
    pub height: Height,
    pub line_height: LineHeight,
    pub margin: Margin,
    pub padding: Padding,
    pub position: Position,
    pub text_align: TextAlign,
    pub width: Width,
}

impl ComputedStyle {
    /// Computes the ComputedStyle for a given node in the DOM.
    ///
    /// # Arguments
    /// * `node_id` - The NodeId of the DOM node to compute the style for.
    /// * `dom` - The DocumentRoot representing the DOM tree.
    /// * `stylesheets` - A slice of CSSStyleSheet references to apply styles from.
    /// * `parent_style` - An optional reference to the ComputedStyle of the parent node for inheritance.
    pub fn from_node(
        node_id: &NodeId,
        dom: &DocumentRoot,
        stylesheets: &[CSSStyleSheet],
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

        let delcarations = &mut collect_declarations(node, dom, stylesheets);
        let properties = cascade(delcarations);

        for (key, value) in properties {
            let v = value.as_str();
            match key.as_str() {
                "background" | "background-color" => {
                    if let Some(color) = PropertyResolver::resolve_color(v) {
                        computed_style.background_color = color;
                    }
                }
                "border" => {
                    if let Some(border) = PropertyResolver::resolve_border(v) {
                        computed_style.border = border;
                    }
                }
                "color" => {
                    if let Some(color) = PropertyResolver::resolve_color(v) {
                        computed_style.color = color;
                    }
                }
                "display" => {
                    if let Some(display) = PropertyResolver::resolve_display(v) {
                        computed_style.display = display;
                    }
                }
                "font-family" => {
                    if let Some(font_family) = PropertyResolver::resolve_font_family(v) {
                        computed_style.font_family = font_family;
                    }
                }
                "font-size" => {
                    if let Some(font_size) = PropertyResolver::resolve_font_size(v) {
                        let parent_px = parent_style
                            .map(|p| p.computed_font_size_px)
                            .unwrap_or(16.0);
                        computed_style.computed_font_size_px = font_size.to_px(parent_px);
                        computed_style.font_size = font_size;
                    }
                }
                "height" => {
                    if let Some(height) = PropertyResolver::resolve_height(v) {
                        computed_style.height = height;
                    }
                }
                "line-height" => {
                    if let Some(line_height) = PropertyResolver::resolve_line_height(v) {
                        computed_style.line_height = line_height;
                    }
                }
                "margin" => {
                    if let Some(margin) = PropertyResolver::resolve_margin(v) {
                        computed_style.margin = margin;
                    }
                }
                "padding" => {
                    if let Some(padding) = PropertyResolver::resolve_padding(v) {
                        computed_style.padding = padding;
                    }
                }
                "position" => {
                    if let Some(position) = PropertyResolver::resolve_position(v) {
                        computed_style.position = position;
                    }
                }
                "text-align" => {
                    if let Some(text_align) = PropertyResolver::resolve_text_align(v) {
                        computed_style.text_align = text_align;
                    }
                }
                "width" => {
                    if let Some(width) = PropertyResolver::resolve_width(v) {
                        computed_style.width = width;
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
            color: self.color.clone(),
            font_family: self.font_family.clone(),
            font_size: self.font_size.clone(),
            computed_font_size_px: self.computed_font_size_px,
            line_height: self.line_height.clone(),
            text_align: self.text_align.clone(),
            ..ComputedStyle::default()
        }
    }
}

impl Default for ComputedStyle {
    fn default() -> Self {
        ComputedStyle {
            background_color: Color::Named(NamedColor::Transparent),
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
            computed_font_size_px: 16.0,
            height: Height::Auto,
            line_height: LineHeight::Normal,
            margin: Margin::zero(),
            padding: Padding::zero(),
            position: Position::Static,
            text_align: TextAlign::Left,
            width: Width::Auto,
        }
    }
}

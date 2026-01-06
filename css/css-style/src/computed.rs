use crate::types::{
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
};

#[derive(Clone, Debug)]
pub struct ComputedStyle {
    pub background_color: Color,
    pub border: Border,
    pub color: Color,
    pub display: Display,
    pub font_family: FontFamily,
    pub font_size: FontSize,
    pub height: Height,
    pub line_height: LineHeight,
    pub margin: Margin,
    pub padding: Padding,
    pub position: Position,
    pub text_align: TextAlign,
    pub width: Width,
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

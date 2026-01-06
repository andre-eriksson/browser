use crate::types::{
    border::Border,
    color::Color,
    display::Display,
    font::{FontFamily, FontSize},
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

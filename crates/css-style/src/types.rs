pub mod angle;
pub mod border;
pub mod color;
pub mod display;
pub mod font;
pub mod global;
pub mod height;
pub mod length;
pub mod line_height;
pub mod margin;
pub mod padding;
pub mod position;
pub mod text_align;
pub mod whitespace;
pub mod width;
pub mod writing_mode;

/// A trait for types that can be parsed from a string representation.
pub trait Parseable: Sized {
    fn parse(value: &str) -> Option<Self>;
}

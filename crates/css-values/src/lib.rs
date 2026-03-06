use css_cssom::ComponentValueStream;

pub mod background;
pub mod border;
pub mod calc;
pub mod color;
pub mod combination;
pub mod dimension;
pub mod display;
pub mod global;
pub mod image;
pub mod numeric;
pub mod position;
pub mod quantity;
pub mod text;
//TODO: pub mod shape;

/// Trait for CSS value types that can be parsed from a `ComponentValueStream`.
///
/// This is the primary parsing interface for the css-style crate. Types that
/// implement this trait can be used as the inner value of `CSSProperty<T>`.
///
/// The trait provides a default `try_parse` method that constructs a stream
/// from a `&[ComponentValue]` slice and delegates to `parse`, so call sites
/// that still operate on slices (e.g. `update_property`) continue to work.
pub trait CSSParsable: Sized {
    /// Parse a value from a `ComponentValueStream`.
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, String>;
}


pub struct ComputedBackgroundPositionX {}

#[derive(Debug, Clone, PartialEq)]
pub enum ComputedSize {
    Contain,
    Cover,
    Auto,
    Fixed(f32),
    Percentage(f32),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ComputedBackgroundSize {
    pub sizes: Vec<ComputedSize>,
}

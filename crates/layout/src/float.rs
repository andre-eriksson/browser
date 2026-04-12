use css_values::{
    display::{Clear, Float},
    text::WritingMode,
};

use crate::Rect;

#[derive(Debug, Clone)]
struct FloatBox {
    rect: Rect,
}

#[derive(Debug, Clone, Default)]
pub struct FloatContext {
    left_floats: Vec<FloatBox>,
    right_floats: Vec<FloatBox>,
}

impl FloatContext {
    pub const fn new() -> Self {
        Self {
            left_floats: Vec::new(),
            right_floats: Vec::new(),
        }
    }

    /// Add a float to the context. The rect should already have clearance applied
    /// (i.e., the y position should be the final cleared position).
    pub fn add_float(&mut self, rect: Rect, writing_mode: WritingMode, float: Float) {
        let float_box = FloatBox { rect };
        match float {
            Float::Left => self.left_floats.push(float_box),
            Float::Right => self.right_floats.push(float_box),
            Float::InlineEnd => match writing_mode {
                WritingMode::HorizontalTb | WritingMode::SidewaysRl | WritingMode::VerticalRl => {
                    self.right_floats.push(float_box)
                }
                WritingMode::SidewaysLr | WritingMode::VerticalLr => self.left_floats.push(float_box),
            },
            Float::InlineStart => match writing_mode {
                WritingMode::HorizontalTb | WritingMode::SidewaysRl | WritingMode::VerticalRl => {
                    self.left_floats.push(float_box)
                }
                WritingMode::SidewaysLr | WritingMode::VerticalLr => self.right_floats.push(float_box),
            },
            Float::None => { /* No float to add */ }
        }
    }

    pub fn available_width_at(&self, y: f32, container_width: f32) -> (f32, f32) {
        let left_offset = self
            .left_floats
            .iter()
            .filter(|f| f.rect.y <= y && y < f.rect.y + f.rect.height)
            .map(|f| f.rect.x + f.rect.width)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);

        let right_offset = self
            .right_floats
            .iter()
            .filter(|f| f.rect.y <= y && y < f.rect.y + f.rect.height)
            .map(|f| container_width - f.rect.x)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);

        (left_offset, container_width - right_offset)
    }

    pub fn clear_y(&self, clear: Clear, writing_mode: WritingMode, current_y: f32) -> f32 {
        match clear {
            Clear::None => current_y,
            Clear::Left => self
                .left_floats
                .iter()
                .map(|f| f.rect.y + f.rect.height)
                .max_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap_or(current_y)
                .max(current_y),
            Clear::Right => self
                .right_floats
                .iter()
                .map(|f| f.rect.y + f.rect.height)
                .max_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap_or(current_y)
                .max(current_y),
            Clear::Both => {
                let left_clear = self.clear_y(Clear::Left, writing_mode, current_y);
                let right_clear = self.clear_y(Clear::Right, writing_mode, current_y);

                left_clear.max(right_clear)
            }
            Clear::InlineEnd => match writing_mode {
                WritingMode::HorizontalTb | WritingMode::SidewaysRl | WritingMode::VerticalRl => {
                    self.clear_y(Clear::Right, writing_mode, current_y)
                }
                WritingMode::SidewaysLr | WritingMode::VerticalLr => self.clear_y(Clear::Left, writing_mode, current_y),
            },
            Clear::InlineStart => match writing_mode {
                WritingMode::HorizontalTb | WritingMode::SidewaysRl | WritingMode::VerticalRl => {
                    self.clear_y(Clear::Left, writing_mode, current_y)
                }
                WritingMode::SidewaysLr | WritingMode::VerticalLr => {
                    self.clear_y(Clear::Right, writing_mode, current_y)
                }
            },
        }
    }
}

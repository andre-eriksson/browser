//! Defines the `Length` struct and related types for representing CSS length values.

use strum::EnumString;

use crate::properties::{AbsoluteContext, RelativeContext};

/// Length units as defined in CSS Values and Units Module Level 4
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, EnumString)]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
pub enum LengthUnit {
    // Relative length units based on font
    /// Equal to the "cap height" (nominal height of capital letters) of the element's font.
    Cap,

    /// Represents the width or, more precisely, the advance measure of the glyph `0` (zero, the Unicode character U+0030)
    /// in the element's font. In cases where determining the measure of the `0` glyph is impossible or impractical,
    /// it must be assumed to be `0.5em` wide by `1em` tall.
    Ch,

    /// Represents the calculated font-size of the element. If used on the font-size property itself,
    /// it represents the inherited font-size of the element.
    Em,

    /// Equal to the x-height of the element's font. In fonts with the `x` letter, this is generally the
    /// height of lowercase letters in the font; `1ex ≈ 0.5em` in many fonts.
    Ex,

    /// Represents the used advance measure of the "水" glyph (CJK water ideograph, U+6C34), found in the font used to render it.
    Ic,

    /// Equal to the computed value of the line-height property of the element on which it is used, converted to an absolute length.
    /// This unit enables length calculations based on the theoretical size of an ideal empty line. However, the size of actual
    /// line boxes may differ based on their content.
    Lh,

    // Relative length units based on root element's font
    /// Equal to the "cap height" (nominal height of capital letters) of the root element's font.
    Rcap,

    /// Equal to the width or the advance measure of the glyph 0 (zero, the Unicode character U+0030) in the root element's font.
    Rch,

    /// Represents the font-size of the root element (typically `<html>`). When used within the root element font-size,
    /// it represents its initial value. The default is `16px` (for this browser), but user-defined preferences may modify this.
    Rem,

    /// Equal to the x-height of the root element's font.
    Rex,

    /// Equal to the value of `ic` unit on the root element's font.
    Ric,

    /// Equal to the value of `lh` unit on the root element's font.
    /// This unit enables length calculations based on the theoretical size of an ideal empty line.
    /// However, the size of actual line boxes may differ based on their content.
    Rlh,

    // Relative length units based on viewport
    /// Represents a percentage of the height of the viewport's initial containing block.
    /// `1vh` is 1% of the viewport height. For example, if the viewport height is `300px`,
    /// then a value of `70vh` on a property will be `210px`.
    Vw,

    /// Represents a percentage of the width of the viewport's initial containing block.
    /// `1vw` is 1% of the viewport width. For example, if the viewport width is `800px`,
    /// then a value of `50vw` on a property will be `400px`.
    Vh,

    /// Represents in percentage the largest of `vw` and `vh`.
    Vmax,

    /// Represents in percentage the smallest of `vw` and `vh`.
    Vmin,

    /// Represents the percentage of the size of the initial containing block, in the direction of the root element's block axis.
    Vb,

    /// Represents a percentage of the size of the initial containing block, in the direction of the root element's inline axis.
    Vi,

    // Small
    /// The small viewport height variant, see `vh` for details.
    Svh,

    /// The small viewport width variant, see `vw` for details.
    Svw,

    /// The small viewport larger dimension variant, see `vmax` for details.
    Svmax,

    /// The small viewport smaller dimension variant, see `vmin` for details.
    Svmin,

    /// The small viewport block size variant, see `vb` for details.
    Svb,

    /// The small viewport inline size variant, see `vi` for details.
    Svi,

    // Large
    /// The large viewport height variant, see `vh` for details.
    Lvh,

    /// The large viewport width variant, see `vw` for details.
    Lvw,

    /// The large viewport larger dimension variant, see `vmax` for details.
    Lvmax,

    /// The large viewport smaller dimension variant, see `vmin` for details.
    Lvmin,

    /// The large viewport block size variant, see `vb` for details.
    Lvb,

    /// The large viewport inline size variant, see `vi` for details.
    Lvi,

    // Dynamic
    /// The dynamic viewport height variant, see `vh` for details.
    Dvh,

    /// The dynamic viewport width variant, see `vw` for details.
    Dvw,

    /// The dynamic viewport larger dimension variant, see `vmax` for details.
    Dvmax,

    /// The dynamic viewport smaller dimension variant, see `vmin` for details.
    Dvmin,

    /// The dynamic viewport block size variant, see `vb` for details.
    Dvb,

    /// The dynamic viewport inline size variant, see `vi` for details.
    Dvi,

    // Container query length units
    /// Represents a percentage of the width of the query container.
    /// `1cqw` is 1% of the query container's width. For example, if the query container's width is `800px`,
    /// then a value of `50cqw` on a property will be `400px`.
    Cqw,

    /// Represents a percentage of the height of the query container.
    /// `1cqh` is 1% of the query container's height. For example, if the query container's height is `300px`,
    /// then a value of `10cqh` on a property will be `30px`.
    Cqh,

    /// Represents a percentage of the inline size of the query container.
    /// `1cqi` is 1% of the query container's inline size. For example, if the query container's inline size is `800px`,
    /// then a value of `50cqi` on a property will be `400px`.
    Cqi,

    /// Represents a percentage of the block size of the query container.
    /// `1cqb` is 1% of the query container's block size. For example, if the query container's block size is `300px`,
    /// then a value of `10cqb` on a property will be `30px`.
    Cqb,

    /// Represents a percentage of the smaller value of either the query container's inline size or block size.
    /// `1cqmin` is 1% of the smaller value of either the query container's inline size or block size. For example,
    /// if the query container's inline size is `800px` and its block size is `300px`, then a value of `50cqmin` on a
    /// property will be `150px`.
    Cqmin,

    /// Represents a percentage of the larger value of either the query container's inline size or block size.
    /// `1cqmax` is 1% of the larger value of either the query container's inline size or block size. For example,
    /// if the query container's inline size is `800px` and its block size is `300px`, then a value of `50cqmax` on a
    /// property will be `400px`.
    Cqmax,

    // Absolute length units
    /// One pixel. For screen displays, it traditionally represents one device pixel (dot).
    /// However, for printers and high-resolution screens, one CSS pixel implies multiple device pixels.
    /// `1px` = `1in / 96`.
    #[default]
    Px,

    /// One centimeter. `1cm` = `96px / 2.54`.
    Cm,

    /// One millimeter. `1mm` = `1cm / 10`.
    Mm,

    /// One quarter of a millimeter. `1Q` = `1cm / 40`.
    Q,

    /// One inch. `1in` = `2.54cm = 96px`.
    In,

    /// One pica. `1pc` = `12pt = 1in / 6`.
    Pc,

    /// One point. `1pt` = `1in / 72`.
    Pt,
}

/// Represents a CSS length value with a numeric value and a unit.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Length {
    /// The numeric value of the length.
    value: f32,

    /// The unit of the length.
    unit: LengthUnit,
}

impl Length {
    /// Creates a new Length with the given value and unit.
    pub fn new(value: f32, unit: LengthUnit) -> Self {
        Self { value, unit }
    }

    /// Returns the numeric value of the length.
    pub fn value(&self) -> f32 {
        self.value
    }

    /// Returns the unit of the length.
    pub fn unit(&self) -> LengthUnit {
        self.unit
    }

    /// Constructs a Length of zero, for convenience.
    pub fn zero() -> Self {
        Self {
            value: 0.0,
            unit: LengthUnit::Px,
        }
    }

    /// Constructs a Length in pixels, for convenience.
    pub fn px(value: f32) -> Self {
        Self {
            value,
            unit: LengthUnit::Px,
        }
    }

    /// Converts the Length to pixels based on the provided relative and absolute contexts.
    pub fn to_px(self, rel_ctx: &RelativeContext, abs_ctx: &AbsoluteContext) -> f32 {
        match self.unit {
            LengthUnit::Px => self.value,
            LengthUnit::Cm => self.value * 96.0 / 2.54,
            LengthUnit::Mm => self.value * 96.0 / 25.4,
            LengthUnit::Q => self.value * 96.0 / 101.6,
            LengthUnit::In => self.value * 96.0,
            LengthUnit::Pc => self.value * 16.0,
            LengthUnit::Pt => self.value * 96.0 / 72.0,
            LengthUnit::Vw => abs_ctx.viewport_width * self.value / 100.0,
            LengthUnit::Vh => abs_ctx.viewport_height * self.value / 100.0,

            LengthUnit::Ch | LengthUnit::Cap => rel_ctx.parent.font_size * 0.5 * self.value,
            LengthUnit::Rem => abs_ctx.root_font_size * self.value,
            LengthUnit::Em => rel_ctx.parent.font_size * self.value,
            _ => self.value, // TODO: Handle other units properly
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::ComputedStyle;

    use super::*;

    #[test]
    fn test_length_to_px() {
        let abs_ctx = AbsoluteContext {
            viewport_width: 800.0,
            viewport_height: 600.0,
            root_font_size: 16.0,
            ..Default::default()
        };
        let rel_ctx = RelativeContext {
            parent: Arc::new(ComputedStyle {
                font_size: 16.0,
                ..Default::default()
            }),
        };

        let length = Length::new(2.0, LengthUnit::In);
        assert_eq!(length.to_px(&rel_ctx, &abs_ctx), 192.0);

        let length = Length::new(50.0, LengthUnit::Vw);
        assert_eq!(length.to_px(&rel_ctx, &abs_ctx), 400.0);
    }
}

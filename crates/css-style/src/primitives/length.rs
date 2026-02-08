use std::str::FromStr;

use strum::EnumString;

use crate::properties::{AbsoluteContext, RelativeContext};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, EnumString)]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
pub enum LengthUnit {
    // Relative length units based on font
    Cap,
    Ch,
    Em,
    Ex,
    Ic,
    Lh,

    // Relative length units based on root element's font
    Rcap,
    Rch,
    Rem,
    Rex,
    Ric,
    Rlh,

    // Relative length units based on viewport
    Vw,
    Vh,
    Vmin,
    Vmax,
    Vb,
    Vi,

    // Small
    Svh,
    Svw,
    Svmax,
    Svmin,
    Svb,
    Svi,

    // Large
    Lvh,
    Lvw,
    Lvmax,
    Lvmin,
    Lvb,
    Lvi,

    // Dynamic
    Dvh,
    Dvw,
    Dvmax,
    Dvmin,
    Dvb,
    Dvi,

    // Container query length units
    Cqw,
    Cqh,
    Cqi,
    Cqb,
    Cqmin,
    Cqmax,

    // Absolute length units
    #[default]
    Px,
    Cm,
    Mm,
    Q,
    In,
    Pc,
    Pt,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Length {
    value: f32,
    unit: LengthUnit,
}

impl Length {
    pub fn new(value: f32, unit: LengthUnit) -> Self {
        Self { value, unit }
    }

    pub fn value(&self) -> f32 {
        self.value
    }

    pub fn unit(&self) -> LengthUnit {
        self.unit
    }

    pub fn zero() -> Self {
        Self {
            value: 0.0,
            unit: LengthUnit::Px,
        }
    }

    pub fn px(value: f32) -> Self {
        Self {
            value,
            unit: LengthUnit::Px,
        }
    }

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

            LengthUnit::Ch | LengthUnit::Cap => rel_ctx.font_size * 0.5 * self.value,
            LengthUnit::Rem => abs_ctx.root_font_size * self.value,
            LengthUnit::Em => rel_ctx.font_size * self.value,
            _ => self.value, // TODO: Handle other units properly
        }
    }
}

impl FromStr for Length {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        let split_idx = s.find(|c: char| c.is_alphabetic()).unwrap_or(s.len());
        let (value_str, unit_str) = s.split_at(split_idx);

        let value = value_str.trim().parse::<f32>().map_err(|e| e.to_string())?;
        let unit = unit_str
            .parse::<LengthUnit>()
            .map_err(|_| format!("Invalid length unit: {}", unit_str))?;

        Ok(Self::new(value, unit))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_length_parse() {
        let length = "12px".parse::<Length>().unwrap();
        assert_eq!(length.value, 12.0);
        assert_eq!(length.unit, LengthUnit::Px);

        let length = "5.5em".parse::<Length>().unwrap();
        assert_eq!(length.value, 5.5);
        assert_eq!(length.unit, LengthUnit::Em);

        let length = "100%".parse::<Length>();
        assert!(length.is_err());
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
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

#[derive(Debug, Clone)]
pub struct Length {
    pub value: f32,
    pub unit: LengthUnit,
}

impl Length {
    pub fn new(value: f32, unit: LengthUnit) -> Self {
        Self { value, unit }
    }

    pub fn zero() -> Self {
        Self {
            value: 0.0,
            unit: LengthUnit::Px,
        }
    }

    pub fn to_px(&self, relative_to: f32) -> f32 {
        match self.unit {
            LengthUnit::Px => self.value,
            LengthUnit::Cm => self.value * 96.0 / 2.54,
            LengthUnit::Mm => self.value * 96.0 / 25.4,
            LengthUnit::Q => self.value * 96.0 / 101.6,
            LengthUnit::In => self.value * 96.0,
            LengthUnit::Pc => self.value * 16.0,
            LengthUnit::Pt => self.value * 96.0 / 72.0,
            LengthUnit::Vw => relative_to * self.value / 100.0,
            LengthUnit::Vh => relative_to * self.value / 100.0,
            _ => self.value, // TODO: Handle other units properly
        }
    }
}

impl From<&str> for LengthUnit {
    fn from(s: &str) -> Self {
        match s {
            "cap" => LengthUnit::Cap,
            "ch" => LengthUnit::Ch,
            "em" => LengthUnit::Em,
            "ex" => LengthUnit::Ex,
            "ic" => LengthUnit::Ic,
            "lh" => LengthUnit::Lh,
            "rcap" => LengthUnit::Rcap,
            "rch" => LengthUnit::Rch,
            "rem" => LengthUnit::Rem,
            "rex" => LengthUnit::Rex,
            "ric" => LengthUnit::Ric,
            "rlh" => LengthUnit::Rlh,
            "vw" => LengthUnit::Vw,
            "vh" => LengthUnit::Vh,
            "vmin" => LengthUnit::Vmin,
            "vmax" => LengthUnit::Vmax,
            "vb" => LengthUnit::Vb,
            "vi" => LengthUnit::Vi,
            "svh" => LengthUnit::Svh,
            "svw" => LengthUnit::Svw,
            "svmax" => LengthUnit::Svmax,
            "svmin" => LengthUnit::Svmin,
            "svb" => LengthUnit::Svb,
            "svi" => LengthUnit::Svi,
            "lvh" => LengthUnit::Lvh,
            "lvw" => LengthUnit::Lvw,
            "lvmax" => LengthUnit::Lvmax,
            "lvmin" => LengthUnit::Lvmin,
            "lvb" => LengthUnit::Lvb,
            "lvi" => LengthUnit::Lvi,
            "dvh" => LengthUnit::Dvh,
            "dvw" => LengthUnit::Dvw,
            "dvmax" => LengthUnit::Dvmax,
            "dvmin" => LengthUnit::Dvmin,
            "dvb" => LengthUnit::Dvb,
            "dvi" => LengthUnit::Dvi,
            "cqw" => LengthUnit::Cqw,
            "cqh" => LengthUnit::Cqh,
            "cqi" => LengthUnit::Cqi,
            "cqb" => LengthUnit::Cqb,
            "cqmin" => LengthUnit::Cqmin,
            "cqmax" => LengthUnit::Cqmax,
            "px" => LengthUnit::Px,
            "cm" => LengthUnit::Cm,
            "mm" => LengthUnit::Mm,
            "q" => LengthUnit::Q,
            "in" => LengthUnit::In,
            "pc" => LengthUnit::Pc,
            "pt" => LengthUnit::Pt,
            _ => LengthUnit::Px,
        }
    }
}

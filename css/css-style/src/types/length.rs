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

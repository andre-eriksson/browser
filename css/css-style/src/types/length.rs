#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
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

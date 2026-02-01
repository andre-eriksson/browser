use crate::types::Parseable;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
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

    pub fn px(value: f32) -> Self {
        Self {
            value,
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
            LengthUnit::Rem | LengthUnit::Em => relative_to * self.value,
            _ => self.value, // TODO: Handle other units properly
        }
    }
}

impl Parseable for LengthUnit {
    fn parse(value: &str) -> Option<Self> {
        match value.len() {
            1 => {
                if value.eq_ignore_ascii_case("q") {
                    Some(Self::Q)
                } else {
                    None
                }
            }
            2 => {
                if value.eq_ignore_ascii_case("ch") {
                    Some(Self::Ch)
                } else if value.eq_ignore_ascii_case("em") {
                    Some(Self::Em)
                } else if value.eq_ignore_ascii_case("ex") {
                    Some(Self::Ex)
                } else if value.eq_ignore_ascii_case("ic") {
                    Some(Self::Ic)
                } else if value.eq_ignore_ascii_case("lh") {
                    Some(Self::Lh)
                } else if value.eq_ignore_ascii_case("vw") {
                    Some(Self::Vw)
                } else if value.eq_ignore_ascii_case("vh") {
                    Some(Self::Vh)
                } else if value.eq_ignore_ascii_case("vb") {
                    Some(Self::Vw)
                } else if value.eq_ignore_ascii_case("vi") {
                    Some(Self::Vh)
                } else if value.eq_ignore_ascii_case("px") {
                    Some(Self::Px)
                } else if value.eq_ignore_ascii_case("cm") {
                    Some(Self::Cm)
                } else if value.eq_ignore_ascii_case("mm") {
                    Some(Self::Mm)
                } else if value.eq_ignore_ascii_case("in") {
                    Some(Self::In)
                } else if value.eq_ignore_ascii_case("pc") {
                    Some(Self::Pc)
                } else if value.eq_ignore_ascii_case("pt") {
                    Some(Self::Pt)
                } else {
                    None
                }
            }
            3 => {
                if value.eq_ignore_ascii_case("cap") {
                    Some(Self::Cap)
                } else if value.eq_ignore_ascii_case("rch") {
                    Some(Self::Rch)
                } else if value.eq_ignore_ascii_case("rem") {
                    Some(Self::Rem)
                } else if value.eq_ignore_ascii_case("rex") {
                    Some(Self::Rex)
                } else if value.eq_ignore_ascii_case("ric") {
                    Some(Self::Ric)
                } else if value.eq_ignore_ascii_case("rlh") {
                    Some(Self::Rlh)
                } else if value.eq_ignore_ascii_case("svh") {
                    Some(Self::Svh)
                } else if value.eq_ignore_ascii_case("svw") {
                    Some(Self::Svw)
                } else if value.eq_ignore_ascii_case("svb") {
                    Some(Self::Svb)
                } else if value.eq_ignore_ascii_case("svi") {
                    Some(Self::Svi)
                } else if value.eq_ignore_ascii_case("lvh") {
                    Some(Self::Lvh)
                } else if value.eq_ignore_ascii_case("lvw") {
                    Some(Self::Lvw)
                } else if value.eq_ignore_ascii_case("lvb") {
                    Some(Self::Lvb)
                } else if value.eq_ignore_ascii_case("lvi") {
                    Some(Self::Lvi)
                } else if value.eq_ignore_ascii_case("dvh") {
                    Some(Self::Dvh)
                } else if value.eq_ignore_ascii_case("dvw") {
                    Some(Self::Dvw)
                } else if value.eq_ignore_ascii_case("dvb") {
                    Some(Self::Dvb)
                } else if value.eq_ignore_ascii_case("dvi") {
                    Some(Self::Dvi)
                } else if value.eq_ignore_ascii_case("cqw") {
                    Some(Self::Cqw)
                } else if value.eq_ignore_ascii_case("cqh") {
                    Some(Self::Cqh)
                } else if value.eq_ignore_ascii_case("cqi") {
                    Some(Self::Cqi)
                } else if value.eq_ignore_ascii_case("cqb") {
                    Some(Self::Cqb)
                } else {
                    None
                }
            }
            4 => {
                if value.eq_ignore_ascii_case("rcap") {
                    Some(Self::Rcap)
                } else if value.eq_ignore_ascii_case("vmin") {
                    Some(Self::Vmin)
                } else if value.eq_ignore_ascii_case("vmax") {
                    Some(Self::Vmax)
                } else {
                    None
                }
            }
            5 => {
                if value.eq_ignore_ascii_case("svmax") {
                    Some(Self::Svmax)
                } else if value.eq_ignore_ascii_case("svmin") {
                    Some(Self::Svmin)
                } else if value.eq_ignore_ascii_case("lvmax") {
                    Some(Self::Lvmax)
                } else if value.eq_ignore_ascii_case("lvmin") {
                    Some(Self::Lvmin)
                } else if value.eq_ignore_ascii_case("dvmax") {
                    Some(Self::Dvmax)
                } else if value.eq_ignore_ascii_case("dvmin") {
                    Some(Self::Dvmin)
                } else if value.eq_ignore_ascii_case("cqmax") {
                    Some(Self::Cqmax)
                } else if value.eq_ignore_ascii_case("cqmin") {
                    Some(Self::Cqmin)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

impl Parseable for Length {
    fn parse(value: &str) -> Option<Self> {
        let s = value.trim();
        let split_idx = s.find(|c: char| c.is_alphabetic()).unwrap_or(s.len());
        let (value_str, unit_str) = s.split_at(split_idx);

        let value = value_str.trim().parse::<f32>().ok()?;
        let unit = LengthUnit::parse(unit_str)?;

        Some(Self::new(value, unit))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_length_parse() {
        let length = Length::parse("12px").unwrap();
        assert_eq!(length.value, 12.0);
        assert_eq!(length.unit, LengthUnit::Px);

        let length = Length::parse("5.5em").unwrap();
        assert_eq!(length.value, 5.5);
        assert_eq!(length.unit, LengthUnit::Em);

        let length = Length::parse("100%"); // Should be None
        assert!(length.is_none());
    }
}

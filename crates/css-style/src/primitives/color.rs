use std::str::FromStr;

use strum::EnumString;

use crate::primitives::{angle::Angle, percentage::Percentage};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, EnumString)]
#[strum(serialize_all = "kebab_case", ascii_case_insensitive)]
pub enum SystemColor {
    AccentColor,
    AccentColorText,
    ActiveText,
    ButtonBorder,
    ButtonFace,
    ButtonText,
    Canvas,
    CanvasText,
    Field,
    FieldText,
    GrayText,
    Highlight,
    HighlightText,
    LinkText,
    Mark,
    MarkText,
    SelectedItem,
    SelectedItemText,
    VisitedText,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, EnumString)]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
pub enum NamedColor {
    AliceBlue,
    AntiqueWhite,
    Aqua,
    Aquamarine,
    Azure,
    Beige,
    Bisque,
    Black,
    BlanchedAlmond,
    Blue,
    BlueViolet,
    Brown,
    BurlyWood,
    CadetBlue,
    Chartreuse,
    Chocolate,
    Coral,
    CornflowerBlue,
    Cornsilk,
    Crimson,
    Cyan,
    DarkBlue,
    DarkCyan,
    DarkGoldenRod,
    DarkGray,
    DarkGreen,
    DarkKhaki,
    DarkMagenta,
    DarkOliveGreen,
    DarkOrange,
    DarkOrchid,
    DarkRed,
    DarkSalmon,
    DarkSeaGreen,
    DarkSlateBlue,
    DarkSlateGray,
    DarkTurquoise,
    DarkViolet,
    DeepPink,
    DeepSkyBlue,
    DimGray,
    DodgerBlue,
    FireBrick,
    FloralWhite,
    ForestGreen,
    Fuchsia,
    Gainsboro,
    GhostWhite,
    Gold,
    GoldenRod,
    Gray,
    Green,
    GreenYellow,
    HoneyDew,
    HotPink,
    IndianRed,
    Indigo,
    Ivory,
    Khaki,
    Lavender,
    LavenderBlush,
    LawnGreen,
    LemonChiffon,
    LightBlue,
    LightCoral,
    LightCyan,
    LightGoldenRodYellow,
    LightGray,
    LightGreen,
    LightPink,
    LightSalmon,
    LightSeaGreen,
    LightSkyBlue,
    LightSlateGray,
    LightSteelBlue,
    LightYellow,
    Lime,
    LimeGreen,
    Linen,
    Magenta,
    Maroon,
    MediumAquaMarine,
    MediumBlue,
    MediumOrchid,
    MediumPurple,
    MediumSeaGreen,
    MediumSlateBlue,
    MediumSpringGreen,
    MediumTurquoise,
    MediumVioletRed,
    MidnightBlue,
    MintCream,
    MistyRose,
    Moccasin,
    NavajoWhite,
    Navy,
    OldLace,
    Olive,
    OliveDrab,
    Orange,
    OrangeRed,
    Orchid,
    PaleGoldenRod,
    PaleGreen,
    PaleTurquoise,
    PaleVioletRed,
    PapayaWhip,
    PeachPuff,
    Peru,
    Pink,
    Plum,
    PowderBlue,
    Purple,
    RebeccaPurple,
    Red,
    RosyBrown,
    RoyalBlue,
    SaddleBrown,
    Salmon,
    SandyBrown,
    SeaGreen,
    SeaShell,
    Sienna,
    Silver,
    SkyBlue,
    SlateBlue,
    SlateGray,
    Snow,
    SpringGreen,
    SteelBlue,
    Tan,
    Teal,
    Thistle,
    Tomato,
    Turquoise,
    Violet,
    Wheat,
    White,
    WhiteSmoke,
    Yellow,
    YellowGreen,
}

impl NamedColor {
    /// Converts the NamedColor to its hexadecimal string representation
    ///
    /// # Returns
    /// An Option containing the hex string if the color is valid, or None if not.
    pub fn to_hex(self) -> Option<&'static str> {
        match self {
            NamedColor::AliceBlue => Some("#F0F8FF"),
            NamedColor::AntiqueWhite => Some("#FAEBD7"),
            NamedColor::Aqua => Some("#00FFFF"),
            NamedColor::Aquamarine => Some("#7FFFD4"),
            NamedColor::Azure => Some("#F0FFFF"),
            NamedColor::Beige => Some("#F5F5DC"),
            NamedColor::Bisque => Some("#FFE4C4"),
            NamedColor::Black => Some("#000000"),
            NamedColor::BlanchedAlmond => Some("#FFEBCD"),
            NamedColor::Blue => Some("#0000FF"),
            NamedColor::BlueViolet => Some("#8A2BE2"),
            NamedColor::Brown => Some("#A52A2A"),
            NamedColor::BurlyWood => Some("#DEB887"),
            NamedColor::CadetBlue => Some("#5F9EA0"),
            NamedColor::Chartreuse => Some("#7FFF00"),
            NamedColor::Chocolate => Some("#D2691E"),
            NamedColor::Coral => Some("#FF7F50"),
            NamedColor::CornflowerBlue => Some("#6495ED"),
            NamedColor::Cornsilk => Some("#FFF8DC"),
            NamedColor::Crimson => Some("#DC143C"),
            NamedColor::Cyan => Some("#00FFFF"),
            NamedColor::DarkBlue => Some("#00008B"),
            NamedColor::DarkCyan => Some("#008B8B"),
            NamedColor::DarkGoldenRod => Some("#B8860B"),
            NamedColor::DarkGray => Some("#A9A9A9"),
            NamedColor::DarkGreen => Some("#006400"),
            NamedColor::DarkKhaki => Some("#BDB76B"),
            NamedColor::DarkMagenta => Some("#8B008B"),
            NamedColor::DarkOliveGreen => Some("#556B2F"),
            NamedColor::DarkOrange => Some("#FF8C00"),
            NamedColor::DarkOrchid => Some("#9932CC"),
            NamedColor::DarkRed => Some("#8B0000"),
            NamedColor::DarkSalmon => Some("#E9967A"),
            NamedColor::DarkSeaGreen => Some("#8FBC8F"),
            NamedColor::DarkSlateBlue => Some("#483D8B"),
            NamedColor::DarkSlateGray => Some("#2F4F4F"),
            NamedColor::DarkTurquoise => Some("#00CED1"),
            NamedColor::DarkViolet => Some("#9400D3"),
            NamedColor::DeepPink => Some("#FF1493"),
            NamedColor::DeepSkyBlue => Some("#00BFFF"),
            NamedColor::DimGray => Some("#696969"),
            NamedColor::DodgerBlue => Some("#1E90FF"),
            NamedColor::FireBrick => Some("#B22222"),
            NamedColor::FloralWhite => Some("#FFFAF0"),
            NamedColor::ForestGreen => Some("#228B22"),
            NamedColor::Fuchsia => Some("#FF00FF"),
            NamedColor::Gainsboro => Some("#DCDCDC"),
            NamedColor::GhostWhite => Some("#F8F8FF"),
            NamedColor::Gold => Some("#FFD700"),
            NamedColor::GoldenRod => Some("#DAA520"),
            NamedColor::Gray => Some("#808080"),
            NamedColor::Green => Some("#008000"),
            NamedColor::GreenYellow => Some("#ADFF2F"),
            NamedColor::HoneyDew => Some("#F0FFF0"),
            NamedColor::HotPink => Some("#FF69B4"),
            NamedColor::IndianRed => Some("#CD5C5C"),
            NamedColor::Indigo => Some("#4B0082"),
            NamedColor::Ivory => Some("#FFFFF0"),
            NamedColor::Khaki => Some("#F0E68C"),
            NamedColor::Lavender => Some("#E6E6FA"),
            NamedColor::LavenderBlush => Some("#FFF0F5"),
            NamedColor::LawnGreen => Some("#7CFC00"),
            NamedColor::LemonChiffon => Some("#FFFACD"),
            NamedColor::LightBlue => Some("#ADD8E6"),
            NamedColor::LightCoral => Some("#F08080"),
            NamedColor::LightCyan => Some("#E0FFFF"),
            NamedColor::LightGoldenRodYellow => Some("#FAFAD2"),
            NamedColor::LightGray => Some("#D3D3D3"),
            NamedColor::LightGreen => Some("#90EE90"),
            NamedColor::LightPink => Some("#FFB6C1"),
            NamedColor::LightSalmon => Some("#FFA07A"),
            NamedColor::LightSeaGreen => Some("#20B2AA"),
            NamedColor::LightSkyBlue => Some("#87CEFA"),
            NamedColor::LightSlateGray => Some("#778899"),
            NamedColor::LightSteelBlue => Some("#B0C4DE"),
            NamedColor::LightYellow => Some("#FFFFE0"),
            NamedColor::Lime => Some("#00FF00"),
            NamedColor::LimeGreen => Some("#32CD32"),
            NamedColor::Linen => Some("#FAF0E6"),
            NamedColor::Magenta => Some("#FF00FF"),
            NamedColor::Maroon => Some("#800000"),
            NamedColor::MediumAquaMarine => Some("#66CDAA"),
            NamedColor::MediumBlue => Some("#0000CD"),
            NamedColor::MediumOrchid => Some("#BA55D3"),
            NamedColor::MediumPurple => Some("#9370DB"),
            NamedColor::MediumSeaGreen => Some("#3CB371"),
            NamedColor::MediumSlateBlue => Some("#7B68EE"),
            NamedColor::MediumSpringGreen => Some("#00FA9A"),
            NamedColor::MediumTurquoise => Some("#48D1CC"),
            NamedColor::MediumVioletRed => Some("#C71585"),
            NamedColor::MidnightBlue => Some("#191970"),
            NamedColor::MintCream => Some("#F5FFFA"),
            NamedColor::MistyRose => Some("#FFE4E1"),
            NamedColor::Moccasin => Some("#FFE4B5"),
            NamedColor::NavajoWhite => Some("#FFDEAD"),
            NamedColor::Navy => Some("#000080"),
            NamedColor::OldLace => Some("#FDF5E6"),
            NamedColor::Olive => Some("#808000"),
            NamedColor::OliveDrab => Some("#6B8E23"),
            NamedColor::Orange => Some("#FFA500"),
            NamedColor::OrangeRed => Some("#FF4500"),
            NamedColor::Orchid => Some("#DA70D6"),
            NamedColor::PaleGoldenRod => Some("#EEE8AA"),
            NamedColor::PaleGreen => Some("#98FB98"),
            NamedColor::PaleTurquoise => Some("#AFEEEE"),
            NamedColor::PaleVioletRed => Some("#DB7093"),
            NamedColor::PapayaWhip => Some("#FFEFD5"),
            NamedColor::PeachPuff => Some("#FFDAB9"),
            NamedColor::Peru => Some("#CD853F"),
            NamedColor::Pink => Some("#FFC0CB"),
            NamedColor::Plum => Some("#DDA0DD"),
            NamedColor::PowderBlue => Some("#B0E0E6"),
            NamedColor::Purple => Some("#800080"),
            NamedColor::RebeccaPurple => Some("#663399"),
            NamedColor::Red => Some("#FF0000"),
            NamedColor::RosyBrown => Some("#BC8F8F"),
            NamedColor::RoyalBlue => Some("#4169E1"),
            NamedColor::SaddleBrown => Some("#8B4513"),
            NamedColor::Salmon => Some("#FA8072"),
            NamedColor::SandyBrown => Some("#F4A460"),
            NamedColor::SeaGreen => Some("#2E8B57"),
            NamedColor::SeaShell => Some("#FFF5EE"),
            NamedColor::Sienna => Some("#A0522D"),
            NamedColor::Silver => Some("#C0C0C0"),
            NamedColor::SkyBlue => Some("#87CEEB"),
            NamedColor::SlateBlue => Some("#6A5ACD"),
            NamedColor::SlateGray => Some("#708090"),
            NamedColor::Snow => Some("#FFFAFA"),
            NamedColor::SpringGreen => Some("#00FF7F"),
            NamedColor::SteelBlue => Some("#4682B4"),
            NamedColor::Tan => Some("#D2B48C"),
            NamedColor::Teal => Some("#008080"),
            NamedColor::Thistle => Some("#D8BFD8"),
            NamedColor::Tomato => Some("#FF6347"),
            NamedColor::Turquoise => Some("#40E0D0"),
            NamedColor::Violet => Some("#EE82EE"),
            NamedColor::Wheat => Some("#F5DEB3"),
            NamedColor::White => Some("#FFFFFF"),
            NamedColor::WhiteSmoke => Some("#F5F5F5"),
            NamedColor::Yellow => Some("#FFFF00"),
            NamedColor::YellowGreen => Some("#9ACD32"),
        }
    }

    /// Converts the NamedColor to an RGB tuple (r, g, b)
    ///
    /// # Returns
    /// An Option containing a tuple of (r, g, b) if the color is valid, or None if not.
    pub fn to_rgb_tuple(self) -> Option<(u8, u8, u8)> {
        self.to_hex().and_then(|hex| {
            let hex = hex.trim_start_matches('#');
            if hex.len() == 6
                && let Ok(parsed) = u32::from_str_radix(hex, 16)
            {
                let r = ((parsed >> 16) & 0xFF) as u8;
                let g = ((parsed >> 8) & 0xFF) as u8;
                let b = (parsed & 0xFF) as u8;
                return Some((r, g, b));
            }
            None
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SRGBAColor {
    Rgb(u8, u8, u8),
    Rgba(u8, u8, u8, f32),
    Hsl(f32, f32, f32),
    Hsla(f32, f32, f32, f32),
    Hwb(f32, f32, f32),
}

impl FromStr for SRGBAColor {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let value = value.trim();

        if let Some(hex) = value.strip_prefix('#') {
            let parsed = u32::from_str_radix(hex, 16).map_err(|e| e.to_string())?;

            match hex.len() {
                3 => {
                    let r = (((parsed >> 8) & 0xF) * 17) as u8;
                    let g = (((parsed >> 4) & 0xF) * 17) as u8;
                    let b = ((parsed & 0xF) * 17) as u8;
                    Ok(Self::Rgb(r, g, b))
                }
                4 => {
                    let r = (((parsed >> 12) & 0xF) * 17) as u8;
                    let g = (((parsed >> 8) & 0xF) * 17) as u8;
                    let b = (((parsed >> 4) & 0xF) * 17) as u8;
                    let a = ((parsed & 0xF) * 17) as f32 / 255.0;
                    Ok(Self::Rgba(r, g, b, a))
                }
                6 => {
                    let r = ((parsed >> 16) & 0xFF) as u8;
                    let g = ((parsed >> 8) & 0xFF) as u8;
                    let b = (parsed & 0xFF) as u8;
                    Ok(Self::Rgb(r, g, b))
                }
                8 => {
                    let r = ((parsed >> 24) & 0xFF) as u8;
                    let g = ((parsed >> 16) & 0xFF) as u8;
                    let b = ((parsed >> 8) & 0xFF) as u8;
                    let a = (parsed & 0xFF) as f32 / 255.0;
                    Ok(Self::Rgba(r, g, b, a))
                }
                _ => Err(format!("'{}', Invalid hex color format", value)),
            }
        } else if value.starts_with("rgb(") && value.ends_with(')') {
            let content = &value[4..value.len() - 1];
            let parts: Vec<&str> = content.split(',').map(|s| s.trim()).collect();
            if parts.len() == 3
                && let (Ok(r), Ok(g), Ok(b)) = (
                    parts[0].parse::<u8>(),
                    parts[1].parse::<u8>(),
                    parts[2].parse::<u8>(),
                )
            {
                Ok(Self::Rgb(r, g, b))
            } else {
                Err(format!("'{}', Invalid rgb() format", value))
            }
        } else if value.starts_with("rgba(") && value.ends_with(')') {
            let content = &value[5..value.len() - 1];
            let parts: Vec<&str> = content.split(',').map(|s| s.trim()).collect();
            if parts.len() == 4
                && let (Ok(r), Ok(g), Ok(b), Ok(a)) = (
                    parts[0].parse::<u8>(),
                    parts[1].parse::<u8>(),
                    parts[2].parse::<u8>(),
                    parts[3].parse::<f32>(),
                )
            {
                Ok(Self::Rgba(r, g, b, a))
            } else {
                Err(format!("'{}', Invalid rgb() format", value))
            }
        } else {
            Err("Invalid color format".to_string())
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Cielab {
    Lab(f32, f32, f32),
    Lch(f32, f32, f32),
}

impl FromStr for Cielab {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let value = value.trim();

        if value.starts_with("lab(") && value.ends_with(')') {
            let content = &value[4..value.len() - 1];
            // TODO: Handle relative values: `from <color> L C H [ / A]`
            let parts: Vec<&str> = if content.contains(',') {
                content.split(',').map(|s| s.trim()).collect()
            } else {
                content.split_whitespace().map(|s| s.trim()).collect()
            };

            if parts.len() != 3 {
                // TODO: Handle optional alpha channel
                return Err("Alpha channel not supported yet".to_string());
            }

            let l = if parts[0].contains('%') {
                parts[0].parse::<Percentage>()?.value()
            } else if parts[0].eq_ignore_ascii_case("none") {
                0.0
            } else {
                parts[0].parse::<f32>().map_err(|e| e.to_string())?
            };

            let a = if parts[1].contains('%') {
                (parts[1].parse::<Percentage>()?.value() / 100.0) * 125.0
            } else if parts[1].eq_ignore_ascii_case("none") {
                0.0
            } else {
                parts[1].parse::<f32>().map_err(|e| e.to_string())?
            };

            let b = if parts[2].contains('%') {
                (parts[2].parse::<Percentage>()?.value() / 100.0 / 100.0) * 125.0
            } else if parts[2].eq_ignore_ascii_case("none") {
                0.0
            } else {
                parts[2].parse::<f32>().map_err(|e| e.to_string())?
            };

            Ok(Self::Lab(l, a, b))
        } else if value.starts_with("lch(") && value.ends_with(')') {
            let content = &value[4..value.len() - 1];
            // TODO: Handle relative values: `from <color> L C H [ / A]`
            let parts: Vec<&str> = if content.contains(',') {
                content.split(',').map(|s| s.trim()).collect()
            } else {
                content.split_whitespace().map(|s| s.trim()).collect()
            };

            if parts.len() != 3 {
                // TODO: Handle optional alpha channel
                return Err("Alpha channel not supported yet".to_string());
            }

            let l = if parts[0].contains('%') {
                parts[0].parse::<Percentage>()?.value()
            } else if parts[0].eq_ignore_ascii_case("none") {
                0.0
            } else {
                parts[0].parse::<f32>().map_err(|e| e.to_string())?
            };

            let c = if parts[1].contains('%') {
                (parts[1].parse::<Percentage>()?.value() / 100.0) * 150.0
            } else if parts[1].eq_ignore_ascii_case("none") {
                0.0
            } else {
                parts[1].parse::<f32>().map_err(|e| e.to_string())?
            };

            let h = if let Ok(angle) = parts[2].parse::<Angle>() {
                angle.to_degrees()
            } else if parts[2].eq_ignore_ascii_case("none") {
                0.0
            } else {
                parts[2].parse::<f32>().map_err(|e| e.to_string())?
            };

            Ok(Self::Lch(l, c, h))
        } else {
            Err(format!("'{}', Invalid CIELAB color format", value))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Oklab {
    Oklab(f32, f32, f32),
    Oklch(f32, f32, f32),
}

impl FromStr for Oklab {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = s.trim();

        if value.starts_with("oklch(") && value.ends_with(')') {
            let content = &value[6..value.len() - 1];

            // TODO: Handle relative values: `from <color> L C H [ / A]`
            let parts: Vec<&str> = if content.contains(',') {
                content.split(',').map(|s| s.trim()).collect()
            } else {
                content.split_whitespace().map(|s| s.trim()).collect()
            };

            if parts.len() != 3 {
                // TODO: Handle optional alpha channel
                return Err("Alpha channel not supported yet".to_string());
            }

            let l = if parts[0].contains('%') {
                parts[0].parse::<Percentage>()?.value() / 100.0
            } else if parts[0].eq_ignore_ascii_case("none") {
                0.0
            } else {
                parts[0]
                    .parse::<f32>()
                    .map_err(|e| format!("Error parsing L component in oklch(): {}", e))?
            };

            let c = if parts[1].contains('%') {
                parts[1].parse::<Percentage>()?.value() / 100.0
            } else if parts[1].eq_ignore_ascii_case("none") {
                0.0
            } else {
                parts[1]
                    .parse::<f32>()
                    .map_err(|e| format!("Error parsing C component in oklch(): {}", e))?
            };

            let h = if let Ok(angle) = parts[2].parse::<Angle>() {
                angle.to_degrees()
            } else if parts[2].eq_ignore_ascii_case("none") {
                0.0
            } else {
                parts[2]
                    .parse::<f32>()
                    .map_err(|e| format!("Error parsing H component in oklch(): {}", e))?
            };

            Ok(Self::Oklch(l, c, h))
        } else if value.starts_with("oklab(") && value.ends_with(')') {
            let content = &value[6..value.len() - 1];

            let parts: Vec<&str> = if content.contains(',') {
                content.split(',').map(|s| s.trim()).collect()
            } else {
                content.split_whitespace().map(|s| s.trim()).collect()
            };

            if parts.len() != 3 {
                // TODO: Handle optional alpha channel
                return Err("Alpha channel not supported yet".to_string());
            }

            let l = if parts[0].contains('%') {
                parts[0].parse::<Percentage>()?.value() / 100.0
            } else if parts[0].eq_ignore_ascii_case("none") {
                0.0
            } else {
                parts[0]
                    .parse::<f32>()
                    .map_err(|e| format!("Error parsing L component in oklab(): {}", e))?
            };

            let a = if parts[1].contains('%') {
                parts[1].parse::<Percentage>()?.value() / 100.0 * 0.4
            } else if parts[1].eq_ignore_ascii_case("none") {
                0.0
            } else {
                parts[1]
                    .parse::<f32>()
                    .map_err(|e| format!("Error parsing a component in oklab(): {}", e))?
            };

            let b = if parts[2].contains('%') {
                parts[2].parse::<Percentage>()?.value() / 100.0 * 0.4
            } else if parts[2].eq_ignore_ascii_case("none") {
                0.0
            } else {
                parts[2]
                    .parse::<f32>()
                    .map_err(|e| format!("Error parsing b component in oklab(): {}", e))?
            };

            Ok(Self::Oklab(l, a, b))
        } else {
            Err(format!("'{}', Invalid Oklab color format", value))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FunctionColor {
    Srgba(SRGBAColor),
    Cielab(Cielab),
    Oklab(Oklab),
}

impl FromStr for FunctionColor {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if let Ok(srgba) = value.parse::<SRGBAColor>() {
            return Ok(Self::Srgba(srgba));
        }

        if let Ok(cielab) = value.parse::<Cielab>() {
            return Ok(Self::Cielab(cielab));
        }

        if let Ok(oklab) = value.parse::<Oklab>() {
            return Ok(Self::Oklab(oklab));
        }

        Err(format!("'{}', Invalid functional color format", value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_color_parsing() {
        assert_eq!(
            SystemColor::try_from("highlight"),
            Ok(SystemColor::Highlight)
        );
        assert!(SystemColor::try_from("unknowncolor").is_err());
    }

    #[test]
    fn test_named_color_parsing() {
        assert_eq!(
            NamedColor::try_from("rebeCCapurPLE"),
            Ok(NamedColor::RebeccaPurple)
        );
        assert!(NamedColor::try_from("invalidcolor").is_err());
    }

    #[test]
    fn test_color_hex_parsing() {
        assert_eq!("#FF5733".parse(), Ok(SRGBAColor::Rgb(255, 87, 51)));
        assert!("#ZZZZZZ".parse::<SRGBAColor>().is_err());
    }

    #[test]
    fn test_color_srgba_parsing() {
        assert_eq!("rgb(255, 0, 0)".parse(), Ok(SRGBAColor::Rgb(255, 0, 0)));
        assert_eq!(
            "rgba(0, 255, 0, 0.5)".parse(),
            Ok(SRGBAColor::Rgba(0, 255, 0, 0.5))
        );
        assert!("invalid".parse::<SRGBAColor>().is_err());
    }

    #[test]
    fn test_color_cielab_parsing() {
        assert_eq!("lab(50, 20, 30)".parse(), Ok(Cielab::Lab(50.0, 20.0, 30.0)));
        assert_eq!(
            "lch(60, 40, 120)".parse(),
            Ok(Cielab::Lch(60.0, 40.0, 120.0))
        );
        assert!("invalid".parse::<Cielab>().is_err());
    }

    #[test]
    fn test_color_oklab_parsing() {
        assert_eq!(
            "oklch(70%, 50%, 180)".parse(),
            Ok(Oklab::Oklch(0.7, 0.5, 180.0))
        );
        assert_eq!(
            "oklab(0.6, 0.1, -0.1)".parse(),
            Ok(Oklab::Oklab(0.6, 0.1, -0.1))
        );

        assert!("invalid".parse::<Oklab>().is_err());
    }
}

use crate::{resolver::PropertyResolver, types::angle::Angle};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
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

    /// Used only for getting a default value in From<&str>
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
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
    Transparent,
    Turquoise,
    Violet,
    Wheat,
    White,
    WhiteSmoke,
    Yellow,
    YellowGreen,

    /// Used only for getting a default value in From<&str>
    Unknown,
}

#[derive(Debug, Clone, Copy)]
pub enum SRGBAColor {
    RGB(u8, u8, u8),
    RGBA(u8, u8, u8, f32),
    HSL(f32, f32, f32),
    HSLA(f32, f32, f32, f32),
    HWB(f32, f32, f32),
}

#[derive(Debug, Clone, Copy)]
pub enum CIELAB {
    Lab(f32, f32, f32),
    Lch(f32, f32, f32),
}

#[derive(Debug, Clone, Copy)]
pub enum Oklab {
    Oklab(f32, f32, f32),
    Oklch(f32, f32, f32),
}

#[derive(Debug, Clone, Copy)]
pub enum FunctionColor {
    SRGBA(SRGBAColor),
    CIELAB(CIELAB),
    Oklab(Oklab),
}

#[derive(Debug, Clone, Copy)]
pub enum Color {
    System(SystemColor),
    Named(NamedColor),
    Hex([u8; 3]),
    Functional(FunctionColor),
    CurrentColor,
}

impl Color {
    pub fn hex(value: &str) -> Option<Self> {
        let hex = value.trim_start_matches('#');
        if hex.len() == 6
            && let Ok(parsed) = u32::from_str_radix(hex, 16)
        {
            let r = ((parsed >> 16) & 0xFF) as u8;
            let g = ((parsed >> 8) & 0xFF) as u8;
            let b = (parsed & 0xFF) as u8;
            return Some(Color::Hex([r, g, b]));
        }

        None
    }

    pub fn from_rgb_string(value: &str) -> Option<Self> {
        let value = value.trim();

        if value.starts_with("rgb(") && value.ends_with(')') {
            let content = &value[4..value.len() - 1];
            let parts: Vec<&str> = content.split(',').map(|s| s.trim()).collect();
            if parts.len() == 3
                && let (Ok(r), Ok(g), Ok(b)) = (
                    parts[0].parse::<u8>(),
                    parts[1].parse::<u8>(),
                    parts[2].parse::<u8>(),
                )
            {
                return Some(Color::Functional(FunctionColor::SRGBA(SRGBAColor::RGB(
                    r, g, b,
                ))));
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
                return Some(Color::Functional(FunctionColor::SRGBA(SRGBAColor::RGBA(
                    r, g, b, a,
                ))));
            }
        }

        None
    }

    pub fn from_oklch_string(value: &str) -> Option<Self> {
        let value = value.trim();

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
                return None;
            }

            let l = if parts[0].contains('%') {
                PropertyResolver::resolve_percentage(parts[0])? / 100.0
            } else if parts[0].eq_ignore_ascii_case("none") {
                0.0
            } else {
                parts[0].parse::<f32>().ok()?
            };

            let c = if parts[1].contains('%') {
                PropertyResolver::resolve_percentage(parts[1])? / 100.0
            } else if parts[1].eq_ignore_ascii_case("none") {
                0.0
            } else {
                parts[1].parse::<f32>().ok()?
            };

            let h = if let Some(angle) = Angle::parse(parts[2]) {
                angle.to_degrees()
            } else {
                parts[2].parse::<f32>().ok()?
            };

            return Some(Color::Functional(FunctionColor::Oklab(Oklab::Oklch(
                l, c, h,
            ))));
        }

        None
    }
}

impl NamedColor {
    /// Converts the NamedColor to its hexadecimal string representation
    ///
    /// # Returns
    /// An Option containing the hex string if the color is valid, or None if not.
    pub fn to_hex(&self) -> Option<&'static str> {
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
            _ => None,
        }
    }

    /// Converts the NamedColor to an RGB tuple (r, g, b)
    ///
    /// # Returns
    /// An Option containing a tuple of (r, g, b) if the color is valid, or None if not.
    pub fn to_rgb_tuple(&self) -> Option<(u8, u8, u8)> {
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

impl From<&str> for SystemColor {
    fn from(value: &str) -> Self {
        match value {
            "AccentColor" => SystemColor::AccentColor,
            "AccentColorText" => SystemColor::AccentColorText,
            "ActiveText" => SystemColor::ActiveText,
            "ButtonBorder" => SystemColor::ButtonBorder,
            "ButtonFace" => SystemColor::ButtonFace,
            "ButtonText" => SystemColor::ButtonText,
            "Canvas" => SystemColor::Canvas,
            "CanvasText" => SystemColor::CanvasText,
            "Field" => SystemColor::Field,
            "FieldText" => SystemColor::FieldText,
            "GrayText" => SystemColor::GrayText,
            "Highlight" => SystemColor::Highlight,
            "HighlightText" => SystemColor::HighlightText,
            "LinkText" => SystemColor::LinkText,
            "Mark" => SystemColor::Mark,
            "MarkText" => SystemColor::MarkText,
            "SelectedItem" => SystemColor::SelectedItem,
            "SelectedItemText" => SystemColor::SelectedItemText,
            "VisitedText" => SystemColor::VisitedText,
            _ => SystemColor::Unknown, // Default case
        }
    }
}

impl From<String> for SystemColor {
    fn from(value: String) -> Self {
        SystemColor::from(value.as_str())
    }
}

impl From<&str> for NamedColor {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "aliceblue" => NamedColor::AliceBlue,
            "antiquewhite" => NamedColor::AntiqueWhite,
            "aqua" => NamedColor::Aqua,
            "aquamarine" => NamedColor::Aquamarine,
            "azure" => NamedColor::Azure,
            "beige" => NamedColor::Beige,
            "bisque" => NamedColor::Bisque,
            "black" => NamedColor::Black,
            "blanchedalmond" => NamedColor::BlanchedAlmond,
            "blue" => NamedColor::Blue,
            "blueviolet" => NamedColor::BlueViolet,
            "brown" => NamedColor::Brown,
            "burlywood" => NamedColor::BurlyWood,
            "cadetblue" => NamedColor::CadetBlue,
            "chartreuse" => NamedColor::Chartreuse,
            "chocolate" => NamedColor::Chocolate,
            "coral" => NamedColor::Coral,
            "cornflowerblue" => NamedColor::CornflowerBlue,
            "cornsilk" => NamedColor::Cornsilk,
            "crimson" => NamedColor::Crimson,
            "cyan" => NamedColor::Cyan,
            "darkblue" => NamedColor::DarkBlue,
            "darkcyan" => NamedColor::DarkCyan,
            "darkgoldenrod" => NamedColor::DarkGoldenRod,
            "darkgray" => NamedColor::DarkGray,
            "darkgreen" => NamedColor::DarkGreen,
            "darkkhaki" => NamedColor::DarkKhaki,
            "darkmagenta" => NamedColor::DarkMagenta,
            "darkolivegreen" => NamedColor::DarkOliveGreen,
            "darkorange" => NamedColor::DarkOrange,
            "darkorchid" => NamedColor::DarkOrchid,
            "darkred" => NamedColor::DarkRed,
            "darksalmon" => NamedColor::DarkSalmon,
            "darkseagreen" => NamedColor::DarkSeaGreen,
            "darkslateblue" => NamedColor::DarkSlateBlue,
            "darkslategray" | "darkslategrey" => NamedColor::DarkSlateGray,
            "darkturquoise" => NamedColor::DarkTurquoise,
            "darkviolet" => NamedColor::DarkViolet,
            "deeppink" => NamedColor::DeepPink,
            "deepskyblue" => NamedColor::DeepSkyBlue,
            "dimgray" | "dimgrey" => NamedColor::DimGray,
            "dodgerblue" => NamedColor::DodgerBlue,
            "firebrick" => NamedColor::FireBrick,
            "floralwhite" => NamedColor::FloralWhite,
            "forestgreen" => NamedColor::ForestGreen,
            "fuchsia" => NamedColor::Fuchsia,
            "gainsboro" => NamedColor::Gainsboro,
            "ghostwhite" => NamedColor::GhostWhite,
            "gold" => NamedColor::Gold,
            "goldenrod" => NamedColor::GoldenRod,
            "gray" | "grey" => NamedColor::Gray,
            "green" => NamedColor::Green,
            "greenyellow" => NamedColor::GreenYellow,
            "honeydew" => NamedColor::HoneyDew,
            "hotpink" => NamedColor::HotPink,
            "indianred" => NamedColor::IndianRed,
            "indigo" => NamedColor::Indigo,
            "ivory" => NamedColor::Ivory,
            "khaki" => NamedColor::Khaki,
            "lavender" => NamedColor::Lavender,
            "lavenderblush" => NamedColor::LavenderBlush,
            "lawngreen" => NamedColor::LawnGreen,
            "lemonchiffon" => NamedColor::LemonChiffon,
            "lightblue" => NamedColor::LightBlue,
            "lightcoral" => NamedColor::LightCoral,
            "lightcyan" => NamedColor::LightCyan,
            "lightgoldenrodyellow" => NamedColor::LightGoldenRodYellow,
            "lightgray" | "lightgrey" => NamedColor::LightGray,
            "lightgreen" => NamedColor::LightGreen,
            "lightpink" => NamedColor::LightPink,
            "lightsalmon" => NamedColor::LightSalmon,
            "lightseagreen" => NamedColor::LightSeaGreen,
            "lightskyblue" => NamedColor::LightSkyBlue,
            "lightslategray" | "lightslategrey" => NamedColor::LightSlateGray,
            "lightsteelblue" => NamedColor::LightSteelBlue,
            "lightyellow" => NamedColor::LightYellow,
            "lime" => NamedColor::Lime,
            "limegreen" => NamedColor::LimeGreen,
            "linen" => NamedColor::Linen,
            "magenta" => NamedColor::Magenta,
            "maroon" => NamedColor::Maroon,
            "mediumaquamarine" => NamedColor::MediumAquaMarine,
            "mediumblue" => NamedColor::MediumBlue,
            "mediumorchid" => NamedColor::MediumOrchid,
            "mediumpurple" => NamedColor::MediumPurple,
            "mediumseagreen" => NamedColor::MediumSeaGreen,
            "mediumslateblue" => NamedColor::MediumSlateBlue,
            "mediumspringgreen" => NamedColor::MediumSpringGreen,
            "mediumturquoise" => NamedColor::MediumTurquoise,
            "mediumvioletred" => NamedColor::MediumVioletRed,
            "midnightblue" => NamedColor::MidnightBlue,
            "mintcream" => NamedColor::MintCream,
            "mistyrose" => NamedColor::MistyRose,
            "moccasin" => NamedColor::Moccasin,
            "mavajowhite" => NamedColor::NavajoWhite,
            "navy" => NamedColor::Navy,
            "oldlace" => NamedColor::OldLace,
            "olive" => NamedColor::Olive,
            "olivedrab" => NamedColor::OliveDrab,
            "orange" => NamedColor::Orange,
            "orangered" => NamedColor::OrangeRed,
            "orchid" => NamedColor::Orchid,
            "palegoldenrod" => NamedColor::PaleGoldenRod,
            "palegreen" => NamedColor::PaleGreen,
            "paleturquoise" => NamedColor::PaleTurquoise,
            "palevioletred" => NamedColor::PaleVioletRed,
            "papayawhip" => NamedColor::PapayaWhip,
            "peachpuff" => NamedColor::PeachPuff,
            "peru" => NamedColor::Peru,
            "pink" => NamedColor::Pink,
            "plum" => NamedColor::Plum,
            "powderblue" => NamedColor::PowderBlue,
            "purple" => NamedColor::Purple,
            "rebeccapurple" => NamedColor::RebeccaPurple,
            "red" => NamedColor::Red,
            "rosybrown" => NamedColor::RosyBrown,
            "royalblue" => NamedColor::RoyalBlue,
            "saddlebrown" => NamedColor::SaddleBrown,
            "salmon" => NamedColor::Salmon,
            "sandybrown" => NamedColor::SandyBrown,
            "seagreen" => NamedColor::SeaGreen,
            "seashell" => NamedColor::SeaShell,
            "sienna" => NamedColor::Sienna,
            "silver" => NamedColor::Silver,
            "skyblue" => NamedColor::SkyBlue,
            "slateblue" => NamedColor::SlateBlue,
            "slategray" | "slategrey" => NamedColor::SlateGray,
            "snow" => NamedColor::Snow,
            "springgreen" => NamedColor::SpringGreen,
            "steelblue" => NamedColor::SteelBlue,
            "tan" => NamedColor::Tan,
            "teal" => NamedColor::Teal,
            "thistle" => NamedColor::Thistle,
            "tomato" => NamedColor::Tomato,
            "transparent" => NamedColor::Transparent,
            "turquoise" => NamedColor::Turquoise,
            "violet" => NamedColor::Violet,
            "wheat" => NamedColor::Violet,
            "white" => NamedColor::White,
            "whitesmoke" => NamedColor::WhiteSmoke,
            "yellow" => NamedColor::Yellow,
            "yellowgreen" => NamedColor::YellowGreen,
            _ => NamedColor::Unknown,
        }
    }
}

impl From<String> for NamedColor {
    fn from(value: String) -> Self {
        NamedColor::from(value.as_str())
    }
}

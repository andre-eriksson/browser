use crate::{
    types::{Parseable, angle::Angle, global::Global},
    unit::Unit,
};

pub type HexColor = [u8; 3];

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
    Turquoise,
    Violet,
    Wheat,
    White,
    WhiteSmoke,
    Yellow,
    YellowGreen,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SRGBAColor {
    RGB(u8, u8, u8),
    RGBA(u8, u8, u8, f32),
    HSL(f32, f32, f32),
    HSLA(f32, f32, f32, f32),
    HWB(f32, f32, f32),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CIELAB {
    Lab(f32, f32, f32),
    Lch(f32, f32, f32),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Oklab {
    Oklab(f32, f32, f32),
    Oklch(f32, f32, f32),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FunctionColor {
    SRGBA(SRGBAColor),
    CIELAB(CIELAB),
    Oklab(Oklab),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Color {
    System(SystemColor),
    Named(NamedColor),
    Hex(HexColor),
    Functional(FunctionColor),
    CurrentColor,
    Transparent,
    Global(Global),
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

impl Parseable for SystemColor {
    fn parse(s: &str) -> Option<Self> {
        match s.len() {
            4 if s.eq_ignore_ascii_case("mark") => Some(SystemColor::Mark),
            6 if s.eq_ignore_ascii_case("canvas") => Some(SystemColor::Canvas),
            9 if s.eq_ignore_ascii_case("gray-text") => Some(SystemColor::GrayText),
            9 if s.eq_ignore_ascii_case("highlight") => Some(SystemColor::Highlight),
            9 if s.eq_ignore_ascii_case("mark-text") => Some(SystemColor::MarkText),
            9 if s.eq_ignore_ascii_case("link-text") => Some(SystemColor::LinkText),
            11 if s.eq_ignore_ascii_case("active-text") => Some(SystemColor::ActiveText),
            11 if s.eq_ignore_ascii_case("button-face") => Some(SystemColor::ButtonFace),
            11 if s.eq_ignore_ascii_case("button-text") => Some(SystemColor::ButtonText),
            11 if s.eq_ignore_ascii_case("canvas-text") => Some(SystemColor::CanvasText),
            12 if s.eq_ignore_ascii_case("accent-color") => Some(SystemColor::AccentColor),
            12 if s.eq_ignore_ascii_case("visited-text") => Some(SystemColor::VisitedText),
            13 if s.eq_ignore_ascii_case("button-border") => Some(SystemColor::ButtonBorder),
            13 if s.eq_ignore_ascii_case("selected-item") => Some(SystemColor::SelectedItem),
            14 if s.eq_ignore_ascii_case("highlight-text") => Some(SystemColor::HighlightText),
            17 if s.eq_ignore_ascii_case("accent-color-text") => Some(SystemColor::AccentColorText),
            18 if s.eq_ignore_ascii_case("selected-item-text") => {
                Some(SystemColor::SelectedItemText)
            }
            _ => None,
        }
    }
}

impl Parseable for NamedColor {
    fn parse(value: &str) -> Option<Self> {
        let s = value.trim();

        if s.is_empty() || s.len() > 24 {
            return None;
        }

        let mut buf = [0u8; 24];
        let len = s.len();

        for (i, b) in s.bytes().enumerate() {
            buf[i] = b.to_ascii_lowercase();
        }

        match &buf[..len] {
            // A
            b"aliceblue" => Some(NamedColor::AliceBlue),
            b"antiquewhite" => Some(NamedColor::AntiqueWhite),
            b"aqua" => Some(NamedColor::Aqua),
            b"aquamarine" => Some(NamedColor::Aquamarine),
            b"azure" => Some(NamedColor::Azure),

            // B
            b"beige" => Some(NamedColor::Beige),
            b"bisque" => Some(NamedColor::Bisque),
            b"black" => Some(NamedColor::Black),
            b"blanchedalmond" => Some(NamedColor::BlanchedAlmond),
            b"blue" => Some(NamedColor::Blue),
            b"blueviolet" => Some(NamedColor::BlueViolet),
            b"brown" => Some(NamedColor::Brown),
            b"burlywood" => Some(NamedColor::BurlyWood),

            // C
            b"cadetblue" => Some(NamedColor::CadetBlue),
            b"chartreuse" => Some(NamedColor::Chartreuse),
            b"chocolate" => Some(NamedColor::Chocolate),
            b"coral" => Some(NamedColor::Coral),
            b"cornflowerblue" => Some(NamedColor::CornflowerBlue),
            b"cornsilk" => Some(NamedColor::Cornsilk),
            b"crimson" => Some(NamedColor::Crimson),
            b"cyan" => Some(NamedColor::Cyan),

            // D
            b"darkblue" => Some(NamedColor::DarkBlue),
            b"darkcyan" => Some(NamedColor::DarkCyan),
            b"darkgoldenrod" => Some(NamedColor::DarkGoldenRod),
            b"darkgray" => Some(NamedColor::DarkGray),
            b"darkgrey" => Some(NamedColor::DarkGray),
            b"darkgreen" => Some(NamedColor::DarkGreen),
            b"darkkhaki" => Some(NamedColor::DarkKhaki),
            b"darkmagenta" => Some(NamedColor::DarkMagenta),
            b"darkolivegreen" => Some(NamedColor::DarkOliveGreen),
            b"darkorange" => Some(NamedColor::DarkOrange),
            b"darkorchid" => Some(NamedColor::DarkOrchid),
            b"darkred" => Some(NamedColor::DarkRed),
            b"darksalmon" => Some(NamedColor::DarkSalmon),
            b"darkseagreen" => Some(NamedColor::DarkSeaGreen),
            b"darkslateblue" => Some(NamedColor::DarkSlateBlue),
            b"darkslategray" => Some(NamedColor::DarkSlateGray),
            b"darkslategrey" => Some(NamedColor::DarkSlateGray),
            b"darkturquoise" => Some(NamedColor::DarkTurquoise),
            b"darkviolet" => Some(NamedColor::DarkViolet),
            b"deeppink" => Some(NamedColor::DeepPink),
            b"deepskyblue" => Some(NamedColor::DeepSkyBlue),
            b"dimgray" => Some(NamedColor::DimGray),
            b"dimgrey" => Some(NamedColor::DimGray),
            b"dodgerblue" => Some(NamedColor::DodgerBlue),

            // F
            b"firebrick" => Some(NamedColor::FireBrick),
            b"floralwhite" => Some(NamedColor::FloralWhite),
            b"forestgreen" => Some(NamedColor::ForestGreen),
            b"fuchsia" => Some(NamedColor::Fuchsia),

            // G
            b"gainsboro" => Some(NamedColor::Gainsboro),
            b"ghostwhite" => Some(NamedColor::GhostWhite),
            b"gold" => Some(NamedColor::Gold),
            b"goldenrod" => Some(NamedColor::GoldenRod),
            b"gray" => Some(NamedColor::Gray),
            b"grey" => Some(NamedColor::Gray),
            b"green" => Some(NamedColor::Green),
            b"greenyellow" => Some(NamedColor::GreenYellow),

            // H
            b"honeydew" => Some(NamedColor::HoneyDew),
            b"hotpink" => Some(NamedColor::HotPink),

            // I
            b"indianred" => Some(NamedColor::IndianRed),
            b"indigo" => Some(NamedColor::Indigo),
            b"ivory" => Some(NamedColor::Ivory),

            // K
            b"khaki" => Some(NamedColor::Khaki),

            // L
            b"lavender" => Some(NamedColor::Lavender),
            b"lavenderblush" => Some(NamedColor::LavenderBlush),
            b"lawngreen" => Some(NamedColor::LawnGreen),
            b"lemonchiffon" => Some(NamedColor::LemonChiffon),
            b"lightblue" => Some(NamedColor::LightBlue),
            b"lightcoral" => Some(NamedColor::LightCoral),
            b"lightcyan" => Some(NamedColor::LightCyan),
            b"lightgoldenrodyellow" => Some(NamedColor::LightGoldenRodYellow),
            b"lightgray" => Some(NamedColor::LightGray),
            b"lightgrey" => Some(NamedColor::LightGray),
            b"lightgreen" => Some(NamedColor::LightGreen),
            b"lightpink" => Some(NamedColor::LightPink),
            b"lightsalmon" => Some(NamedColor::LightSalmon),
            b"lightseagreen" => Some(NamedColor::LightSeaGreen),
            b"lightskyblue" => Some(NamedColor::LightSkyBlue),
            b"lightslategray" => Some(NamedColor::LightSlateGray),
            b"lightslategrey" => Some(NamedColor::LightSlateGray),
            b"lightsteelblue" => Some(NamedColor::LightSteelBlue),
            b"lightyellow" => Some(NamedColor::LightYellow),
            b"lime" => Some(NamedColor::Lime),
            b"limegreen" => Some(NamedColor::LimeGreen),
            b"linen" => Some(NamedColor::Linen),

            // M
            b"magenta" => Some(NamedColor::Magenta),
            b"maroon" => Some(NamedColor::Maroon),
            b"mediumaquamarine" => Some(NamedColor::MediumAquaMarine),
            b"mediumblue" => Some(NamedColor::MediumBlue),
            b"mediumorchid" => Some(NamedColor::MediumOrchid),
            b"mediumpurple" => Some(NamedColor::MediumPurple),
            b"mediumseagreen" => Some(NamedColor::MediumSeaGreen),
            b"mediumslateblue" => Some(NamedColor::MediumSlateBlue),
            b"mediumspringgreen" => Some(NamedColor::MediumSpringGreen),
            b"mediumturquoise" => Some(NamedColor::MediumTurquoise),
            b"mediumvioletred" => Some(NamedColor::MediumVioletRed),
            b"midnightblue" => Some(NamedColor::MidnightBlue),
            b"mintcream" => Some(NamedColor::MintCream),
            b"mistyrose" => Some(NamedColor::MistyRose),
            b"moccasin" => Some(NamedColor::Moccasin),

            // N
            b"navajowhite" => Some(NamedColor::NavajoWhite),
            b"navy" => Some(NamedColor::Navy),

            // O
            b"oldlace" => Some(NamedColor::OldLace),
            b"olive" => Some(NamedColor::Olive),
            b"olivedrab" => Some(NamedColor::OliveDrab),
            b"orange" => Some(NamedColor::Orange),
            b"orangered" => Some(NamedColor::OrangeRed),
            b"orchid" => Some(NamedColor::Orchid),

            // P
            b"palegoldenrod" => Some(NamedColor::PaleGoldenRod),
            b"palegreen" => Some(NamedColor::PaleGreen),
            b"paleturquoise" => Some(NamedColor::PaleTurquoise),
            b"palevioletred" => Some(NamedColor::PaleVioletRed),
            b"papayawhip" => Some(NamedColor::PapayaWhip),
            b"peachpuff" => Some(NamedColor::PeachPuff),
            b"peru" => Some(NamedColor::Peru),
            b"pink" => Some(NamedColor::Pink),
            b"plum" => Some(NamedColor::Plum),
            b"powderblue" => Some(NamedColor::PowderBlue),
            b"purple" => Some(NamedColor::Purple),

            // R
            b"rebeccapurple" => Some(NamedColor::RebeccaPurple),
            b"red" => Some(NamedColor::Red),
            b"rosybrown" => Some(NamedColor::RosyBrown),
            b"royalblue" => Some(NamedColor::RoyalBlue),

            // S
            b"saddlebrown" => Some(NamedColor::SaddleBrown),
            b"salmon" => Some(NamedColor::Salmon),
            b"sandybrown" => Some(NamedColor::SandyBrown),
            b"seagreen" => Some(NamedColor::SeaGreen),
            b"seashell" => Some(NamedColor::SeaShell),
            b"sienna" => Some(NamedColor::Sienna),
            b"silver" => Some(NamedColor::Silver),
            b"skyblue" => Some(NamedColor::SkyBlue),
            b"slateblue" => Some(NamedColor::SlateBlue),
            b"slategray" => Some(NamedColor::SlateGray),
            b"slategrey" => Some(NamedColor::SlateGray),
            b"snow" => Some(NamedColor::Snow),
            b"springgreen" => Some(NamedColor::SpringGreen),
            b"steelblue" => Some(NamedColor::SteelBlue),

            // T
            b"tan" => Some(NamedColor::Tan),
            b"teal" => Some(NamedColor::Teal),
            b"thistle" => Some(NamedColor::Thistle),
            b"tomato" => Some(NamedColor::Tomato),
            b"turquoise" => Some(NamedColor::Turquoise),

            // V
            b"violet" => Some(NamedColor::Violet),

            // W
            b"wheat" => Some(NamedColor::Wheat),
            b"white" => Some(NamedColor::White),
            b"whitesmoke" => Some(NamedColor::WhiteSmoke),

            // Y
            b"yellow" => Some(NamedColor::Yellow),
            b"yellowgreen" => Some(NamedColor::YellowGreen),
            _ => None,
        }
    }
}

impl Parseable for HexColor {
    fn parse(value: &str) -> Option<Self> {
        let hex = value.trim_start_matches('#');
        if hex.len() == 6
            && let Ok(parsed) = u32::from_str_radix(hex, 16)
        {
            let r = ((parsed >> 16) & 0xFF) as u8;
            let g = ((parsed >> 8) & 0xFF) as u8;
            let b = (parsed & 0xFF) as u8;
            return Some([r, g, b]);
        }

        None
    }
}

impl Parseable for SRGBAColor {
    fn parse(value: &str) -> Option<Self> {
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
                return Some(Self::RGB(r, g, b));
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
                return Some(Self::RGBA(r, g, b, a));
            }
        }

        None
    }
}

impl Parseable for CIELAB {
    fn parse(value: &str) -> Option<Self> {
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
                return None;
            }

            let l = if parts[0].contains('%') {
                Unit::resolve_percentage(parts[0])?
            } else if parts[0].eq_ignore_ascii_case("none") {
                0.0
            } else {
                parts[0].parse::<f32>().ok()?
            };

            let a = if parts[1].contains('%') {
                (Unit::resolve_percentage(parts[1])? / 100.0) * 125.0
            } else if parts[1].eq_ignore_ascii_case("none") {
                0.0
            } else {
                parts[1].parse::<f32>().ok()?
            };

            let b = if parts[2].contains('%') {
                (Unit::resolve_percentage(parts[2])? / 100.0) * 125.0
            } else if parts[2].eq_ignore_ascii_case("none") {
                0.0
            } else {
                parts[2].parse::<f32>().ok()?
            };

            return Some(Self::Lab(l, a, b));
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
                return None;
            }

            let l = if parts[0].contains('%') {
                Unit::resolve_percentage(parts[0])?
            } else if parts[0].eq_ignore_ascii_case("none") {
                0.0
            } else {
                parts[0].parse::<f32>().ok()?
            };

            let c = if parts[1].contains('%') {
                (Unit::resolve_percentage(parts[1])? / 100.0) * 150.0
            } else if parts[1].eq_ignore_ascii_case("none") {
                0.0
            } else {
                parts[1].parse::<f32>().ok()?
            };

            let h = if let Some(angle) = Angle::parse(parts[2]) {
                angle.to_degrees()
            } else if parts[2].eq_ignore_ascii_case("none") {
                0.0
            } else {
                parts[2].parse::<f32>().ok()?
            };

            return Some(Self::Lch(l, c, h));
        }
        None
    }
}

impl Parseable for Oklab {
    fn parse(value: &str) -> Option<Self> {
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
                Unit::resolve_percentage(parts[0])? / 100.0
            } else if parts[0].eq_ignore_ascii_case("none") {
                0.0
            } else {
                parts[0].parse::<f32>().ok()?
            };

            let c = if parts[1].contains('%') {
                Unit::resolve_percentage(parts[1])? / 100.0
            } else if parts[1].eq_ignore_ascii_case("none") {
                0.0
            } else {
                parts[1].parse::<f32>().ok()?
            };

            let h = if let Some(angle) = Angle::parse(parts[2]) {
                angle.to_degrees()
            } else if parts[2].eq_ignore_ascii_case("none") {
                0.0
            } else {
                parts[2].parse::<f32>().ok()?
            };

            return Some(Self::Oklch(l, c, h));
        } else if value.starts_with("oklab(") && value.ends_with(')') {
            let content = &value[6..value.len() - 1];

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
                Unit::resolve_percentage(parts[0])? / 100.0
            } else if parts[0].eq_ignore_ascii_case("none") {
                0.0
            } else {
                parts[0].parse::<f32>().ok()?
            };

            let a = if parts[1].contains('%') {
                Unit::resolve_percentage(parts[1])? / 100.0 * 0.4
            } else if parts[1].eq_ignore_ascii_case("none") {
                0.0
            } else {
                parts[1].parse::<f32>().ok()?
            };

            let b = if parts[2].contains('%') {
                Unit::resolve_percentage(parts[2])? / 100.0 * 0.4
            } else if parts[2].eq_ignore_ascii_case("none") {
                0.0
            } else {
                parts[2].parse::<f32>().ok()?
            };

            return Some(Self::Oklab(l, a, b));
        }

        None
    }
}

impl Parseable for FunctionColor {
    fn parse(value: &str) -> Option<Self> {
        if let Some(srgba) = SRGBAColor::parse(value) {
            return Some(Self::SRGBA(srgba));
        }

        if let Some(cielab) = CIELAB::parse(value) {
            return Some(Self::CIELAB(cielab));
        }

        if let Some(oklab) = Oklab::parse(value) {
            return Some(Self::Oklab(oklab));
        }

        None
    }
}

impl Parseable for Color {
    fn parse(value: &str) -> Option<Self> {
        let s = value.trim();

        if s.eq_ignore_ascii_case("currentColor") {
            return Some(Self::CurrentColor);
        } else if s.eq_ignore_ascii_case("transparent") {
            return Some(Self::Transparent);
        }

        if let Some(global) = Global::parse(s) {
            return Some(Self::Global(global));
        }

        if let Some(hex_color) = HexColor::parse(s) {
            return Some(Self::Hex(hex_color));
        }

        if let Some(function_color) = FunctionColor::parse(s) {
            return Some(Self::Functional(function_color));
        }

        if let Some(system_color) = SystemColor::parse(s) {
            return Some(Self::System(system_color));
        }

        if let Some(named_color) = NamedColor::parse(s) {
            return Some(Self::Named(named_color));
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use crate::types::{Parseable, color::HexColor};

    use super::*;

    #[test]
    fn test_system_color_parsing() {
        assert_eq!(
            SystemColor::parse("highlight"),
            Some(SystemColor::Highlight)
        );
        assert_eq!(SystemColor::parse("unknowncolor"), None);
    }

    #[test]
    fn test_named_color_parsing() {
        assert_eq!(
            NamedColor::parse("rebeCCapurPLE"),
            Some(NamedColor::RebeccaPurple)
        );
        assert_eq!(NamedColor::parse("invalidcolor"), None);
    }

    #[test]
    fn test_color_hex_parsing() {
        assert_eq!(HexColor::parse("#FF5733"), Some([255, 87, 51]));
        assert_eq!(HexColor::parse("#ZZZZZZ"), None);
    }

    #[test]
    fn test_color_srgba_parsing() {
        assert_eq!(
            SRGBAColor::parse("rgb(255, 0, 0)"),
            Some(SRGBAColor::RGB(255, 0, 0))
        );
        assert_eq!(
            SRGBAColor::parse("rgba(0, 255, 0, 0.5)"),
            Some(SRGBAColor::RGBA(0, 255, 0, 0.5))
        );
        assert_eq!(SRGBAColor::parse("invalid"), None);
    }

    #[test]
    fn test_color_cielab_parsing() {
        assert_eq!(
            CIELAB::parse("lab(50, 20, 30)"),
            Some(CIELAB::Lab(50.0, 20.0, 30.0))
        );
        assert_eq!(
            CIELAB::parse("lch(60, 40, 120)"),
            Some(CIELAB::Lch(60.0, 40.0, 120.0))
        );
        assert_eq!(CIELAB::parse("invalid"), None);
    }

    #[test]
    fn test_color_oklab_parsing() {
        assert_eq!(
            Oklab::parse("oklch(70%, 50%, 180)"),
            Some(Oklab::Oklch(0.7, 0.5, 180.0))
        );
        assert_eq!(
            Oklab::parse("oklab(0.6, 0.1, -0.1)"),
            Some(Oklab::Oklab(0.6, 0.1, -0.1))
        );

        assert_eq!(Oklab::parse("invalid"), None);
    }
}

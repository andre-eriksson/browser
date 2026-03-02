//! Named and system colors (e.g., "red", "blue", "LinkText", "CanvasText")

use css_cssom::{ComponentValue, CssTokenKind};
use strum::EnumString;

/// System colors defined in CSS specifications.
///
/// These are colors that correspond to the user's operating system or browser theme settings,
/// for now only a fixed set of colors is provided.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, EnumString)]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
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

impl SystemColor {
    /// Converts the SystemColor to its hexadecimal string representation, or returns None if the color is not recognized.
    pub fn to_hex(self) -> Option<&'static str> {
        match self {
            SystemColor::AccentColor => Some("#0078D7"),
            SystemColor::AccentColorText => Some("#FFFFFF"),
            SystemColor::ActiveText => Some("#0000FF"),
            SystemColor::ButtonBorder => Some("#A9A9A9"),
            SystemColor::ButtonFace => Some("#F0F0F0"),
            SystemColor::ButtonText => Some("#000000"),
            SystemColor::Canvas => Some("#FFFFFF"),
            SystemColor::CanvasText => Some("#000000"),
            SystemColor::Field => Some("#FFFFFF"),
            SystemColor::FieldText => Some("#000000"),
            SystemColor::GrayText => Some("#A9A9A9"),
            SystemColor::Highlight => Some("#3399FF"),
            SystemColor::HighlightText => Some("#FFFFFF"),
            SystemColor::LinkText => Some("#0000FF"),
            SystemColor::Mark => Some("#FFFF00"),
            SystemColor::MarkText => Some("#000000"),
            SystemColor::SelectedItem => Some("#3399FF"),
            SystemColor::SelectedItemText => Some("#FFFFFF"),
            SystemColor::VisitedText => Some("#800080"),
        }
    }
}

impl TryFrom<&[ComponentValue]> for SystemColor {
    type Error = String;

    fn try_from(value: &[ComponentValue]) -> Result<Self, Self::Error> {
        for cv in value {
            match cv {
                ComponentValue::Token(token) => match &token.kind {
                    CssTokenKind::Ident(ident) => {
                        return ident
                            .parse()
                            .map_err(|_| format!("Invalid system color: '{}'", ident));
                    }
                    _ => continue,
                },
                _ => continue,
            }
        }

        Err(format!("No valid system color token found in component values: {:?}", value))
    }
}

/// Named colors defined in CSS specifications.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, EnumString)]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
pub enum NamedColor {
    /// #F0F8FF
    AliceBlue,

    /// #FAEBD7
    AntiqueWhite,

    /// #00FFFF
    Aqua,

    /// #7FFFD4
    Aquamarine,

    /// #F0FFFF
    Azure,

    /// #F5F5DC
    Beige,

    /// #FFE4C4
    Bisque,

    /// #000000
    Black,

    /// #FFEBCD
    BlanchedAlmond,

    /// #0000FF
    Blue,

    /// #8A2BE2
    BlueViolet,

    /// #A52A2A
    Brown,

    /// #DEB887
    BurlyWood,

    /// #5F9EA0
    CadetBlue,

    /// #7FFF00
    Chartreuse,

    /// #D2691E
    Chocolate,

    /// #FF7F50
    Coral,

    /// #6495ED
    CornflowerBlue,

    /// #FFF8DC
    Cornsilk,

    /// #DC143C
    Crimson,

    /// #00FFFF (alias of Aqua)
    Cyan,

    /// #00008B
    DarkBlue,

    /// #008B8B
    DarkCyan,

    /// #B8860B
    DarkGoldenRod,

    /// #A9A9A9
    DarkGray,

    /// #A9A9A9 (alias of DarkGray)
    DarkGrey,

    /// #006400
    DarkGreen,

    /// #BDB76B
    DarkKhaki,

    /// #8B008B
    DarkMagenta,

    /// #556B2F
    DarkOliveGreen,

    /// #FF8C00
    DarkOrange,

    /// #9932CC
    DarkOrchid,

    /// #8B0000
    DarkRed,

    /// #E9967A
    DarkSalmon,

    /// #8FBC8F
    DarkSeaGreen,

    /// #483D8B
    DarkSlateBlue,

    /// #2F4F4F
    DarkSlateGray,

    /// #2F4F4F (alias of DarkSlateGray)
    DarkSlateGrey,

    /// #00CED1
    DarkTurquoise,

    /// #9400D3
    DarkViolet,

    /// #FF1493
    DeepPink,

    /// #00BFFF
    DeepSkyBlue,

    /// #696969
    DimGray,
    /// #696969
    DimGrey,

    /// #1E90FF
    DodgerBlue,

    /// #B22222
    FireBrick,

    /// #FFFAF0
    FloralWhite,

    /// #228B22
    ForestGreen,

    /// #FF00FF
    Fuchsia,

    /// #DCDCDC
    Gainsboro,

    /// #F8F8FF
    GhostWhite,

    /// #FFD700
    Gold,

    /// #DAA520
    GoldenRod,

    /// #808080
    Gray,

    /// #808080 (alias of Gray)
    Grey,

    /// #008000
    Green,

    /// #ADFF2F
    GreenYellow,

    /// #F0FFF0
    HoneyDew,

    /// #FF69B4
    HotPink,

    /// #CD5C5C
    IndianRed,

    /// #4B0082
    Indigo,

    /// #FFFFF0
    Ivory,

    /// #F0E68C
    Khaki,

    /// #E6E6FA
    Lavender,

    /// #FFF0F5
    LavenderBlush,

    /// #7CFC00
    LawnGreen,

    /// #FFFACD
    LemonChiffon,

    /// #ADD8E6
    LightBlue,

    /// #F08080
    LightCoral,

    /// #E0FFFF
    LightCyan,

    /// #FAFAD2
    LightGoldenRodYellow,

    /// #D3D3D3
    LightGray,

    /// #D3D3D3 (alias of LightGray)
    LightGrey,

    /// #90EE90
    LightGreen,

    /// #FFB6C1
    LightPink,

    /// #FFA07A
    LightSalmon,

    /// #20B2AA
    LightSeaGreen,

    /// #87CEFA
    LightSkyBlue,

    /// #708090
    LightSlateGray,

    /// #708090 (alias of LightSlateGray)
    LightSlateGrey,

    /// #B0C4DE
    LightSteelBlue,

    /// #FFFFE0
    LightYellow,

    /// #00FF00
    Lime,

    /// #32CD32
    LimeGreen,

    /// #FAF0E6
    Linen,

    /// #FF00FF
    Magenta,

    /// #800000
    Maroon,

    /// #66CDAA
    MediumAquaMarine,

    /// #0000CD
    MediumBlue,

    /// #BA55D3
    MediumOrchid,

    /// #9370DB
    MediumPurple,

    /// #3CB371
    MediumSeaGreen,

    /// #7B68EE
    MediumSlateBlue,

    /// #00FA9A
    MediumSpringGreen,

    /// #48D1CC
    MediumTurquoise,

    /// #C71585
    MediumVioletRed,

    /// #191970
    MidnightBlue,

    /// #F5FFFA
    MintCream,

    /// #FFE4E1
    MistyRose,

    /// #FFE4B5
    Moccasin,

    /// #FFDEAD
    NavajoWhite,

    /// #000080
    Navy,

    /// #FDF5E6
    OldLace,

    /// #808000
    Olive,

    /// #6B8E23
    OliveDrab,

    /// #FFA500
    Orange,

    /// #FF4500
    OrangeRed,

    /// #DA70D6
    Orchid,

    /// #EEE8AA
    PaleGoldenRod,

    /// #98FB98
    PaleGreen,

    /// #AFEEEE
    PaleTurquoise,

    /// #DB7093
    PaleVioletRed,

    /// #FFEFD5
    PapayaWhip,

    /// #FFDAB9
    PeachPuff,

    /// #CD853F
    Peru,

    /// #FFC0CB
    Pink,

    /// #DDA0DD
    Plum,

    /// #B0E0E6
    PowderBlue,

    /// #800080
    Purple,

    /// #663399
    RebeccaPurple,

    /// #FF0000
    Red,

    /// #BC8F8F
    RosyBrown,

    /// #4169E1
    RoyalBlue,

    /// #8B4513
    SaddleBrown,

    /// #FA8072
    Salmon,

    /// #F4A460
    SandyBrown,

    /// #2E8B57
    SeaGreen,

    /// #FFF5EE
    SeaShell,

    /// #A0522D
    Sienna,

    /// #C0C0C0
    Silver,

    /// #87CEEB
    SkyBlue,

    /// #6A5ACD
    SlateBlue,

    /// #708090
    SlateGray,

    /// #708090 (alias of SlateGray)
    SlateGrey,

    /// #FFFAFA
    Snow,

    /// #00FF7F
    SpringGreen,

    /// #4682B4
    SteelBlue,

    /// #D2B48C
    Tan,

    /// #008080
    Teal,

    /// #D8BFD8
    Thistle,

    /// #FF6347
    Tomato,

    /// #40E0D0
    Turquoise,

    /// #EE82EE
    Violet,

    /// #F5DEB3
    Wheat,

    /// #FFFFFF
    White,

    /// #F5F5F5
    WhiteSmoke,

    /// #FFFF00
    Yellow,

    /// #9ACD32
    YellowGreen,
}

impl NamedColor {
    /// Converts the NamedColor to its hexadecimal string representation, or returns None if the color is not recognized.
    pub fn to_hex(self) -> Option<&'static str> {
        match self {
            NamedColor::AliceBlue => Some("#F0F8FF"),
            NamedColor::AntiqueWhite => Some("#FAEBD7"),
            NamedColor::Aqua | NamedColor::Cyan => Some("#00FFFF"),
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
            NamedColor::DarkBlue => Some("#00008B"),
            NamedColor::DarkCyan => Some("#008B8B"),
            NamedColor::DarkGoldenRod => Some("#B8860B"),
            NamedColor::DarkGray | NamedColor::DarkGrey => Some("#A9A9A9"),
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
            NamedColor::DarkSlateGray | NamedColor::DarkSlateGrey => Some("#2F4F4F"),
            NamedColor::DarkTurquoise => Some("#00CED1"),
            NamedColor::DarkViolet => Some("#9400D3"),
            NamedColor::DeepPink => Some("#FF1493"),
            NamedColor::DeepSkyBlue => Some("#00BFFF"),
            NamedColor::DimGray | NamedColor::DimGrey => Some("#696969"),
            NamedColor::DodgerBlue => Some("#1E90FF"),
            NamedColor::FireBrick => Some("#B22222"),
            NamedColor::FloralWhite => Some("#FFFAF0"),
            NamedColor::ForestGreen => Some("#228B22"),
            NamedColor::Fuchsia => Some("#FF00FF"),
            NamedColor::Gainsboro => Some("#DCDCDC"),
            NamedColor::GhostWhite => Some("#F8F8FF"),
            NamedColor::Gold => Some("#FFD700"),
            NamedColor::GoldenRod => Some("#DAA520"),
            NamedColor::Gray | NamedColor::Grey => Some("#808080"),
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
            NamedColor::LightGray | NamedColor::LightGrey => Some("#D3D3D3"),
            NamedColor::LightGreen => Some("#90EE90"),
            NamedColor::LightPink => Some("#FFB6C1"),
            NamedColor::LightSalmon => Some("#FFA07A"),
            NamedColor::LightSeaGreen => Some("#20B2AA"),
            NamedColor::LightSkyBlue => Some("#87CEFA"),
            NamedColor::LightSlateGray | NamedColor::LightSlateGrey => Some("#778899"),
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
            NamedColor::SlateGray | NamedColor::SlateGrey => Some("#708090"),
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
}

impl TryFrom<&[ComponentValue]> for NamedColor {
    type Error = String;

    fn try_from(value: &[ComponentValue]) -> Result<Self, Self::Error> {
        for cv in value {
            match cv {
                ComponentValue::Token(token) => match &token.kind {
                    CssTokenKind::Ident(ident) => {
                        return ident
                            .parse()
                            .map_err(|_| format!("Invalid named color: '{}'", ident));
                    }
                    _ => continue,
                },
                _ => continue,
            }
        }

        Err(format!("No valid named color token found in component values: {:?}", value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_system_color() {
        let color: SystemColor = "accentColor".parse().unwrap();
        assert_eq!(color, SystemColor::AccentColor);

        let color: SystemColor = "LinkText".parse().unwrap();
        assert_eq!(color, SystemColor::LinkText);

        let color = "invalidColor".parse::<SystemColor>();
        assert!(color.is_err());
    }

    #[test]
    fn parse_named_color() {
        let color: NamedColor = "AliceBlue".parse().unwrap();
        assert_eq!(color, NamedColor::AliceBlue);

        let color: NamedColor = "rebeccapurple".parse().unwrap();
        assert_eq!(color, NamedColor::RebeccaPurple);

        let color = "invalidColor".parse::<NamedColor>();
        assert!(color.is_err());
    }
}

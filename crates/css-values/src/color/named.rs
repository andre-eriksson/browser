/// Named colors defined in CSS specifications.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
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

    /// #00008B
    DarkBlue,

    /// #008B8B
    DarkCyan,

    /// #B8860B
    DarkGoldenRod,

    /// #A9A9A9
    DarkGray,

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
    pub fn from_str_insensitive(s: &str) -> Option<Self> {
        let bytes = s.as_bytes();
        match bytes.len() {
            3 => {
                match [
                    bytes[0].to_ascii_lowercase(),
                    bytes[1].to_ascii_lowercase(),
                    bytes[2].to_ascii_lowercase(),
                ] {
                    [b'r', b'e', b'd'] => Some(Self::Red),
                    [b't', b'a', b'n'] => Some(Self::Tan),
                    _ => None,
                }
            }
            4 => {
                match [
                    bytes[0].to_ascii_lowercase(),
                    bytes[1].to_ascii_lowercase(),
                    bytes[2].to_ascii_lowercase(),
                    bytes[3].to_ascii_lowercase(),
                ] {
                    [b'a', b'q', b'u', b'a'] => Some(Self::Aqua),
                    [b'b', b'l', b'u', b'e'] => Some(Self::Blue),
                    [b'c', b'y', b'a', b'n'] => Some(Self::Aqua),
                    [b'g', b'o', b'l', b'd'] => Some(Self::Gold),
                    [b'g', b'r', b'a', b'y'] => Some(Self::Gray),
                    [b'g', b'r', b'e', b'y'] => Some(Self::Gray),
                    [b'l', b'i', b'm', b'e'] => Some(Self::Lime),
                    [b'n', b'a', b'v', b'y'] => Some(Self::Navy),
                    [b'p', b'e', b'r', b'u'] => Some(Self::Peru),
                    [b'p', b'i', b'n', b'k'] => Some(Self::Pink),
                    [b'p', b'l', b'u', b'm'] => Some(Self::Plum),
                    [b's', b'n', b'o', b'w'] => Some(Self::Snow),
                    [b't', b'e', b'a', b'l'] => Some(Self::Teal),
                    _ => None,
                }
            }
            5 => {
                match [
                    bytes[0].to_ascii_lowercase(),
                    bytes[1].to_ascii_lowercase(),
                    bytes[2].to_ascii_lowercase(),
                    bytes[3].to_ascii_lowercase(),
                    bytes[4].to_ascii_lowercase(),
                ] {
                    [b'a', b'z', b'u', b'r', b'e'] => Some(Self::Azure),
                    [b'b', b'e', b'i', b'g', b'e'] => Some(Self::Beige),
                    [b'b', b'l', b'a', b'c', b'k'] => Some(Self::Black),
                    [b'b', b'r', b'o', b'w', b'n'] => Some(Self::Brown),
                    [b'c', b'o', b'r', b'a', b'l'] => Some(Self::Coral),
                    [b'g', b'r', b'e', b'e', b'n'] => Some(Self::Green),
                    [b'i', b'v', b'o', b'r', b'y'] => Some(Self::Ivory),
                    [b'k', b'h', b'a', b'k', b'i'] => Some(Self::Khaki),
                    [b'l', b'i', b'n', b'e', b'n'] => Some(Self::Linen),
                    [b'o', b'l', b'i', b'v', b'e'] => Some(Self::Olive),
                    [b'w', b'h', b'e', b'a', b't'] => Some(Self::Wheat),
                    [b'w', b'h', b'i', b't', b'e'] => Some(Self::White),
                    _ => None,
                }
            }
            6 => {
                match [
                    bytes[0].to_ascii_lowercase(),
                    bytes[1].to_ascii_lowercase(),
                    bytes[2].to_ascii_lowercase(),
                    bytes[3].to_ascii_lowercase(),
                    bytes[4].to_ascii_lowercase(),
                    bytes[5].to_ascii_lowercase(),
                ] {
                    [b'b', b'i', b's', b'q', b'u', b'e'] => Some(Self::Bisque),
                    [b'i', b'n', b'd', b'i', b'g', b'o'] => Some(Self::Indigo),
                    [b'm', b'a', b'r', b'o', b'o', b'n'] => Some(Self::Maroon),
                    [b'o', b'r', b'a', b'n', b'g', b'e'] => Some(Self::Orange),
                    [b'o', b'r', b'c', b'h', b'i', b'd'] => Some(Self::Orchid),
                    [b'p', b'u', b'r', b'p', b'l', b'e'] => Some(Self::Purple),
                    [b's', b'a', b'l', b'm', b'o', b'n'] => Some(Self::Salmon),
                    [b's', b'i', b'e', b'n', b'n', b'a'] => Some(Self::Sienna),
                    [b's', b'i', b'l', b'v', b'e', b'r'] => Some(Self::Silver),
                    [b't', b'o', b'm', b'a', b't', b'o'] => Some(Self::Tomato),
                    [b'v', b'i', b'o', b'l', b'e', b't'] => Some(Self::Violet),
                    [b'y', b'e', b'l', b'l', b'o', b'w'] => Some(Self::Yellow),
                    _ => None,
                }
            }
            7 => {
                match [
                    bytes[0].to_ascii_lowercase(),
                    bytes[1].to_ascii_lowercase(),
                    bytes[2].to_ascii_lowercase(),
                    bytes[3].to_ascii_lowercase(),
                    bytes[4].to_ascii_lowercase(),
                    bytes[5].to_ascii_lowercase(),
                    bytes[6].to_ascii_lowercase(),
                ] {
                    [b'c', b'r', b'i', b'm', b's', b'o', b'n'] => Some(Self::Crimson),
                    [b'd', b'a', b'r', b'k', b'r', b'e', b'd'] => Some(Self::DarkRed),
                    [b'd', b'i', b'm', b'g', b'r', b'a', b'y'] => Some(Self::DimGray),
                    [b'd', b'i', b'm', b'g', b'r', b'e', b'y'] => Some(Self::DimGray),
                    [b'f', b'u', b'c', b'h', b's', b'i', b'a'] => Some(Self::Fuchsia),
                    [b'h', b'o', b't', b'p', b'i', b'n', b'k'] => Some(Self::HotPink),
                    [b'm', b'a', b'g', b'e', b'n', b't', b'a'] => Some(Self::Magenta),
                    [b'o', b'l', b'd', b'l', b'a', b'c', b'e'] => Some(Self::OldLace),
                    [b's', b'k', b'y', b'b', b'l', b'u', b'e'] => Some(Self::SkyBlue),
                    [b't', b'h', b'i', b's', b't', b'l', b'e'] => Some(Self::Thistle),
                    _ => None,
                }
            }
            8 => {
                match [
                    bytes[0].to_ascii_lowercase(),
                    bytes[1].to_ascii_lowercase(),
                    bytes[2].to_ascii_lowercase(),
                    bytes[3].to_ascii_lowercase(),
                    bytes[4].to_ascii_lowercase(),
                    bytes[5].to_ascii_lowercase(),
                    bytes[6].to_ascii_lowercase(),
                    bytes[7].to_ascii_lowercase(),
                ] {
                    [b'c', b'o', b'r', b'n', b's', b'i', b'l', b'k'] => Some(Self::Cornsilk),
                    [b'd', b'a', b'r', b'k', b'b', b'l', b'u', b'e'] => Some(Self::DarkBlue),
                    [b'd', b'a', b'r', b'k', b'c', b'y', b'a', b'n'] => Some(Self::DarkCyan),
                    [b'd', b'a', b'r', b'k', b'g', b'r', b'a', b'y'] => Some(Self::DarkGray),
                    [b'd', b'a', b'r', b'k', b'g', b'r', b'e', b'y'] => Some(Self::DarkGray),
                    [b'd', b'e', b'e', b'p', b'p', b'i', b'n', b'k'] => Some(Self::DeepPink),
                    [b'h', b'o', b'n', b'e', b'y', b'd', b'e', b'w'] => Some(Self::HoneyDew),
                    [b'l', b'a', b'v', b'e', b'n', b'd', b'e', b'r'] => Some(Self::Lavender),
                    [b'm', b'o', b'c', b'c', b'a', b's', b'i', b'n'] => Some(Self::Moccasin),
                    [b's', b'e', b'a', b'g', b'r', b'e', b'e', b'n'] => Some(Self::SeaGreen),
                    [b's', b'e', b'a', b's', b'h', b'e', b'l', b'l'] => Some(Self::SeaShell),
                    _ => None,
                }
            }
            9 => {
                match [
                    bytes[0].to_ascii_lowercase(),
                    bytes[1].to_ascii_lowercase(),
                    bytes[2].to_ascii_lowercase(),
                    bytes[3].to_ascii_lowercase(),
                    bytes[4].to_ascii_lowercase(),
                    bytes[5].to_ascii_lowercase(),
                    bytes[6].to_ascii_lowercase(),
                    bytes[7].to_ascii_lowercase(),
                    bytes[8].to_ascii_lowercase(),
                ] {
                    [b'a', b'l', b'i', b'c', b'e', b'b', b'l', b'u', b'e'] => Some(Self::AliceBlue),
                    [b'b', b'u', b'r', b'l', b'y', b'w', b'o', b'o', b'd'] => Some(Self::BurlyWood),
                    [b'c', b'a', b'd', b'e', b't', b'b', b'l', b'u', b'e'] => Some(Self::CadetBlue),
                    [b'c', b'h', b'o', b'c', b'o', b'l', b'a', b't', b'e'] => Some(Self::Chocolate),
                    [b'd', b'a', b'r', b'k', b'g', b'r', b'e', b'e', b'n'] => Some(Self::DarkGreen),
                    [b'd', b'a', b'r', b'k', b'k', b'h', b'a', b'k', b'i'] => Some(Self::DarkKhaki),
                    [b'f', b'i', b'r', b'e', b'b', b'r', b'i', b'c', b'k'] => Some(Self::FireBrick),
                    [b'g', b'a', b'i', b'n', b's', b'b', b'o', b'r', b'o'] => Some(Self::Gainsboro),
                    [b'g', b'o', b'l', b'd', b'e', b'n', b'r', b'o', b'd'] => Some(Self::GoldenRod),
                    [b'i', b'n', b'd', b'i', b'a', b'n', b'r', b'e', b'd'] => Some(Self::IndianRed),
                    [b'l', b'a', b'w', b'n', b'g', b'r', b'e', b'e', b'n'] => Some(Self::LawnGreen),
                    [b'l', b'i', b'g', b'h', b't', b'b', b'l', b'u', b'e'] => Some(Self::LightBlue),
                    [b'l', b'i', b'g', b'h', b't', b'c', b'y', b'a', b'n'] => Some(Self::LightCyan),
                    [b'l', b'i', b'g', b'h', b't', b'g', b'r', b'a', b'y'] => Some(Self::LightGray),
                    [b'l', b'i', b'g', b'h', b't', b'g', b'r', b'e', b'y'] => Some(Self::LightGray),
                    [b'l', b'i', b'g', b'h', b't', b'p', b'i', b'n', b'k'] => Some(Self::LightPink),
                    [b'l', b'i', b'm', b'e', b'g', b'r', b'e', b'e', b'n'] => Some(Self::LimeGreen),
                    [b'm', b'i', b'n', b't', b'c', b'r', b'e', b'a', b'm'] => Some(Self::MintCream),
                    [b'm', b'i', b's', b't', b'y', b'r', b'o', b's', b'e'] => Some(Self::MistyRose),
                    [b'o', b'l', b'i', b'v', b'e', b'd', b'r', b'a', b'b'] => Some(Self::OliveDrab),
                    [b'o', b'r', b'a', b'n', b'g', b'e', b'r', b'e', b'd'] => Some(Self::OrangeRed),
                    [b'p', b'a', b'l', b'e', b'g', b'r', b'e', b'e', b'n'] => Some(Self::PaleGreen),
                    [b'p', b'e', b'a', b'c', b'h', b'p', b'u', b'f', b'f'] => Some(Self::PeachPuff),
                    [b'r', b'o', b's', b'y', b'b', b'r', b'o', b'w', b'n'] => Some(Self::RosyBrown),
                    [b'r', b'o', b'y', b'a', b'l', b'b', b'l', b'u', b'e'] => Some(Self::RoyalBlue),
                    [b's', b'l', b'a', b't', b'e', b'b', b'l', b'u', b'e'] => Some(Self::SlateBlue),
                    [b's', b'l', b'a', b't', b'e', b'g', b'r', b'a', b'y'] => Some(Self::SlateGray),
                    [b's', b'l', b'a', b't', b'e', b'g', b'r', b'e', b'y'] => Some(Self::SlateGray),
                    [b's', b't', b'e', b'e', b'l', b'b', b'l', b'u', b'e'] => Some(Self::SteelBlue),
                    [b't', b'u', b'r', b'q', b'u', b'o', b'i', b's', b'e'] => Some(Self::Turquoise),
                    _ => None,
                }
            }
            10 => {
                match [
                    bytes[0].to_ascii_lowercase(),
                    bytes[1].to_ascii_lowercase(),
                    bytes[2].to_ascii_lowercase(),
                    bytes[3].to_ascii_lowercase(),
                    bytes[4].to_ascii_lowercase(),
                    bytes[5].to_ascii_lowercase(),
                    bytes[6].to_ascii_lowercase(),
                    bytes[7].to_ascii_lowercase(),
                    bytes[8].to_ascii_lowercase(),
                    bytes[9].to_ascii_lowercase(),
                ] {
                    [b'a', b'q', b'u', b'a', b'm', b'a', b'r', b'i', b'n', b'e'] => Some(Self::Aquamarine),
                    [b'b', b'l', b'u', b'e', b'v', b'i', b'o', b'l', b'e', b't'] => Some(Self::BlueViolet),
                    [b'c', b'h', b'a', b'r', b't', b'r', b'e', b'u', b's', b'e'] => Some(Self::Chartreuse),
                    [b'd', b'a', b'r', b'k', b'o', b'r', b'a', b'n', b'g', b'e'] => Some(Self::DarkOrange),
                    [b'd', b'a', b'r', b'k', b'o', b'r', b'c', b'h', b'i', b'd'] => Some(Self::DarkOrchid),
                    [b'd', b'a', b'r', b'k', b's', b'a', b'l', b'm', b'o', b'n'] => Some(Self::DarkSalmon),
                    [b'd', b'a', b'r', b'k', b'v', b'i', b'o', b'l', b'e', b't'] => Some(Self::DarkViolet),
                    [b'd', b'o', b'd', b'g', b'e', b'r', b'b', b'l', b'u', b'e'] => Some(Self::DodgerBlue),
                    [b'g', b'h', b'o', b's', b't', b'w', b'h', b'i', b't', b'e'] => Some(Self::GhostWhite),
                    [b'l', b'i', b'g', b'h', b't', b'c', b'o', b'r', b'a', b'l'] => Some(Self::LightCoral),
                    [b'l', b'i', b'g', b'h', b't', b'g', b'r', b'e', b'e', b'n'] => Some(Self::LightGreen),
                    [b'm', b'e', b'd', b'i', b'u', b'm', b'b', b'l', b'u', b'e'] => Some(Self::MediumBlue),
                    [b'p', b'a', b'p', b'a', b'y', b'a', b'w', b'h', b'i', b'p'] => Some(Self::PapayaWhip),
                    [b'p', b'o', b'w', b'd', b'e', b'r', b'b', b'l', b'u', b'e'] => Some(Self::PowderBlue),
                    [b's', b'a', b'n', b'd', b'y', b'b', b'r', b'o', b'w', b'n'] => Some(Self::SandyBrown),
                    [b'w', b'h', b'i', b't', b'e', b's', b'm', b'o', b'k', b'e'] => Some(Self::WhiteSmoke),
                    _ => None,
                }
            }
            11 => {
                match [
                    bytes[0].to_ascii_lowercase(),
                    bytes[1].to_ascii_lowercase(),
                    bytes[2].to_ascii_lowercase(),
                    bytes[3].to_ascii_lowercase(),
                    bytes[4].to_ascii_lowercase(),
                    bytes[5].to_ascii_lowercase(),
                    bytes[6].to_ascii_lowercase(),
                    bytes[7].to_ascii_lowercase(),
                    bytes[8].to_ascii_lowercase(),
                    bytes[9].to_ascii_lowercase(),
                    bytes[10].to_ascii_lowercase(),
                ] {
                    [
                        b'd',
                        b'a',
                        b'r',
                        b'k',
                        b'm',
                        b'a',
                        b'g',
                        b'e',
                        b'n',
                        b't',
                        b'a',
                    ] => Some(Self::DarkMagenta),
                    [
                        b'd',
                        b'e',
                        b'e',
                        b'p',
                        b's',
                        b'k',
                        b'y',
                        b'b',
                        b'l',
                        b'u',
                        b'e',
                    ] => Some(Self::DeepSkyBlue),
                    [
                        b'f',
                        b'l',
                        b'o',
                        b'r',
                        b'a',
                        b'l',
                        b'w',
                        b'h',
                        b'i',
                        b't',
                        b'e',
                    ] => Some(Self::FloralWhite),
                    [
                        b'f',
                        b'o',
                        b'r',
                        b'e',
                        b's',
                        b't',
                        b'g',
                        b'r',
                        b'e',
                        b'e',
                        b'n',
                    ] => Some(Self::ForestGreen),
                    [
                        b'g',
                        b'r',
                        b'e',
                        b'e',
                        b'n',
                        b'y',
                        b'e',
                        b'l',
                        b'l',
                        b'o',
                        b'w',
                    ] => Some(Self::GreenYellow),
                    [
                        b'l',
                        b'i',
                        b'g',
                        b'h',
                        b't',
                        b's',
                        b'a',
                        b'l',
                        b'm',
                        b'o',
                        b'n',
                    ] => Some(Self::LightSalmon),
                    [
                        b'l',
                        b'i',
                        b'g',
                        b'h',
                        b't',
                        b'y',
                        b'e',
                        b'l',
                        b'l',
                        b'o',
                        b'w',
                    ] => Some(Self::LightYellow),
                    [
                        b'n',
                        b'a',
                        b'v',
                        b'a',
                        b'j',
                        b'o',
                        b'w',
                        b'h',
                        b'i',
                        b't',
                        b'e',
                    ] => Some(Self::NavajoWhite),
                    [
                        b's',
                        b'a',
                        b'd',
                        b'd',
                        b'l',
                        b'e',
                        b'b',
                        b'r',
                        b'o',
                        b'w',
                        b'n',
                    ] => Some(Self::SaddleBrown),
                    [
                        b's',
                        b'p',
                        b'r',
                        b'i',
                        b'n',
                        b'g',
                        b'g',
                        b'r',
                        b'e',
                        b'e',
                        b'n',
                    ] => Some(Self::SpringGreen),
                    [
                        b'y',
                        b'e',
                        b'l',
                        b'l',
                        b'o',
                        b'w',
                        b'g',
                        b'r',
                        b'e',
                        b'e',
                        b'n',
                    ] => Some(Self::YellowGreen),
                    _ => None,
                }
            }
            12 => {
                match [
                    bytes[0].to_ascii_lowercase(),
                    bytes[1].to_ascii_lowercase(),
                    bytes[2].to_ascii_lowercase(),
                    bytes[3].to_ascii_lowercase(),
                    bytes[4].to_ascii_lowercase(),
                    bytes[5].to_ascii_lowercase(),
                    bytes[6].to_ascii_lowercase(),
                    bytes[7].to_ascii_lowercase(),
                    bytes[8].to_ascii_lowercase(),
                    bytes[9].to_ascii_lowercase(),
                    bytes[10].to_ascii_lowercase(),
                    bytes[11].to_ascii_lowercase(),
                ] {
                    [
                        b'a',
                        b'n',
                        b't',
                        b'i',
                        b'q',
                        b'u',
                        b'e',
                        b'w',
                        b'h',
                        b'i',
                        b't',
                        b'e',
                    ] => Some(Self::AntiqueWhite),
                    [
                        b'd',
                        b'a',
                        b'r',
                        b'k',
                        b's',
                        b'e',
                        b'a',
                        b'g',
                        b'r',
                        b'e',
                        b'e',
                        b'n',
                    ] => Some(Self::DarkSeaGreen),
                    [
                        b'l',
                        b'e',
                        b'm',
                        b'o',
                        b'n',
                        b'c',
                        b'h',
                        b'i',
                        b'f',
                        b'f',
                        b'o',
                        b'n',
                    ] => Some(Self::LemonChiffon),
                    [
                        b'l',
                        b'i',
                        b'g',
                        b'h',
                        b't',
                        b's',
                        b'k',
                        b'y',
                        b'b',
                        b'l',
                        b'u',
                        b'e',
                    ] => Some(Self::LightSkyBlue),
                    [
                        b'm',
                        b'e',
                        b'd',
                        b'i',
                        b'u',
                        b'm',
                        b'o',
                        b'r',
                        b'c',
                        b'h',
                        b'i',
                        b'd',
                    ] => Some(Self::MediumOrchid),
                    [
                        b'm',
                        b'e',
                        b'd',
                        b'i',
                        b'u',
                        b'm',
                        b'p',
                        b'u',
                        b'r',
                        b'p',
                        b'l',
                        b'e',
                    ] => Some(Self::MediumPurple),
                    [
                        b'm',
                        b'i',
                        b'd',
                        b'n',
                        b'i',
                        b'g',
                        b'h',
                        b't',
                        b'b',
                        b'l',
                        b'u',
                        b'e',
                    ] => Some(Self::MidnightBlue),
                    _ => None,
                }
            }
            13 => {
                match [
                    bytes[0].to_ascii_lowercase(),
                    bytes[1].to_ascii_lowercase(),
                    bytes[2].to_ascii_lowercase(),
                    bytes[3].to_ascii_lowercase(),
                    bytes[4].to_ascii_lowercase(),
                    bytes[5].to_ascii_lowercase(),
                    bytes[6].to_ascii_lowercase(),
                    bytes[7].to_ascii_lowercase(),
                    bytes[8].to_ascii_lowercase(),
                    bytes[9].to_ascii_lowercase(),
                    bytes[10].to_ascii_lowercase(),
                    bytes[11].to_ascii_lowercase(),
                    bytes[12].to_ascii_lowercase(),
                ] {
                    [
                        b'd',
                        b'a',
                        b'r',
                        b'k',
                        b'g',
                        b'o',
                        b'l',
                        b'd',
                        b'e',
                        b'n',
                        b'r',
                        b'o',
                        b'd',
                    ] => Some(Self::DarkGoldenRod),
                    [
                        b'd',
                        b'a',
                        b'r',
                        b'k',
                        b's',
                        b'l',
                        b'a',
                        b't',
                        b'e',
                        b'b',
                        b'l',
                        b'u',
                        b'e',
                    ] => Some(Self::DarkSlateBlue),
                    [
                        b'd',
                        b'a',
                        b'r',
                        b'k',
                        b's',
                        b'l',
                        b'a',
                        b't',
                        b'e',
                        b'g',
                        b'r',
                        b'a',
                        b'y',
                    ] => Some(Self::DarkSlateGray),
                    [
                        b'd',
                        b'a',
                        b'r',
                        b'k',
                        b's',
                        b'l',
                        b'a',
                        b't',
                        b'e',
                        b'g',
                        b'r',
                        b'e',
                        b'y',
                    ] => Some(Self::DarkSlateGray),
                    [
                        b'd',
                        b'a',
                        b'r',
                        b'k',
                        b't',
                        b'u',
                        b'r',
                        b'q',
                        b'u',
                        b'o',
                        b'i',
                        b's',
                        b'e',
                    ] => Some(Self::DarkTurquoise),
                    [
                        b'l',
                        b'a',
                        b'v',
                        b'e',
                        b'n',
                        b'd',
                        b'e',
                        b'r',
                        b'b',
                        b'l',
                        b'u',
                        b's',
                        b'h',
                    ] => Some(Self::LavenderBlush),
                    [
                        b'l',
                        b'i',
                        b'g',
                        b'h',
                        b't',
                        b's',
                        b'e',
                        b'a',
                        b'g',
                        b'r',
                        b'e',
                        b'e',
                        b'n',
                    ] => Some(Self::LightSeaGreen),
                    [
                        b'p',
                        b'a',
                        b'l',
                        b'e',
                        b'g',
                        b'o',
                        b'l',
                        b'd',
                        b'e',
                        b'n',
                        b'r',
                        b'o',
                        b'd',
                    ] => Some(Self::PaleGoldenRod),
                    [
                        b'p',
                        b'a',
                        b'l',
                        b'e',
                        b't',
                        b'u',
                        b'r',
                        b'q',
                        b'u',
                        b'o',
                        b'i',
                        b's',
                        b'e',
                    ] => Some(Self::PaleTurquoise),
                    [
                        b'p',
                        b'a',
                        b'l',
                        b'e',
                        b'v',
                        b'i',
                        b'o',
                        b'l',
                        b'e',
                        b't',
                        b'r',
                        b'e',
                        b'd',
                    ] => Some(Self::PaleVioletRed),
                    [
                        b'r',
                        b'e',
                        b'b',
                        b'e',
                        b'c',
                        b'c',
                        b'a',
                        b'p',
                        b'u',
                        b'r',
                        b'p',
                        b'l',
                        b'e',
                    ] => Some(Self::RebeccaPurple),
                    _ => None,
                }
            }
            14 => {
                match [
                    bytes[0].to_ascii_lowercase(),
                    bytes[1].to_ascii_lowercase(),
                    bytes[2].to_ascii_lowercase(),
                    bytes[3].to_ascii_lowercase(),
                    bytes[4].to_ascii_lowercase(),
                    bytes[5].to_ascii_lowercase(),
                    bytes[6].to_ascii_lowercase(),
                    bytes[7].to_ascii_lowercase(),
                    bytes[8].to_ascii_lowercase(),
                    bytes[9].to_ascii_lowercase(),
                    bytes[10].to_ascii_lowercase(),
                    bytes[11].to_ascii_lowercase(),
                    bytes[12].to_ascii_lowercase(),
                    bytes[13].to_ascii_lowercase(),
                ] {
                    [
                        b'b',
                        b'l',
                        b'a',
                        b'n',
                        b'c',
                        b'h',
                        b'e',
                        b'd',
                        b'a',
                        b'l',
                        b'm',
                        b'o',
                        b'n',
                        b'd',
                    ] => Some(Self::BlanchedAlmond),
                    [
                        b'c',
                        b'o',
                        b'r',
                        b'n',
                        b'f',
                        b'l',
                        b'o',
                        b'w',
                        b'e',
                        b'r',
                        b'b',
                        b'l',
                        b'u',
                        b'e',
                    ] => Some(Self::CornflowerBlue),
                    [
                        b'd',
                        b'a',
                        b'r',
                        b'k',
                        b'o',
                        b'l',
                        b'i',
                        b'v',
                        b'e',
                        b'g',
                        b'r',
                        b'e',
                        b'e',
                        b'n',
                    ] => Some(Self::DarkOliveGreen),
                    [
                        b'l',
                        b'i',
                        b'g',
                        b'h',
                        b't',
                        b's',
                        b'l',
                        b'a',
                        b't',
                        b'e',
                        b'g',
                        b'r',
                        b'a',
                        b'y',
                    ] => Some(Self::LightSlateGray),
                    [
                        b'l',
                        b'i',
                        b'g',
                        b'h',
                        b't',
                        b's',
                        b'l',
                        b'a',
                        b't',
                        b'e',
                        b'g',
                        b'r',
                        b'e',
                        b'y',
                    ] => Some(Self::LightSlateGray),
                    [
                        b'l',
                        b'i',
                        b'g',
                        b'h',
                        b't',
                        b's',
                        b't',
                        b'e',
                        b'e',
                        b'l',
                        b'b',
                        b'l',
                        b'u',
                        b'e',
                    ] => Some(Self::LightSteelBlue),
                    [
                        b'm',
                        b'e',
                        b'd',
                        b'i',
                        b'u',
                        b'm',
                        b's',
                        b'e',
                        b'a',
                        b'g',
                        b'r',
                        b'e',
                        b'e',
                        b'n',
                    ] => Some(Self::MediumSeaGreen),
                    _ => None,
                }
            }
            15 => {
                match [
                    bytes[0].to_ascii_lowercase(),
                    bytes[1].to_ascii_lowercase(),
                    bytes[2].to_ascii_lowercase(),
                    bytes[3].to_ascii_lowercase(),
                    bytes[4].to_ascii_lowercase(),
                    bytes[5].to_ascii_lowercase(),
                    bytes[6].to_ascii_lowercase(),
                    bytes[7].to_ascii_lowercase(),
                    bytes[8].to_ascii_lowercase(),
                    bytes[9].to_ascii_lowercase(),
                    bytes[10].to_ascii_lowercase(),
                    bytes[11].to_ascii_lowercase(),
                    bytes[12].to_ascii_lowercase(),
                    bytes[13].to_ascii_lowercase(),
                    bytes[14].to_ascii_lowercase(),
                ] {
                    [
                        b'm',
                        b'e',
                        b'd',
                        b'i',
                        b'u',
                        b'm',
                        b's',
                        b'l',
                        b'a',
                        b't',
                        b'e',
                        b'b',
                        b'l',
                        b'u',
                        b'e',
                    ] => Some(Self::MediumSlateBlue),
                    [
                        b'm',
                        b'e',
                        b'd',
                        b'i',
                        b'u',
                        b'm',
                        b't',
                        b'u',
                        b'r',
                        b'q',
                        b'u',
                        b'o',
                        b'i',
                        b's',
                        b'e',
                    ] => Some(Self::MediumTurquoise),
                    [
                        b'm',
                        b'e',
                        b'd',
                        b'i',
                        b'u',
                        b'm',
                        b'v',
                        b'i',
                        b'o',
                        b'l',
                        b'e',
                        b't',
                        b'r',
                        b'e',
                        b'd',
                    ] => Some(Self::MediumVioletRed),
                    _ => None,
                }
            }
            16 => {
                match [
                    bytes[0].to_ascii_lowercase(),
                    bytes[1].to_ascii_lowercase(),
                    bytes[2].to_ascii_lowercase(),
                    bytes[3].to_ascii_lowercase(),
                    bytes[4].to_ascii_lowercase(),
                    bytes[5].to_ascii_lowercase(),
                    bytes[6].to_ascii_lowercase(),
                    bytes[7].to_ascii_lowercase(),
                    bytes[8].to_ascii_lowercase(),
                    bytes[9].to_ascii_lowercase(),
                    bytes[10].to_ascii_lowercase(),
                    bytes[11].to_ascii_lowercase(),
                    bytes[12].to_ascii_lowercase(),
                    bytes[13].to_ascii_lowercase(),
                    bytes[14].to_ascii_lowercase(),
                    bytes[15].to_ascii_lowercase(),
                ] {
                    [
                        b'm',
                        b'e',
                        b'd',
                        b'i',
                        b'u',
                        b'm',
                        b'a',
                        b'q',
                        b'u',
                        b'a',
                        b'm',
                        b'a',
                        b'r',
                        b'i',
                        b'n',
                        b'e',
                    ] => Some(Self::MediumAquaMarine),
                    _ => None,
                }
            }
            17 => {
                match [
                    bytes[0].to_ascii_lowercase(),
                    bytes[1].to_ascii_lowercase(),
                    bytes[2].to_ascii_lowercase(),
                    bytes[3].to_ascii_lowercase(),
                    bytes[4].to_ascii_lowercase(),
                    bytes[5].to_ascii_lowercase(),
                    bytes[6].to_ascii_lowercase(),
                    bytes[7].to_ascii_lowercase(),
                    bytes[8].to_ascii_lowercase(),
                    bytes[9].to_ascii_lowercase(),
                    bytes[10].to_ascii_lowercase(),
                    bytes[11].to_ascii_lowercase(),
                    bytes[12].to_ascii_lowercase(),
                    bytes[13].to_ascii_lowercase(),
                    bytes[14].to_ascii_lowercase(),
                    bytes[15].to_ascii_lowercase(),
                    bytes[16].to_ascii_lowercase(),
                ] {
                    [
                        b'm',
                        b'e',
                        b'd',
                        b'i',
                        b'u',
                        b'm',
                        b's',
                        b'p',
                        b'r',
                        b'i',
                        b'n',
                        b'g',
                        b'g',
                        b'r',
                        b'e',
                        b'e',
                        b'n',
                    ] => Some(Self::MediumSpringGreen),
                    _ => None,
                }
            }
            20 => {
                match [
                    bytes[0].to_ascii_lowercase(),
                    bytes[1].to_ascii_lowercase(),
                    bytes[2].to_ascii_lowercase(),
                    bytes[3].to_ascii_lowercase(),
                    bytes[4].to_ascii_lowercase(),
                    bytes[5].to_ascii_lowercase(),
                    bytes[6].to_ascii_lowercase(),
                    bytes[7].to_ascii_lowercase(),
                    bytes[8].to_ascii_lowercase(),
                    bytes[9].to_ascii_lowercase(),
                    bytes[10].to_ascii_lowercase(),
                    bytes[11].to_ascii_lowercase(),
                    bytes[12].to_ascii_lowercase(),
                    bytes[13].to_ascii_lowercase(),
                    bytes[14].to_ascii_lowercase(),
                    bytes[15].to_ascii_lowercase(),
                    bytes[16].to_ascii_lowercase(),
                    bytes[17].to_ascii_lowercase(),
                    bytes[18].to_ascii_lowercase(),
                    bytes[19].to_ascii_lowercase(),
                ] {
                    [
                        b'l',
                        b'i',
                        b'g',
                        b'h',
                        b't',
                        b'g',
                        b'o',
                        b'l',
                        b'd',
                        b'e',
                        b'n',
                        b'r',
                        b'o',
                        b'd',
                        b'y',
                        b'e',
                        b'l',
                        b'l',
                        b'o',
                        b'w',
                    ] => Some(Self::LightGoldenRodYellow),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    /// Converts the NamedColor to its hexadecimal string representation, or returns None if the color is not recognized.
    pub const fn to_hex(self) -> Option<&'static str> {
        match self {
            Self::AliceBlue => Some("#F0F8FF"),
            Self::AntiqueWhite => Some("#FAEBD7"),
            Self::Aqua => Some("#00FFFF"),
            Self::Aquamarine => Some("#7FFFD4"),
            Self::Azure => Some("#F0FFFF"),
            Self::Beige => Some("#F5F5DC"),
            Self::Bisque => Some("#FFE4C4"),
            Self::Black => Some("#000000"),
            Self::BlanchedAlmond => Some("#FFEBCD"),
            Self::Blue => Some("#0000FF"),
            Self::BlueViolet => Some("#8A2BE2"),
            Self::Brown => Some("#A52A2A"),
            Self::BurlyWood => Some("#DEB887"),
            Self::CadetBlue => Some("#5F9EA0"),
            Self::Chartreuse => Some("#7FFF00"),
            Self::Chocolate => Some("#D2691E"),
            Self::Coral => Some("#FF7F50"),
            Self::CornflowerBlue => Some("#6495ED"),
            Self::Cornsilk => Some("#FFF8DC"),
            Self::Crimson => Some("#DC143C"),
            Self::DarkBlue => Some("#00008B"),
            Self::DarkCyan => Some("#008B8B"),
            Self::DarkGoldenRod => Some("#B8860B"),
            Self::DarkGray => Some("#A9A9A9"),
            Self::DarkGreen => Some("#006400"),
            Self::DarkKhaki => Some("#BDB76B"),
            Self::DarkMagenta => Some("#8B008B"),
            Self::DarkOliveGreen => Some("#556B2F"),
            Self::DarkOrange => Some("#FF8C00"),
            Self::DarkOrchid => Some("#9932CC"),
            Self::DarkRed => Some("#8B0000"),
            Self::DarkSalmon => Some("#E9967A"),
            Self::DarkSeaGreen => Some("#8FBC8F"),
            Self::DarkSlateBlue => Some("#483D8B"),
            Self::DarkSlateGray => Some("#2F4F4F"),
            Self::DarkTurquoise => Some("#00CED1"),
            Self::DarkViolet => Some("#9400D3"),
            Self::DeepPink => Some("#FF1493"),
            Self::DeepSkyBlue => Some("#00BFFF"),
            Self::DimGray => Some("#696969"),
            Self::DodgerBlue => Some("#1E90FF"),
            Self::FireBrick => Some("#B22222"),
            Self::FloralWhite => Some("#FFFAF0"),
            Self::ForestGreen => Some("#228B22"),
            Self::Fuchsia => Some("#FF00FF"),
            Self::Gainsboro => Some("#DCDCDC"),
            Self::GhostWhite => Some("#F8F8FF"),
            Self::Gold => Some("#FFD700"),
            Self::GoldenRod => Some("#DAA520"),
            Self::Gray => Some("#808080"),
            Self::Green => Some("#008000"),
            Self::GreenYellow => Some("#ADFF2F"),
            Self::HoneyDew => Some("#F0FFF0"),
            Self::HotPink => Some("#FF69B4"),
            Self::IndianRed => Some("#CD5C5C"),
            Self::Indigo => Some("#4B0082"),
            Self::Ivory => Some("#FFFFF0"),
            Self::Khaki => Some("#F0E68C"),
            Self::Lavender => Some("#E6E6FA"),
            Self::LavenderBlush => Some("#FFF0F5"),
            Self::LawnGreen => Some("#7CFC00"),
            Self::LemonChiffon => Some("#FFFACD"),
            Self::LightBlue => Some("#ADD8E6"),
            Self::LightCoral => Some("#F08080"),
            Self::LightCyan => Some("#E0FFFF"),
            Self::LightGoldenRodYellow => Some("#FAFAD2"),
            Self::LightGray => Some("#D3D3D3"),
            Self::LightGreen => Some("#90EE90"),
            Self::LightPink => Some("#FFB6C1"),
            Self::LightSalmon => Some("#FFA07A"),
            Self::LightSeaGreen => Some("#20B2AA"),
            Self::LightSkyBlue => Some("#87CEFA"),
            Self::LightSlateGray => Some("#778899"),
            Self::LightSteelBlue => Some("#B0C4DE"),
            Self::LightYellow => Some("#FFFFE0"),
            Self::Lime => Some("#00FF00"),
            Self::LimeGreen => Some("#32CD32"),
            Self::Linen => Some("#FAF0E6"),
            Self::Magenta => Some("#FF00FF"),
            Self::Maroon => Some("#800000"),
            Self::MediumAquaMarine => Some("#66CDAA"),
            Self::MediumBlue => Some("#0000CD"),
            Self::MediumOrchid => Some("#BA55D3"),
            Self::MediumPurple => Some("#9370DB"),
            Self::MediumSeaGreen => Some("#3CB371"),
            Self::MediumSlateBlue => Some("#7B68EE"),
            Self::MediumSpringGreen => Some("#00FA9A"),
            Self::MediumTurquoise => Some("#48D1CC"),
            Self::MediumVioletRed => Some("#C71585"),
            Self::MidnightBlue => Some("#191970"),
            Self::MintCream => Some("#F5FFFA"),
            Self::MistyRose => Some("#FFE4E1"),
            Self::Moccasin => Some("#FFE4B5"),
            Self::NavajoWhite => Some("#FFDEAD"),
            Self::Navy => Some("#000080"),
            Self::OldLace => Some("#FDF5E6"),
            Self::Olive => Some("#808000"),
            Self::OliveDrab => Some("#6B8E23"),
            Self::Orange => Some("#FFA500"),
            Self::OrangeRed => Some("#FF4500"),
            Self::Orchid => Some("#DA70D6"),
            Self::PaleGoldenRod => Some("#EEE8AA"),
            Self::PaleGreen => Some("#98FB98"),
            Self::PaleTurquoise => Some("#AFEEEE"),
            Self::PaleVioletRed => Some("#DB7093"),
            Self::PapayaWhip => Some("#FFEFD5"),
            Self::PeachPuff => Some("#FFDAB9"),
            Self::Peru => Some("#CD853F"),
            Self::Pink => Some("#FFC0CB"),
            Self::Plum => Some("#DDA0DD"),
            Self::PowderBlue => Some("#B0E0E6"),
            Self::Purple => Some("#800080"),
            Self::RebeccaPurple => Some("#663399"),
            Self::Red => Some("#FF0000"),
            Self::RosyBrown => Some("#BC8F8F"),
            Self::RoyalBlue => Some("#4169E1"),
            Self::SaddleBrown => Some("#8B4513"),
            Self::Salmon => Some("#FA8072"),
            Self::SandyBrown => Some("#F4A460"),
            Self::SeaGreen => Some("#2E8B57"),
            Self::SeaShell => Some("#FFF5EE"),
            Self::Sienna => Some("#A0522D"),
            Self::Silver => Some("#C0C0C0"),
            Self::SkyBlue => Some("#87CEEB"),
            Self::SlateBlue => Some("#6A5ACD"),
            Self::SlateGray => Some("#708090"),
            Self::Snow => Some("#FFFAFA"),
            Self::SpringGreen => Some("#00FF7F"),
            Self::SteelBlue => Some("#4682B4"),
            Self::Tan => Some("#D2B48C"),
            Self::Teal => Some("#008080"),
            Self::Thistle => Some("#D8BFD8"),
            Self::Tomato => Some("#FF6347"),
            Self::Turquoise => Some("#40E0D0"),
            Self::Violet => Some("#EE82EE"),
            Self::Wheat => Some("#F5DEB3"),
            Self::White => Some("#FFFFFF"),
            Self::WhiteSmoke => Some("#F5F5F5"),
            Self::Yellow => Some("#FFFF00"),
            Self::YellowGreen => Some("#9ACD32"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_named_color() {
        let color = NamedColor::from_str_insensitive("AliceBlue").unwrap();
        assert_eq!(color, NamedColor::AliceBlue);

        let color = NamedColor::from_str_insensitive("rebeccapurple").unwrap();
        assert_eq!(color, NamedColor::RebeccaPurple);

        let color = NamedColor::from_str_insensitive("LIGHTGOLDENRODYELLOW").unwrap();
        assert_eq!(color, NamedColor::LightGoldenRodYellow);

        let color = NamedColor::from_str_insensitive("invalidcolor");
        assert!(color.is_none());
    }
}

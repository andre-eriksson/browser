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
                    [b'r', b'e', b'd'] => Some(NamedColor::Red),
                    [b't', b'a', b'n'] => Some(NamedColor::Tan),
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
                    [b'a', b'q', b'u', b'a'] => Some(NamedColor::Aqua),
                    [b'b', b'l', b'u', b'e'] => Some(NamedColor::Blue),
                    [b'c', b'y', b'a', b'n'] => Some(NamedColor::Aqua),
                    [b'g', b'o', b'l', b'd'] => Some(NamedColor::Gold),
                    [b'g', b'r', b'a', b'y'] => Some(NamedColor::Gray),
                    [b'g', b'r', b'e', b'y'] => Some(NamedColor::Gray),
                    [b'l', b'i', b'm', b'e'] => Some(NamedColor::Lime),
                    [b'n', b'a', b'v', b'y'] => Some(NamedColor::Navy),
                    [b'p', b'e', b'r', b'u'] => Some(NamedColor::Peru),
                    [b'p', b'i', b'n', b'k'] => Some(NamedColor::Pink),
                    [b'p', b'l', b'u', b'm'] => Some(NamedColor::Plum),
                    [b's', b'n', b'o', b'w'] => Some(NamedColor::Snow),
                    [b't', b'e', b'a', b'l'] => Some(NamedColor::Teal),
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
                    [b'a', b'z', b'u', b'r', b'e'] => Some(NamedColor::Azure),
                    [b'b', b'e', b'i', b'g', b'e'] => Some(NamedColor::Beige),
                    [b'b', b'l', b'a', b'c', b'k'] => Some(NamedColor::Black),
                    [b'b', b'r', b'o', b'w', b'n'] => Some(NamedColor::Brown),
                    [b'c', b'o', b'r', b'a', b'l'] => Some(NamedColor::Coral),
                    [b'g', b'r', b'e', b'e', b'n'] => Some(NamedColor::Green),
                    [b'i', b'v', b'o', b'r', b'y'] => Some(NamedColor::Ivory),
                    [b'k', b'h', b'a', b'k', b'i'] => Some(NamedColor::Khaki),
                    [b'l', b'i', b'n', b'e', b'n'] => Some(NamedColor::Linen),
                    [b'o', b'l', b'i', b'v', b'e'] => Some(NamedColor::Olive),
                    [b'w', b'h', b'e', b'a', b't'] => Some(NamedColor::Wheat),
                    [b'w', b'h', b'i', b't', b'e'] => Some(NamedColor::White),
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
                    [b'b', b'i', b's', b'q', b'u', b'e'] => Some(NamedColor::Bisque),
                    [b'i', b'n', b'd', b'i', b'g', b'o'] => Some(NamedColor::Indigo),
                    [b'm', b'a', b'r', b'o', b'o', b'n'] => Some(NamedColor::Maroon),
                    [b'o', b'r', b'a', b'n', b'g', b'e'] => Some(NamedColor::Orange),
                    [b'o', b'r', b'c', b'h', b'i', b'd'] => Some(NamedColor::Orchid),
                    [b'p', b'u', b'r', b'p', b'l', b'e'] => Some(NamedColor::Purple),
                    [b's', b'a', b'l', b'm', b'o', b'n'] => Some(NamedColor::Salmon),
                    [b's', b'i', b'e', b'n', b'n', b'a'] => Some(NamedColor::Sienna),
                    [b's', b'i', b'l', b'v', b'e', b'r'] => Some(NamedColor::Silver),
                    [b't', b'o', b'm', b'a', b't', b'o'] => Some(NamedColor::Tomato),
                    [b'v', b'i', b'o', b'l', b'e', b't'] => Some(NamedColor::Violet),
                    [b'y', b'e', b'l', b'l', b'o', b'w'] => Some(NamedColor::Yellow),
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
                    [b'c', b'r', b'i', b'm', b's', b'o', b'n'] => Some(NamedColor::Crimson),
                    [b'd', b'a', b'r', b'k', b'r', b'e', b'd'] => Some(NamedColor::DarkRed),
                    [b'd', b'i', b'm', b'g', b'r', b'a', b'y'] => Some(NamedColor::DimGray),
                    [b'd', b'i', b'm', b'g', b'r', b'e', b'y'] => Some(NamedColor::DimGray),
                    [b'f', b'u', b'c', b'h', b's', b'i', b'a'] => Some(NamedColor::Fuchsia),
                    [b'h', b'o', b't', b'p', b'i', b'n', b'k'] => Some(NamedColor::HotPink),
                    [b'm', b'a', b'g', b'e', b'n', b't', b'a'] => Some(NamedColor::Magenta),
                    [b'o', b'l', b'd', b'l', b'a', b'c', b'e'] => Some(NamedColor::OldLace),
                    [b's', b'k', b'y', b'b', b'l', b'u', b'e'] => Some(NamedColor::SkyBlue),
                    [b't', b'h', b'i', b's', b't', b'l', b'e'] => Some(NamedColor::Thistle),
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
                    [b'c', b'o', b'r', b'n', b's', b'i', b'l', b'k'] => Some(NamedColor::Cornsilk),
                    [b'd', b'a', b'r', b'k', b'b', b'l', b'u', b'e'] => Some(NamedColor::DarkBlue),
                    [b'd', b'a', b'r', b'k', b'c', b'y', b'a', b'n'] => Some(NamedColor::DarkCyan),
                    [b'd', b'a', b'r', b'k', b'g', b'r', b'a', b'y'] => Some(NamedColor::DarkGray),
                    [b'd', b'a', b'r', b'k', b'g', b'r', b'e', b'y'] => Some(NamedColor::DarkGray),
                    [b'd', b'e', b'e', b'p', b'p', b'i', b'n', b'k'] => Some(NamedColor::DeepPink),
                    [b'h', b'o', b'n', b'e', b'y', b'd', b'e', b'w'] => Some(NamedColor::HoneyDew),
                    [b'l', b'a', b'v', b'e', b'n', b'd', b'e', b'r'] => Some(NamedColor::Lavender),
                    [b'm', b'o', b'c', b'c', b'a', b's', b'i', b'n'] => Some(NamedColor::Moccasin),
                    [b's', b'e', b'a', b'g', b'r', b'e', b'e', b'n'] => Some(NamedColor::SeaGreen),
                    [b's', b'e', b'a', b's', b'h', b'e', b'l', b'l'] => Some(NamedColor::SeaShell),
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
                    [b'a', b'l', b'i', b'c', b'e', b'b', b'l', b'u', b'e'] => Some(NamedColor::AliceBlue),
                    [b'b', b'u', b'r', b'l', b'y', b'w', b'o', b'o', b'd'] => Some(NamedColor::BurlyWood),
                    [b'c', b'a', b'd', b'e', b't', b'b', b'l', b'u', b'e'] => Some(NamedColor::CadetBlue),
                    [b'c', b'h', b'o', b'c', b'o', b'l', b'a', b't', b'e'] => Some(NamedColor::Chocolate),
                    [b'd', b'a', b'r', b'k', b'g', b'r', b'e', b'e', b'n'] => Some(NamedColor::DarkGreen),
                    [b'd', b'a', b'r', b'k', b'k', b'h', b'a', b'k', b'i'] => Some(NamedColor::DarkKhaki),
                    [b'f', b'i', b'r', b'e', b'b', b'r', b'i', b'c', b'k'] => Some(NamedColor::FireBrick),
                    [b'g', b'a', b'i', b'n', b's', b'b', b'o', b'r', b'o'] => Some(NamedColor::Gainsboro),
                    [b'g', b'o', b'l', b'd', b'e', b'n', b'r', b'o', b'd'] => Some(NamedColor::GoldenRod),
                    [b'i', b'n', b'd', b'i', b'a', b'n', b'r', b'e', b'd'] => Some(NamedColor::IndianRed),
                    [b'l', b'a', b'w', b'n', b'g', b'r', b'e', b'e', b'n'] => Some(NamedColor::LawnGreen),
                    [b'l', b'i', b'g', b'h', b't', b'b', b'l', b'u', b'e'] => Some(NamedColor::LightBlue),
                    [b'l', b'i', b'g', b'h', b't', b'c', b'y', b'a', b'n'] => Some(NamedColor::LightCyan),
                    [b'l', b'i', b'g', b'h', b't', b'g', b'r', b'a', b'y'] => Some(NamedColor::LightGray),
                    [b'l', b'i', b'g', b'h', b't', b'g', b'r', b'e', b'y'] => Some(NamedColor::LightGray),
                    [b'l', b'i', b'g', b'h', b't', b'p', b'i', b'n', b'k'] => Some(NamedColor::LightPink),
                    [b'l', b'i', b'm', b'e', b'g', b'r', b'e', b'e', b'n'] => Some(NamedColor::LimeGreen),
                    [b'm', b'i', b'n', b't', b'c', b'r', b'e', b'a', b'm'] => Some(NamedColor::MintCream),
                    [b'm', b'i', b's', b't', b'y', b'r', b'o', b's', b'e'] => Some(NamedColor::MistyRose),
                    [b'o', b'l', b'i', b'v', b'e', b'd', b'r', b'a', b'b'] => Some(NamedColor::OliveDrab),
                    [b'o', b'r', b'a', b'n', b'g', b'e', b'r', b'e', b'd'] => Some(NamedColor::OrangeRed),
                    [b'p', b'a', b'l', b'e', b'g', b'r', b'e', b'e', b'n'] => Some(NamedColor::PaleGreen),
                    [b'p', b'e', b'a', b'c', b'h', b'p', b'u', b'f', b'f'] => Some(NamedColor::PeachPuff),
                    [b'r', b'o', b's', b'y', b'b', b'r', b'o', b'w', b'n'] => Some(NamedColor::RosyBrown),
                    [b'r', b'o', b'y', b'a', b'l', b'b', b'l', b'u', b'e'] => Some(NamedColor::RoyalBlue),
                    [b's', b'l', b'a', b't', b'e', b'b', b'l', b'u', b'e'] => Some(NamedColor::SlateBlue),
                    [b's', b'l', b'a', b't', b'e', b'g', b'r', b'a', b'y'] => Some(NamedColor::SlateGray),
                    [b's', b'l', b'a', b't', b'e', b'g', b'r', b'e', b'y'] => Some(NamedColor::SlateGray),
                    [b's', b't', b'e', b'e', b'l', b'b', b'l', b'u', b'e'] => Some(NamedColor::SteelBlue),
                    [b't', b'u', b'r', b'q', b'u', b'o', b'i', b's', b'e'] => Some(NamedColor::Turquoise),
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
                    [b'a', b'q', b'u', b'a', b'm', b'a', b'r', b'i', b'n', b'e'] => Some(NamedColor::Aquamarine),
                    [b'b', b'l', b'u', b'e', b'v', b'i', b'o', b'l', b'e', b't'] => Some(NamedColor::BlueViolet),
                    [b'c', b'h', b'a', b'r', b't', b'r', b'e', b'u', b's', b'e'] => Some(NamedColor::Chartreuse),
                    [b'd', b'a', b'r', b'k', b'o', b'r', b'a', b'n', b'g', b'e'] => Some(NamedColor::DarkOrange),
                    [b'd', b'a', b'r', b'k', b'o', b'r', b'c', b'h', b'i', b'd'] => Some(NamedColor::DarkOrchid),
                    [b'd', b'a', b'r', b'k', b's', b'a', b'l', b'm', b'o', b'n'] => Some(NamedColor::DarkSalmon),
                    [b'd', b'a', b'r', b'k', b'v', b'i', b'o', b'l', b'e', b't'] => Some(NamedColor::DarkViolet),
                    [b'd', b'o', b'd', b'g', b'e', b'r', b'b', b'l', b'u', b'e'] => Some(NamedColor::DodgerBlue),
                    [b'g', b'h', b'o', b's', b't', b'w', b'h', b'i', b't', b'e'] => Some(NamedColor::GhostWhite),
                    [b'l', b'i', b'g', b'h', b't', b'c', b'o', b'r', b'a', b'l'] => Some(NamedColor::LightCoral),
                    [b'l', b'i', b'g', b'h', b't', b'g', b'r', b'e', b'e', b'n'] => Some(NamedColor::LightGreen),
                    [b'm', b'e', b'd', b'i', b'u', b'm', b'b', b'l', b'u', b'e'] => Some(NamedColor::MediumBlue),
                    [b'p', b'a', b'p', b'a', b'y', b'a', b'w', b'h', b'i', b'p'] => Some(NamedColor::PapayaWhip),
                    [b'p', b'o', b'w', b'd', b'e', b'r', b'b', b'l', b'u', b'e'] => Some(NamedColor::PowderBlue),
                    [b's', b'a', b'n', b'd', b'y', b'b', b'r', b'o', b'w', b'n'] => Some(NamedColor::SandyBrown),
                    [b'w', b'h', b'i', b't', b'e', b's', b'm', b'o', b'k', b'e'] => Some(NamedColor::WhiteSmoke),
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
                    ] => Some(NamedColor::DarkMagenta),
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
                    ] => Some(NamedColor::DeepSkyBlue),
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
                    ] => Some(NamedColor::FloralWhite),
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
                    ] => Some(NamedColor::ForestGreen),
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
                    ] => Some(NamedColor::GreenYellow),
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
                    ] => Some(NamedColor::LightSalmon),
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
                    ] => Some(NamedColor::LightYellow),
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
                    ] => Some(NamedColor::NavajoWhite),
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
                    ] => Some(NamedColor::SaddleBrown),
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
                    ] => Some(NamedColor::SpringGreen),
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
                    ] => Some(NamedColor::YellowGreen),
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
                    ] => Some(NamedColor::AntiqueWhite),
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
                    ] => Some(NamedColor::DarkSeaGreen),
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
                    ] => Some(NamedColor::LemonChiffon),
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
                    ] => Some(NamedColor::LightSkyBlue),
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
                    ] => Some(NamedColor::MediumOrchid),
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
                    ] => Some(NamedColor::MediumPurple),
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
                    ] => Some(NamedColor::MidnightBlue),
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
                    ] => Some(NamedColor::DarkGoldenRod),
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
                    ] => Some(NamedColor::DarkSlateBlue),
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
                    ] => Some(NamedColor::DarkSlateGray),
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
                    ] => Some(NamedColor::DarkSlateGray),
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
                    ] => Some(NamedColor::DarkTurquoise),
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
                    ] => Some(NamedColor::LavenderBlush),
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
                    ] => Some(NamedColor::LightSeaGreen),
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
                    ] => Some(NamedColor::PaleGoldenRod),
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
                    ] => Some(NamedColor::PaleTurquoise),
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
                    ] => Some(NamedColor::PaleVioletRed),
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
                    ] => Some(NamedColor::RebeccaPurple),
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
                    ] => Some(NamedColor::BlanchedAlmond),
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
                    ] => Some(NamedColor::CornflowerBlue),
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
                    ] => Some(NamedColor::DarkOliveGreen),
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
                    ] => Some(NamedColor::LightSlateGray),
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
                    ] => Some(NamedColor::LightSlateGray),
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
                    ] => Some(NamedColor::LightSteelBlue),
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
                    ] => Some(NamedColor::MediumSeaGreen),
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
                    ] => Some(NamedColor::MediumSlateBlue),
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
                    ] => Some(NamedColor::MediumTurquoise),
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
                    ] => Some(NamedColor::MediumVioletRed),
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
                    ] => Some(NamedColor::MediumAquaMarine),
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
                    ] => Some(NamedColor::MediumSpringGreen),
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
                    ] => Some(NamedColor::LightGoldenRodYellow),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    /// Converts the NamedColor to its hexadecimal string representation, or returns None if the color is not recognized.
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

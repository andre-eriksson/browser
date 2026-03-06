use std::str::FromStr;

/// System colors defined in CSS specifications.
///
/// These are colors that correspond to the user's operating system or browser theme settings,
/// for now only a fixed set of colors is provided.
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
    Deprecated(DeprecatedColor),
}

impl FromStr for SystemColor {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq_ignore_ascii_case("accentcolor") {
            Ok(SystemColor::AccentColor)
        } else if s.eq_ignore_ascii_case("accentcolortext") {
            Ok(SystemColor::AccentColorText)
        } else if s.eq_ignore_ascii_case("activetext") {
            Ok(SystemColor::ActiveText)
        } else if s.eq_ignore_ascii_case("buttonborder") {
            Ok(SystemColor::ButtonBorder)
        } else if s.eq_ignore_ascii_case("buttonface") {
            Ok(SystemColor::ButtonFace)
        } else if s.eq_ignore_ascii_case("buttontext") {
            Ok(SystemColor::ButtonText)
        } else if s.eq_ignore_ascii_case("canvas") {
            Ok(SystemColor::Canvas)
        } else if s.eq_ignore_ascii_case("canvastext") {
            Ok(SystemColor::CanvasText)
        } else if s.eq_ignore_ascii_case("field") {
            Ok(SystemColor::Field)
        } else if s.eq_ignore_ascii_case("fieldtext") {
            Ok(SystemColor::FieldText)
        } else if s.eq_ignore_ascii_case("graytext") {
            Ok(SystemColor::GrayText)
        } else if s.eq_ignore_ascii_case("highlight") {
            Ok(SystemColor::Highlight)
        } else if s.eq_ignore_ascii_case("highlighttext") {
            Ok(SystemColor::HighlightText)
        } else if s.eq_ignore_ascii_case("linktext") {
            Ok(SystemColor::LinkText)
        } else if s.eq_ignore_ascii_case("mark") {
            Ok(SystemColor::Mark)
        } else if s.eq_ignore_ascii_case("marktext") {
            Ok(SystemColor::MarkText)
        } else if s.eq_ignore_ascii_case("selecteditem") {
            Ok(SystemColor::SelectedItem)
        } else if s.eq_ignore_ascii_case("selecteditemtext") {
            Ok(SystemColor::SelectedItemText)
        } else if s.eq_ignore_ascii_case("visitedtext") {
            Ok(SystemColor::VisitedText)
        } else if let Ok(deprecated) = DeprecatedColor::from_str(s) {
            Ok(SystemColor::Deprecated(deprecated))
        } else {
            Err(format!("Unknown system color: {}", s))
        }
    }
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
            SystemColor::Deprecated(deprecated) => deprecated.to_hex(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DeprecatedColor {
    ActiveBorder,
    ActiveCaption,
    AppWorkspace,
    Background,
    ButtonHighlight,
    ButtonShadow,
    CaptionText,
    InactiveBorder,
    InactiveCaption,
    InactiveCaptionText,
    InfoBackground,
    InfoText,
    Menu,
    MenuText,
    Scrollbar,
    ThreeDDarkShadow,
    ThreeDFace,
    ThreeDHighlight,
    ThreeDLightShadow,
    ThreeDShadow,
    Window,
    WindowFrame,
    WindowText,
}

impl DeprecatedColor {
    /// Converts the DeprecatedColor to its hexadecimal string representation, or returns None if the color is not recognized.
    pub fn to_hex(self) -> Option<&'static str> {
        match self {
            DeprecatedColor::ActiveBorder => Some("#A9A9A9"),
            DeprecatedColor::ActiveCaption => Some("#000080"),
            DeprecatedColor::AppWorkspace => Some("#ABABAB"),
            DeprecatedColor::Background => Some("#FFFFFF"),
            DeprecatedColor::ButtonHighlight => Some("#FFFFFF"),
            DeprecatedColor::ButtonShadow => Some("#A9A9A9"),
            DeprecatedColor::CaptionText => Some("#FFFFFF"),
            DeprecatedColor::InactiveBorder => Some("#A9A9A9"),
            DeprecatedColor::InactiveCaption => Some("#808080"),
            DeprecatedColor::InactiveCaptionText => Some("#C0C0C0"),
            DeprecatedColor::InfoBackground => Some("#FFFFE1"),
            DeprecatedColor::InfoText => Some("#000000"),
            DeprecatedColor::Menu => Some("#F0F0F0"),
            DeprecatedColor::MenuText => Some("#000000"),
            DeprecatedColor::Scrollbar => Some("#C0C0C0"),
            DeprecatedColor::ThreeDDarkShadow => Some("#696969"),
            DeprecatedColor::ThreeDFace => Some("#F0F0F0"),
            DeprecatedColor::ThreeDHighlight => Some("#FFFFFF"),
            DeprecatedColor::ThreeDLightShadow => Some("#D3D3D3"),
            DeprecatedColor::ThreeDShadow => Some("#A9A9A9"),
            DeprecatedColor::Window => Some("#FFFFFF"),
            DeprecatedColor::WindowFrame => Some("#000000"),
            DeprecatedColor::WindowText => Some("#000000"),
        }
    }
}

impl FromStr for DeprecatedColor {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq_ignore_ascii_case("activeborder") {
            Ok(DeprecatedColor::ActiveBorder)
        } else if s.eq_ignore_ascii_case("activecaption") {
            Ok(DeprecatedColor::ActiveCaption)
        } else if s.eq_ignore_ascii_case("appworkspace") {
            Ok(DeprecatedColor::AppWorkspace)
        } else if s.eq_ignore_ascii_case("background") {
            Ok(DeprecatedColor::Background)
        } else if s.eq_ignore_ascii_case("buttonhighlight") {
            Ok(DeprecatedColor::ButtonHighlight)
        } else if s.eq_ignore_ascii_case("buttonshadow") {
            Ok(DeprecatedColor::ButtonShadow)
        } else if s.eq_ignore_ascii_case("captiontext") {
            Ok(DeprecatedColor::CaptionText)
        } else if s.eq_ignore_ascii_case("inactiveborder") {
            Ok(DeprecatedColor::InactiveBorder)
        } else if s.eq_ignore_ascii_case("inactivecaption") {
            Ok(DeprecatedColor::InactiveCaption)
        } else if s.eq_ignore_ascii_case("inactivecaptiontext") {
            Ok(DeprecatedColor::InactiveCaptionText)
        } else if s.eq_ignore_ascii_case("infobackground") {
            Ok(DeprecatedColor::InfoBackground)
        } else if s.eq_ignore_ascii_case("infotext") {
            Ok(DeprecatedColor::InfoText)
        } else if s.eq_ignore_ascii_case("menu") {
            Ok(DeprecatedColor::Menu)
        } else if s.eq_ignore_ascii_case("menutext") {
            Ok(DeprecatedColor::MenuText)
        } else if s.eq_ignore_ascii_case("scrollbar") {
            Ok(DeprecatedColor::Scrollbar)
        } else if s.eq_ignore_ascii_case("3ddarkshadow") {
            Ok(DeprecatedColor::ThreeDDarkShadow)
        } else if s.eq_ignore_ascii_case("3dface") {
            Ok(DeprecatedColor::ThreeDFace)
        } else if s.eq_ignore_ascii_case("3dhighlight") {
            Ok(DeprecatedColor::ThreeDHighlight)
        } else if s.eq_ignore_ascii_case("3dlightshadow") {
            Ok(DeprecatedColor::ThreeDLightShadow)
        } else if s.eq_ignore_ascii_case("3dshadow") {
            Ok(DeprecatedColor::ThreeDShadow)
        } else if s.eq_ignore_ascii_case("window") {
            Ok(DeprecatedColor::Window)
        } else if s.eq_ignore_ascii_case("windowframe") {
            Ok(DeprecatedColor::WindowFrame)
        } else if s.eq_ignore_ascii_case("windowtext") {
            Ok(DeprecatedColor::WindowText)
        } else {
            Err(format!("Unknown deprecated system color: {}", s))
        }
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
    fn parse_deprecated_color() {
        let color: DeprecatedColor = "activeBorder".parse().unwrap();
        assert_eq!(color, DeprecatedColor::ActiveBorder);

        let color: DeprecatedColor = "3dFace".parse().unwrap();
        assert_eq!(color, DeprecatedColor::ThreeDFace);

        let color = "invalidDeprecatedColor".parse::<DeprecatedColor>();
        assert!(color.is_err());
    }

    #[test]
    fn parse_system_color_deprecated() {
        let color: SystemColor = "activeBorder".parse().unwrap();
        assert_eq!(color, SystemColor::Deprecated(DeprecatedColor::ActiveBorder));
    }
}

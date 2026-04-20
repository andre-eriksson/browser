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
            Ok(Self::AccentColor)
        } else if s.eq_ignore_ascii_case("accentcolortext") {
            Ok(Self::AccentColorText)
        } else if s.eq_ignore_ascii_case("activetext") {
            Ok(Self::ActiveText)
        } else if s.eq_ignore_ascii_case("buttonborder") {
            Ok(Self::ButtonBorder)
        } else if s.eq_ignore_ascii_case("buttonface") {
            Ok(Self::ButtonFace)
        } else if s.eq_ignore_ascii_case("buttontext") {
            Ok(Self::ButtonText)
        } else if s.eq_ignore_ascii_case("canvas") {
            Ok(Self::Canvas)
        } else if s.eq_ignore_ascii_case("canvastext") {
            Ok(Self::CanvasText)
        } else if s.eq_ignore_ascii_case("field") {
            Ok(Self::Field)
        } else if s.eq_ignore_ascii_case("fieldtext") {
            Ok(Self::FieldText)
        } else if s.eq_ignore_ascii_case("graytext") {
            Ok(Self::GrayText)
        } else if s.eq_ignore_ascii_case("highlight") {
            Ok(Self::Highlight)
        } else if s.eq_ignore_ascii_case("highlighttext") {
            Ok(Self::HighlightText)
        } else if s.eq_ignore_ascii_case("linktext") {
            Ok(Self::LinkText)
        } else if s.eq_ignore_ascii_case("mark") {
            Ok(Self::Mark)
        } else if s.eq_ignore_ascii_case("marktext") {
            Ok(Self::MarkText)
        } else if s.eq_ignore_ascii_case("selecteditem") {
            Ok(Self::SelectedItem)
        } else if s.eq_ignore_ascii_case("selecteditemtext") {
            Ok(Self::SelectedItemText)
        } else if s.eq_ignore_ascii_case("visitedtext") {
            Ok(Self::VisitedText)
        } else if let Ok(deprecated) = DeprecatedColor::from_str(s) {
            Ok(Self::Deprecated(deprecated))
        } else {
            Err(format!("Unknown system color: {s}"))
        }
    }
}

impl SystemColor {
    /// Converts the `SystemColor` to its hexadecimal string representation, or returns None if the color is not recognized.
    #[must_use]
    pub const fn to_hex(self) -> Option<&'static str> {
        match self {
            Self::AccentColor => Some("#0078D7"),
            Self::AccentColorText => Some("#FFFFFF"),
            Self::ActiveText => Some("#0000FF"),
            Self::ButtonBorder => Some("#A9A9A9"),
            Self::ButtonFace => Some("#F0F0F0"),
            Self::ButtonText => Some("#000000"),
            Self::Canvas => Some("#FFFFFF"),
            Self::CanvasText => Some("#000000"),
            Self::Field => Some("#FFFFFF"),
            Self::FieldText => Some("#000000"),
            Self::GrayText => Some("#A9A9A9"),
            Self::Highlight => Some("#3399FF"),
            Self::HighlightText => Some("#FFFFFF"),
            Self::LinkText => Some("#0000FF"),
            Self::Mark => Some("#FFFF00"),
            Self::MarkText => Some("#000000"),
            Self::SelectedItem => Some("#3399FF"),
            Self::SelectedItemText => Some("#FFFFFF"),
            Self::VisitedText => Some("#800080"),
            Self::Deprecated(deprecated) => deprecated.to_hex(),
        }
    }
}

/// Deprecated system colors that were defined in older versions of CSS but are no longer recommended for use.
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
    /// Converts the `DeprecatedColor` to its hexadecimal string representation, or returns None if the color is not recognized.
    #[must_use]
    pub const fn to_hex(self) -> Option<&'static str> {
        match self {
            Self::ActiveBorder => Some("#A9A9A9"),
            Self::ActiveCaption => Some("#000080"),
            Self::AppWorkspace => Some("#ABABAB"),
            Self::Background => Some("#FFFFFF"),
            Self::ButtonHighlight => Some("#FFFFFF"),
            Self::ButtonShadow => Some("#A9A9A9"),
            Self::CaptionText => Some("#FFFFFF"),
            Self::InactiveBorder => Some("#A9A9A9"),
            Self::InactiveCaption => Some("#808080"),
            Self::InactiveCaptionText => Some("#C0C0C0"),
            Self::InfoBackground => Some("#FFFFE1"),
            Self::InfoText => Some("#000000"),
            Self::Menu => Some("#F0F0F0"),
            Self::MenuText => Some("#000000"),
            Self::Scrollbar => Some("#C0C0C0"),
            Self::ThreeDDarkShadow => Some("#696969"),
            Self::ThreeDFace => Some("#F0F0F0"),
            Self::ThreeDHighlight => Some("#FFFFFF"),
            Self::ThreeDLightShadow => Some("#D3D3D3"),
            Self::ThreeDShadow => Some("#A9A9A9"),
            Self::Window => Some("#FFFFFF"),
            Self::WindowFrame => Some("#000000"),
            Self::WindowText => Some("#000000"),
        }
    }
}

impl FromStr for DeprecatedColor {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq_ignore_ascii_case("activeborder") {
            Ok(Self::ActiveBorder)
        } else if s.eq_ignore_ascii_case("activecaption") {
            Ok(Self::ActiveCaption)
        } else if s.eq_ignore_ascii_case("appworkspace") {
            Ok(Self::AppWorkspace)
        } else if s.eq_ignore_ascii_case("background") {
            Ok(Self::Background)
        } else if s.eq_ignore_ascii_case("buttonhighlight") {
            Ok(Self::ButtonHighlight)
        } else if s.eq_ignore_ascii_case("buttonshadow") {
            Ok(Self::ButtonShadow)
        } else if s.eq_ignore_ascii_case("captiontext") {
            Ok(Self::CaptionText)
        } else if s.eq_ignore_ascii_case("inactiveborder") {
            Ok(Self::InactiveBorder)
        } else if s.eq_ignore_ascii_case("inactivecaption") {
            Ok(Self::InactiveCaption)
        } else if s.eq_ignore_ascii_case("inactivecaptiontext") {
            Ok(Self::InactiveCaptionText)
        } else if s.eq_ignore_ascii_case("infobackground") {
            Ok(Self::InfoBackground)
        } else if s.eq_ignore_ascii_case("infotext") {
            Ok(Self::InfoText)
        } else if s.eq_ignore_ascii_case("menu") {
            Ok(Self::Menu)
        } else if s.eq_ignore_ascii_case("menutext") {
            Ok(Self::MenuText)
        } else if s.eq_ignore_ascii_case("scrollbar") {
            Ok(Self::Scrollbar)
        } else if s.eq_ignore_ascii_case("3ddarkshadow") {
            Ok(Self::ThreeDDarkShadow)
        } else if s.eq_ignore_ascii_case("3dface") {
            Ok(Self::ThreeDFace)
        } else if s.eq_ignore_ascii_case("3dhighlight") {
            Ok(Self::ThreeDHighlight)
        } else if s.eq_ignore_ascii_case("3dlightshadow") {
            Ok(Self::ThreeDLightShadow)
        } else if s.eq_ignore_ascii_case("3dshadow") {
            Ok(Self::ThreeDShadow)
        } else if s.eq_ignore_ascii_case("window") {
            Ok(Self::Window)
        } else if s.eq_ignore_ascii_case("windowframe") {
            Ok(Self::WindowFrame)
        } else if s.eq_ignore_ascii_case("windowtext") {
            Ok(Self::WindowText)
        } else {
            Err(format!("Unknown deprecated system color: {s}"))
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

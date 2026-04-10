//! This module defines the `EmbededResource` struct and the `EmbededType` enum,
//! which represent embedded resources in the application. The `rust_embed` crate
//! is used to include files from the specified folder, allowing for easy access
//! to assets such as icons, fonts, shaders, and browser-related files.

use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "../../assets/"]
#[include = "**/*"]
/// A struct representing embedded resources in the application,
/// using the `rust_embed` crate to include files from the specified folder.
pub(crate) struct EmbededResource;

/// Represents different types of embedded resources in the application.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EmbededType<'path> {
    Icon(&'path str),
    Font(&'path str),
    Image(&'path str),
    Shader(&'path str),
    Browser(&'path str),
    Root(&'path str),
}

impl EmbededType<'_> {
    /// Returns the path of the embedded resource based on its type and name.
    pub fn path(&self) -> String {
        match self {
            EmbededType::Icon(name) => format!("icon/{}", name),
            EmbededType::Font(name) => format!("font/{}", name),
            EmbededType::Image(name) => format!("image/{}", name),
            EmbededType::Shader(name) => format!("shader/{}", name),
            EmbededType::Browser(name) => format!("browser/{}", name),
            EmbededType::Root(name) => name.to_string(),
        }
    }
}

// === Icon Assets ===

/// Represents the main icon asset.
pub const WINDOW_ICON: EmbededType = EmbededType::Icon("icon.ico");

/// Represents the DevTools variant of the main icon asset.
pub const DEVTOOLS_ICON: EmbededType = EmbededType::Icon("devtools.ico");

pub const PLUS_ICON: EmbededType = EmbededType::Icon("plus.svg");
pub const LEFT_CHEVRON_ICON: EmbededType = EmbededType::Icon("chevron-left.svg");
pub const RIGHT_CHEVRON_ICON: EmbededType = EmbededType::Icon("chevron-right.svg");
pub const REFRESH_ICON: EmbededType = EmbededType::Icon("rotate-cw.svg");

// === Font Assets ===

pub const OPEN_SANS_BOLD: EmbededType = EmbededType::Font("OpenSans-Bold.ttf");
pub const OPEN_SANS_EXTRA_BOLD: EmbededType = EmbededType::Font("OpenSans-ExtraBold.ttf");
pub const OPEN_SANS_LIGHT: EmbededType = EmbededType::Font("OpenSans-Light.ttf");
pub const OPEN_SANS_MEDIUM: EmbededType = EmbededType::Font("OpenSans-Medium.ttf");
pub const OPEN_SANS_REGULAR: EmbededType = EmbededType::Font("OpenSans-Regular.ttf");
pub const OPEN_SANS_SEMI_BOLD: EmbededType = EmbededType::Font("OpenSans-SemiBold.ttf");

pub const ROBOTO_MONO_BOLD: EmbededType = EmbededType::Font("RobotoMono-Bold.ttf");
pub const ROBOTO_MONO_EXTRA_LIGHT: EmbededType = EmbededType::Font("RobotoMono-ExtraLight.ttf");
pub const ROBOTO_MONO_LIGHT: EmbededType = EmbededType::Font("RobotoMono-Light.ttf");
pub const ROBOTO_MONO_MEDIUM: EmbededType = EmbededType::Font("RobotoMono-Medium.ttf");
pub const ROBOTO_MONO_REGULAR: EmbededType = EmbededType::Font("RobotoMono-Regular.ttf");
pub const ROBOTO_MONO_SEMI_BOLD: EmbededType = EmbededType::Font("RobotoMono-SemiBold.ttf");
pub const ROBOTO_MONO_THIN: EmbededType = EmbededType::Font("RobotoMono-Thin.ttf");

pub const ROBOTO_SERIF_BLACK: EmbededType = EmbededType::Font("RobotoSerif-Black.ttf");
pub const ROBOTO_SERIF_BOLD: EmbededType = EmbededType::Font("RobotoSerif-Bold.ttf");
pub const ROBOTO_SERIF_EXTRA_BOLD: EmbededType = EmbededType::Font("RobotoSerif-ExtraBold.ttf");
pub const ROBOTO_SERIF_EXTRA_LIGHT: EmbededType = EmbededType::Font("RobotoSerif-ExtraLight.ttf");
pub const ROBOTO_SERIF_LIGHT: EmbededType = EmbededType::Font("RobotoSerif-Light.ttf");
pub const ROBOTO_SERIF_MEDIUM: EmbededType = EmbededType::Font("RobotoSerif-Medium.ttf");
pub const ROBOTO_SERIF_REGULAR: EmbededType = EmbededType::Font("RobotoSerif-Regular.ttf");
pub const ROBOTO_SERIF_SEMI_BOLD: EmbededType = EmbededType::Font("RobotoSerif-SemiBold.ttf");
pub const ROBOTO_SERIF_THIN: EmbededType = EmbededType::Font("RobotoSerif-Thin.ttf");

// === Shader Assets ===

/// Shader for rendering solid colors.
pub const SOLID_SHADER: EmbededType = EmbededType::Shader("solid.wgsl");

/// Shader for rendering textures.
pub const TEXTURE_SHADER: EmbededType = EmbededType::Shader("texture.wgsl");

// === Browser Assets ===

/// Represents the default CSS stylesheet.
pub const DEFAULT_CSS: EmbededType = EmbededType::Browser("default.css");
pub const DEVTOOLS_CSS: EmbededType = EmbededType::Browser("devtools.css");

/// Represents the "about:blank" HTML page.
pub const ABOUT_BLANK_HTML: EmbededType = EmbededType::Browser("about_blank.html");

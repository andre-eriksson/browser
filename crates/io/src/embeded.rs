use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "../../assets/"]
#[include = "**/*"]
pub(crate) struct EmbededResource;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EmbededType<'a> {
    Icon(&'a str),
    Font(&'a str),
    Image(&'a str),
    Shader(&'a str),
    Browser(&'a str),
    Root(&'a str),
}

impl EmbededType<'_> {
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

// === Font Assets ===

pub const OPEN_SANS_LIGHT: EmbededType = EmbededType::Font("OpenSans-Light.ttf");
pub const OPEN_SANS_MEDIUM: EmbededType = EmbededType::Font("OpenSans-Medium.ttf");
pub const OPEN_SANS_REGULAR: EmbededType = EmbededType::Font("OpenSans-Regular.ttf");
pub const OPEN_SANS_SEMI_BOLD: EmbededType = EmbededType::Font("OpenSans-SemiBold.ttf");
pub const OPEN_SANS_BOLD: EmbededType = EmbededType::Font("OpenSans-Bold.ttf");
pub const OPEN_SANS_EXTRA_BOLD: EmbededType = EmbededType::Font("OpenSans-ExtraBold.ttf");

pub const ROBOTO_MONO_THIN: EmbededType = EmbededType::Font("RobotoMono-Thin.ttf");
pub const ROBOTO_MONO_EXTRA_LIGHT: EmbededType = EmbededType::Font("RobotoMono-ExtraLight.ttf");
pub const ROBOTO_MONO_LIGHT: EmbededType = EmbededType::Font("RobotoMono-Light.ttf");
pub const ROBOTO_MONO_MEDIUM: EmbededType = EmbededType::Font("RobotoMono-Medium.ttf");
pub const ROBOTO_MONO_REGULAR: EmbededType = EmbededType::Font("RobotoMono-Regular.ttf");
pub const ROBOTO_MONO_SEMI_BOLD: EmbededType = EmbededType::Font("RobotoMono-SemiBold.ttf");
pub const ROBOTO_MONO_BOLD: EmbededType = EmbededType::Font("RobotoMono-Bold.ttf");

// === Shader Assets ===

/// Shader for rendering solid colors.
pub const SOLID_SHADER: EmbededType = EmbededType::Shader("solid.wgsl");

/// Shader for rendering textures.
pub const TEXTURE_SHADER: EmbededType = EmbededType::Shader("texture.wgsl");

// === Browser Assets ===

/// Represents the default CSS stylesheet.
pub const DEFAULT_CSS: EmbededType = EmbededType::Browser("default.css");

/// Represents the "about:blank" HTML page.
pub const ABOUT_BLANK_HTML: EmbededType = EmbededType::Browser("about_blank.html");

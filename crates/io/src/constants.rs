use crate::manager::AssetType;

// === Icon Assets ===

/// Represents the main icon asset.
pub const WINDOW_ICON: AssetType = AssetType::Icon("icon.ico");

/// Represents the DevTools variant of the main icon asset.
pub const DEVTOOLS_ICON: AssetType = AssetType::Icon("devtools.ico");

// === Font Assets ===

pub const OPEN_SANS_LIGHT: AssetType = AssetType::Font("OpenSans-Light.ttf");
pub const OPEN_SANS_MEDIUM: AssetType = AssetType::Font("OpenSans-Medium.ttf");
pub const OPEN_SANS_REGULAR: AssetType = AssetType::Font("OpenSans-Regular.ttf");
pub const OPEN_SANS_SEMI_BOLD: AssetType = AssetType::Font("OpenSans-SemiBold.ttf");
pub const OPEN_SANS_BOLD: AssetType = AssetType::Font("OpenSans-Bold.ttf");
pub const OPEN_SANS_EXTRA_BOLD: AssetType = AssetType::Font("OpenSans-ExtraBold.ttf");

pub const ROBOTO_MONO_THIN: AssetType = AssetType::Font("RobotoMono-Thin.ttf");
pub const ROBOTO_MONO_EXTRA_LIGHT: AssetType = AssetType::Font("RobotoMono-ExtraLight.ttf");
pub const ROBOTO_MONO_LIGHT: AssetType = AssetType::Font("RobotoMono-Light.ttf");
pub const ROBOTO_MONO_MEDIUM: AssetType = AssetType::Font("RobotoMono-Medium.ttf");
pub const ROBOTO_MONO_REGULAR: AssetType = AssetType::Font("RobotoMono-Regular.ttf");
pub const ROBOTO_MONO_SEMI_BOLD: AssetType = AssetType::Font("RobotoMono-SemiBold.ttf");
pub const ROBOTO_MONO_BOLD: AssetType = AssetType::Font("RobotoMono-Bold.ttf");

// === Shader Assets ===

/// Shader for rendering solid colors.
pub const SOLID_SHADER: AssetType = AssetType::Shader("solid.wgsl");

/// Shader for rendering textures.
pub const TEXTURE_SHADER: AssetType = AssetType::Shader("texture.wgsl");

// === Browser Assets ===

/// Represents the default CSS stylesheet.
pub const DEFAULT_CSS: AssetType = AssetType::Browser("default.css");

/// Represents the "about:blank" HTML page.
pub const ABOUT_BLANK_HTML: AssetType = AssetType::Browser("about_blank.html");

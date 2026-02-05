use crate::manager::EmbededAsset;

// === Icon Assets ===

/// Represents the main icon asset.
pub const WINDOW_ICON: EmbededAsset = EmbededAsset::Icon("icon.ico");

/// Represents the DevTools variant of the main icon asset.
pub const DEVTOOLS_ICON: EmbededAsset = EmbededAsset::Icon("devtools.ico");

// === Font Assets ===

pub const OPEN_SANS_LIGHT: EmbededAsset = EmbededAsset::Font("OpenSans-Light.ttf");
pub const OPEN_SANS_MEDIUM: EmbededAsset = EmbededAsset::Font("OpenSans-Medium.ttf");
pub const OPEN_SANS_REGULAR: EmbededAsset = EmbededAsset::Font("OpenSans-Regular.ttf");
pub const OPEN_SANS_SEMI_BOLD: EmbededAsset = EmbededAsset::Font("OpenSans-SemiBold.ttf");
pub const OPEN_SANS_BOLD: EmbededAsset = EmbededAsset::Font("OpenSans-Bold.ttf");
pub const OPEN_SANS_EXTRA_BOLD: EmbededAsset = EmbededAsset::Font("OpenSans-ExtraBold.ttf");

pub const ROBOTO_MONO_THIN: EmbededAsset = EmbededAsset::Font("RobotoMono-Thin.ttf");
pub const ROBOTO_MONO_EXTRA_LIGHT: EmbededAsset = EmbededAsset::Font("RobotoMono-ExtraLight.ttf");
pub const ROBOTO_MONO_LIGHT: EmbededAsset = EmbededAsset::Font("RobotoMono-Light.ttf");
pub const ROBOTO_MONO_MEDIUM: EmbededAsset = EmbededAsset::Font("RobotoMono-Medium.ttf");
pub const ROBOTO_MONO_REGULAR: EmbededAsset = EmbededAsset::Font("RobotoMono-Regular.ttf");
pub const ROBOTO_MONO_SEMI_BOLD: EmbededAsset = EmbededAsset::Font("RobotoMono-SemiBold.ttf");
pub const ROBOTO_MONO_BOLD: EmbededAsset = EmbededAsset::Font("RobotoMono-Bold.ttf");

// === Shader Assets ===

/// Shader for rendering solid colors.
pub const SOLID_SHADER: EmbededAsset = EmbededAsset::Shader("solid.wgsl");

/// Shader for rendering textures.
pub const TEXTURE_SHADER: EmbededAsset = EmbededAsset::Shader("texture.wgsl");

// === Browser Assets ===

/// Represents the default CSS stylesheet.
pub const DEFAULT_CSS: EmbededAsset = EmbededAsset::Browser("default.css");

/// Represents the "about:blank" HTML page.
pub const ABOUT_BLANK_HTML: EmbededAsset = EmbededAsset::Browser("about_blank.html");

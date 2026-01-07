use crate::manager::AssetType;

// === Icon Assets ===

/// Represents the main icon asset.
pub const WINDOW_ICON: AssetType = AssetType::Icon("icon.ico");

/// Represents the DevTools variant of the main icon asset.
pub const DEVTOOLS_ICON: AssetType = AssetType::Icon("devtools.ico");

// === Font Assets ===

/// Standard rendering font for most content.
pub const DEFAULT_FONT: AssetType = AssetType::Font("OpenSans-Regular.ttf");

/// Represents a monospaced font for code and other fixed-width content.
pub const MONOSPACE_FONT: AssetType = AssetType::Font("RobotoMono-Regular.ttf");

/// Test shader for development purposes.
pub const TEST_SHADER: AssetType = AssetType::Shader("test.wgsl");

// === Browser Assets ===

/// Represents the default CSS stylesheet.
pub const DEFAULT_CSS: AssetType = AssetType::Browser("default.css");


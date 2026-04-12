pub mod events;
pub mod keys;

pub const APP_NAME: &str = "MyBrowser";
pub const APP_ID: &str = "com.andree.mybrowser";

pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const APP_MAJOR_VERSION: &str = env!("CARGO_PKG_VERSION_MAJOR");
pub const APP_MINOR_VERSION: &str = env!("CARGO_PKG_VERSION_MINOR");
pub const APP_PATCH_VERSION: &str = env!("CARGO_PKG_VERSION_PATCH");

pub const DEVTOOLS_NAME: &str = "MyBrowser DevTools";
pub const DEVTOOLS_ID: &str = "com.andree.mybrowser.devtools";

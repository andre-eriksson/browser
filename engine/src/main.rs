pub mod headers;

use std::sync::{Arc, Mutex};

use cookies::cookie_store::CookieJar;
use tracing::Level;

use crate::headers::create_default_browser_headers;

use ui::runtime::UiRuntime;

/// The main entry point for the application
fn main() {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    let browser_headers = Arc::new(create_default_browser_headers());

    // TODO: Load cookies from persistent storage
    let cookie_jar = Arc::new(Mutex::new(CookieJar::new()));

    let ui_runtime = UiRuntime::new(browser_headers, cookie_jar);
    ui_runtime.run();
}

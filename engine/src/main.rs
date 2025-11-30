pub mod headers;

use cookies::cookie_store::CookieJar;
use tracing::Level;

use crate::headers::create_default_browser_headers;

//use ui::runtime::UiRuntime;

/// The main entry point for the application
fn main() {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    let _browser_headers = create_default_browser_headers();

    // TODO: Load cookies from persistent storage
    let _cookie_jar = CookieJar::new();

    //UiRuntime::run();
}

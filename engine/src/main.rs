use tracing::Level;

use ui::runtime::UiRuntime;

/// The main entry point for the application
fn main() {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    UiRuntime::run();
}

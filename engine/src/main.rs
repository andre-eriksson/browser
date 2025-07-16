use tracing::Level;

use ui::runtime::UiRuntime;

fn main() {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    UiRuntime::run();
}

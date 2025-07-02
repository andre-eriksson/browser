use tracing::Level;
use ui::browser::Browser;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    let browser = Browser::new();
    browser.start();
}

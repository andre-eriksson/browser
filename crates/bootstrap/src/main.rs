use std::{str::FromStr, sync::Arc};

use browser_config::BrowserConfig;
use browser_core::Browser;
use browser_headless::HeadlessEngine;
use browser_ui::Ui;
use tracing::{error, info};
use tracing_subscriber::{
    EnvFilter,
    filter::Directive,
    fmt::{self},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

/// The main entry point for the application
fn main() {
    let filter = EnvFilter::new("warn")
        .add_directive(Directive::from_str("browser=debug").unwrap())
        .add_directive(Directive::from_str("io=debug").unwrap())
        .add_directive(Directive::from_str("layout=debug").unwrap())
        .add_directive(Directive::from_str("css=debug").unwrap())
        .add_directive(Directive::from_str("cookies=debug").unwrap())
        .add_directive(Directive::from_str("html=debug").unwrap())
        .add_directive(Directive::from_str("network=debug").unwrap());

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_file(false).with_line_number(false))
        .init();

    let config = Box::leak(Box::new(BrowserConfig::new()));
    let browser = Browser::new(config);

    if config.args().headless {
        let mut engine = HeadlessEngine::new(browser);

        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to create Tokio runtime");

        return runtime.block_on(engine.run(config));
    }

    let browser = Arc::new(tokio::sync::Mutex::new(browser));

    let ui = Ui::run(browser, config);

    if let Err(e) = ui {
        error!("Application exited with error: {:?}", e);
    }

    info!("Application exited successfully.");
}

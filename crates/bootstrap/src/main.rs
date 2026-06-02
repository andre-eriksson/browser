use std::{str::FromStr, sync::Arc};

use browser_args::{BrowserArgs, Parser};
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
        .add_directive(Directive::from_str("layout=trace").unwrap())
        .add_directive(Directive::from_str("css=debug").unwrap())
        .add_directive(Directive::from_str("cookies=debug").unwrap())
        .add_directive(Directive::from_str("html=debug").unwrap())
        .add_directive(Directive::from_str("network=debug").unwrap());

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_file(false).with_line_number(false))
        .init();

    let args = BrowserArgs::parse();
    let browser = Browser::new(&args);

    if args.headless.enabled {
        let mut engine = HeadlessEngine::new(browser);

        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to create Tokio runtime");

        return runtime.block_on(engine.run(args));
    }

    let browser = Arc::new(browser);
    let ui = Ui::run(browser, args);

    if let Err(error) = ui {
        error!(%error, "Application exited unsuccessfully.");
    } else {
        info!("Application exited successfully.");
    }
}

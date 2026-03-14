use std::{str::FromStr, sync::Arc};

use cli::{Parser, args::BrowserArgs};
use kernel::{Browser, HeadlessBrowser, HeadlessEngine};
use preferences::BrowserConfig;
use tracing::{error, info};
use tracing_subscriber::{
    EnvFilter,
    filter::Directive,
    fmt::{self},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};
use ui::Ui;

/// The main entry point for the application
fn main() {
    let filter = EnvFilter::new("warn")
        .add_directive(Directive::from_str("engine=info").unwrap())
        .add_directive(Directive::from_str("kernel=debug").unwrap())
        .add_directive(Directive::from_str("io=debug").unwrap())
        .add_directive(Directive::from_str("css=debug").unwrap())
        .add_directive(Directive::from_str("cookies=debug").unwrap())
        .add_directive(Directive::from_str("html=debug").unwrap())
        .add_directive(Directive::from_str("ui=debug").unwrap())
        .add_directive(Directive::from_str("network=debug").unwrap());

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_file(false).with_line_number(false))
        .init();

    let args = BrowserArgs::parse();
    let config = if let Some(theme) = &args.theme {
        BrowserConfig::new(theme.clone())
    } else {
        BrowserConfig::load()
    };

    if args.headless {
        let browser = HeadlessBrowser::new(&args);
        let mut engine = HeadlessEngine::new(browser);

        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        return runtime.block_on(engine.main(&args));
    }

    let browser = Browser::new(&args);
    let browser = Arc::new(tokio::sync::Mutex::new(browser));

    let ui_runtime = Ui::new(browser, args, config);
    let res = ui_runtime.run();

    if let Err(e) = res {
        error!("Application exited with error: {:?}", e);
    }

    info!("Application exited successfully.");
}

mod message;

use std::{str::FromStr, sync::Arc};

use browser_config::BrowserConfig;
use browser_core::{Browser, BrowserEvent, HeadlessBrowser, HeadlessEngine};
use cli::{Parser, args::BrowserArgs};
use tokio::sync::mpsc::unbounded_channel;
use tracing::{error, info};
use tracing_subscriber::{
    EnvFilter,
    filter::Directive,
    fmt::{self},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};
use ui::Ui;

use crate::message::ChannelEmitter;

/// The main entry point for the application
fn main() {
    let filter = EnvFilter::new("warn")
        .add_directive(Directive::from_str("engine=info").unwrap())
        .add_directive(Directive::from_str("browser_core=debug").unwrap())
        .add_directive(Directive::from_str("assets=debug").unwrap())
        .add_directive(Directive::from_str("css=debug").unwrap())
        .add_directive(Directive::from_str("cookies=debug").unwrap())
        .add_directive(Directive::from_str("html=debug").unwrap())
        .add_directive(Directive::from_str("ui=info").unwrap())
        .add_directive(Directive::from_str("network=debug").unwrap());

    tracing_subscriber::registry()
        .with(filter)
        .with(
            fmt::layer()
                .pretty()
                .with_file(false)
                .with_line_number(false),
        )
        .init();

    let args = BrowserArgs::parse();
    let config = BrowserConfig::default();

    let (event_sender, event_receiver) = unbounded_channel::<BrowserEvent>();
    let emitter = Box::new(ChannelEmitter::new(event_sender));

    if args.headless {
        let browser = HeadlessBrowser::new(&args, emitter);
        let mut engine = HeadlessEngine::new(browser);

        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        return runtime.block_on(engine.main(&args));
    }

    let browser = Browser::new(&args, emitter);
    let browser = Arc::new(tokio::sync::Mutex::new(browser));

    let ui_runtime = Ui::new(browser, event_receiver, args, config);
    let res = ui_runtime.run();

    if let Err(e) = res {
        error!("Application exited with error: {:?}", e);
    }

    info!("Application exited successfully.");
}

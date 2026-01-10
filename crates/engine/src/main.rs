pub mod cli;
pub mod headers;
pub mod headless;
pub mod message;

use std::{str::FromStr, sync::Arc};

use browser_core::{Browser, BrowserEvent, HeadlessBrowser};
use clap::Parser;
use cookies::cookie_store::CookieJar;
use network::clients::reqwest::ReqwestClient;
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

use crate::{
    cli::Args, headers::create_default_browser_headers, headless::HeadlessEngine,
    message::ChannelEmitter,
};

/// The main entry point for the application
fn main() {
    let filter = EnvFilter::new("warn")
        .add_directive(Directive::from_str("engine=info").unwrap())
        .add_directive(Directive::from_str("browser_core=debug").unwrap())
        .add_directive(Directive::from_str("assets=debug").unwrap())
        .add_directive(Directive::from_str("css=debug").unwrap())
        .add_directive(Directive::from_str("html_dom=info").unwrap())
        .add_directive(Directive::from_str("html_parser=debug").unwrap())
        .add_directive(Directive::from_str("ui=info").unwrap())
        .add_directive(Directive::from_str("network=debug").unwrap());

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer())
        .init();

    let args = Args::parse();

    let (event_sender, event_receiver) = unbounded_channel::<BrowserEvent>();
    let emitter = Box::new(ChannelEmitter::new(event_sender));
    let http_client = Box::new(ReqwestClient::new());
    let browser_headers = Arc::new(create_default_browser_headers());

    // TODO: Load cookies from persistent storage
    let cookie_jar = Arc::new(std::sync::Mutex::new(CookieJar::new()));

    if args.headless {
        let browser = HeadlessBrowser::new(emitter, http_client, cookie_jar, browser_headers);
        let mut engine = HeadlessEngine::new(browser);

        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        return runtime.block_on(engine.main(&args));
    }

    let browser = Arc::new(tokio::sync::Mutex::new(Browser::new(
        emitter,
        http_client,
        cookie_jar,
        browser_headers,
    )));

    let ui_runtime = Ui::new(browser, event_receiver);
    let res = ui_runtime.run();

    if let Err(e) = res {
        error!("Application exited with error: {:?}", e);
    }

    info!("Application exited successfully.");
}

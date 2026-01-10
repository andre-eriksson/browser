pub mod cli;
pub mod headers;
pub mod headless;

use std::{str::FromStr, sync::Arc};

use browser_core::{Browser, BrowserEvent, Emitter};
use clap::Parser;
use cookies::cookie_store::CookieJar;
use network::clients::reqwest::ReqwestClient;
use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};
use tracing::{error, info};
use tracing_subscriber::{
    EnvFilter,
    filter::Directive,
    fmt::{self},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};
use ui::Ui;

use crate::{cli::Args, headers::create_default_browser_headers, headless::headless_main};

pub struct ChannelEmitter<T> {
    sender: UnboundedSender<T>,
}

impl<T: Send + 'static> ChannelEmitter<T> {
    pub fn new(sender: UnboundedSender<T>) -> Self {
        ChannelEmitter { sender }
    }
}

impl<T: Send + 'static> Emitter<T> for ChannelEmitter<T> {
    fn emit(&self, event: T) {
        let sender = self.sender.clone();
        tokio::spawn(async move {
            if let Err(e) = sender.send(event) {
                error!("Failed to send event: {:?}", e);
            }
        });
    }

    fn clone_box(&self) -> Box<dyn Emitter<T>> {
        Box::new(ChannelEmitter {
            sender: self.sender.clone(),
        })
    }
}

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

    let browser = Arc::new(tokio::sync::Mutex::new(Browser::new(
        emitter,
        http_client,
        cookie_jar,
        browser_headers,
    )));

    if args.headless {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        return runtime.block_on(headless_main(args, browser));
    }

    let ui_runtime = Ui::new(browser, event_receiver);
    let res = ui_runtime.run();

    if let Err(e) = res {
        error!("Application exited with error: {:?}", e);
    }

    info!("Application exited successfully.");
}

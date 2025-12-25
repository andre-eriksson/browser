pub mod cli;
pub mod headers;

use std::{
    str::FromStr,
    sync::{Arc, Mutex},
};

use browser_core::{
    browser::{Browser, Commandable},
    commands::BrowserCommand,
};
use clap::Parser;
use cookies::cookie_store::CookieJar;
use event::{Emitter, browser::BrowserEvent};
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
use ui::runtime::Ui;

use crate::{cli::Args, headers::create_default_browser_headers};

//use ui::runtime::UiRuntime;

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
#[tokio::main]
async fn main() {
    let filter = EnvFilter::new("warn")
        .add_directive(Directive::from_str("engine=info").unwrap())
        .add_directive(Directive::from_str("browser_core=debug").unwrap())
        .add_directive(Directive::from_str("assets=debug").unwrap())
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
    let cookie_jar = Arc::new(Mutex::new(CookieJar::new()));

    let browser = Arc::new(tokio::sync::Mutex::new(Browser::new(
        emitter,
        http_client,
        cookie_jar,
        browser_headers,
    )));

    if !args.headless {
        let ui_runtime = Ui::new(browser, event_receiver);
        let res = ui_runtime.run();

        if let Err(e) = res {
            error!("Application exited with error: {:?}", e);
        }

        info!("Application exited successfully.");
    } else {
        if !args.url.is_empty() {
            let res = browser
                .lock()
                .await
                .execute(BrowserCommand::Navigate {
                    tab_id: 0,
                    url: args.url,
                })
                .await;

            if let Err(e) = res {
                error!("Failed to navigate to URL: {:?}", e);
            }
        }

        loop {
            print!("Enter a new URL to navigate to (or press Enter to exit): ");
            std::io::Write::flush(&mut std::io::stdout()).unwrap();

            let mut input = String::new();
            if let Err(e) = std::io::stdin().read_line(&mut input) {
                error!("Failed to read input: {:?}", e);
                continue;
            }

            let command = input.trim().to_string();
            if command.is_empty() {
                break;
            }

            if command == "body" {
                browser.lock().await.print_body(0);
                continue;
            }

            let res = browser
                .lock()
                .await
                .execute(BrowserCommand::Navigate {
                    tab_id: 0,
                    url: command,
                })
                .await;

            if let Err(e) = res {
                error!("Failed to navigate to URL: {:?}", e);
                break;
            }
        }
    }
}

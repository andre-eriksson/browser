use browser_core::{BrowserCommand, Commandable, HeadlessBrowser, TabId};
use std::io::{self, Write};
use tracing::{error, info};

use crate::cli::Args;

pub struct HeadlessEngine {
    browser: HeadlessBrowser,
}

impl HeadlessEngine {
    pub fn new(browser: HeadlessBrowser) -> Self {
        HeadlessEngine { browser }
    }

    pub async fn main(&mut self, args: &Args) {
        if !args.url.is_empty() {
            let navigation_result = self
                .browser
                .execute(BrowserCommand::Navigate {
                    tab_id: TabId(0),
                    url: args.url.clone(),
                })
                .await;

            match navigation_result {
                Ok(_) => {
                    info!("Success");
                }
                Err(e) => {
                    error!("{}", e);
                }
            }
        }

        loop {
            print!("headless > ");
            io::stdout().flush().unwrap();

            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .expect("Failed to read line");

            match input.trim() {
                "exit" | "quit" => {
                    break;
                }
                cmd if cmd.starts_with("navigate ") => {
                    if let Some(command) = BrowserCommand::parse_navigate(&cmd["navigate ".len()..])
                    {
                        match self.browser.execute(command).await {
                            Ok(_) => info!("Success"),
                            Err(e) => error!("{}", e),
                        }
                    } else {
                        error!("Invalid navigate command format. Use: navigate <tab_id> <url>");
                    }
                }
                "body" => {
                    self.browser.print_body();
                }
                _ => {
                    error!("Unknown command: {}", input.trim());
                }
            }
        }

        info!("Exiting headless engine.");
    }
}

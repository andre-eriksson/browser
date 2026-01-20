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

    /// Handle a single command input
    async fn handle_command(&mut self, input: &str) -> Result<(), String> {
        match input.trim() {
            "exit" | "quit" => {
                info!("Exiting headless engine.");
                std::process::exit(0);
            }
            cmd if cmd.starts_with("navigate ") => {
                let parts: Vec<&str> = cmd["navigate ".len()..].splitn(2, ' ').collect();
                if parts.len() != 2 {
                    return Err("Usage: navigate <tab_id> <url>".to_string());
                }
                let tab_id = parts[0]
                    .parse::<usize>()
                    .map_err(|_| "Invalid tab_id".to_string())?;
                let url = parts[1].to_string();

                let navigation_result = self
                    .browser
                    .execute(BrowserCommand::Navigate {
                        tab_id: TabId(tab_id),
                        url,
                    })
                    .await;

                match navigation_result {
                    Ok(_) => {
                        info!("Navigation successful");
                        Ok(())
                    }
                    Err(e) => Err(format!("Navigation error: {}", e)),
                }
            }
            "body" => {
                self.browser.print_body();
                Ok(())
            }
            "cookies" => {
                self.browser.print_cookies();
                Ok(())
            }
            _ => Err(format!("Unknown command: {}", input.trim())),
        }
    }

    /// Main loop to process commands
    pub async fn main(&mut self, args: &Args) {
        if args.url.is_some() {
            let navigation_result = self
                .browser
                .execute(BrowserCommand::Navigate {
                    tab_id: TabId(0),
                    url: args.url.clone().unwrap(),
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

        if let Some(input_path) = args.input.as_deref()
            && !input_path.is_empty()
        {
            let content = std::fs::read_to_string(input_path).expect("Failed to read input file");
            for line in content.lines() {
                if let Err(e) = self.handle_command(line).await {
                    error!("{}", e);
                }
            }
            return;
        }

        if !args.commands.is_empty() {
            for cmd in &args.commands {
                if let Err(e) = self.handle_command(cmd).await {
                    error!("{}", e);
                }
            }
            return;
        }

        loop {
            print!("headless > ");
            io::stdout().flush().unwrap();

            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .expect("Failed to read line");

            if let Err(e) = self.handle_command(&input).await {
                error!("{}", e);
            }
        }
    }
}

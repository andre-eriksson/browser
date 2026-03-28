use browser_config::BrowserConfig;
use browser_core::{Browser, Commandable, EngineCommand, TabId};
use std::io::{self, Write};
use tracing::{error, info};

pub struct HeadlessEngine {
    browser: Browser,
}

impl HeadlessEngine {
    pub fn new(browser: Browser) -> Self {
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
                    .execute(EngineCommand::Navigate {
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
            "headers" => {
                self.print_headers();
                Ok(())
            }
            "body" => {
                self.print_body();
                Ok(())
            }
            "cookies" => {
                self.print_cookies(None);
                Ok(())
            }
            cmd if cmd.starts_with("cookies ") => {
                let domain = cmd["cookies ".len()..].trim();
                self.print_cookies(Some(domain));
                Ok(())
            }
            _ => Err(format!("Unknown command: {}", input.trim())),
        }
    }

    fn print_headers(&mut self) {
        for header in self.browser.headers().iter() {
            println!("{}: {}", header.0, header.1.to_str().unwrap_or(""));
        }
    }

    fn print_body(&mut self) {
        if let Some(active_tab) = self.browser.tab_manager().active_tab() {
            println!("{}", active_tab.page().document());
        } else {
            println!("No active tab.");
        }
    }

    fn print_cookies(&mut self, domain: Option<&str>) {
        let jar = self.browser.cookie_jar().lock().unwrap();

        if domain.is_none() {
            for cookie in jar.cookies() {
                println!("{}", cookie);
            }
            return;
        }

        let domain = domain.unwrap();

        for cookie in jar.get_cookies_for_domain(domain) {
            println!("{}", cookie);
        }
    }

    /// Main loop to process commands
    pub async fn run(&mut self, config: &BrowserConfig) {
        if config.args().url.is_some() {
            let navigation_result = self
                .browser
                .execute(EngineCommand::Navigate {
                    tab_id: TabId(0),
                    url: config.args().url.clone().unwrap(),
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

        if let Some(input_path) = config.args().input.as_deref()
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

        if !config.args().commands.is_empty() {
            for cmd in &config.args().commands {
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

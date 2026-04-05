use browser_config::BrowserConfig;
use browser_core::{Browser, History, Page};
use browser_preferences::theme::ThemeCategory;
use css_style::{AbsoluteContext, StyleTree};
use layout::{LayoutEngine, LayoutTree, Rect, TextContext};
use std::{
    io::{self, Write},
    sync::Arc,
};
use tracing::{error, info};

use crate::commands::{
    HeadlessCommand,
    content::{cmd_body, cmd_cookies, cmd_headers, cmd_info, cmd_title, cmd_url},
    dom::cmd_dom,
    layout::{cmd_layout, cmd_node, cmd_resize},
    navigation::{cmd_back, cmd_forward, cmd_navigate, cmd_reload},
};

const DEFAULT_VIEWPORT_WIDTH: f32 = 1280.0;
const DEFAULT_VIEWPORT_HEIGHT: f32 = 800.0;

pub struct HeadlessEngine {
    pub browser: Browser,
    pub page: Arc<Page>,
    pub history: History,
    pub viewport_width: f32,
    pub viewport_height: f32,
    pub layout_tree: Option<LayoutTree>,
    pub style_tree: Option<StyleTree>,
    pub text_ctx: TextContext,
}

impl HeadlessEngine {
    pub fn new(browser: Browser) -> Self {
        HeadlessEngine {
            browser,
            page: Arc::new(Page::blank()),
            history: History::new(),
            viewport_width: DEFAULT_VIEWPORT_WIDTH,
            viewport_height: DEFAULT_VIEWPORT_HEIGHT,
            layout_tree: None,
            style_tree: None,
            text_ctx: TextContext::default(),
        }
    }

    /// Handle a single command input
    async fn handle_command(&mut self, input: &str) -> Result<(), String> {
        let command = match HeadlessCommand::parse(input) {
            Ok(cmd) => cmd,
            Err(e) if e.is_empty() => return Ok(()),
            Err(e) => return Err(e),
        };

        match command {
            HeadlessCommand::Help => {
                println!("{}", HeadlessCommand::help_text());
                Ok(())
            }
            HeadlessCommand::Exit | HeadlessCommand::Quit => {
                info!("Exiting headless engine.");
                std::process::exit(0);
            }
            HeadlessCommand::Navigate { url } => cmd_navigate(self, &url).await,
            HeadlessCommand::Back => cmd_back(self).await,
            HeadlessCommand::Forward => cmd_forward(self).await,
            HeadlessCommand::Reload => cmd_reload(self).await,
            HeadlessCommand::Title => cmd_title(self),
            HeadlessCommand::Url => cmd_url(self),
            HeadlessCommand::Headers => cmd_headers(self),
            HeadlessCommand::Body => cmd_body(self),
            HeadlessCommand::Cookies { domain } => cmd_cookies(self, domain.as_deref()),
            HeadlessCommand::Dom { selector } => cmd_dom(self, &selector),
            HeadlessCommand::Node { x, y } => cmd_node(self, x, y),
            HeadlessCommand::Resize { width, height } => cmd_resize(self, width, height),
            HeadlessCommand::Layout => cmd_layout(self),
            HeadlessCommand::Info => cmd_info(self),
        }
    }

    pub(crate) fn ensure_layout(&mut self) -> Result<(), String> {
        if self.layout_tree.is_some() {
            return Ok(());
        }
        self.recompute_layout();
        if self.layout_tree.is_none() {
            return Err("Could not compute layout - no active page".to_string());
        }
        Ok(())
    }

    pub(crate) fn recompute_layout(&mut self) {
        let document = self.page.document();
        let stylesheets = self.page.stylesheets();

        let ctx = AbsoluteContext {
            root_font_size: 16.0,
            root_line_height_multiplier: 1.2,
            viewport_width: self.viewport_width,
            viewport_height: self.viewport_height,
            root_color: css_values::color::Color::BLACK,
            theme_category: ThemeCategory::Light,
            document_url: self.page.document_url(),
        };

        let style_tree = StyleTree::build(&ctx, document, stylesheets);

        let viewport = Rect::new(0.0, 0.0, self.viewport_width, self.viewport_height);
        let layout_tree = LayoutEngine::compute_layout(&style_tree, viewport, &mut self.text_ctx, None);

        self.style_tree = Some(style_tree);
        self.layout_tree = Some(layout_tree);
    }

    /// Main loop to process commands
    pub async fn run(&mut self, config: &BrowserConfig) {
        if let Some(ref url) = config.args().url
            && let Err(e) = cmd_navigate(self, url).await
        {
            error!("{}", e);
        }

        if let Some(input_path) = config.args().input.as_deref()
            && !input_path.is_empty()
        {
            let content = std::fs::read_to_string(input_path).expect("Failed to read input file");
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
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

        println!("Headless browser ready. Type 'help' for commands.");
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

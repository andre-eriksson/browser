use clap::{Parser, Subcommand};

pub mod content;
pub mod dom;
pub mod layout;
pub mod navigation;

/// Headless browser command parser
#[derive(Parser, Debug)]
#[command(
    name = "",
    no_binary_name = true,
    disable_help_subcommand = true,
    subcommand_required = true,
    arg_required_else_help = true
)]
pub struct HeadlessArgs {
    #[command(subcommand)]
    pub command: HeadlessCommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum HeadlessCommand {
    /// Show help for all commands
    Help,

    /// Exit the headless browser
    Exit,

    /// Exit the headless browser (alias for exit)
    Quit,

    /// Navigate the active tab to a URL
    Navigate {
        /// The URL to navigate to
        url: String,
    },

    /// Navigate back in history
    Back,

    /// Navigate forward in history
    Forward,

    /// Reload the current page
    Reload,

    /// Print the current page title
    Title,

    /// Print the current page URL
    Url,

    /// Print response headers
    Headers,

    /// Print the HTML document body
    Body,

    /// Print cookies (optionally filtered by domain)
    Cookies {
        /// Optional domain to filter cookies
        domain: Option<String>,
    },

    /// Query the DOM with a CSS selector and print matching elements
    Dom {
        /// CSS selector to query
        selector: String,
    },

    /// Get the layout node at the specified coordinates
    Node {
        /// X coordinate
        x: f32,
        /// Y coordinate
        y: f32,
    },

    /// Set the viewport size for layout computation
    Resize {
        /// Viewport width
        width: f32,
        /// Viewport height
        height: f32,
    },

    /// Print the computed layout tree
    Layout,

    /// Print information about the current page (title, URL, document size)
    Info,
}

impl HeadlessCommand {
    /// Parse a command string into a HeadlessCommand
    pub fn parse(input: &str) -> Result<Self, String> {
        let input = input.trim();
        if input.is_empty() {
            return Err(String::new());
        }

        let args = shell_words::split(input).map_err(|e| format!("Parse error: {}", e))?;

        if args.is_empty() {
            return Err(String::new());
        }

        HeadlessArgs::try_parse_from(&args)
            .map(|parsed| parsed.command)
            .map_err(|e| e.to_string())
    }

    /// Generate help text for all commands
    pub fn help_text() -> String {
        let mut help = String::from("Available commands:\n\n");

        help.push_str("  help                  Show this help message\n");
        help.push_str("  exit, quit            Exit the headless browser\n");
        help.push('\n');
        help.push_str("Navigation:\n");
        help.push_str("  navigate <url>        Navigate to URL\n");
        help.push_str("  back                  Navigate back in history\n");
        help.push_str("  forward               Navigate forward in history\n");
        help.push_str("  reload                Reload the current page\n");
        help.push('\n');
        help.push_str("Page Content:\n");
        help.push_str("  title                 Print page title\n");
        help.push_str("  url                   Print current URL\n");
        help.push_str("  headers               Print response headers\n");
        help.push_str("  body                  Print HTML document\n");
        help.push_str("  cookies [domain]      Print cookies\n");
        help.push_str("  info                  Print page summary\n");
        help.push('\n');
        help.push_str("Layout & DOM:\n");
        help.push_str("  dom <selector>        Query DOM with CSS selector\n");
        help.push_str("  node <x> <y>          Get layout node at coordinates\n");
        help.push_str("  layout                Print layout tree\n");
        help.push_str("  resize <w> <h>        Set viewport size\n");

        help
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_exit() {
        let cmd = HeadlessCommand::parse("exit").unwrap();
        assert!(matches!(cmd, HeadlessCommand::Exit));
    }

    #[test]
    fn test_parse_quit() {
        let cmd = HeadlessCommand::parse("quit").unwrap();
        assert!(matches!(cmd, HeadlessCommand::Quit));
    }

    #[test]
    fn test_parse_navigate() {
        let cmd = HeadlessCommand::parse("navigate https://example.com").unwrap();
        match cmd {
            HeadlessCommand::Navigate { url } => assert_eq!(url, "https://example.com"),
            _ => panic!("Expected Navigate command"),
        }
    }

    #[test]
    fn test_parse_cookies_no_domain() {
        let cmd = HeadlessCommand::parse("cookies").unwrap();
        match cmd {
            HeadlessCommand::Cookies { domain } => assert!(domain.is_none()),
            _ => panic!("Expected Cookies command"),
        }
    }

    #[test]
    fn test_parse_cookies_with_domain() {
        let cmd = HeadlessCommand::parse("cookies example.com").unwrap();
        match cmd {
            HeadlessCommand::Cookies { domain } => assert_eq!(domain, Some("example.com".to_string())),
            _ => panic!("Expected Cookies command"),
        }
    }

    #[test]
    fn test_parse_node() {
        let cmd = HeadlessCommand::parse("node 100.5 200.0").unwrap();
        match cmd {
            HeadlessCommand::Node { x, y } => {
                assert!((x - 100.5).abs() < f32::EPSILON);
                assert!((y - 200.0).abs() < f32::EPSILON);
            }
            _ => panic!("Expected Node command"),
        }
    }

    #[test]
    fn test_parse_resize() {
        let cmd = HeadlessCommand::parse("resize 1920 1080").unwrap();
        match cmd {
            HeadlessCommand::Resize { width, height } => {
                assert!((width - 1920.0).abs() < f32::EPSILON);
                assert!((height - 1080.0).abs() < f32::EPSILON);
            }
            _ => panic!("Expected Resize command"),
        }
    }

    #[test]
    fn test_parse_dom() {
        let cmd = HeadlessCommand::parse("dom div.container").unwrap();
        match cmd {
            HeadlessCommand::Dom { selector } => assert_eq!(selector, "div.container"),
            _ => panic!("Expected Dom command"),
        }
    }

    #[test]
    fn test_parse_dom_quoted() {
        let cmd = HeadlessCommand::parse(r#"dom "div.container > p""#).unwrap();
        match cmd {
            HeadlessCommand::Dom { selector } => assert_eq!(selector, "div.container > p"),
            _ => panic!("Expected Dom command"),
        }
    }

    #[test]
    fn test_parse_empty() {
        let result = HeadlessCommand::parse("");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_whitespace() {
        let result = HeadlessCommand::parse("   ");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_unknown() {
        let result = HeadlessCommand::parse("unknown_command");
        assert!(result.is_err());
    }

    #[test]
    fn test_case_insensitive_commands() {
        // Clap is case-sensitive by default, but our commands are lowercase
        let result = HeadlessCommand::parse("EXIT");
        assert!(result.is_err()); // Should fail - commands are case-sensitive
    }

    #[test]
    fn test_help() {
        let cmd = HeadlessCommand::parse("help").unwrap();
        assert!(matches!(cmd, HeadlessCommand::Help));
    }
}

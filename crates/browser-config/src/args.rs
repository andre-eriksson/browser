use clap::{ArgAction, Parser};
use manifest::{APP_NAME, APP_VERSION};

#[derive(Parser, Debug, Clone)]
#[command(
    name = APP_NAME,
    version = APP_VERSION,
    about = "A web browser implemented in Rust."
)]
pub struct BrowserArgs {
    #[arg(short = 'u', long, help = "The initial URL to load")]
    pub url: Option<String>,

    #[arg(
        short = 't',
        long,
        value_enum,
        help = "Override the theme for the session.",
        conflicts_with = "headless"
    )]
    pub theme: Option<String>,

    #[arg(long = "disable-ua-css", action = ArgAction::SetFalse, help = "Disable user agent stylesheets.")]
    pub enable_ua_css: bool,

    #[arg(
        long = "ua-compatibility",
        help = "Enable user agent compatibility mode, which makes the browser identify itself as a more common browser to improve compatibility with websites that perform user agent sniffing."
    )]
    pub ua_compatibility: bool,

    #[arg(
        short = 'U',
        long,
        long_help = "Override the default user agent string sent in HTTP requests. This can be used to improve compatibility with websites that perform user agent sniffing, or to test how a website behaves with different user agents. If not specified, a default user agent string will be used based on the operating system and compatibility mode settings."
    )]
    pub user_agent: Option<String>,

    #[arg(
        short = 'T',
        long,
        default_value_t = false,
        group = "mode",
        help_heading = "Headless Mode",
        help = "Run the browser in headless mode."
    )]
    pub headless: bool,

    #[arg(
        short = 'I',
        long,
        group = "headless-mode",
        help_heading = "Headless Mode",
        help = "Path to a file containing commands to execute in headless mode, one per line. Can't be used with --commands.",
        requires = "headless",
        conflicts_with = "command"
    )]
    pub input: Option<String>,

    #[arg(
        name = "command",
        short = 'C',
        long,
        group = "headless-mode",
        help_heading = "Headless Mode",
        help = "Commands to execute in headless mode, separated by commas. Can't be used with --input.",
        requires = "headless",
        value_delimiter = ',',
        conflicts_with = "input"
    )]
    pub commands: Vec<String>,
}

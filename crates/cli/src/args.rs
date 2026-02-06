use clap::{ArgAction, Parser};
use constants::APP_NAME;
use preferences::PresetTheme;

#[derive(Parser, Debug)]
#[command(
    name = APP_NAME,
    version,
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
    pub theme: Option<PresetTheme>,

    #[arg(long = "disable-ua-css", action = ArgAction::SetFalse, help = "Disable user agent stylesheets.")]
    pub enable_ua_css: bool,

    #[arg(
        name = "header",
        short = 'H',
        long,
        long_help = "Custom headers to include in requests.\nFormat: 'Header-Name: Header-Value'.\nWill override default headers if there are any conflicts."
    )]
    pub headers: Vec<String>,

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

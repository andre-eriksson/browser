use clap::{ArgAction, Parser};
use manifest::{APP_NAME, APP_VERSION};

use crate::args::{headless::HeadlessArgs, preferences::PreferencesArgs};

pub mod headless;
pub mod preferences;

#[derive(Parser, Debug, Clone)]
#[command(
    name = APP_NAME,
    version = APP_VERSION,
    about = "A web browser implemented in Rust."
)]
pub struct BrowserArgs {
    #[arg(short = 'u', long, help = "The initial URL to load")]
    pub url: Option<String>,

    #[command(flatten)]
    pub preferences: PreferencesArgs,

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

    #[command(flatten)]
    pub headless: HeadlessArgs,
}

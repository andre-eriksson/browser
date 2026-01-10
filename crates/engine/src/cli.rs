use clap::Parser;
use constants::APP_NAME;

#[derive(Parser, Debug)]
#[command(
    name = APP_NAME,
    version,
    about = "A web browser implemented in Rust."
)]
pub struct Args {
    #[arg(
        long,
        default_value_t = String::from("https://www.example.com"),
        help = "The initial URL to load"
    )]
    pub url: String,

    #[arg(
        short = 'H',
        long,
        default_value_t = false,
        group = "mode",
        help_heading = "Headless Mode",
        help = "Run the browser in headless mode, without a graphical user interface. Can't be used with --interactive.",
        conflicts_with = "interactive"
    )]
    pub headless: bool,

    #[arg(
        short = 'I',
        long,
        group = "headless-mode",
        help_heading = "Headless Mode",
        help = "Path to a file containing commands to execute in headless mode, one per line. Can't be used with --commands.",
        requires = "headless",
        conflicts_with = "commands"
    )]
    pub input: Option<String>,

    #[arg(
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

    #[arg(
        short,
        long,
        default_value_t = false,
        group = "mode",
        help = "Run the browser in interactive terminal mode (TUI). Can't be used with --headless. (Not yet implemented.)",
        conflicts_with = "headless"
    )]
    pub interactive: bool,
}

use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "Browser Engine",
    version,
    about = "A web browser engine implemented in Rust."
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
        help = "Run the browser in headless mode, without a graphical user interface, exclusive with interactive mode"
    )]
    pub headless: bool,

    #[arg(
        short = 'I',
        long,
        help_heading = "Headless Mode",
        help = "Path to a file containing commands to execute in headless mode (See documentation for command format)",
        requires = "headless"
    )]
    pub input: Option<String>,

    #[arg(
        short = 'C',
        long,
        help_heading = "Headless Mode",
        help = "Commands to execute in headless mode, separated by commas. Will exit after executing all commands.",
        requires = "headless",
        value_delimiter = ','
    )]
    pub commands: Vec<String>,

    #[arg(
        short,
        long,
        default_value_t = false,
        group = "mode",
        help = "Run the browser in interactive terminal mode (TUI), exclusive with headless mode",
        conflicts_with = "headless"
    )]
    pub interactive: bool,
}

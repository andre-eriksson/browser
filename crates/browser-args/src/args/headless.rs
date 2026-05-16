use clap::Args;

#[derive(Debug, Clone, Args)]
pub struct HeadlessArgs {
    #[arg(
        short = 'T',
        long = "headless",
        name = "headless",
        group = "mode",
        help_heading = "Headless Mode",
        help = "Run the browser in headless mode."
    )]
    pub enabled: bool,

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

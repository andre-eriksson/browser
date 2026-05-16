use clap::Args;

#[derive(Debug, Clone, Args)]
pub struct PreferencesArgs {
    #[arg(
        short = 't',
        long,
        help_heading = "Preferences",
        help = "Override the theme for the session.",
        conflicts_with = "headless"
    )]
    pub theme: Option<String>,

    #[arg(
        long = "force-dark",
        help_heading = "Preferences",
        help = "Force dark mode for all websites, ignoring their preferences. This can be used to improve readability in low-light environments or to reduce eye strain.",
        conflicts_with = "headless"
    )]
    pub force_dark: bool,
}

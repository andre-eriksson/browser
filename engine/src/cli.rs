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
        default_value_t = false,
        help = "Run the browser in headless mode"
    )]
    pub headless: bool,

    #[arg(
        short,
        long,
        default_value_t = String::from("https://www.example.com"),
        help = "The initial URL to load"
    )]
    pub url: String,
}

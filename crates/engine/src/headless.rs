use std::sync::Arc;

use browser_core::Browser;

use crate::cli::Args;

pub async fn headless_main(args: Args, browser: Arc<tokio::sync::Mutex<Browser>>) {
    todo!()
}

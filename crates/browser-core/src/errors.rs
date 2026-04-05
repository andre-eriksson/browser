use html_parser::errors::HtmlParsingError;
use network::errors::RequestError;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum NavigationError {
    #[error("Navigation failed due to a parsing error: {0}")]
    ParsingError(#[from] HtmlParsingError),

    #[error("Navigation failed due to a request error: {0}")]
    RequestError(#[from] RequestError),

    #[error("Navigation failed because the cookie jar is locked")]
    CookieJarLocked,
}

#[derive(Error, Debug, Clone)]
pub enum KernelError {
    #[error("Navigation error: {0}")]
    NavigationError(#[from] NavigationError),

    #[error("Failed to fetch image: {0}")]
    ImageFetchError(String),

    #[error("The current browser doesn't support this command.")]
    UnsupportedCommand,

    #[error("Failed to generate devtools HTML: {0}")]
    DevtoolsGenerationError(String),
}

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
pub enum BrowserError {
    #[error("Tab error: {0}")]
    TabError(#[from] TabError),

    #[error("Navigation error: {0}")]
    NavigationError(#[from] NavigationError),
}

#[derive(Error, Debug, Clone)]
pub enum TabError {
    #[error("Tab with ID {0:?} not found")]
    TabNotFound(usize),

    #[error("No tabs available")]
    NoTabsAvailable,

    #[error("No active tab available")]
    NoActiveTab,
}

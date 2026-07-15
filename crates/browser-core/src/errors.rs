use html_parser::errors::HtmlParsingError;
use http_fetch::errors::FetchError;
use io::errors::{MiddlewareError, ResourceError};
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum NavigationError {
    #[error("failed to parse HTML for {url}")]
    Parsing {
        url: String,
        #[source]
        source: HtmlParsingError,
    },

    #[error(transparent)]
    Middleware(#[from] MiddlewareError),

    #[error("request failed for {url}")]
    Request {
        url: String,
        #[source]
        source: FetchError,
    },

    #[error(transparent)]
    Resource(#[from] ResourceError),

    #[error("invalid navigation target: {0}")]
    Forbidden(String),

    #[error("cookie jar is locked")]
    CookieJarLocked,
}

#[derive(Error, Debug, Clone)]
pub enum CoreError {
    #[error(transparent)]
    Navigation(#[from] NavigationError),

    #[error("failed to initialize database: {0}")]
    InitializeDatabase(String),

    #[error("failed to fetch an image: {0}")]
    Image(String),

    #[error("failed to generate devtools HTML: {0}")]
    DevtoolsGeneration(String),
}

use html_parser::errors::HtmlParsingError;
use io::errors::ResourceError;
use network::errors::RequestError;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum NavigationError {
    #[error("failed to parse HTML for {url}")]
    Parsing {
        url: String,
        #[source]
        source: HtmlParsingError,
    },

    #[error("request failed for {url}")]
    Request {
        url: String,
        #[source]
        source: RequestError,
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

    #[error("failed to get an image")]
    Image,

    #[error("the current browser doesn't support this command.")]
    UnsupportedCommand,

    #[error("failed to generate devtools HTML: {0}")]
    DevtoolsGeneration(String),
}

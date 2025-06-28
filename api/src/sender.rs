use reqwest::{
    Method, Response,
    header::{HeaderMap, HeaderValue},
};
use tokio::sync::oneshot;

/// Represents a message sent over the network channel for handling various network operations.
/// This enum defines the different types of messages that can be sent to the network thread,
///
/// # Variants
/// * `InitializePage` - Initializes a page with a given URL and sends a response back.
/// * `FetchContent` - Fetches content from a specified URL with optional headers, method, and body,
///   and sends the response back.
/// * `Shutdown` - Signals the network thread to shut down gracefully.
#[derive(Debug)]
pub enum NetworkMessage {
    InitializePage {
        full_url: String,
        response: oneshot::Sender<Result<String, String>>,
    },
    FetchContent {
        url: String,
        headers: Option<HeaderMap<HeaderValue>>,
        method: Option<Method>,
        body: Option<String>,
        tag_name: String,
        response: oneshot::Sender<Result<Response, String>>,
    },
    Shutdown,
}

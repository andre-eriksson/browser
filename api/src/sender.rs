use reqwest::{
    Method, Response,
    header::{HeaderMap, HeaderValue},
};
use tokio::sync::oneshot;

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

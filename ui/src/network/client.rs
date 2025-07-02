use network::web::client::WebClient;
use reqwest::{
    Client,
    header::{
        ACCEPT, ACCEPT_ENCODING, ACCEPT_LANGUAGE, CONNECTION, HeaderMap, UPGRADE_INSECURE_REQUESTS,
    },
    redirect::Policy,
};
use tracing::error;

const USER_AGENT_HEADER: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) egui/0.31.0 (KHTML, like Gecko) Rust/1.87.0 MiniBrowser/0.1.0";
const ACCEPT_HEADER: &str =
    "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8";
const ACCEPT_LANGUAGE_HEADER: &str = "en-US,en;q=0.9";
const ACCEPT_ENCODING_HEADER: &str = "gzip, deflate, br";
const CONNECTION_HEADER: &str = "keep-alive";
const UPGRADE_INSECURE_REQUESTS_HEADER: &str = "1";

/// Sets up a new HTTP client with default headers and configuration.
pub fn setup_new_client() -> Result<WebClient, String> {
    let client_result = Client::builder()
        .user_agent(USER_AGENT_HEADER)
        .redirect(Policy::limited(10))
        .build();

    if let Err(e) = client_result {
        error!("Failed to create HTTP client: {}", e);
        return Err(format!("Failed to create HTTP client: {}", e));
    }

    let client = client_result.unwrap();

    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, ACCEPT_HEADER.parse().unwrap());
    headers.insert(ACCEPT_LANGUAGE, ACCEPT_LANGUAGE_HEADER.parse().unwrap());
    headers.insert(ACCEPT_ENCODING, ACCEPT_ENCODING_HEADER.parse().unwrap());
    headers.insert(CONNECTION, CONNECTION_HEADER.parse().unwrap());
    headers.insert(
        UPGRADE_INSECURE_REQUESTS,
        UPGRADE_INSECURE_REQUESTS_HEADER.parse().unwrap(),
    );

    Ok(WebClient::builder(client).with_headers(headers).build())
}

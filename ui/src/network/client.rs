use network::web::client::WebClient;

const USER_AGENT_HEADER: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) egui/0.31.0 (KHTML, like Gecko) Rust/1.87.0 MiniBrowser/0.1.0";
const ACCEPT_HEADER: &str =
    "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8";
const ACCEPT_LANGUAGE_HEADER: &str = "en-US,en;q=0.9";
const ACCEPT_ENCODING_HEADER: &str = "gzip, deflate, br";
const CONNECTION_HEADER: &str = "keep-alive";
const UPGRADE_INSECURE_REQUESTS_HEADER: &str = "1";

/// Sets up a new HTTP client with default headers and configuration.
pub fn setup_new_client() -> WebClient {
    WebClient::builder()
        .max_redirects(10)
        .with_user_agent(USER_AGENT_HEADER)
        .with_accept(ACCEPT_HEADER)
        .with_accept_language(ACCEPT_LANGUAGE_HEADER)
        .with_accept_encoding(ACCEPT_ENCODING_HEADER)
        .with_connection(CONNECTION_HEADER)
        .with_upgrade_insecure_requests(UPGRADE_INSECURE_REQUESTS_HEADER)
        .build()
}

use api::sender::NetworkMessage;
use engine::util::constants::{
    ACCEPT_ENCODING_HEADER, ACCEPT_HEADER, ACCEPT_LANGUAGE_HEADER, CONNECTION_HEADER,
    UPGRADE_INSECURE_REQUESTS_HEADER, USER_AGENT_HEADER,
};
use network::web::client::WebClient;
use reqwest::{
    Client,
    header::{
        ACCEPT, ACCEPT_ENCODING, ACCEPT_LANGUAGE, CONNECTION, HeaderMap, UPGRADE_INSECURE_REQUESTS,
    },
    redirect::Policy,
};
use std::thread;
use tokio::sync::mpsc::{self};
use tracing::{Level, debug, error, info};
use ui::browser::Browser;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    let client_result = Client::builder()
        .user_agent(USER_AGENT_HEADER)
        .redirect(Policy::limited(10))
        .build();

    if let Err(e) = client_result {
        error!("Failed to create HTTP client: {}", e);
        return;
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

    let web_client = WebClient::builder(client).with_headers(headers).build();

    let network_sender = spawn_network_thread(web_client);
    let browser = Browser::new(network_sender);
    browser.start();
}

fn spawn_network_thread(mut client: WebClient) -> mpsc::UnboundedSender<NetworkMessage> {
    let (sender, mut receiver) = mpsc::unbounded_channel::<NetworkMessage>();

    thread::spawn(move || {
        let runtime = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");

        runtime.block_on(async move {
            info!("[Network Thread] Starting network thread");

            while let Some(msg) = receiver.recv().await {
                match msg {
                    NetworkMessage::InitializePage { full_url, response } => {
                        debug!(
                            "[Network Thread] Received InitializePage message with URL: {}",
                            full_url
                        );

                        let result = client.setup_client_from_url(&full_url).await;

                        let _ = response.send(result);
                    }

                    NetworkMessage::FetchContent {
                        url,
                        headers,
                        method,
                        body,
                        tag_name,
                        response,
                    } => {
                        debug!(
                            "[Network Thread] Received FetchContent message for URL: {}",
                            url
                        );

                        let result = client
                            .fetch(&tag_name, &client.origin, &url, method, headers, body)
                            .await;

                        let _ = response.send(result);
                    }

                    NetworkMessage::Shutdown => {
                        info!("[Network Thread] Shutting down network thread");
                        break;
                    }
                }
            }
        })
    });

    sender
}

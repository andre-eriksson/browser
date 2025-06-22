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
use ui::browser::Browser;

fn spawn_network_thread(mut client: WebClient) -> mpsc::UnboundedSender<NetworkMessage> {
    let (sender, mut receiver) = mpsc::unbounded_channel::<NetworkMessage>();

    thread::spawn(move || {
        let runtime = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");

        runtime.block_on(async move {
            println!("Network thread started");

            while let Some(msg) = receiver.recv().await {
                match msg {
                    NetworkMessage::InitializePage { full_url, response } => {
                        println!("Received SetupClient message",);

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
                        println!("Received FetchContent message",);

                        let result = client
                            .fetch(&tag_name, &client.origin, &url, method, headers, body)
                            .await;

                        let _ = response.send(result);
                    }

                    NetworkMessage::Shutdown => {
                        println!("Network thread shutting down");
                        break;
                    }
                }
            }
        })
    });

    sender
}

#[tokio::main]
async fn main() {
    let client = Client::builder()
        .user_agent(USER_AGENT_HEADER)
        .redirect(Policy::limited(10))
        .build()
        .expect("Failed to build HTTP client");

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

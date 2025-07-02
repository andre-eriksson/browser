use std::thread;

use api::{
    logging::{EVENT, EVENT_NETWORK_THREAD_STARTED, EVENT_NETWORK_THREAD_STOPPED},
    sender::NetworkMessage,
};
use network::web::client::WebClient;
use tokio::sync::mpsc;
use tracing::{Span, debug, info};

/// Spawns a network thread that handles network requests using the provided `WebClient`.
///
/// # Arguments
/// * `client` - An instance of `WebClient` that will be used to make network requests.
/// * `span` - A tracing `Span` that will be entered when the thread starts.
///
/// # Returns
/// An `mpsc::UnboundedSender<NetworkMessage>` that can be used to send messages to the network thread.
pub fn spawn_network_thread(
    mut client: WebClient,
    span: Span,
) -> mpsc::UnboundedSender<NetworkMessage> {
    let (sender, mut receiver) = mpsc::unbounded_channel::<NetworkMessage>();

    thread::spawn(move || {
        let runtime = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");

        runtime.block_on(async move {
            let _enter = span.enter();
            info!({ EVENT } = EVENT_NETWORK_THREAD_STARTED);

            while let Some(msg) = receiver.recv().await {
                match msg {
                    NetworkMessage::InitializePage { full_url, response } => {
                        debug!("Received InitializePage message with URL: {}", full_url);

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
                        debug!("Received FetchContent message for URL: {}", url);

                        let result = client
                            .fetch(&tag_name, &client.origin, &url, method, headers, body)
                            .await;

                        let _ = response.send(result);
                    }

                    NetworkMessage::Shutdown => {
                        info!({ EVENT } = EVENT_NETWORK_THREAD_STOPPED);
                        break;
                    }
                }
            }
        })
    });

    sender
}

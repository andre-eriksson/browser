use std::{
    hash::{Hash, Hasher},
    pin::Pin,
    sync::Arc,
};

use iced::futures::{Stream, stream::unfold};
use kernel::BrowserEvent;
use tokio::sync::{Mutex, mpsc::UnboundedReceiver};

use crate::core::app::Event;

/// A hashable wrapper around the browser event receiver.
///
/// With a static hash implementation since there is only one receiver in the application.
pub struct ReceiverHandle {
    /// The receiver for browser events.
    receiver: Arc<Mutex<UnboundedReceiver<BrowserEvent>>>,
}

impl ReceiverHandle {
    pub fn new(receiver: Arc<Mutex<UnboundedReceiver<BrowserEvent>>>) -> Self {
        Self { receiver }
    }
}

impl Hash for ReceiverHandle {
    fn hash<H: Hasher>(&self, state: &mut H) {
        "browser-core-events".hash(state);
    }
}

/// Creates a stream that receives browser events and converts them to UI events.
///
/// # Arguments
/// * `handle` - A reference to the `ReceiverHandle` containing the browser event receiver.
///
/// # Returns
/// A pinned boxed stream of `Event` items.
pub fn create_browser_event_stream(
    handle: &ReceiverHandle,
) -> Pin<Box<dyn Stream<Item = Event> + Send>> {
    let receiver = handle.receiver.clone();
    Box::pin(unfold(receiver, |receiver| async move {
        let event = {
            let mut lock = receiver.lock().await;
            lock.recv().await
        };

        event.map(|browser_event| (Event::Browser(browser_event), receiver))
    }))
}

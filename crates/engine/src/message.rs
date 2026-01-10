use browser_core::Emitter;
use tokio::sync::mpsc::UnboundedSender;
use tracing::error;

/// An emitter that sends events through a Tokio unbounded channel.
pub struct ChannelEmitter<T> {
    sender: UnboundedSender<T>,
}

impl<T: Send + 'static> ChannelEmitter<T> {
    pub fn new(sender: UnboundedSender<T>) -> Self {
        ChannelEmitter { sender }
    }
}

impl<T: Send + 'static> Emitter<T> for ChannelEmitter<T> {
    fn emit(&self, event: T) {
        let sender = self.sender.clone();
        tokio::spawn(async move {
            if let Err(e) = sender.send(event) {
                error!("Failed to send event: {:?}", e);
            }
        });
    }

    fn clone_box(&self) -> Box<dyn Emitter<T>> {
        Box::new(ChannelEmitter {
            sender: self.sender.clone(),
        })
    }
}

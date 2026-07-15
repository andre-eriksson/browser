use std::{
    fmt::Debug,
    pin::Pin,
    task::{Context, Poll},
};

use bytes::{Bytes, BytesMut};
use futures::stream::once;
use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;
use tokio_stream::{Stream, StreamExt, empty};

pub type BodyStream = Pin<Box<dyn Stream<Item = Result<Bytes, String>> + Send>>;

pub enum HttpBody {
    Empty,
    Buffered(Bytes),
    Streaming(BodyStream),
}

impl HttpBody {
    pub async fn into_complete(self, max_size: usize) -> Option<CompleteHttpBody> {
        match self {
            HttpBody::Empty => Some(CompleteHttpBody(Bytes::new())),
            HttpBody::Buffered(bytes) if bytes.len() <= max_size => Some(CompleteHttpBody(bytes)),
            HttpBody::Buffered(_) => None,
            HttpBody::Streaming(mut stream) => {
                let mut buf = BytesMut::new();

                while let Some(next_chunk) = stream.next().await {
                    let chunk_data = next_chunk.ok()?;

                    if buf.len() + chunk_data.len() > max_size {
                        return None;
                    }

                    buf.extend_from_slice(&chunk_data);
                }

                Some(CompleteHttpBody(buf.freeze()))
            }
        }
    }

    pub fn into_stream(self) -> BodyStream {
        match self {
            HttpBody::Empty => Box::pin(empty()),
            HttpBody::Buffered(b) => Box::pin(once(async { Ok(b) })),
            HttpBody::Streaming(s) => s,
        }
    }
}

impl Debug for HttpBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => f.debug_tuple("Empty").finish(),
            Self::Buffered(bytes) => f.debug_tuple("Buffered").field(bytes).finish(),
            Self::Streaming(_) => f.debug_tuple("Streaming").field(&"<BodyStream>").finish(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteHttpBody(pub Bytes);

pub struct TeeStream<S> {
    inner: S,
    /// Should be none once we've given up on caching this one
    acc: Option<BytesMut>,
    max_size: usize,
    on_complete: Option<oneshot::Sender<Option<Bytes>>>,
}

impl<S> TeeStream<S> {
    pub fn new(inner: S, max_size: usize, on_complete: oneshot::Sender<Option<Bytes>>) -> Self {
        Self {
            inner,
            acc: Some(BytesMut::new()),
            max_size,
            on_complete: Some(on_complete),
        }
    }

    fn finish(&mut self) {
        if let Some(tx) = self.on_complete.take() {
            let _ = tx.send(self.acc.take().map(BytesMut::freeze));
        }
    }
}

impl<S> Stream for TeeStream<S>
where
    S: Stream<Item = Result<Bytes, String>> + Unpin,
{
    type Item = Result<Bytes, String>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let max_size = self.max_size;

        match Pin::new(&mut self.inner).poll_next(cx) {
            Poll::Ready(Some(Ok(chunk))) => {
                if let Some(acc) = &mut self.acc {
                    if acc.len() + chunk.len() > max_size {
                        self.acc = None;
                    } else {
                        acc.extend_from_slice(&chunk);
                    }
                }
                Poll::Ready(Some(Ok(chunk)))
            }
            Poll::Ready(Some(Err(e))) => {
                self.acc = None;
                self.finish();
                Poll::Ready(Some(Err(e)))
            }
            Poll::Ready(None) => {
                self.finish();
                Poll::Ready(None)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<S> Drop for TeeStream<S> {
    fn drop(&mut self) {
        self.acc = None;
        self.finish();
    }
}

use async_trait::async_trait;

use http_types::{
    body::HttpBody,
    response::{HeaderResponse, Response},
};

use crate::{
    client::ResponseHandle,
    decoding::{decode, decode_stream, get_encoding_order},
    errors::NetworkError,
};

pub struct DecodeResponse {
    inner: Box<dyn ResponseHandle>,
}

impl DecodeResponse {
    pub fn wrap_handle(inner: Box<dyn ResponseHandle>) -> Box<dyn ResponseHandle> {
        Box::new(Self { inner })
    }
}

#[async_trait]
impl ResponseHandle for DecodeResponse {
    fn head(&self) -> &HeaderResponse {
        self.inner.head()
    }

    async fn response(self: Box<Self>) -> Result<Response, NetworkError> {
        let mut response = self.inner.response().await?;
        let encoding_order = get_encoding_order(&response.head.headers)?;

        if encoding_order.is_empty() {
            return Ok(response);
        }

        response.body = match response.body {
            HttpBody::Empty => HttpBody::Empty,
            HttpBody::Buffered(bytes) => HttpBody::Buffered(decode(&encoding_order, bytes).await?),
            HttpBody::Streaming(stream) => HttpBody::Streaming(decode_stream(&encoding_order, stream)),
        };

        Ok(response)
    }
}

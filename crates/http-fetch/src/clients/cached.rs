use async_trait::async_trait;
use bytes::Bytes;
use http::{HeaderMap, StatusCode};
use http_cache::{block::MAX_BLOCK_SIZE, http::HttpCache};
use http_types::{
    body::{CompleteHttpBody, HttpBody, TeeStream},
    response::{CompleteResponse, HeaderResponse, Response},
};
use storage::Directory;
use tokio::sync::oneshot;
use tracing::debug;

use crate::{client::ResponseHandle, errors::NetworkError};

pub struct CachingResponse {
    dirs: Directory,
    inner: Box<dyn ResponseHandle>,
    cache: HttpCache,
    cache_key: String,
    request_headers: HeaderMap,
}

impl CachingResponse {
    pub fn new(
        dirs: Directory,
        inner: Box<dyn ResponseHandle>,
        cache: HttpCache,
        cache_key: String,
        request_headers: HeaderMap,
    ) -> Self {
        Self {
            dirs,
            inner,
            cache,
            cache_key,
            request_headers,
        }
    }

    pub fn wrap_handle(
        dirs: Directory,
        http_cache: &HttpCache,
        cache_key: String,
        handle: Box<dyn ResponseHandle>,
        request_headers: HeaderMap,
    ) -> Box<dyn ResponseHandle> {
        Box::new(CachingResponse::new(dirs, handle, http_cache.clone(), cache_key, request_headers))
    }
}

#[async_trait]
impl ResponseHandle for CachingResponse {
    fn metadata(&self) -> &HeaderResponse {
        self.inner.metadata()
    }

    async fn response(self: Box<Self>) -> Result<Response, NetworkError> {
        let CachingResponse {
            dirs,
            inner,
            cache,
            cache_key,
            request_headers,
        } = *self;

        let mut response = inner.response().await?;

        if response.head.status_code != StatusCode::OK {
            return Ok(response);
        }

        match response.body {
            HttpBody::Empty | HttpBody::Buffered(_) => {
                let resp = response
                    .into_complete(MAX_BLOCK_SIZE as usize)
                    .await
                    .ok_or(NetworkError::Decode("Unable to decode the response".to_string()))?;

                if let Err(err) = cache.store(&dirs, cache_key, resp.clone(), &request_headers) {
                    debug!(%err);
                }

                Ok(resp.into())
            }
            HttpBody::Streaming(stream) => {
                let (tx, rx) = oneshot::channel();
                let tee = TeeStream::new(stream, MAX_BLOCK_SIZE as usize, tx);
                response.body = HttpBody::Streaming(Box::pin(tee));

                let headers = response.head.headers.clone();
                let status = response.head.status_code;

                tokio::spawn(async move {
                    if let Ok(Some(bytes)) = rx.await {
                        let cached = CompleteResponse {
                            head: HeaderResponse {
                                headers,
                                status_code: status,
                            },
                            body: CompleteHttpBody(bytes),
                        };

                        if let Err(err) = cache.store(&dirs, cache_key, cached, &request_headers) {
                            debug!(%err);
                        }
                    }
                });

                Ok(response)
            }
        }
    }
}

pub struct CachedResponse {
    metadata: HeaderResponse,
    body: Bytes,
}

impl CachedResponse {
    pub fn new(response: CompleteResponse) -> Self {
        Self {
            metadata: HeaderResponse {
                status_code: response.head.status_code,
                headers: response.head.headers,
            },
            body: response.body.0,
        }
    }
}

#[async_trait]
impl ResponseHandle for CachedResponse {
    fn metadata(&self) -> &HeaderResponse {
        &self.metadata
    }

    async fn response(self: Box<Self>) -> Result<Response, NetworkError> {
        Ok(Response {
            head: HeaderResponse {
                headers: self.metadata.headers,
                status_code: self.metadata.status_code,
            },
            body: HttpBody::Buffered(self.body),
        })
    }
}

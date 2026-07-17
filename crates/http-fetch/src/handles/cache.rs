use async_trait::async_trait;
use http::{HeaderMap, StatusCode};
use tokio::sync::oneshot;
use tracing::debug;

use http_cache::{block::MAX_BLOCK_SIZE, http::HttpCache};
use http_types::{
    body::{CompleteHttpBody, HttpBody, TeeStream},
    response::{CompleteResponse, HeaderResponse, Response},
};
use io::paths::AppPaths;

use crate::{errors::NetworkError, handle::ResponseHandle};

/// A response handle that wraps another response handle and caches the response if it is cacheable.
pub struct CacheHandle {
    paths: AppPaths,
    inner: Box<dyn ResponseHandle>,
    cache: HttpCache,
    cache_key: String,
    request_headers: HeaderMap,
}

impl CacheHandle {
    pub fn new(
        paths: AppPaths,
        inner: Box<dyn ResponseHandle>,
        cache: HttpCache,
        cache_key: String,
        request_headers: HeaderMap,
    ) -> Self {
        Self {
            paths,
            inner,
            cache,
            cache_key,
            request_headers,
        }
    }

    pub fn wrap_handle(
        paths: AppPaths,
        http_cache: &HttpCache,
        cache_key: String,
        handle: Box<dyn ResponseHandle>,
        request_headers: HeaderMap,
    ) -> Box<dyn ResponseHandle> {
        Box::new(CacheHandle::new(paths, handle, http_cache.clone(), cache_key, request_headers))
    }
}

#[async_trait]
impl ResponseHandle for CacheHandle {
    fn head(&self) -> &HeaderResponse {
        self.inner.head()
    }

    async fn response(self: Box<Self>) -> Result<Response, NetworkError> {
        let CacheHandle {
            paths,
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
                if let Some(complete) = response.to_cacheable(MAX_BLOCK_SIZE as usize)
                    && let Err(err) = cache.store(&paths, cache_key, complete, &request_headers)
                {
                    debug!(%err);
                }

                Ok(response)
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

                        if let Err(err) = cache.store(&paths, cache_key, cached, &request_headers) {
                            debug!(%err);
                        }
                    }
                });

                Ok(response)
            }
        }
    }
}

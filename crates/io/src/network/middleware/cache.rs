use async_trait::async_trait;
use http_serde::http::StatusCode;
use network::{
    client::ResponseHandle,
    errors::NetworkError,
    request::Request,
    response::{HeaderResponse, Response},
};
use tracing::debug;

use crate::{CacheEntry, HttpCache, cache::errors::CacheError};

pub struct CachingResponse {
    inner: Box<dyn ResponseHandle>,
    cache: HttpCache,
    cache_key: String,
}

impl CachingResponse {
    pub fn new(inner: Box<dyn ResponseHandle>, cache: HttpCache, cache_key: String) -> Self {
        Self {
            inner,
            cache,
            cache_key,
        }
    }
}

#[async_trait]
impl ResponseHandle for CachingResponse {
    fn metadata(&self) -> &HeaderResponse {
        self.inner.metadata()
    }

    async fn response(self: Box<Self>) -> Result<Response, NetworkError> {
        let CachingResponse {
            inner,
            cache,
            cache_key,
        } = *self;

        let response = inner.response().await?;

        // TODO: Use background worker
        if response.status_code == StatusCode::OK
            && let Err(err) = cache.store(cache_key, response.clone(), &response.headers)
        {
            debug!(%err);
        }

        Ok(response)
    }
}

pub struct CachedResponse {
    metadata: HeaderResponse,
    body: Option<Vec<u8>>,
}

impl CachedResponse {
    pub fn new(response: Response) -> Self {
        Self {
            metadata: HeaderResponse {
                status_code: response.status_code,
                headers: response.headers,
            },
            body: response.body,
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
            status_code: self.metadata.status_code,
            headers: self.metadata.headers,
            body: self.body,
        })
    }
}

pub struct CacheMiddleware;

impl CacheMiddleware {
    pub fn lookup(request: &Request, http_cache: &HttpCache) -> Result<CacheEntry, CacheError> {
        match http_cache.get(request.url.as_str(), &request.headers) {
            Ok(entry) => Ok(entry),
            Err(e) => Err(e),
        }
    }

    pub fn wrap_handle(
        http_cache: &HttpCache,
        cache_key: String,
        handle: Box<dyn ResponseHandle>,
    ) -> Box<dyn ResponseHandle> {
        Box::new(CachingResponse::new(handle, http_cache.clone(), cache_key))
    }
}

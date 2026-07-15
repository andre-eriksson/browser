use std::sync::Arc;

use http::{HeaderMap, StatusCode};
use tracing::debug;

use http_cache::{
    errors::CacheError,
    http::{CacheEntry, HttpCache},
};
use http_types::{
    request::{Request, RequestContext},
    response::CompleteResponse,
};
use storage::AppPaths;

use crate::{
    client::{HttpClient, ResponseHandle},
    clients::cached::{CachedResponse, CachingResponse},
    errors::NetworkError,
    request::FetchResult,
};

pub(crate) fn cache_lookup(
    paths: &AppPaths,
    request_context: &RequestContext,
    http_cache: &HttpCache,
) -> Result<CacheEntry, CacheError> {
    http_cache.get(paths, request_context.url.as_str(), &request_context.headers)
}

pub(crate) async fn make_revalidation_request(
    mut request: Request,
    client: &dyn HttpClient,
    paths: &AppPaths,
    http_cache: &HttpCache,
    stale_data: CompleteResponse,
    revalidation_headers: HeaderMap,
) -> Result<FetchResult<Box<dyn ResponseHandle>>, NetworkError> {
    let request_headers = request.context.headers.clone();
    request.context.headers.extend(revalidation_headers);

    let url = request.context.url.to_string();
    let context = Arc::new(request.context);

    let Ok(network_request) = client.send(context, request.body).await else {
        return Err(NetworkError::ConnectionRefused);
    };

    let req = network_request.into();

    if let FetchResult::Success(handle) = req {
        let status = handle.head().status_code;
        let headers = handle.head().headers.clone();

        if status == StatusCode::NOT_MODIFIED {
            match http_cache.revalidate(&url, &headers) {
                Ok(()) => {
                    return Ok(FetchResult::Success(Box::new(CachedResponse::new(stale_data))));
                }
                Err(error) => {
                    debug!(%error, "failure to revalidate the cache entry");
                }
            }

            return Ok(FetchResult::Success(handle));
        } else if status == StatusCode::OK {
            return Ok(FetchResult::Success(CachingResponse::wrap_handle(
                paths.clone(),
                http_cache,
                url,
                handle,
                request_headers,
            )));
        }

        return Ok(FetchResult::Success(handle));
    }

    Ok(req)
}

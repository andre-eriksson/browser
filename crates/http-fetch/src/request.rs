use std::sync::Arc;

use cookies::CookieJar;
use http::HeaderMap;
use http_cache::http::HttpCache;
use http_policy::{cors::needs_preflight, referrer::apply_referrer};
use http_types::request::{Request, RequestContext};
use tracing::debug;
use url::Url;

use crate::{
    client::{HttpClient, ResponseHandle},
    errors::FetchError,
    headers::add_forbidden_headers,
};

pub enum FetchResult<T> {
    Success(T),
    ClientError(T),
    ServerError(T),
    Failed(FetchError),
}

impl From<Box<dyn ResponseHandle>> for FetchResult<Box<dyn ResponseHandle>> {
    fn from(response: Box<dyn ResponseHandle>) -> Self {
        let status_code = response.metadata().status_code;
        // debug!({ STATUS_CODE } = status_code.to_string());

        match status_code {
            _ if status_code.is_client_error() => FetchResult::ClientError(response),
            _ if status_code.is_server_error() => FetchResult::ServerError(response),
            _ => FetchResult::Success(response),
        }
    }
}

pub async fn fetch(
    current_url: Option<&Url>,
    mut request: Request,
    client: &dyn HttpClient,
    browser_headers: &HeaderMap,
    cookie_jar: &CookieJar,
    http_cache: &HttpCache,
) {
    pre(current_url, &mut request, browser_headers, cookie_jar, http_cache);

    let context = Arc::new(request.context);
    let body = request.body;

    let result = client.send(context.clone(), body).await;

    let response_handle = match result {
        Ok(handle) => handle,
        Err(error) => {
            debug!(%error);
            panic!()
            // return FetchResult::Failed(FetchError::Network(error));
        }
    };

    post(context, &*response_handle, cookie_jar, http_cache);
}

/// Pre fetch processing, such as setting request headers and cookies.
fn pre(
    current_url: Option<&Url>,
    request: &mut Request,
    browser_headers: &HeaderMap,
    cookie_jar: &CookieJar,
    http_cache: &HttpCache,
) {
    let needs_preflight =
        needs_preflight(current_url, &request.context.url, &request.context.headers, &request.context.method);

    request.context.headers.extend(browser_headers.clone());

    add_forbidden_headers(request, current_url);

    if let Some(url) = current_url {
        apply_referrer(url, request);
    }
}

/// Post fetch processing, such as decoding the response body and handling response cookies.
fn post(context: Arc<RequestContext>, response: &dyn ResponseHandle, cookie_jar: &CookieJar, http_cache: &HttpCache) {}

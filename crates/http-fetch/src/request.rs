use std::sync::Arc;

use http::{HeaderMap, header::SET_COOKIE};
use tracing::{debug, instrument, trace};
use url::Url;

use cookies::CookieJar;
use http_cache::http::{CacheEntry, HttpCache};
use http_policy::{
    cors::{is_cross_origin_request_allowed, make_preflight_request, needs_preflight},
    errors::PolicyError,
    referrer::apply_referrer,
};
use http_types::{
    properties::Credentials,
    request::{Request, RequestContext},
};
use storage::Directory;

use crate::{
    cache::{cache_lookup, make_revalidation_request},
    client::{HttpClient, ResponseHandle},
    clients::{
        cached::{CachedResponse, CachingResponse},
        decode::DecodeResponse,
    },
    cookies::{apply_cookies, handle_response_cookie},
    errors::FetchError,
    headers::add_forbidden_headers,
};

const STATUS_CODE: &str = "status_code";
const CACHE: &str = "cache";

pub enum FetchResult<T> {
    Success(T),
    ClientError(T),
    ServerError(T),
    Failed(FetchError),
}

impl From<Box<dyn ResponseHandle>> for FetchResult<Box<dyn ResponseHandle>> {
    fn from(response: Box<dyn ResponseHandle>) -> Self {
        let status_code = response.head().status_code;
        debug!({ STATUS_CODE } = status_code.to_string(), { CACHE } = "miss");

        match status_code {
            _ if status_code.is_client_error() => FetchResult::ClientError(response),
            _ if status_code.is_server_error() => FetchResult::ServerError(response),
            _ => FetchResult::Success(response),
        }
    }
}

#[instrument(skip_all, fields(url = %request.context.url, method = %request.context.method))]
pub async fn fetch(
    current_url: Option<&Url>,
    mut request: Request,
    client: &dyn HttpClient,
    browser_headers: &HeaderMap,
    dirs: &Directory,
    cookie_jar: &CookieJar,
    http_cache: &HttpCache,
) -> FetchResult<Box<dyn ResponseHandle>> {
    let needs_preflight =
        needs_preflight(current_url, &request.context.url, &request.context.headers, &request.context.method);

    add_headers(current_url, &mut request, browser_headers);

    match cache_lookup(dirs, &request.context, http_cache) {
        Ok(entry) => match entry {
            CacheEntry::Hit(data) => {
                debug!({ STATUS_CODE } = data.head.status_code.to_string(), { CACHE } = "hit");
                let cached_response = Box::new(CachedResponse::new(data));
                return FetchResult::Success(DecodeResponse::wrap_handle(cached_response));
            }
            CacheEntry::RequiresRevalidation {
                stale_data,
                revalidation_headers,
            } => {
                trace!("Cache requires revalidation for {}", request.context.url);
                return match make_revalidation_request(
                    request,
                    client,
                    dirs,
                    http_cache,
                    stale_data,
                    revalidation_headers,
                )
                .await
                {
                    Ok(res) => res,
                    Err(error) => FetchResult::Failed(FetchError::Network(error)),
                };
            }
            CacheEntry::Miss => {
                trace!("Cache Miss");
            }
        },
        Err(error) => {
            dbg!(&error);
            debug!(%error);
        }
    }

    if needs_preflight && let Err(error) = handle_preflight(current_url, client, &request.context).await {
        return error;
    }

    if !matches!(request.context.credentials, Credentials::Omit)
        && let Some(host) = &request.context.url.host()
    {
        let is_secure = request.context.url.scheme().eq_ignore_ascii_case("https");
        let cookies = cookie_jar.get_cookies(host, request.context.url.path(), is_secure);

        apply_cookies(&mut request, &cookies);
    }

    let request_context = Arc::new(request.context);
    let request_body = request.body;

    let result = client.send(request_context.clone(), request_body).await;

    let response_handle = match result {
        Ok(handle) => handle,
        Err(error) => {
            debug!(%error);
            return FetchResult::Failed(FetchError::Network(error));
        }
    };

    let response_head = response_handle.head();

    if !matches!(request_context.credentials, Credentials::Omit)
        && let Some(response_cookies) = response_head.headers.get(SET_COOKIE)
    {
        handle_response_cookie(cookie_jar, &request_context.url, response_cookies);
    }

    let cache_key = request_context.url.to_string();
    let final_handle = if response_head.status_code.is_success() {
        CachingResponse::wrap_handle(
            dirs.clone(),
            http_cache,
            cache_key,
            response_handle,
            request_context.headers.clone(),
        )
    } else {
        response_handle
    };

    let decode_handle = DecodeResponse::wrap_handle(final_handle);

    decode_handle.into()
}

fn add_headers(current_url: Option<&Url>, request: &mut Request, browser_headers: &HeaderMap) {
    request.context.headers.extend(browser_headers.clone());

    add_forbidden_headers(request, current_url);

    if let Some(url) = current_url {
        apply_referrer(url, request);
    }
}

async fn handle_preflight(
    current_url: Option<&Url>,
    client: &dyn HttpClient,
    request_context: &RequestContext,
) -> Result<(), FetchResult<Box<dyn ResponseHandle>>> {
    let current_url = current_url.as_ref().unwrap();
    let preflight_request =
        make_preflight_request(current_url, &request_context.headers, &request_context.url, &request_context.method);

    let preflight_context = Arc::new(preflight_request.context);
    let preflight_body = preflight_request.body;

    let preflight_result = client.send(preflight_context, preflight_body).await;
    match preflight_result {
        Ok(res) => {
            let fetch_result = res.into();
            match fetch_result {
                FetchResult::Success(data) => {
                    let resp = data.head();

                    if let Err(cors_error) = is_cross_origin_request_allowed(
                        &current_url.origin(),
                        &request_context.credentials,
                        &request_context.url,
                        &request_context.method,
                        &request_context.headers,
                        resp,
                    ) {
                        return Err(FetchResult::Failed(FetchError::Policy(PolicyError::Cors(cors_error))));
                    }
                }
                FetchResult::ClientError(e) | FetchResult::ServerError(e) => {
                    let resp = e.head();
                    debug!(%resp.status_code);
                    return Err(FetchResult::Failed(FetchError::PreflightFailed));
                }
                FetchResult::Failed(error) => {
                    return Err(FetchResult::Failed(error));
                }
            }
        }
        Err(error) => return Err(FetchResult::Failed(FetchError::Network(error))),
    }

    Ok(())
}

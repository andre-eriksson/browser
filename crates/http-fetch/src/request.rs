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
use io::paths::AppPaths;

use crate::{
    cache::{cache_lookup, make_revalidation_request},
    client::HttpClient,
    errors::FetchError,
    handle::ResponseHandle,
    handles::{CacheHandle, DecodeHandle, LocalHandle},
    middleware::{add_forbidden_headers, apply_cookies, handle_response_cookie},
};

const STATUS_CODE: &str = "status_code";
const CACHE: &str = "cache";

#[instrument(skip_all, fields(url = %request.context.url, method = %request.context.method))]
pub async fn fetch(
    current_url: Option<&Url>,
    mut request: Request,
    client: &dyn HttpClient,
    browser_headers: &HeaderMap,
    paths: &AppPaths,
    cookie_jar: &CookieJar,
    http_cache: &HttpCache,
) -> Result<Box<dyn ResponseHandle>, FetchError> {
    let needs_preflight =
        needs_preflight(current_url, &request.context.url, &request.context.headers, &request.context.method);

    add_headers(current_url, &mut request, browser_headers);

    match cache_lookup(paths, &request.context, http_cache) {
        Ok(entry) => match entry {
            CacheEntry::Hit(data) => {
                debug!({ STATUS_CODE } = data.head.status_code.as_u16(), { CACHE } = "hit");
                return Ok(DecodeHandle::wrap_handle(LocalHandle::new(data).into()));
            }
            CacheEntry::RequiresRevalidation {
                stale_data,
                revalidation_headers,
            } => {
                trace!("Cache requires revalidation for {}", request.context.url);
                return make_revalidation_request(request, client, paths, http_cache, stale_data, revalidation_headers)
                    .await
                    .map_err(FetchError::Network);
            }
            CacheEntry::Miss => {
                trace!("Cache Miss");
            }
        },
        Err(error) => {
            debug!(%error);
        }
    }

    if needs_preflight && let Err(error) = handle_preflight(current_url, client, &request.context).await {
        return Err(error);
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
            return Err(FetchError::Network(error));
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
        CacheHandle::wrap_handle(paths.clone(), http_cache, cache_key, response_handle, request_context.headers.clone())
    } else {
        response_handle
    };

    let decode_handle = DecodeHandle::wrap_handle(final_handle);

    Ok(decode_handle)
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
) -> Result<Box<dyn ResponseHandle>, FetchError> {
    let current_url = current_url.as_ref().unwrap();
    let preflight_request =
        make_preflight_request(current_url, &request_context.headers, &request_context.url, &request_context.method);

    let preflight_context = Arc::new(preflight_request.context);
    let preflight_body = preflight_request.body;

    let preflight_result = client.send(preflight_context, preflight_body).await;
    let preflight_response = match preflight_result {
        Ok(res) => {
            let status_code = res.head().status_code;

            if status_code.is_success() {
                if let Err(cors_error) = is_cross_origin_request_allowed(
                    &current_url.origin(),
                    &request_context.credentials,
                    &request_context.url,
                    &request_context.method,
                    &request_context.headers,
                    res.head(),
                ) {
                    return Err(FetchError::Policy(PolicyError::Cors(cors_error)));
                }
            } else {
                return Err(FetchError::PreflightFailed);
            }

            res
        }
        Err(error) => return Err(FetchError::Network(error)),
    };

    Ok(preflight_response)
}

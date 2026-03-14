use std::sync::Arc;

use html_dom::Decoder;
use network::errors::{NetworkError, RequestError};
use url::Url;

use crate::{
    BrowserEvent, TabId,
    commands::navigate::resolve_request,
    errors::{BrowserError, NavigationError, TabError},
    navigation::NavigationContext,
};

/// Loads an image from the specified URL using the browser's HTTP client, headers, and cookies.
pub(crate) async fn load_image(
    ctx: &mut dyn NavigationContext,
    tab_id: TabId,
    url: &str,
) -> Result<BrowserEvent, BrowserError> {
    let client = ctx.http_client().box_clone();
    let headers = Arc::clone(ctx.headers());

    let cookies = ctx
        .cookie_jar()
        .lock()
        .map_err(|_| BrowserError::ImageFetchError("Cookie jar locked".to_string()))?
        .cookies()
        .clone();

    let tab = ctx
        .tab_manager()
        .get_tab(tab_id)
        .ok_or(BrowserError::TabError(TabError::TabNotFound(tab_id.0)))?;
    let page = tab.page();
    let document_url = page.document_url.clone();
    let policies = *page.policies();

    let decoder = Decoder::new(url);
    let decoded_url = decoder
        .decode()
        .map_err(|e| NavigationError::RequestError(RequestError::Network(NetworkError::InvalidUrl(e.to_string()))))?;

    let absolute_url = match document_url.as_ref() {
        Some(u) => u.join(&decoded_url),
        None => Url::parse(&decoded_url),
    }
    .map_err(|e| NavigationError::RequestError(RequestError::Network(NetworkError::InvalidUrl(e.to_string()))))?;

    if absolute_url.path().ends_with(".svg") {
        return Err(BrowserError::ImageFetchError("SVG images are not supported yet".to_string()));
    }

    let (_resolved_url, response) =
        resolve_request(absolute_url, ctx, &document_url, &policies, &cookies, &headers, client.as_ref()).await?;

    let body = match response.body {
        Some(body) => body,
        None => {
            return Err(BrowserError::ImageFetchError("No body in response".to_string()));
        }
    };

    Ok(BrowserEvent::ImageFetched(tab_id, url.to_string(), body, response.headers))
}

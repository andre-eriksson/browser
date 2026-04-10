use html_dom::Decoder;
use io::DocumentPolicy;
use network::errors::{NetworkError, RequestError};
use url::Url;

use crate::{
    EngineResponse,
    commands::navigate::resolve_request,
    errors::{KernelError, NavigationError},
    navigation::NavigationContext,
};

/// Loads an image from the specified URL using the browser's HTTP client, headers, and cookies.
pub(crate) async fn load_image(
    ctx: &mut dyn NavigationContext,
    url: Url,
    policies: DocumentPolicy,
    image_url: &str,
) -> Result<EngineResponse, KernelError> {
    let client = ctx.http_client().box_clone();
    let headers = ctx.headers().clone();
    let cookies = ctx
        .cookie_jar()
        .lock()
        .map_err(|_| KernelError::ImageFetchError("Cookie jar locked".to_string()))?
        .cookies()
        .clone();

    let decoder = Decoder::new(image_url);
    let decoded_url = decoder
        .decode()
        .map_err(|e| NavigationError::RequestError(RequestError::Network(NetworkError::InvalidUrl(e.to_string()))))?;

    let absolute_url = url
        .join(&decoded_url)
        .map_err(|e| NavigationError::RequestError(RequestError::Network(NetworkError::InvalidUrl(e.to_string()))))?;

    if absolute_url.path().ends_with(".svg") {
        return Err(KernelError::ImageFetchError("SVG images are not supported yet".to_string()));
    }

    let (_resolved_url, response) =
        resolve_request(absolute_url, ctx, Some(url), &policies, &cookies, &headers, client.as_ref()).await?;

    let body = match response.body {
        Some(body) => body,
        None => {
            return Err(KernelError::ImageFetchError("No body in response".to_string()));
        }
    };

    Ok(EngineResponse::ImageFetched(image_url.to_string(), body, response.headers))
}

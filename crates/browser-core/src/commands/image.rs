use html_dom::Decoder;
use io::DocumentPolicy;
use network::errors::{NetworkError, RequestError};
use tracing::debug;
use url::Url;

use crate::{
    EngineResponse,
    commands::navigate::resolve_request,
    errors::{CoreError, NavigationError},
    navigation::NavigationContext,
};

/// Loads an image from the specified URL using the browser's HTTP client, headers, and cookies.
pub async fn load_image(
    ctx: &mut dyn NavigationContext,
    url: Url,
    policies: DocumentPolicy,
    image_url: &str,
) -> Result<EngineResponse, CoreError> {
    let client = ctx.http_client().box_clone();
    let headers = ctx.headers().clone();
    let cookies = ctx
        .cookie_jar()
        .lock()
        .map_err(|_| CoreError::Image)?
        .cookies()
        .clone();

    let decoder = Decoder::new(image_url);
    let decoded_url = decoder.decode().map_err(|error| NavigationError::Request {
        source: RequestError::Network(NetworkError::Decode(error)),
        url: image_url.to_string(),
    })?;

    let absolute_url = url
        .join(&decoded_url)
        .map_err(|error| NavigationError::Request {
            source: RequestError::Network(NetworkError::InvalidUrl(error)),
            url: image_url.to_string(),
        })?;

    if std::path::Path::new(absolute_url.path())
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("svg"))
    {
        debug!("SVG images are not supported, skipping: {}", absolute_url);
        return Err(CoreError::Image);
    }

    let (_resolved_url, response) =
        resolve_request(absolute_url, ctx, Some(url), &policies, &cookies, &headers, client.as_ref()).await?;

    let Some(body) = response.body else {
        return Err(CoreError::Image);
    };

    Ok(EngineResponse::ImageFetched(image_url.to_string(), body, response.headers))
}

use html_dom::NodeId;
use html_escape::decode_html_entities;
use http::header::CONTENT_TYPE;
use http_cache::block::MAX_BLOCK_SIZE;
use http_fetch::{
    clients::RawClient,
    errors::{FetchError, NetworkError},
    request::fetch,
};
use http_types::{
    properties::{Destination, RequestMode},
    request::Request,
};
use io::Readable;
use tracing::debug;
use url::Url;

use crate::{
    Browser, EngineResponse,
    errors::{CoreError, NavigationError},
};

impl Browser {
    /// Loads an image from the specified URL using the browser's HTTP client, headers, and cookies.
    pub async fn load_image(
        &self,
        node_ids: Vec<NodeId>,
        request_url: Url,
        image_url: &str,
    ) -> Result<EngineResponse, CoreError> {
        let client = self.http_client().box_clone();
        let headers = self.profile().config().headers().clone();

        let decoded_url = decode_html_entities(image_url);

        let absolute_url = request_url
            .join(&decoded_url)
            .map_err(|error| NavigationError::Request {
                source: FetchError::Network(NetworkError::InvalidUrl(error)),
                url: image_url.to_string(),
            })?;

        if std::path::Path::new(absolute_url.path())
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("svg"))
        {
            debug!("SVG images are not supported, skipping: {}", absolute_url);
            return Err(CoreError::Image("SVG images are not supported".to_string()));
        }

        let is_http = absolute_url.scheme() == "http" || absolute_url.scheme() == "https";

        let image_request = Request::builder_url(absolute_url)
            .destination(Destination::Image)
            .request_mode(RequestMode::Cors)
            .build();

        let response_handle = if !is_http {
            match image_request.read(&self.profile().dirs().into(), Some(MAX_BLOCK_SIZE)) {
                Ok(data) => RawClient::wrap_handle(data),
                Err(error) => {
                    debug!(%error, "Failed to load image");
                    return Err(CoreError::Image(error.to_string()));
                }
            }
        } else {
            match fetch(
                Some(&request_url),
                image_request,
                client.as_ref(),
                &headers,
                &self.profile().dirs().into(),
                self.profile().cookie_jar(),
                self.profile().http_cache(),
            )
            .await
            {
                Ok(handle) => handle,
                Err(error) => {
                    return Err(CoreError::Image(error.to_string()));
                }
            }
        };

        if !response_handle.head().status_code.is_success() {
            return Err(CoreError::Image(format!("Status Code: {}", response_handle.head().status_code.as_u16())));
        }

        let response = match response_handle.response().await {
            Ok(resp) => resp,
            Err(error) => {
                debug!(%error, "Failed to read image response: {}", image_url);
                return Err(CoreError::Image(format!("Failed to read image response: {}", error)));
            }
        };

        let Some(body) = response.body.into_complete(MAX_BLOCK_SIZE as usize).await else {
            debug!("Image body is too large or failed to read: {}", image_url);
            return Err(CoreError::Image("Image body is too large or failed to read".to_string()));
        };

        let content_type = response
            .head
            .headers
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or("application/octet-stream")
            .to_string();

        Ok(EngineResponse::ImageFetched {
            node_ids,
            content_type,
            url: image_url.to_string(),
            data: body.0.into(),
        })
    }
}

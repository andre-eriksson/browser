use html_dom::NodeId;
use html_escape::decode_html_entities;
use io::{DecodingMiddleware, DocumentPolicy};
use network::errors::{NetworkError, RequestError};
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
        node_id: NodeId,
        request_url: Url,
        policies: DocumentPolicy,
        image_url: &str,
    ) -> Result<EngineResponse, CoreError> {
        let client = self.http_client().box_clone();
        let headers = self.headers().clone();

        let decoded_url = decode_html_entities(image_url);

        let absolute_url = request_url
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

        let (_resolved_url, response) = self
            .resolve_request(absolute_url, Some(request_url), &policies, &headers, client.as_ref())
            .await?;

        let Some(body) = response.body else {
            return Err(CoreError::Image);
        };

        let decoded_data = DecodingMiddleware::decode(&response.headers, body)
            .await
            .map_err(|_| CoreError::Image)?;

        Ok(EngineResponse::ImageFetched {
            id: node_id,
            url: image_url.to_string(),
            data: decoded_data,
        })
    }
}

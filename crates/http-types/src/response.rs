use bytes::Bytes;
use http::{HeaderMap, StatusCode};
use serde::{Deserialize, Serialize};

use crate::body::{CompleteHttpBody, HttpBody};

/// Represents the first part of an HTTP response, containing headers and status code.
///
/// <https://developer.mozilla.org/en-US/docs/Web/API/Response>
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeaderResponse {
    /// The status code of the response.
    ///
    /// <https://developer.mozilla.org/en-US/docs/Web/HTTP/Status>
    #[serde(with = "http_serde::status_code")]
    pub status_code: StatusCode,

    /// The headers of the response.
    #[serde(with = "http_serde::header_map")]
    pub headers: HeaderMap,
}

impl HeaderResponse {
    /// Creates a new HTTP response.
    ///
    /// Useful for testing and constructing responses manually.
    #[must_use]
    pub const fn new(status_code: StatusCode, headers: HeaderMap) -> Self {
        Self {
            status_code,
            headers,
        }
    }
}

/// Represents a complete HTTP response, including headers, status code, and body.
///
/// <https://developer.mozilla.org/en-US/docs/Web/API/Response>
pub struct Response {
    /// The head of the response
    pub head: HeaderResponse,

    /// The body of the response.
    pub body: HttpBody,
}

impl Response {
    pub async fn into_complete(self, max_size: usize) -> Option<CompleteResponse> {
        let body = self.body.into_complete(max_size).await?;

        Some(CompleteResponse {
            head: self.head,
            body,
        })
    }

    pub fn to_cacheable(&self, max_size: usize) -> Option<CompleteResponse> {
        let body = match &self.body {
            HttpBody::Empty => CompleteHttpBody(Bytes::new()),
            HttpBody::Buffered(b) if b.len() <= max_size => CompleteHttpBody(b.clone()),
            _ => return None,
        };
        Some(CompleteResponse {
            head: self.head.clone(),
            body,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteResponse {
    pub head: HeaderResponse,
    pub body: CompleteHttpBody,
}

impl CompleteResponse {
    pub fn new(status_code: StatusCode, headers: HeaderMap, data: Bytes) -> Self {
        CompleteResponse {
            head: HeaderResponse {
                status_code,
                headers,
            },
            body: CompleteHttpBody(data),
        }
    }
}

impl From<CompleteResponse> for Response {
    fn from(complete_response: CompleteResponse) -> Self {
        Response {
            head: complete_response.head,
            body: HttpBody::Buffered(complete_response.body.0),
        }
    }
}

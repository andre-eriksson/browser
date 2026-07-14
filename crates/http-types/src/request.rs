use std::fmt::Debug;

use http::{HeaderMap, HeaderName, HeaderValue, Method};
use url::Url;

use crate::{
    body::HttpBody,
    errors::RequestError,
    properties::{Credentials, Destination, Referrer, RequestMode},
};

#[derive(Debug, Clone)]
pub struct RequestContext {
    pub method: Method,
    pub url: Url,
    pub headers: HeaderMap,
    pub credentials: Credentials,
    pub destination: Destination,
    pub referrer: Referrer,
    pub request_mode: RequestMode,
}

/// Represents an HTTP request.
///
/// Implements `From<Url>` and `From<Request>` for easy conversion to `RequestBuilder`.
///
/// <https://developer.mozilla.org/en-US/docs/Web/API/Request>
pub struct Request {
    pub context: RequestContext,

    /// The body of the request.
    pub body: HttpBody,
}

impl Request {
    pub fn builder(url: &str) -> RequestBuilder {
        RequestBuilder::new(url)
    }

    pub fn builder_url(url: Url) -> RequestBuilder {
        RequestBuilder::new_url(url)
    }
}

/// Builder for constructing HTTP requests.
pub struct RequestBuilder {
    context: RequestContext,

    /// The optional body of the request.
    body: HttpBody,
}

impl RequestBuilder {
    /// Creates a new `RequestBuilder` with the given URL.
    ///
    /// # Arguments
    /// * `url` - The URL for the request.
    ///
    /// # Panics
    /// * Panics if the URL is invalid.
    #[must_use]
    pub fn new(url: &str) -> Self {
        Self::try_new(url).unwrap()
    }

    pub fn new_url(url: Url) -> Self {
        Self::from(url)
    }

    /// Tries to create a new `RequestBuilder` with a URL relative to the current URL in the session.
    ///
    /// # Arguments
    /// * `session` - The network session containing the current URL.
    /// * `url` - The relative URL for the request.
    ///
    /// # Returns
    /// The `RequestBuilder` initialized with the joined URL, default method GET, empty headers, mode CORS, credentials `SameOrigin`, and no body.
    ///
    /// # Errors
    /// * `NetworkError::InvalidUrl` if the URL is invalid or cannot be joined with the current URL.
    pub fn from_relative_url(starting_url: &Url, url: &str) -> Result<Self, RequestError> {
        let joined_url = starting_url.join(url).map_err(RequestError::InvalidUrl)?;

        Ok(Self::from(joined_url))
    }

    /// Tries to create a new `RequestBuilder` with the given URL.
    ///
    /// # Arguments
    /// * `url` - The URL for the request.
    ///
    /// # Returns
    /// The `RequestBuilder` initialized with the given URL, default method GET, empty headers, mode CORS, credentials `SameOrigin`, and no body.
    ///
    /// # Errors
    /// * `NetworkError::InvalidUrl` if the URL is invalid.
    pub fn try_new(url: &str) -> Result<Self, RequestError> {
        let parsed_url = Url::parse(url).map_err(RequestError::InvalidUrl)?;
        let context = RequestContext {
            method: Method::GET,
            url: parsed_url,
            headers: HeaderMap::new(),
            credentials: Credentials::SameOrigin,
            destination: Destination::Document,
            referrer: Referrer::default(),
            request_mode: RequestMode::Navigate,
        };

        Ok(Self {
            context,
            body: HttpBody::Empty,
        })
    }

    /// Sets the HTTP method for the request.
    /// Unnecessary if setting GET method.
    ///
    /// # Arguments
    /// * `method` - The HTTP method to set.
    #[must_use]
    pub fn method(mut self, method: Method) -> Self {
        self.context.method = method;
        self
    }

    /// Sets the headers for the request.
    /// Will override any previously set headers.
    ///
    /// # Arguments
    /// * `headers` - The headers to set.
    #[must_use]
    pub fn headers(mut self, headers: HeaderMap) -> Self {
        self.context.headers = headers;
        self
    }

    pub fn add_headers(mut self, headers: HeaderMap) -> Self {
        self.context.headers.extend(headers);
        self
    }

    /// Appends a header to the request.
    ///
    /// # Arguments
    /// * `key` - The header name.
    /// * `value` - The header value.
    ///
    /// # Panics
    /// * Panics if the header value is invalid.
    #[must_use]
    pub fn header(self, key: HeaderName, value: &str) -> Self {
        self.try_header(key, value).unwrap()
    }

    /// Tries to append a header to the request.
    ///
    /// # Arguments
    /// * `key` - The header name.
    /// * `value` - The header value.
    ///
    /// # Returns
    /// * `Ok(Self)` if the header was added successfully.
    /// * `Err(String)` if the header value was invalid.
    ///
    /// # Errors
    /// * `NetworkError::InvalidHeader` if the header value is invalid.
    pub fn try_header(mut self, key: HeaderName, value: &str) -> Result<Self, RequestError> {
        let header_value = HeaderValue::from_str(value);

        match header_value {
            Ok(v) => {
                self.context.headers.insert(key, v);
                Ok(self)
            }
            Err(err) => Err(RequestError::InvalidHeader(format!("Invalid header value: {err}"))),
        }
    }

    /// Sets the body for the request.
    ///
    /// # Arguments
    /// * `body` - The body to set.
    #[must_use]
    pub fn body(mut self, body: HttpBody) -> Self {
        self.body = body;
        self
    }

    /// Sets the credentials mode for the request.
    ///
    /// # Arguments
    /// * `credentials` - The credentials mode to set.
    #[must_use]
    pub const fn credentials(mut self, credentials: Credentials) -> Self {
        self.context.credentials = credentials;
        self
    }

    pub const fn destination(mut self, destination: Destination) -> Self {
        self.context.destination = destination;
        self
    }

    pub const fn referrer(mut self, referrer: Referrer) -> Self {
        self.context.referrer = referrer;
        self
    }

    pub const fn request_mode(mut self, request_mode: RequestMode) -> Self {
        self.context.request_mode = request_mode;
        self
    }

    /// Finalizes and builds the Request object.
    ///
    /// # Panics
    /// * Panics if the request is invalid.
    #[must_use]
    pub fn build(self) -> Request {
        self.try_build().unwrap()
    }

    /// Finalizes and builds the Request object.
    ///
    /// # Returns
    /// The `Request` object built from the builder.
    ///
    /// # Errors
    /// * `NetworkError::InvalidRequest` if the request is invalid (e.g., GET or HEAD requests with a body).
    pub fn try_build(self) -> Result<Request, RequestError> {
        if (self.context.method == Method::GET || self.context.method == Method::HEAD)
            && !matches!(self.body, HttpBody::Empty)
        {
            return Err(RequestError::InvalidRequest("GET and HEAD requests cannot have a body".to_string()));
        }

        // TODO: More validations can be added here as needed.

        Ok(Request {
            context: self.context,
            body: self.body,
        })
    }
}

impl From<Url> for RequestBuilder {
    /// Creates a `RequestBuilder` from a Url.
    ///
    /// # Arguments
    /// * `url` - The URL for the request.
    ///
    /// # Returns
    /// * `RequestBuilder` initialized with the given URL, default method GET, empty headers, mode CORS, credentials `SameOrigin`, and no body.
    fn from(url: Url) -> Self {
        Self {
            context: RequestContext {
                method: Method::GET,
                url,
                headers: HeaderMap::new(),
                credentials: Credentials::SameOrigin,
                destination: Destination::Document,
                referrer: Referrer::default(),
                request_mode: RequestMode::Navigate,
            },
            body: HttpBody::Empty,
        }
    }
}

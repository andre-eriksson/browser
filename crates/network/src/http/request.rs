use errors::network::NetworkError;
use http::{HeaderMap, HeaderName, HeaderValue, Method};
use url::{Origin, Url};

/// Request mode for the request.
///
/// <https://developer.mozilla.org/en-US/docs/Web/API/Request/mode>
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RequestMode {
    /// Disallows cross-origin requests. If a request is made to another origin with this mode set, the result is an error.
    SameOrigin,

    /// Disables CORS for cross-origin requests. The response is opaque, meaning that its headers and body are not available to JavaScript.
    NoCors,

    /// If the request is cross-origin then it will use the Cross-Origin Resource Sharing (CORS) mechanism.
    Cors,

    /// A mode for supporting navigation. The navigate value is intended to be used only by HTML navigation.
    /// A navigate request is created only while navigating between documents.
    Navigate,
}

/// Credentials mode for the request.
///
/// <https://developer.mozilla.org/en-US/docs/Web/API/Request/credentials>
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Credentials {
    /// Always include credentials even for cross-origin requests.
    Include,

    /// Only send and include credentials for same-origin requests. (default)
    SameOrigin,

    /// Never send or include credentials with the request.
    Omit,
}

/// Represents an HTTP request.
///
/// Implements `From<Url>` and `From<Request>` for easy conversion to `RequestBuilder`.
///
/// <https://developer.mozilla.org/en-US/docs/Web/API/Request>
#[derive(Debug, Clone)]
pub struct Request {
    /// The HTTP method of the request.
    pub method: Method,

    /// The URL of the request.
    pub url: Url,

    /// The headers of the request.
    pub headers: HeaderMap,

    /// The mode of the request.
    pub mode: RequestMode,

    /// The credentials mode of the request.
    pub credentials: Credentials,

    /// The optional body of the request.
    pub body: Option<Vec<u8>>,

    /// The origin of the request.
    pub origin: Origin,
}

/// Builder for constructing HTTP requests.
pub struct RequestBuilder {
    /// The HTTP method of the request, default is GET.
    method: Method,

    /// The URL of the request.
    url: Url,

    /// The headers of the request.
    headers: HeaderMap,

    /// The mode of the request, default is CORS.
    mode: RequestMode,

    /// The credentials mode of the request, default is SameOrigin.
    credentials: Credentials,

    /// The optional body of the request.
    body: Option<Vec<u8>>,
}

impl RequestBuilder {
    /// Creates a new RequestBuilder with the given URL.
    ///
    /// # Arguments
    /// * `url` - The URL for the request.
    ///
    /// # Panics
    /// * Panics if the URL is invalid.
    pub fn new(url: &str) -> Self {
        RequestBuilder::try_new(url).unwrap()
    }

    /// Tries to create a new RequestBuilder with a URL relative to the current URL in the session.
    ///
    /// # Arguments
    /// * `session` - The network session containing the current URL.
    /// * `url` - The relative URL for the request.
    ///
    /// # Returns
    /// * `Ok(RequestBuilder)` if the URL is valid.
    /// * `Err(HttpError)` if the URL is invalid or if there is no base URL in the session.
    pub fn from_relative_url(starting_url: &Url, url: &str) -> Result<Self, NetworkError> {
        let joined_url = starting_url.join(url).map_err(|err| {
            NetworkError::InvalidUrl(format!("Failed to join URL '{}': {}", url, err))
        })?;

        Ok(RequestBuilder::from(joined_url))
    }

    /// Tries to create a new RequestBuilder with the given URL.
    ///
    /// # Arguments
    /// * `url` - The URL for the request.
    ///
    /// # Returns
    /// * `Ok(RequestBuilder)` if the URL is valid.
    /// * `Err(HttpError)` if the URL is invalid.
    pub fn try_new(url: &str) -> Result<Self, NetworkError> {
        let parsed_url = Url::parse(url).map_err(|err| {
            NetworkError::InvalidUrl(format!("Failed to parse URL '{}': {}", url, err))
        })?;

        Ok(RequestBuilder {
            method: Method::GET,
            url: parsed_url,
            headers: HeaderMap::new(),
            mode: RequestMode::Cors,
            credentials: Credentials::SameOrigin,
            body: None,
        })
    }

    /// Sets the HTTP method for the request.
    /// Unnecessary if setting GET method.
    ///
    /// # Arguments
    /// * `method` - The HTTP method to set.
    pub fn method(mut self, method: Method) -> Self {
        self.method = method;
        self
    }

    /// Sets the headers for the request.
    /// Will override any previously set headers.
    ///
    /// # Arguments
    /// * `headers` - The headers to set.
    pub fn headers(mut self, headers: HeaderMap) -> Self {
        self.headers = headers;
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
    pub fn try_header(mut self, key: HeaderName, value: &str) -> Result<Self, NetworkError> {
        let header_value = HeaderValue::from_str(value);

        match header_value {
            Ok(v) => {
                self.headers.insert(key, v);
                Ok(self)
            }
            Err(err) => Err(NetworkError::InvalidHeader(format!(
                "Invalid header value: {}",
                err
            ))),
        }
    }

    /// Sets the body for the request.
    ///
    /// # Arguments
    /// * `body` - The body to set.
    pub fn body(mut self, body: Vec<u8>) -> Self {
        self.body = Some(body);
        self
    }

    /// Sets the mode for the request.
    ///
    /// # Arguments
    /// * `mode` - The request mode to set.
    pub fn mode(mut self, mode: RequestMode) -> Self {
        self.mode = mode;
        self
    }

    /// Sets the credentials mode for the request.
    ///
    /// # Arguments
    /// * `credentials` - The credentials mode to set.
    pub fn credentials(mut self, credentials: Credentials) -> Self {
        self.credentials = credentials;
        self
    }

    /// Finalizes and builds the Request object.
    ///
    /// # Panics
    /// * Panics if the request is invalid.
    pub fn build(self) -> Request {
        self.try_build().unwrap()
    }

    /// Finalizes and builds the Request object.
    ///
    /// # Returns
    /// * `Ok(Request)` if the request is valid.
    /// * `Err(HttpError)` if the request is invalid.
    pub fn try_build(self) -> Result<Request, NetworkError> {
        if (self.method == Method::GET || self.method == Method::HEAD) && self.body.is_some() {
            return Err(NetworkError::InvalidRequest(
                "GET and HEAD requests cannot have a body".to_string(),
            ));
        }

        // TODO: More validations can be added here as needed.

        let origin = self.url.origin();

        Ok(Request {
            method: self.method,
            url: self.url,
            headers: self.headers,
            mode: self.mode,
            credentials: self.credentials,
            body: self.body,
            origin,
        })
    }
}

impl From<Url> for RequestBuilder {
    /// Creates a RequestBuilder from a Url.
    ///
    /// # Arguments
    /// * `url` - The URL for the request.
    ///
    /// # Returns
    /// * `RequestBuilder` initialized with the given URL, default method GET, empty headers, mode CORS, credentials SameOrigin, and no body.
    fn from(url: Url) -> Self {
        RequestBuilder {
            method: Method::GET,
            url,
            headers: HeaderMap::new(),
            mode: RequestMode::Cors,
            credentials: Credentials::SameOrigin,
            body: None,
        }
    }
}

impl From<Request> for RequestBuilder {
    fn from(request: Request) -> Self {
        RequestBuilder {
            method: request.method,
            url: request.url,
            headers: request.headers,
            mode: request.mode,
            credentials: request.credentials,
            body: request.body,
        }
    }
}

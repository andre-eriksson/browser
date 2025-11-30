use http::{HeaderMap, HeaderName, HeaderValue, Method};
use url::{Origin, Url};

/// Request mode for the request.
///
/// <https://developer.mozilla.org/en-US/docs/Web/API/Request/mode>
#[derive(PartialEq, Eq)]
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
#[derive(PartialEq, Eq)]
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
    // TODO: Implement error handling, see .unwrap() usages and GET with body restriction.

    pub fn new(url: &str) -> Self {
        RequestBuilder {
            method: Method::GET,
            url: Url::parse(url).unwrap(),
            headers: HeaderMap::new(),
            mode: RequestMode::Cors,
            credentials: Credentials::SameOrigin,
            body: None,
        }
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
    pub fn header(mut self, key: HeaderName, value: &str) -> Self {
        self.headers
            .insert(key, HeaderValue::from_str(value).unwrap());
        self
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
    pub fn build(self) -> Request {
        // TODO: Validate the request, e.g., body with GET/HEAD methods, etc.

        let origin = self.url.origin();

        Request {
            method: self.method,
            url: self.url,
            headers: self.headers,
            mode: self.mode,
            credentials: self.credentials,
            body: self.body,
            origin,
        }
    }
}

impl From<Url> for RequestBuilder {
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

use strum::{AsRefStr, EnumString};

#[derive(Debug, Clone, Copy, EnumString, AsRefStr)]
#[strum(serialize_all = "lowercase")]
pub enum Destination {
    Audio,
    Audioworklet,
    Document,
    Embed,
    Empty,
    Fencedframe,
    Font,
    Frame,
    Iframe,
    Image,
    Json,
    Manifest,
    Object,
    Paintworklet,
    Report,
    Script,
    Serviceworker,
    Sharedworker,
    Style,
    Track,
    Video,
    Webidentity,
    Worker,
    Xslt,
}

/// Request mode for the request.
///
/// <https://developer.mozilla.org/en-US/docs/Web/API/Request/mode>
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, AsRefStr)]
#[strum(serialize_all = "kebab_case")]
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

    /// `This is a special mode used only when establishing a WebSocket connection.`
    Websocket,

    /// `This is a special mode used only by WebTransport(url, options).`
    Webtransport,
}

/// Credentials mode for the request.
///
/// <https://developer.mozilla.org/en-US/docs/Web/API/Request/credentials>
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Credentials {
    /// Always include credentials even for cross-origin requests.
    Include,

    /// Only send and include credentials for same-origin requests. (default)
    #[default]
    SameOrigin,

    /// Never send or include credentials with the request.
    Omit,
}

/// Defines the referrer policy options for network requests.
///
/// <https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers/Referrer-Policy>
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Referrer {
    /// The Referer header will be omitted: sent requests do not include any referrer information.
    NoReferrer,

    /// Send the origin, path, and query string in Referer when the protocol security level stays the same or improves (HTTP→HTTP, HTTP→HTTPS, HTTPS→HTTPS).
    /// Don't send the Referer header for requests to less secure destinations (HTTPS→HTTP, HTTPS→file).
    NoReferrerWhenDowngrade,

    /// Send only the origin in the Referer header.
    /// For example, a document at <https://example.com/page.html> will send the referrer <https://example.com>/.
    Origin,

    /// When performing a same-origin request, send the origin, path, and query string.
    /// Send only the origin for cross origin requests and requests to less secure destinations (HTTPS→HTTP).
    OriginWhenCrossOrigin,

    /// Send the origin, path, and query string for same-origin requests. Don't send the Referer header for cross-origin requests.
    SameOrigin,

    /// Send only the origin when the protocol security level stays the same (HTTPS→HTTPS).
    /// Don't send the Referer header to less secure destinations (HTTPS→HTTP).
    StrictOrigin,

    /// Send the origin, path, and query string when performing a same-origin request.
    /// For cross-origin requests send the origin (only) when the protocol security level stays same (HTTPS→HTTPS).
    /// Don't send the Referer header to less secure destinations (HTTPS→HTTP).
    #[default]
    StrictOriginWhenCrossOrigin,

    /// Send the origin, path, and query string when performing any request, regardless of security.
    ///
    /// # Notes
    /// * Warning: This policy will leak potentially-private information from HTTPS resource URLs to insecure origins. Carefully consider the impact of this setting.
    UnsafeUrl,
}

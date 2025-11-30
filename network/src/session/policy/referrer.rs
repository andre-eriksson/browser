/// Defines the referrer policy options for network requests.
///
/// <https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers/Referrer-Policy>
pub enum ReferrerPolicy {
    /// The Referer header will be omitted: sent requests do not include any referrer information.
    NoReferrer,

    /// Send the origin, path, and query string in Referer when the protocol security level stays the same or improves (HTTP→HTTP, HTTP→HTTPS, HTTPS→HTTPS).
    /// Don't send the Referer header for requests to less secure destinations (HTTPS→HTTP, HTTPS→file).
    NoReferrerWhenDowngrade,

    /// Send only the origin in the Referer header.
    /// For example, a document at https://example.com/page.html will send the referrer https://example.com/.
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
    StrictOriginWhenCrossOrigin,

    /// Send the origin, path, and query string when performing any request, regardless of security.
    ///
    /// # Notes
    /// * Warning: This policy will leak potentially-private information from HTTPS resource URLs to insecure origins. Carefully consider the impact of this setting.
    UnsafeUrl,
}

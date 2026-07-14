use http::{HeaderValue, header::REFERER};
use url::Url;

use http_types::{properties::Referrer, request::Request};

/// Applies the appropriate Referrer header to the request based on the current URL and referrer policy.
///
/// # Arguments
/// * `current_url` - The URL of the current document.
/// * `request` - The HTTP request to which the Referrer header will be applied.
/// * `referrer_policy` - The referrer policy to be applied.
///
/// # Notes
/// This function modifies the `request` in place by adding the Referrer header if applicable.
pub fn apply_referrer(current_url: &Url, request: &mut Request) {
    match request.context.referrer {
        Referrer::NoReferrer => {}
        Referrer::NoReferrerWhenDowngrade => {
            if current_url.scheme() == "https" && request.context.url.scheme() == "http" {
                return;
            }

            let value = HeaderValue::from_str(current_url.as_str());
            if let Ok(value) = value {
                request.context.headers.insert(REFERER, value);
            }
        }
        Referrer::Origin => {
            let origin = current_url.origin().ascii_serialization();
            let value = HeaderValue::from_str(&origin);

            if let Ok(value) = value {
                request.context.headers.insert(REFERER, value);
            }
        }
        Referrer::OriginWhenCrossOrigin => {
            if current_url.origin() != request.context.url.origin()
                || current_url.scheme() == "https" && request.context.url.scheme() == "http"
            {
                let origin = current_url.origin().ascii_serialization();
                let value = HeaderValue::from_str(&origin);

                if let Ok(value) = value {
                    request.context.headers.insert(REFERER, value);
                }
            } else {
                let value = HeaderValue::from_str(current_url.as_str());
                if let Ok(value) = value {
                    request.context.headers.insert(REFERER, value);
                }
            }
        }
        Referrer::SameOrigin => {
            if current_url.origin() == request.context.url.origin() {
                let value = HeaderValue::from_str(current_url.as_str());
                if let Ok(value) = value {
                    request.context.headers.insert(REFERER, value);
                }
            }
        }
        Referrer::StrictOrigin => {
            if current_url.scheme() == "https" && request.context.url.scheme() == "http" {
                return;
            }

            let origin = current_url.origin().ascii_serialization();
            let value = HeaderValue::from_str(&origin);

            if let Ok(value) = value {
                request.context.headers.insert(REFERER, value);
            }
        }
        Referrer::StrictOriginWhenCrossOrigin => {
            if current_url.scheme() == "https" && request.context.url.scheme() == "http" {
                return;
            }

            if current_url.origin() == request.context.url.origin() {
                let value = HeaderValue::from_str(current_url.as_str());
                if let Ok(value) = value {
                    request.context.headers.insert(REFERER, value);
                }
            } else {
                let origin = current_url.origin().ascii_serialization();
                let value = HeaderValue::from_str(&origin);

                if let Ok(value) = value {
                    request.context.headers.insert(REFERER, value);
                }
            }
        }

        Referrer::UnsafeUrl => {
            let value = HeaderValue::from_str(current_url.as_str());
            if let Ok(value) = value {
                request.context.headers.insert(REFERER, value);
            }
        }
    }
}

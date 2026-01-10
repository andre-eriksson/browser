use http::{HeaderValue, header::REFERER};
use url::Url;

use crate::{http::request::Request, session::policy::referrer::ReferrerPolicy};

/// Applies the appropriate Referrer header to the request based on the current URL and referrer policy.
///
/// # Arguments
/// * `current_url` - The URL of the current document.
/// * `request` - The HTTP request to which the Referrer header will be applied.
/// * `referrer_policy` - The referrer policy to be applied.
///
/// # Notes
/// This function modifies the `request` in place by adding the Referrer header if applicable.
pub fn apply_referrer(current_url: &Url, request: &mut Request, referrer_policy: &ReferrerPolicy) {
    match referrer_policy {
        ReferrerPolicy::NoReferrer => {}
        ReferrerPolicy::NoReferrerWhenDowngrade => {
            if current_url.scheme() == "https" && request.url.scheme() == "http" {
                return;
            }

            let value = HeaderValue::from_str(current_url.as_str());
            if let Ok(value) = value {
                request.headers.insert(REFERER, value);
            }
        }
        ReferrerPolicy::Origin => {
            let origin = current_url.origin().ascii_serialization();
            let value = HeaderValue::from_str(&origin);

            if let Ok(value) = value {
                request.headers.insert(REFERER, value);
            }
        }
        ReferrerPolicy::OriginWhenCrossOrigin => {
            if current_url.origin() != request.url.origin()
                || current_url.scheme() == "https" && request.url.scheme() == "http"
            {
                let origin = current_url.origin().ascii_serialization();
                let value = HeaderValue::from_str(&origin);

                if let Ok(value) = value {
                    request.headers.insert(REFERER, value);
                }
            } else {
                let value = HeaderValue::from_str(current_url.as_str());
                if let Ok(value) = value {
                    request.headers.insert(REFERER, value);
                }
            }
        }
        ReferrerPolicy::SameOrigin => {
            if current_url.origin() == request.url.origin() {
                let value = HeaderValue::from_str(current_url.as_str());
                if let Ok(value) = value {
                    request.headers.insert(REFERER, value);
                }
            }
        }
        ReferrerPolicy::StrictOrigin => {
            if current_url.scheme() == "https" && request.url.scheme() == "http" {
                return;
            }

            let origin = current_url.origin().ascii_serialization();
            let value = HeaderValue::from_str(&origin);

            if let Ok(value) = value {
                request.headers.insert(REFERER, value);
            }
        }
        ReferrerPolicy::StrictOriginWhenCrossOrigin => {
            if current_url.scheme() == "https" && request.url.scheme() == "http" {
                return;
            }

            if current_url.origin() != request.url.origin() {
                let origin = current_url.origin().ascii_serialization();
                let value = HeaderValue::from_str(&origin);

                if let Ok(value) = value {
                    request.headers.insert(REFERER, value);
                }
            } else {
                let value = HeaderValue::from_str(current_url.as_str());
                if let Ok(value) = value {
                    request.headers.insert(REFERER, value);
                }
            }
        }

        ReferrerPolicy::UnsafeUrl => {
            let value = HeaderValue::from_str(current_url.as_str());
            if let Ok(value) = value {
                request.headers.insert(REFERER, value);
            }
        }
    }
}

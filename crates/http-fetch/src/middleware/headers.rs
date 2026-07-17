use http::{HeaderMap, HeaderName, HeaderValue};
use rand::RngExt;
use url::Url;

use http_types::request::Request;
use manifest::{APP_MAJOR_VERSION, APP_NAME};

pub fn add_forbidden_headers(request: &mut Request, current_url: Option<&Url>) {
    add_secure_low_entropy_headers(&mut request.context.headers);
    add_forbidden_fetch_headers(request, current_url);

    // TODO:
    //  * Add high entropy headers.
    //  * Add semi-persistent server response headers, e.g., if the server requests 'Sec-CH-UA-Arch' via the Accept-CH header,
    //    we can store this information and include it in future requests to the same server.
    //  * Add support for user consent via settings for both low and high entropy headers, e.g., if the user has given consent to share high entropy headers, we can include them in future requests.
    //
    // -- High entropy headers --
    //headers.insert(
    //    HeaderName::from_bytes(b"sec-ch-ua-arch").unwrap(),
    //    HeaderValue::from_str(&System::cpu_arch()).unwrap(),
    //);
    //let bitness = if cfg!(target_pointer_width = "64") {
    //    "64"
    //} else {
    //    "32"
    //};
    //headers.insert(HeaderName::from_bytes(b"sec-ch-ua-bitness").unwrap(), HeaderValue::from_str(bitness).unwrap());
    //headers.insert(
    //    HeaderName::from_bytes(b"sec-ch-ua-form-factors").unwrap(),
    //    HeaderValue::from_str("Desktop").unwrap(),
    //);
    //headers.insert(
    //    HeaderName::from_bytes(b"sec-ch-ua-full-version-list").unwrap(),
    //    HeaderValue::from_str(&format!("\" {not_a_brand}\";v=\"99.0.0\", \"{APP_NAME}\";v=\"{APP_VERSION}\"",))
    //        .unwrap(),
    //);
    //headers.insert(HeaderName::from_bytes(b"sec-ch-ua-model").unwrap(), HeaderValue::from_str("").unwrap());
    //if let Some(version) = System::os_version() {
    //    headers.insert(
    //        HeaderName::from_bytes(b"sec-ch-ua-platform-version").unwrap(),
    //        HeaderValue::from_str(&version).unwrap(),
    //    );
    //}
    //headers.insert(HeaderName::from_bytes(b"sec-ch-ua-wow64").unwrap(), HeaderValue::from_str("?0").unwrap());
}

fn add_forbidden_fetch_headers(request: &mut Request, current_url: Option<&Url>) {
    request.context.headers.insert(
        HeaderName::from_bytes(b"sec-fetch-dest").unwrap(),
        HeaderValue::from_str(request.context.destination.as_ref()).unwrap(),
    );
    request.context.headers.insert(
        HeaderName::from_bytes(b"sec-fetch-mode").unwrap(),
        HeaderValue::from_str(request.context.request_mode.as_ref()).unwrap(),
    );

    // TODO: Have a more detailed way to check if the user activated the search and probably wait until it is stable.
    // if is_user {
    //    headers.insert(HeaderName::from_bytes(b"sec-fetch-user").unwrap(), HeaderValue::from_str("?1").unwrap());
    // }

    if let Some(url) = current_url {
        let fetch_site = if url.origin() == request.context.url.origin() {
            "same-origin"
        } else if url.domain() == request.context.url.domain() {
            "same-site"
        } else {
            "cross-site"
        };

        request
            .context
            .headers
            .insert(HeaderName::from_bytes(b"sec-fetch-site").unwrap(), HeaderValue::from_str(fetch_site).unwrap());
    } else {
        request
            .context
            .headers
            .insert(HeaderName::from_bytes(b"sec-fetch-site").unwrap(), HeaderValue::from_str("none").unwrap());
    }
}

fn add_secure_low_entropy_headers(headers: &mut HeaderMap) {
    let not_a_brand = generate_not_a_brand();

    headers.insert(
        HeaderName::from_bytes(b"sec-ch-ua").unwrap(),
        HeaderValue::from_str(&format!("\" {not_a_brand}\";v=\"99\", \"{APP_NAME}\";v=\"{APP_MAJOR_VERSION}\""))
            .unwrap(),
    );
    headers.insert(HeaderName::from_bytes(b"sec-ch-ua-mobile").unwrap(), HeaderValue::from_str("?0").unwrap());
    headers.insert(
        HeaderName::from_bytes(b"sec-ch-ua-platform").unwrap(),
        HeaderValue::from_str(std::env::consts::OS).unwrap(),
    );
}

fn generate_not_a_brand() -> String {
    let mut rng = rand::rng();
    let number = rng.random_range(0..=5);

    match number {
        0 => "Not;A Brand".to_string(),
        1 => "Not A;Brand".to_string(),
        2 => "Not/A)Brand".to_string(),
        3 => "Not:A Brand".to_string(),
        4 => "(Not(A:Brand".to_string(),
        _ => "Not-A Brand".to_string(),
    }
}

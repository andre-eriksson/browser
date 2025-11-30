use cookie::Cookie;
use cookies::cookie_store::CookieJar;
use http::{
    HeaderMap, Method,
    header::{ACCESS_CONTROL_REQUEST_HEADERS, ACCESS_CONTROL_REQUEST_METHOD, ORIGIN, SET_COOKIE},
};
use url::Url;

use crate::{
    http::{
        client::HttpClient,
        request::{Request, RequestBuilder},
        response::Response,
    },
    session::{
        middleware::{
            cookies::apply_cookies,
            cors::is_cors_allowed,
            referrer::apply_referrer,
            simple::{is_simple_method, is_simple_request},
        },
        policy::referrer::ReferrerPolicy,
    },
};

pub struct NetworkSession<'a> {
    current_url: Option<Url>,

    browser_headers: HeaderMap,
    cookie_jar: &'a mut CookieJar,
    client: Box<dyn HttpClient>,

    // Policies
    referrer: ReferrerPolicy,
}

impl<'a> NetworkSession<'a> {
    pub fn new(
        client: Box<dyn HttpClient>,
        browser_headers: HeaderMap,
        cookie_jar: &'a mut CookieJar,
    ) -> Self {
        NetworkSession {
            current_url: None,
            browser_headers,
            cookie_jar,
            referrer: ReferrerPolicy::StrictOriginWhenCrossOrigin,
            client,
        }
    }

    async fn raw_request(&self, request: Request) -> Result<Response, Box<dyn std::error::Error>> {
        self.client.send(request).await
    }

    pub async fn send(&mut self, request: Request) -> Result<Response, Box<dyn std::error::Error>> {
        let user_headers = request.headers.clone();
        let mut request = request;
        request.headers.extend(self.browser_headers.clone());

        let url = &request.url.clone();

        if let Some(current_url) = &self.current_url {
            apply_referrer(current_url, &mut request, &self.referrer);
        }

        if is_simple_request(&request.method, &user_headers) {
            apply_cookies(&mut request, self.cookie_jar);

            let resp = self.raw_request(request).await;

            if let Ok(response) = &resp {
                let domain = url.domain().unwrap_or_default();
                self.handle_response_cookies(response, domain);
            }

            return resp;
        }

        let preflight_response = self
            .preflight_request(&user_headers, &request.url, &request.method)
            .await?;

        if !preflight_response.status_code.to_string().starts_with('2') {
            return Err("CORS preflight request failed".into());
        }

        if !is_cors_allowed(
            &request.origin,
            &request.credentials,
            &request.url,
            &request.method,
            &user_headers,
            preflight_response,
        ) {
            return Err("CORS policy does not allow this request".into());
        }

        apply_cookies(&mut request, self.cookie_jar);

        let resp = self.raw_request(request).await;

        if let Ok(response) = &resp {
            let domain = url.domain().unwrap_or_default();
            self.handle_response_cookies(response, domain);
        }

        resp
    }

    pub fn handle_response_cookies(&mut self, response: &Response, request_domain: &str) {
        for (name, value) in response.headers.iter() {
            if name == SET_COOKIE
                && let Ok(cookie_str) = value.to_str()
                && let Ok(cookie) = Cookie::parse(cookie_str.to_string())
            {
                self.cookie_jar.add_cookie(cookie, request_domain);
            }
        }
    }

    pub fn set_current_url(&mut self, url: Url) {
        self.current_url = Some(url);
    }

    pub fn clear_current_url(&mut self) {
        self.current_url = None;
    }

    async fn preflight_request(
        &self,
        headers: &HeaderMap,
        url: &Url,
        method: &Method,
    ) -> Result<Response, Box<dyn std::error::Error>> {
        if self.current_url.is_none() {
            return Err("No current URL set for CORS preflight request".into());
        }

        let origin = self
            .current_url
            .as_ref()
            .unwrap()
            .origin()
            .ascii_serialization();

        let request_headers = headers
            .iter()
            .map(|(k, _)| k.as_str().to_lowercase())
            .collect::<Vec<String>>()
            .join(", ");

        let preflight_build = RequestBuilder::from(url.clone())
            .method(Method::OPTIONS)
            .header(ORIGIN, &origin)
            .header(ACCESS_CONTROL_REQUEST_HEADERS, &request_headers);

        if !is_simple_method(method) {
            let preflight_request = preflight_build
                .header(ACCESS_CONTROL_REQUEST_METHOD, method.as_str())
                .build();

            return self.raw_request(preflight_request).await;
        }

        let preflight_request = preflight_build.build();

        self.raw_request(preflight_request).await
    }
}

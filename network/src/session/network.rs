use std::sync::{Arc, Mutex};

use cookie::Cookie;
use cookies::cookie_store::CookieJar;
use errors::network::NetworkError;
use http::{
    HeaderMap, Method,
    header::{ACCESS_CONTROL_REQUEST_HEADERS, ACCESS_CONTROL_REQUEST_METHOD, ORIGIN, SET_COOKIE},
};
use url::Url;

use crate::{
    http::{
        client::{HttpClient, ResponseHandle},
        request::{Request, RequestBuilder},
        response::HeaderResponse,
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

#[derive(Clone)]
pub struct NetworkSession {
    current_url: Option<Url>,

    browser_headers: Arc<HeaderMap>,
    cookie_jar: Arc<Mutex<CookieJar>>,
    client: Box<dyn HttpClient>,

    // Policies
    referrer: ReferrerPolicy,
}

impl NetworkSession {
    pub fn new(
        client: Box<dyn HttpClient>,
        browser_headers: Arc<HeaderMap>,
        cookie_jar: Arc<Mutex<CookieJar>>,
        referrer: Option<ReferrerPolicy>,
    ) -> Self {
        NetworkSession {
            current_url: None,
            browser_headers,
            cookie_jar,
            referrer: referrer.unwrap_or(ReferrerPolicy::StrictOriginWhenCrossOrigin),
            client,
        }
    }

    async fn raw_request(&self, request: Request) -> Result<Box<dyn ResponseHandle>, NetworkError> {
        self.client.send(request).await
    }

    pub async fn send(
        &mut self,
        request: Request,
    ) -> Result<Box<dyn ResponseHandle>, NetworkError> {
        let user_headers = request.headers.clone();
        let mut request = request;
        request
            .headers
            .extend(self.browser_headers.as_ref().clone());

        let url = &request.url.clone();

        if let Some(current_url) = &self.current_url {
            apply_referrer(current_url, &mut request, &self.referrer);
        }

        if is_simple_request(&request.method, &user_headers) {
            {
                if let Ok(jar) = self.cookie_jar.lock().as_mut() {
                    apply_cookies(&mut request, jar);
                }
            }

            let resp = self.raw_request(request).await;

            if let Ok(response) = &resp {
                let domain = url.domain().unwrap_or_default();
                self.handle_response_cookies(
                    HeaderResponse {
                        status_code: response.metadata().status_code,
                        headers: response.metadata().headers.clone(),
                    },
                    domain,
                );
            }

            return resp;
        }

        let preflight_response = self
            .preflight_request(&user_headers, &request.url, &request.method)
            .await;

        let preflight_response = match preflight_response {
            Ok(resp) => resp,
            Err(e) => return Err(e),
        };

        let cors = is_cors_allowed(
            &request.origin,
            &request.credentials,
            &request.url,
            &request.method,
            &user_headers,
            preflight_response,
        );

        if let Err(e) = cors {
            return Err(e);
        }

        // TODO: Logging?
        if let Ok(jar) = self.cookie_jar.lock().as_ref() {
            apply_cookies(&mut request, jar);
        }

        let resp = self.raw_request(request).await;

        if let Ok(response) = &resp {
            let domain = url.domain().unwrap_or_default();
            self.handle_response_cookies(
                HeaderResponse {
                    status_code: response.metadata().status_code,
                    headers: response.metadata().headers.clone(),
                },
                domain,
            );
        }

        resp
    }

    pub fn handle_response_cookies(&mut self, response: HeaderResponse, request_domain: &str) {
        for (name, value) in response.headers.iter() {
            // TODO: Logging?

            let cookie_str = match value.to_str() {
                Ok(s) => s,
                Err(_) => continue,
            };

            let cookie = match Cookie::parse(cookie_str.to_string()) {
                Ok(c) => c,
                Err(_) => continue,
            };

            if name == SET_COOKIE {
                let mut jar = self.cookie_jar.lock();
                if let Ok(jar) = jar.as_mut() {
                    jar.add_cookie(cookie, request_domain);
                }
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
    ) -> Result<HeaderResponse, NetworkError> {
        if self.current_url.is_none() {
            return Err(NetworkError::RequestFailed(
                "No current URL set for CORS preflight request".to_string(),
            ));
        }

        let origin = match self.current_url.as_ref() {
            Some(u) => u.origin().ascii_serialization(),
            None => {
                return Err(NetworkError::RequestFailed(
                    "No current Origin set for CORS preflight request".to_string(),
                ));
            }
        };

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

            let res = self.raw_request(preflight_request).await;

            return match res {
                Ok(response) => Ok(HeaderResponse {
                    status_code: response.metadata().status_code,
                    headers: response.metadata().headers.clone(),
                }),
                Err(e) => Err(e),
            };
        }

        let preflight_request = preflight_build.build();

        let res = self.raw_request(preflight_request).await;

        match res {
            Ok(response) => Ok(HeaderResponse {
                status_code: response.metadata().status_code,
                headers: response.metadata().headers.clone(),
            }),
            Err(e) => Err(e),
        }
    }
}

impl std::fmt::Debug for NetworkSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NetworkSession")
            .field("current_url", &self.current_url)
            .finish()
    }
}

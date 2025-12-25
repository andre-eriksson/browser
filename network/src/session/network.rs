use std::sync::{Arc, Mutex};

use cookies::cookie_store::CookieJar;
use errors::network::NetworkError;
use http::{
    HeaderMap, Method,
    header::{ACCESS_CONTROL_REQUEST_HEADERS, ACCESS_CONTROL_REQUEST_METHOD, ORIGIN, SET_COOKIE},
};
use tracing::{debug, instrument, trace};
use url::Url;

use crate::{
    http::{
        client::{HttpClient, ResponseHandle},
        request::{Request, RequestBuilder},
        response::HeaderResponse,
    },
    session::{
        middleware::{
            cookies::{apply_cookies, handle_response_cookie},
            cors::is_cors_allowed,
            referrer::apply_referrer,
            simple::{is_simple_method, is_simple_request},
        },
        policy::referrer::ReferrerPolicy,
    },
};

#[derive(Clone)]
pub struct NetworkSession {
    pub current_url: Option<Url>,

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

    #[instrument(skip(self, request), fields(url = %request.url, method = %request.method))]
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
            trace!("Simple request, skipping CORS preflight");
            {
                if let Ok(jar) = self.cookie_jar.lock().as_mut() {
                    apply_cookies(&mut request, jar);
                }
            }

            let resp = self.raw_request(request).await;

            if let Ok(response) = &resp {
                self.handle_response_headers(
                    &HeaderResponse {
                        status_code: response.metadata().status_code,
                        headers: response.metadata().headers.clone(),
                    },
                    url,
                )
            }

            return resp;
        }

        let preflight_response = self
            .preflight_request(&user_headers, &request.url, &request.method)
            .await;

        let preflight_response = match preflight_response {
            Ok(resp) => resp,
            Err(e) => {
                debug!("CORS preflight request failed: {}", e);
                return Err(e);
            }
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
            debug!("CORS Validation failed: {}", e);
            return Err(e);
        }

        if let Ok(jar) = self.cookie_jar.lock().as_ref() {
            apply_cookies(&mut request, jar);
        }

        let url = &request.url.clone();
        let resp = self.raw_request(request).await;

        if let Ok(response) = &resp {
            self.handle_response_headers(
                &HeaderResponse {
                    status_code: response.metadata().status_code,
                    headers: response.metadata().headers.clone(),
                },
                url,
            )
        }

        resp
    }

    fn handle_response_headers(&self, response: &HeaderResponse, url: &Url) {
        let mut jar = self.cookie_jar.lock();

        for (name, value) in response.headers.iter() {
            if name == SET_COOKIE {
                if let Ok(jar) = jar.as_mut() {
                    handle_response_cookie(jar, url.domain().unwrap_or_default(), value);
                }
            }

            // TODO: Handle other response headers as needed
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

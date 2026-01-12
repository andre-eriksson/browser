use std::sync::{Arc, Mutex};

use cookies::cookie_store::CookieJar;
use errors::network::{NetworkError, RequestError};
use http::{
    HeaderMap, Method,
    header::{ACCESS_CONTROL_REQUEST_HEADERS, ACCESS_CONTROL_REQUEST_METHOD, ORIGIN, SET_COOKIE},
};
use network::http::{
    client::{HttpClient, ResponseHandle},
    request::{Request, RequestBuilder},
    response::HeaderResponse,
};
use telemetry::keys::STATUS_CODE;
use tracing::{debug, instrument, trace};
use url::Url;

use crate::{
    service::network::{
        middleware::{
            cookies::CookieMiddleware, cors::CorsMiddleware, referrer::ReferrerMiddleware,
            simple::SimpleMiddleware,
        },
        request::RequestResult,
    },
    tab::page::Page,
};

#[derive(Clone)]
pub struct NetworkService {
    client: Box<dyn HttpClient>,
    cookie_jar: Arc<Mutex<CookieJar>>,
    browser_headers: Arc<HeaderMap>,
}

impl NetworkService {
    pub fn new(
        client: Box<dyn HttpClient>,
        cookie_jar: Arc<Mutex<CookieJar>>,
        browser_headers: Arc<HeaderMap>,
    ) -> Self {
        NetworkService {
            client,
            cookie_jar,
            browser_headers,
        }
    }

    pub fn cookies(&self) -> Arc<Mutex<CookieJar>> {
        self.cookie_jar.clone()
    }

    fn convert_response(
        response: Result<Box<dyn ResponseHandle>, NetworkError>,
    ) -> RequestResult<Box<dyn ResponseHandle>> {
        match response {
            Ok(resp) => match resp.metadata().status_code {
                _ if resp.metadata().status_code.is_client_error() => {
                    RequestResult::ClientError(resp)
                }
                _ if resp.metadata().status_code.is_server_error() => {
                    RequestResult::ServerError(resp)
                }
                _ => RequestResult::Success(resp),
            },
            Err(e) => RequestResult::Failed(RequestError::Network(e)),
        }
    }

    async fn raw_fetch(&mut self, request: Request) -> RequestResult<Box<dyn ResponseHandle>> {
        let response = self.client.send(request).await;

        Self::convert_response(response)
    }

    #[instrument(skip(self, page, request), fields(method = %request.method, url = %request.url))]
    pub async fn fetch(
        &mut self,
        page: &mut Page,
        request: Request,
    ) -> RequestResult<Box<dyn ResponseHandle>> {
        let mut request = request;
        let user_headers = request.headers.clone();
        request
            .headers
            .extend(self.browser_headers.as_ref().clone());

        // First request to set document URL
        if page.document_url.is_none() {
            if request.method != Method::GET {
                return RequestResult::Failed(RequestError::InvalidMethod(
                    "First request must be a GET request".to_string(),
                ));
            }

            let url = request.url.clone();
            let resp = self.raw_fetch(request).await;

            return match resp {
                RequestResult::Failed(e) => {
                    debug!("{}", e);

                    RequestResult::Failed(e)
                }
                RequestResult::Success(resp) => {
                    self.handle_response_headers(
                        &HeaderResponse {
                            status_code: resp.metadata().status_code,
                            headers: resp.metadata().headers.clone(),
                        },
                        &url,
                    );
                    page.document_url = Some(url);

                    debug!({ STATUS_CODE } = resp.metadata().status_code.to_string());

                    RequestResult::Success(resp)
                }
                RequestResult::ServerError(resp) => {
                    debug!({ STATUS_CODE } = resp.metadata().status_code.to_string());

                    RequestResult::ServerError(resp)
                }
                RequestResult::ClientError(resp) => {
                    debug!({ STATUS_CODE } = resp.metadata().status_code.to_string());

                    RequestResult::ClientError(resp)
                }
            };
        }

        if let Some(current_url) = &page.document_url {
            ReferrerMiddleware::apply_referrer(
                current_url,
                &mut request,
                &page.policies().as_ref().referrer,
            );
        }

        if SimpleMiddleware::is_simple_request(&request.method, &user_headers) {
            trace!("Simple request detected, skipping CORS preflight");
            if let Ok(jar) = self.cookie_jar.lock().as_mut() {
                CookieMiddleware::apply_cookies(&mut request, jar);
            }

            let url = &request.url.clone();
            let resp = self.client.send(request).await;

            if let Ok(response) = &resp {
                debug!({ STATUS_CODE } = response.metadata().status_code.to_string());

                self.handle_response_headers(
                    &HeaderResponse {
                        status_code: response.metadata().status_code,
                        headers: response.metadata().headers.clone(),
                    },
                    url,
                );
            }

            return Self::convert_response(resp);
        }

        let preflight_response = match self
            .preflight_request(page, &user_headers, &request.url, &request.method)
            .await
        {
            RequestResult::Success(resp) => resp,
            _ => {
                return RequestResult::Failed(RequestError::PreflightFailed);
            }
        };

        if let Err(e) = CorsMiddleware::is_allowed(
            &request.origin,
            &request.credentials,
            &request.url,
            &request.method,
            &user_headers,
            preflight_response,
        ) {
            return RequestResult::Failed(e);
        }

        if let Ok(jar) = self.cookie_jar.lock().as_ref() {
            CookieMiddleware::apply_cookies(&mut request, jar);
        }

        let url = &request.url.clone();
        let resp = self.client.send(request).await;

        if let Ok(response) = &resp {
            debug!({ STATUS_CODE } = response.metadata().status_code.to_string());

            self.handle_response_headers(
                &HeaderResponse {
                    status_code: response.metadata().status_code,
                    headers: response.metadata().headers.clone(),
                },
                url,
            );
        }

        Self::convert_response(resp)
    }

    fn handle_response_headers(&self, response: &HeaderResponse, url: &Url) {
        let mut jar = self.cookie_jar.lock();

        for (name, value) in response.headers.iter() {
            if name == SET_COOKIE
                && let Ok(jar) = jar.as_mut()
            {
                CookieMiddleware::handle_response_cookie(
                    jar,
                    url.domain().unwrap_or_default(),
                    value,
                );
            }

            // TODO: Handle other response headers as needed
        }
    }

    async fn preflight_request(
        &self,
        page: &Page,
        headers: &HeaderMap,
        url: &Url,
        method: &Method,
    ) -> RequestResult<HeaderResponse> {
        let page = match page.document_url.as_ref() {
            Some(u) => u,
            None => {
                return RequestResult::Failed(RequestError::InvalidMethod(
                    "First request can not be a OPTION request".to_string(),
                ));
            }
        };

        let origin = page.origin().ascii_serialization();

        let request_headers = headers
            .iter()
            .map(|(k, _)| k.as_str().to_lowercase())
            .collect::<Vec<String>>()
            .join(", ");

        let preflight_build = RequestBuilder::from(url.clone())
            .method(Method::OPTIONS)
            .header(ORIGIN, &origin)
            .header(ACCESS_CONTROL_REQUEST_HEADERS, &request_headers);

        if !SimpleMiddleware::is_simple_method(method) {
            let preflight_request = preflight_build
                .header(ACCESS_CONTROL_REQUEST_METHOD, method.as_str())
                .build();

            let res = self.client.send(preflight_request).await;

            return match res {
                Ok(response) => RequestResult::Success(HeaderResponse {
                    status_code: response.metadata().status_code,
                    headers: response.metadata().headers.clone(),
                }),
                Err(e) => RequestResult::Failed(RequestError::Network(e)),
            };
        }

        let preflight_request = preflight_build.build();

        let res = self.client.send(preflight_request).await;

        match res {
            Ok(response) => RequestResult::Success(HeaderResponse {
                status_code: response.metadata().status_code,
                headers: response.metadata().headers.clone(),
            }),
            Err(e) => RequestResult::Failed(RequestError::Network(e)),
        }
    }
}

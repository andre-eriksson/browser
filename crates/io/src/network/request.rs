use std::sync::Arc;

use constants::keys::STATUS_CODE;
use cookies::Cookie;
use network::{
    ACCESS_CONTROL_REQUEST_HEADERS, ACCESS_CONTROL_REQUEST_METHOD, HeaderMap, Method, ORIGIN,
    client::{HttpClient, ResponseHandle},
    errors::{NetworkError, RequestError},
    request::{Request, RequestBuilder},
    response::HeaderResponse,
};
use tracing::{debug, instrument, trace};
use url::Url;

use crate::network::{
    middleware::{
        cookies::CookieMiddleware, cors::CorsMiddleware, referrer::ReferrerMiddleware,
        simple::SimpleMiddleware,
    },
    policy::DocumentPolicy,
};

pub enum RequestResult<T> {
    Success(T),
    ClientError(T),
    ServerError(T),
    Failed(RequestError),
}

pub struct NetworkService<'a> {
    client: &'a dyn HttpClient,
    cookies: &'a Vec<Cookie>,
    browser_headers: &'a Arc<HeaderMap>,
}

impl<'a> NetworkService<'a> {
    pub fn new(
        client: &'a dyn HttpClient,
        cookies: &'a Vec<Cookie>,
        browser_headers: &'a Arc<HeaderMap>,
    ) -> Self {
        NetworkService {
            client,
            cookies,
            browser_headers,
        }
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

    #[instrument(skip(self, page_url, policies, request), fields(method = %request.method, url = %request.url))]
    pub async fn fetch(
        &mut self,
        page_url: Option<Url>,
        policies: &DocumentPolicy,
        request: Request,
    ) -> RequestResult<Box<dyn ResponseHandle>> {
        let mut request = request;
        let user_headers = request.headers.clone();
        request
            .headers
            .extend(self.browser_headers.as_ref().clone());

        // First request to set document URL
        if page_url.is_none() {
            if request.method != Method::GET {
                return RequestResult::Failed(RequestError::InvalidMethod(
                    "First request must be a GET request".to_string(),
                ));
            }

            CookieMiddleware::apply_cookies(&mut request, self.cookies);

            let resp = self.raw_fetch(request).await;

            return match resp {
                RequestResult::Failed(e) => {
                    debug!("{}", e);

                    RequestResult::Failed(e)
                }
                RequestResult::Success(resp) => {
                    // TODO: page.document_url = Some(url);

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

        if let Some(current_url) = &page_url {
            ReferrerMiddleware::apply_referrer(current_url, &mut request, &policies.referrer);
        }

        if SimpleMiddleware::is_simple_request(&request.method, &user_headers) {
            trace!("Simple request detected, skipping CORS preflight");
            CookieMiddleware::apply_cookies(&mut request, self.cookies);

            return Self::convert_response(self.client.send(request).await);
        }

        let preflight_response = match self
            .preflight_request(page_url, &user_headers, &request.url, &request.method)
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

        CookieMiddleware::apply_cookies(&mut request, self.cookies);

        Self::convert_response(self.client.send(request).await)
    }

    async fn preflight_request(
        &self,
        page_url: Option<Url>,
        headers: &HeaderMap,
        url: &Url,
        method: &Method,
    ) -> RequestResult<HeaderResponse> {
        let page = match page_url.as_ref() {
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

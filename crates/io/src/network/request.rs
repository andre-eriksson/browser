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
        cookies::CookieMiddleware,
        cors::CorsMiddleware,
        headers::{Destination, HeadersMiddleware, RequestMode},
        referrer::ReferrerMiddleware,
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

pub struct NetworkService<'client> {
    client: &'client dyn HttpClient,
    cookies: &'client [Cookie],
    browser_headers: &'client HeaderMap,
}

impl<'client> NetworkService<'client> {
    pub fn new(
        client: &'client dyn HttpClient,
        cookies: &'client [Cookie],
        browser_headers: &'client HeaderMap,
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
            Ok(resp) => {
                let status_code = resp.metadata().status_code;
                debug!({ STATUS_CODE } = status_code.to_string());

                match status_code {
                    _ if status_code.is_client_error() => RequestResult::ClientError(resp),
                    _ if status_code.is_server_error() => RequestResult::ServerError(resp),
                    _ => RequestResult::Success(resp),
                }
            }
            Err(e) => {
                debug!("{}", e);
                RequestResult::Failed(RequestError::Network(e))
            }
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
        request.headers.extend(self.browser_headers.clone());

        HeadersMiddleware::apply_forbidden_headers(&mut request.headers);

        if page_url.is_none() {
            if request.method != Method::GET {
                return RequestResult::Failed(RequestError::InvalidMethod(
                    "First request must be a GET request".to_string(),
                ));
            }

            CookieMiddleware::apply_cookies(&mut request, self.cookies);

            HeadersMiddleware::add_forbidden_fetch_headers(
                &request.url,
                None,
                &mut request.headers,
                Destination::Document,
                RequestMode::Navigate,
                true,
            );

            return self.raw_fetch(request).await;
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

use std::sync::{Arc, Mutex};

use cookies::cookie_store::CookieJar;
use errors::network::RequestError;
use http::{
    HeaderMap, Method,
    header::{ACCESS_CONTROL_REQUEST_HEADERS, ACCESS_CONTROL_REQUEST_METHOD, ORIGIN, SET_COOKIE},
};
use network::http::{
    client::{HttpClient, ResponseHandle},
    request::{Request, RequestBuilder},
    response::HeaderResponse,
};
use tracing::instrument;
use url::Url;

use crate::{
    service::network::middleware::{
        cookies::CookieMiddleware, cors::CorsMiddleware, referrer::ReferrerMiddleware,
        simple::SimpleMiddleware,
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

    async fn raw_fetch(
        &mut self,
        request: Request,
    ) -> Result<Box<dyn ResponseHandle>, RequestError> {
        self.client.send(request).await
    }

    #[instrument(skip(self, page, request), fields(url = %request.url, method = %request.method))]
    pub async fn fetch(
        &mut self,
        page: &mut Page,
        request: Request,
    ) -> Result<Box<dyn ResponseHandle>, RequestError> {
        // First request to set document URL
        if page.document_url.is_none() {
            if request.method != Method::GET {
                return Err(RequestError::RequestFailed(
                    "First request must be a GET request".to_string(),
                ));
            }

            let url = request.url.clone();
            let resp = self.raw_fetch(request).await?;
            page.document_url = Some(url);
            return Ok(resp);
        }

        let user_headers = request.headers.clone();
        let mut request = request;
        request
            .headers
            .extend(self.browser_headers.as_ref().clone());

        if let Some(current_url) = &page.document_url {
            ReferrerMiddleware::apply_referrer(
                current_url,
                &mut request,
                &page.policies().as_ref().referrer,
            );
        }

        if SimpleMiddleware::is_simple_request(&request.method, &user_headers) {
            if let Ok(jar) = self.cookie_jar.lock().as_mut() {
                CookieMiddleware::apply_cookies(&mut request, jar);
            }

            let url = &request.url.clone();
            let resp = self.client.send(request).await;

            if let Ok(response) = &resp {
                self.handle_response_headers(
                    &HeaderResponse {
                        status_code: response.metadata().status_code,
                        headers: response.metadata().headers.clone(),
                    },
                    url,
                );
            }

            return resp;
        }

        let preflight_response = self
            .preflight_request(page, &user_headers, &request.url, &request.method)
            .await?;

        CorsMiddleware::is_allowed(
            &request.origin,
            &request.credentials,
            &request.url,
            &request.method,
            &user_headers,
            preflight_response,
        )?;

        if let Ok(jar) = self.cookie_jar.lock().as_ref() {
            CookieMiddleware::apply_cookies(&mut request, jar);
        }

        let url = &request.url.clone();
        let resp = self.client.send(request).await?;

        self.handle_response_headers(
            &HeaderResponse {
                status_code: resp.metadata().status_code,
                headers: resp.metadata().headers.clone(),
            },
            url,
        );

        Ok(resp)
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
    ) -> Result<HeaderResponse, RequestError> {
        if page.document_url.is_none() {
            return Err(RequestError::RequestFailed(
                "No document URL in context".to_string(),
            ));
        }

        let origin = match page.document_url.as_ref() {
            Some(u) => u.origin().ascii_serialization(),
            None => {
                return Err(RequestError::RequestFailed(
                    "No document URL in context".to_string(),
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

        if !SimpleMiddleware::is_simple_method(method) {
            let preflight_request = preflight_build
                .header(ACCESS_CONTROL_REQUEST_METHOD, method.as_str())
                .build();

            let res = self.client.send(preflight_request).await;

            return match res {
                Ok(response) => Ok(HeaderResponse {
                    status_code: response.metadata().status_code,
                    headers: response.metadata().headers.clone(),
                }),
                Err(e) => Err(e),
            };
        }

        let preflight_request = preflight_build.build();

        let res = self.client.send(preflight_request).await;

        match res {
            Ok(response) => Ok(HeaderResponse {
                status_code: response.metadata().status_code,
                headers: response.metadata().headers.clone(),
            }),
            Err(e) => Err(e),
        }
    }
}

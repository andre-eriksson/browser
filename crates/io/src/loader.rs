use async_trait::async_trait;
use network::{
    errors::{NetworkError, RequestError},
    request::RequestBuilder,
};
use tokio::runtime::Runtime;
use url::Url;

use crate::{
    RequestResult,
    embeded::EmbededResource,
    errors::AssetError,
    manager::{EmbededType, ResourceType},
    network::request::NetworkService,
};

#[async_trait]
pub trait AsyncLoader {
    async fn await_asset(self) -> Result<Vec<u8>, AssetError>;
}

pub trait SyncLoader {
    fn load_asset(self) -> Result<Vec<u8>, AssetError>;
}

#[async_trait]
impl<'a> AsyncLoader for ResourceType<'a> {
    async fn await_asset(self) -> Result<Vec<u8>, AssetError> {
        match self {
            ResourceType::FileSystem(path) => {
                std::fs::read(path).map_err(|_| AssetError::LoadFailed(path.to_string()))
            }
            ResourceType::Embeded(asset) => {
                let path = asset.path();

                if !path.contains('.') {
                    let html_path = format!("{}.html", path);
                    EmbededResource::get(&html_path)
                        .map(|file| file.data.into_owned())
                        .ok_or(AssetError::NotFound(html_path))
                } else {
                    EmbededResource::get(&asset.path())
                        .map(|file| file.data.into_owned())
                        .ok_or(AssetError::NotFound(asset.path()))
                }
            }
            ResourceType::Remote {
                url,
                client,
                cookie_jar,
                browser_headers,
                page_url,
                policies,
            } => {
                let url = page_url.as_ref().map_or(
                    Url::parse(url).map_err(|e| {
                        RequestError::Network(NetworkError::InvalidUrl(e.to_string()))
                    }),
                    |u| {
                        u.join(url).map_err(|e| {
                            RequestError::Network(NetworkError::InvalidUrl(e.to_string()))
                        })
                    },
                )?;

                let request = RequestBuilder::from(url).build();
                let mut service = NetworkService::new(client, cookie_jar, browser_headers);
                let header_response = match service.fetch(page_url.clone(), policies, request).await
                {
                    RequestResult::Failed(err) => return Err(AssetError::RemoteFailed(err)),
                    RequestResult::ClientError(resp)
                    | RequestResult::ServerError(resp)
                    | RequestResult::Success(resp) => resp,
                };

                let body = match header_response.body().await {
                    Ok(resp) => match resp.body {
                        Some(b) => b,
                        None => {
                            return Err(AssetError::RemoteFailed(RequestError::EmptyBody));
                        }
                    },
                    Err(e) => {
                        return Err(AssetError::RemoteFailed(RequestError::Network(e)));
                    }
                };

                Ok(body)
            }
            ResourceType::Absolute {
                protocol,
                location,
                client,
                cookie_jar,
                browser_headers,
                page_url,
                policies,
            } => match protocol {
                "file" => {
                    let adjusted_location = if location.starts_with("file://") {
                        location.trim_start_matches("file://")
                    } else {
                        location
                    };

                    Self::await_asset(ResourceType::FileSystem(adjusted_location)).await
                }
                "embed" => {
                    let adjusted_location = if location.starts_with("embed://") {
                        location.trim_start_matches("embed://")
                    } else {
                        location
                    };

                    Self::await_asset(ResourceType::Embeded(EmbededType::Root(adjusted_location)))
                        .await
                }
                "about" => {
                    let adjusted_location = if location.starts_with("about:") {
                        location.trim_start_matches("about:")
                    } else {
                        location
                    };

                    Self::await_asset(ResourceType::Embeded(EmbededType::Browser(
                        adjusted_location,
                    )))
                    .await
                }
                _ => {
                    Self::await_asset(ResourceType::Remote {
                        url: location,
                        client,
                        cookie_jar,
                        browser_headers,
                        page_url,
                        policies,
                    })
                    .await
                }
            },
        }
    }
}

impl<'a> SyncLoader for ResourceType<'a> {
    fn load_asset(self) -> Result<Vec<u8>, AssetError> {
        match self {
            ResourceType::FileSystem(path) => {
                std::fs::read(path).map_err(|_| AssetError::LoadFailed(path.to_string()))
            }
            ResourceType::Embeded(asset) => EmbededResource::get(&asset.path())
                .map(|file| file.data.into_owned())
                .ok_or_else(|| AssetError::NotFound(asset.path())),
            ResourceType::Remote {
                url,
                client,
                cookie_jar,
                browser_headers,
                page_url,
                policies,
            } => {
                let runtime = Runtime::new().map_err(|e| {
                    AssetError::RemoteFailed(RequestError::Network(NetworkError::RuntimeError(
                        e.to_string(),
                    )))
                })?;

                runtime.block_on(async {
                    let url = page_url.as_ref().map_or(
                        Url::parse(url).map_err(|e| {
                            RequestError::Network(NetworkError::InvalidUrl(e.to_string()))
                        }),
                        |u| {
                            u.join(url).map_err(|e| {
                                RequestError::Network(NetworkError::InvalidUrl(e.to_string()))
                            })
                        },
                    )?;

                    let request = RequestBuilder::from(url).build();
                    let mut service = NetworkService::new(client, cookie_jar, browser_headers);
                    let header_response = match service
                        .fetch(page_url.clone(), policies, request)
                        .await
                    {
                        RequestResult::Failed(err) => return Err(AssetError::RemoteFailed(err)),
                        RequestResult::ClientError(resp)
                        | RequestResult::ServerError(resp)
                        | RequestResult::Success(resp) => resp,
                    };

                    let body = match header_response.body().await {
                        Ok(resp) => match resp.body {
                            Some(b) => b,
                            None => {
                                return Err(AssetError::RemoteFailed(RequestError::EmptyBody));
                            }
                        },
                        Err(e) => {
                            return Err(AssetError::RemoteFailed(RequestError::Network(e)));
                        }
                    };

                    Ok(body)
                })
            }
            ResourceType::Absolute {
                protocol,
                location,
                client,
                cookie_jar,
                browser_headers,
                page_url,
                policies,
            } => match protocol {
                "file://" => {
                    let adjusted_location = location.trim_start_matches("file://");
                    Self::load_asset(ResourceType::FileSystem(adjusted_location))
                }
                "embed://" => {
                    let adjusted_location = location.trim_start_matches("embed://");
                    Self::load_asset(ResourceType::Embeded(EmbededType::Root(adjusted_location)))
                }
                "about:" => {
                    let adjusted_location = location.trim_start_matches("about:");
                    Self::load_asset(ResourceType::Embeded(EmbededType::Browser(
                        adjusted_location,
                    )))
                }
                _ => Self::load_asset(ResourceType::Remote {
                    url: location,
                    client,
                    cookie_jar,
                    browser_headers,
                    page_url,
                    policies,
                }),
            },
        }
    }
}

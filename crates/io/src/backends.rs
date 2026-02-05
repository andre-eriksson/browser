use async_trait::async_trait;
use cookies::CookieJar;
use network::{
    HeaderMap,
    client::HttpClient,
    errors::{NetworkError, RequestError},
    request::RequestBuilder,
};
use rust_embed::RustEmbed;
use std::{path::PathBuf, sync::Arc};
use tokio::runtime::Runtime;
use url::Url;

use crate::{
    errors::AssetError,
    network::{
        policy::DocumentPolicy,
        request::{NetworkService, RequestResult},
    },
};

#[derive(RustEmbed)]
#[folder = "../../assets/"]
#[include = "**/*"]
struct Asset;

#[async_trait]
pub trait AsyncBackend {
    async fn await_asset(self, key: &str) -> Result<Vec<u8>, AssetError>;
}

pub trait SyncBackend {
    fn load_asset(self, key: &str) -> Result<Vec<u8>, AssetError>;
}

/// Backend represents different types of asset backends that can be used to load assets.
/// It can be a file system path, an embedded asset.
#[derive(Debug)]
pub enum Backend<'a> {
    /// Represents a file system backend with a specified path.
    ///
    /// Used for files that don't need the program to be compiled again e.g., config files.
    FileSystem(PathBuf),

    /// Represents the apps default assets embedded in the binary.
    ///
    /// Used for assets that are unlikely to change frequently e.g., UI assets, logos, etc.
    Embedded,

    /// Represents a remote backend (e.g., CDN or web server).
    ///
    /// Used for assets that need to be fetched over the network.
    Remote {
        client: &'a dyn HttpClient,
        cookie_jar: &'a mut CookieJar,
        browser_headers: &'a Arc<HeaderMap>,
        page_url: &'a Option<Url>,
        policies: &'a DocumentPolicy,
    },
}

#[async_trait]
impl<'a> AsyncBackend for Backend<'a> {
    async fn await_asset(self, location: &str) -> Result<Vec<u8>, AssetError> {
        match self {
            Backend::FileSystem(path) => {
                let full_path = path.join(location);
                std::fs::read(full_path).map_err(|_| AssetError::LoadFailed(location.to_string()))
            }
            Backend::Embedded => Asset::get(location)
                .map(|file| file.data.into_owned())
                .ok_or_else(|| AssetError::NotFound(location.to_string())),
            Backend::Remote {
                client,
                cookie_jar,
                browser_headers,
                page_url,
                policies,
            } => {
                let url = page_url.as_ref().map_or(
                    Url::parse(location).map_err(|e| {
                        RequestError::Network(NetworkError::InvalidUrl(e.to_string()))
                    }),
                    |u| {
                        u.join(location).map_err(|e| {
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
        }
    }
}

impl<'a> SyncBackend for Backend<'a> {
    fn load_asset(self, location: &str) -> Result<Vec<u8>, AssetError> {
        match self {
            Backend::FileSystem(path) => {
                let full_path = path.join(location);
                std::fs::read(full_path).map_err(|_| AssetError::LoadFailed(location.to_string()))
            }
            Backend::Embedded => Asset::get(location)
                .map(|file| file.data.into_owned())
                .ok_or_else(|| AssetError::NotFound(location.to_string())),
            Backend::Remote {
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
                        Url::parse(location).map_err(|e| {
                            RequestError::Network(NetworkError::InvalidUrl(e.to_string()))
                        }),
                        |u| {
                            u.join(location).map_err(|e| {
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
        }
    }
}

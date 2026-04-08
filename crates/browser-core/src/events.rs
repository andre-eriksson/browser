use crate::{
    context::page::PageMetadata,
    errors::{KernelError, NavigationError},
};
use async_trait::async_trait;
use html_dom::DocumentRoot;
use io::DocumentPolicy;
use network::HeaderMap;
use url::Url;

use crate::context::page::Page;

#[async_trait]
pub trait Commandable {
    async fn execute(&mut self, command: EngineCommand) -> Result<EngineResponse, KernelError>;
}

#[derive(Debug, Clone)]
pub enum NavigationType {
    Normal,
    Reload,
    Back,
    Forward,
}

/// Represents various events that can occur within the browser.
#[derive(Debug, Clone)]
pub enum EngineResponse {
    /// The DevTools page for a tab is ready.
    DevtoolsPageReady(Page),

    /// Navigation succeeded.
    NavigateSuccess(Page, PageMetadata, NavigationType),

    /// Navigation failed with a network error.
    NavigateError(NavigationError),

    /// An image was successfully fetched from the network.
    ImageFetched(String, Vec<u8>, HeaderMap),

    /// A general browser error occurred (for errors that don't fit other categories).
    Error(KernelError),
}

/// Represents commands that can be issued to the browser.
#[derive(Debug)]
pub enum EngineCommand {
    /// Command to navigate a tab to a specified URL.
    Navigate {
        url: String,
        navigation_type: NavigationType,
    },

    /// Get the DevTools page for a specific tab.
    GetDevtoolsPage { document: DocumentRoot },

    /// Command to fetch an image resource using the browser's HTTP client, headers, and cookies.
    FetchImage {
        request_url: Url,
        request_policies: DocumentPolicy,
        image_url: String,
    },
}

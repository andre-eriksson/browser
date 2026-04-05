use thiserror::Error;

/// Subsystem errors sits at the boundary of a subsystem and the engine.
#[derive(Error, Debug)]
pub enum UiError {
    #[error("UI Runtime error: {0}")]
    RuntimeError(String),
}

#[derive(Error, Debug, Clone)]
pub enum BrowserError {
    #[error("Browser error: {0}")]
    TabError(TabError),
}

#[derive(Error, Debug, Clone)]
pub enum TabError {
    #[error("Tab with ID {0:?} not found")]
    TabNotFound(usize),

    #[error("No tabs available")]
    NoTabsAvailable,

    #[error("No active tab available")]
    NoActiveTab,

    #[error("Tab has no URL to navigate to")]
    NoUrl,

    #[error("Tab has no history to navigate back or forward")]
    NoHistory,

    #[error("DevTools error: {0}")]
    DevtoolsError(String),
}

use thiserror::Error;

/// Subsystem errors sits at the boundary of a subsystem and the engine.
#[derive(Error, Debug)]
pub enum UiError {
    #[error(transparent)]
    Runtime(#[from] iced::Error),
}

#[derive(Error, Debug, Clone)]
pub enum BrowserError {
    #[error(transparent)]
    Tab(#[from] TabError),

    #[error("unable to load image: {0}")]
    ImageLoad(String),
}

#[derive(Error, Debug, Clone)]
pub enum TabError {
    #[error("tab id {0:?} not found")]
    TabNotFound(usize),

    #[error("no tabs available")]
    NoTabsAvailable,

    #[error("no active tab available")]
    NoActiveTab,

    #[error("tab has no URL to navigate to")]
    NoUrl,

    #[error("tab has no backward history")]
    NoBackHistory,

    #[error("tab has no forward history")]
    NoForwardHistory,
}

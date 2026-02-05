use thiserror::Error;

/// Subsystem errors sits at the boundary of a subsystem and the engine.
#[derive(Error, Debug)]
pub enum UiError {
    #[error("UI Runtime error: {0}")]
    RuntimeError(String),
}

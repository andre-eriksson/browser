use thiserror::Error;

/// Subsystem errors sits at the boundary of a subsystem and the engine.
#[derive(Error, Debug)]
pub enum SubsystemError {
    #[error("Runtime error: {0}")]
    RuntimeError(String),
}

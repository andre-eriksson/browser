//! This module defines various error types used across different components of the application.

/// Errors related to network operations
pub mod network;

/// Errors related to subsystem failures, that sit at the boundary of a subsystem and the engine
pub mod subsystem;

/// Errors related to tokenization processes
pub mod tokenization;

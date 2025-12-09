//! Utility state functions for HTML tokenization.
//!
//! This module contains various state functions used during the tokenization process of HTML documents.

/// Utility state functions for attributes
pub mod attributes;

/// Utility state functions for comments
pub mod comment;

/// Utility state functions for data
pub mod data;

/// Utility state functions for declarations like DOCTYPE and XML.
pub mod declaration;

/// Utility state functions for script
pub mod script;

/// Utility state functions for tags
pub mod tag;

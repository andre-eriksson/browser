//! HTML Syntax Module
//!
//! This module provides the core components for parsing and representing HTML syntax.

/// Defines traits and implementations for collecting metadata during HTML parsing.
pub mod collector;

/// Defines structures and functions for representing and building the Document Object Model (DOM) of HTML documents.
///
/// Contains representations for both single threaded and multi-threaded DOM structures, and utilities for converting between them.
pub mod dom;

/// Defines the standard HTML elements and their properties.
pub mod tag;

/// Defines the tokenization process for HTML syntax.
pub mod token;

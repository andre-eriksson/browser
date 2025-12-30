//! Consumers for different CSS token types.

/// Identifier token consumer §4.3.4 and §4.3.11
pub mod ident;

/// Numeric token consumer §4.3.3 and §4.3.12
pub mod numeric;

/// String token consumer §4.3.5 and §4.3.7
pub mod string;

/// Main token consumers module §4.3.1 and §4.3.2
pub mod token;

/// URL token consumer §4.3.6 and §4.3.14
pub mod url;

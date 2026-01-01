//! Consumers for various CSS constructs.

/// Block consumers
///
/// * <https://www.w3.org/TR/css-syntax-3/#consume-a-simple-block>
pub mod block;

/// Component value consumers
///
/// * <https://www.w3.org/TR/css-syntax-3/#consume-a-component-value>
pub mod component;

/// Declaration consumers
///
/// * <https://www.w3.org/TR/css-syntax-3/#consume-a-list-of-declarations>
pub mod declaration;

/// Function consumers
///
/// * <https://www.w3.org/TR/css-syntax-3/#consume-a-function>
pub mod function;

/// Rule consumers
///
/// * <https://www.w3.org/TR/css-syntax-3/#consume-a-list-of-rules>
/// * <https://www.w3.org/TR/css-syntax-3/#consume-an-at-rule>
/// * <https://www.w3.org/TR/css-syntax-3/#consume-a-qualified-rule>
pub mod rule;

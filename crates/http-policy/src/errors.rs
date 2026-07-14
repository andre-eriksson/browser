use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum PolicyError {
    #[error(transparent)]
    Cors(CorsError),
    // Csp(CspError),
    // Other(String),
}

#[derive(Debug, Clone, Error)]
pub enum CorsError {
    #[error("Invalid preflight response {0}")]
    InvalidPreflightResponse(String),

    #[error("Request from origin '{0}' to origin '{1}' is not allowed")]
    InvalidOrigin(String, String),

    #[error("Request method {0} not allowed by server")]
    InvalidMethod(String),

    #[error("Request header {0} not allowed by server")]
    InvalidHeader(String),

    #[error("Request contains non-simple headers not allowed by server")]
    NonSimpleHeaders,

    #[error("Request with credentials not allowed by server")]
    CredentialsNotAllowed,

    #[error("Request with '{0}' credential not allowed {1}")]
    CredentialNotAllowed(String, String),

    #[error("No Access-Control-Allow-Origin header in preflight response")]
    NoAccessControlAllowOrigin,
}

use thiserror::Error;

/// Errors that can occur in the Canvas MCP server
#[derive(Error, Debug)]
pub enum CanvasError {
    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// HTTP request error
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    /// JSON parsing error
    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    /// Canvas API error
    #[error("Canvas API error: {status} - {message}")]
    Api { status: u16, message: String },

    /// Resource not found
    #[error("Resource not found: {0}")]
    NotFound(String),

    /// Authentication error
    #[error("Authentication failed: {0}")]
    Auth(String),

    /// Rate limit exceeded
    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),

    /// Invalid parameter
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Result type for Canvas operations
pub type Result<T> = std::result::Result<T, CanvasError>;

impl CanvasError {
    /// Create a configuration error
    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }

    /// Create an API error from status and message
    pub fn api(status: u16, message: impl Into<String>) -> Self {
        Self::Api {
            status,
            message: message.into(),
        }
    }

    /// Create a not found error
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }

    /// Create an authentication error
    pub fn auth(msg: impl Into<String>) -> Self {
        Self::Auth(msg.into())
    }

    /// Create an internal error
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }
}

use crate::error::{CanvasError, Result};
use std::env;

/// Canvas MCP Server Configuration
#[derive(Debug, Clone)]
pub struct CanvasConfig {
    /// Canvas API access token
    pub api_token: String,

    /// Canvas API base URL (e.g., https://institution.instructure.com/api/v1)
    pub api_url: String,

    /// Institution name (optional, for display purposes)
    pub institution_name: Option<String>,

    /// Timezone for date/time operations (optional)
    pub timezone: Option<String>,

    /// Enable data anonymization for student information
    pub enable_anonymization: bool,

    /// Debug mode
    pub debug: bool,
}

impl CanvasConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self> {
        // Load .env file if it exists
        dotenvy::dotenv().ok();

        let api_token = env::var("CANVAS_API_TOKEN").map_err(|_| {
            CanvasError::config("CANVAS_API_TOKEN environment variable is required")
        })?;

        let api_url = env::var("CANVAS_API_URL")
            .map_err(|_| CanvasError::config("CANVAS_API_URL environment variable is required"))?;

        // Validate API URL
        if !api_url.starts_with("http://") && !api_url.starts_with("https://") {
            return Err(CanvasError::config(
                "CANVAS_API_URL must start with http:// or https://",
            ));
        }

        // Ensure API URL ends with /api/v1
        let api_url = if api_url.ends_with("/api/v1") {
            api_url
        } else if api_url.ends_with('/') {
            format!("{}api/v1", api_url)
        } else {
            format!("{}/api/v1", api_url)
        };

        let institution_name = env::var("INSTITUTION_NAME").ok();
        let timezone = env::var("TIMEZONE").ok();

        let enable_anonymization = env::var("ENABLE_DATA_ANONYMIZATION")
            .unwrap_or_else(|_| "false".to_string())
            .parse::<bool>()
            .unwrap_or(false);

        let debug = env::var("DEBUG")
            .unwrap_or_else(|_| "false".to_string())
            .parse::<bool>()
            .unwrap_or(false);

        Ok(Self {
            api_token,
            api_url,
            institution_name,
            timezone,
            enable_anonymization,
            debug,
        })
    }

    /// Create a new configuration with the given values
    pub fn new(api_token: String, api_url: String) -> Self {
        // Normalize API URL to ensure it ends with /api/v1
        let api_url = if api_url.ends_with("/api/v1") {
            api_url
        } else if api_url.ends_with('/') {
            format!("{}api/v1", api_url)
        } else {
            format!("{}/api/v1", api_url)
        };

        Self {
            api_token,
            api_url,
            institution_name: None,
            timezone: None,
            enable_anonymization: false,
            debug: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_url_normalization() {
        let config = CanvasConfig::new(
            "token".to_string(),
            "https://example.instructure.com".to_string(),
        );
        assert!(config.api_url.ends_with("/api/v1"));

        let config2 = CanvasConfig::new(
            "token".to_string(),
            "https://example.instructure.com/".to_string(),
        );
        assert!(config2.api_url.ends_with("/api/v1"));

        let config3 = CanvasConfig::new(
            "token".to_string(),
            "https://example.instructure.com/api/v1".to_string(),
        );
        assert!(config3.api_url.ends_with("/api/v1"));
    }
}

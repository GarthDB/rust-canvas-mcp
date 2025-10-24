use crate::config::CanvasConfig;
use crate::error::{CanvasError, Result};
use reqwest::{header, Client, Method, Response, StatusCode};
use serde::de::DeserializeOwned;
use std::sync::Arc;
use std::time::Duration;

/// Canvas API HTTP client
#[derive(Clone)]
pub struct CanvasClient {
    client: Client,
    config: Arc<CanvasConfig>,
}

impl CanvasClient {
    /// Create a new Canvas client
    pub fn new(config: Arc<CanvasConfig>) -> Result<Self> {
        let mut headers = header::HeaderMap::new();

        // Add authorization header
        let auth_value = format!("Bearer {}", config.api_token);
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&auth_value)
                .map_err(|e| CanvasError::config(format!("Invalid API token: {}", e)))?,
        );

        // Add user agent
        headers.insert(
            header::USER_AGENT,
            header::HeaderValue::from_static("rust-canvas-mcp/0.1.0"),
        );

        // Build HTTP client with connection pooling and timeouts
        let client = Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .pool_idle_timeout(Duration::from_secs(90))
            .pool_max_idle_per_host(10)
            .build()
            .map_err(|e| CanvasError::config(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { client, config })
    }

    /// Get the base API URL
    pub fn base_url(&self) -> &str {
        &self.config.api_url
    }

    /// Build a URL for a Canvas API endpoint
    pub fn build_url(&self, path: &str) -> String {
        let base = self.config.api_url.trim_end_matches('/');
        let path = path.trim_start_matches('/');
        format!("{}/{}", base, path)
    }

    /// Execute a GET request and deserialize the response
    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = self.build_url(path);
        let response = self.client.get(&url).send().await?;
        self.handle_response(response).await
    }

    /// Execute a POST request with JSON body
    pub async fn post<T: DeserializeOwned, B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let url = self.build_url(path);
        let response = self.client.post(&url).json(body).send().await?;
        self.handle_response(response).await
    }

    /// Execute a PUT request with JSON body
    pub async fn put<T: DeserializeOwned, B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let url = self.build_url(path);
        let response = self.client.put(&url).json(body).send().await?;
        self.handle_response(response).await
    }

    /// Execute a DELETE request
    pub async fn delete<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = self.build_url(path);
        let response = self.client.delete(&url).send().await?;
        self.handle_response(response).await
    }

    /// Execute a request and return the raw response
    pub async fn request(&self, method: Method, path: &str) -> Result<Response> {
        let url = self.build_url(path);
        let response = self.client.request(method, &url).send().await?;

        if response.status().is_success() {
            Ok(response)
        } else {
            Err(self.error_from_response(response).await)
        }
    }

    /// Handle response and deserialize or return error
    async fn handle_response<T: DeserializeOwned>(&self, response: Response) -> Result<T> {
        let status = response.status();

        if status.is_success() {
            let text = response.text().await?;
            serde_json::from_str(&text).map_err(|e| {
                CanvasError::internal(format!(
                    "Failed to parse Canvas API response: {}. Response: {}",
                    e,
                    text.chars().take(200).collect::<String>()
                ))
            })
        } else {
            Err(self.error_from_response(response).await)
        }
    }

    /// Convert an error response into a CanvasError
    async fn error_from_response(&self, response: Response) -> CanvasError {
        let status = response.status();
        let status_code = status.as_u16();

        // Try to get error message from response body
        let message = match response.text().await {
            Ok(body) => {
                // Try to parse JSON error
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
                    json.get("message")
                        .or_else(|| json.get("error"))
                        .and_then(|v| v.as_str())
                        .unwrap_or(&body)
                        .to_string()
                } else {
                    body
                }
            }
            Err(_) => status
                .canonical_reason()
                .unwrap_or("Unknown error")
                .to_string(),
        };

        match status {
            StatusCode::UNAUTHORIZED => CanvasError::auth(message),
            StatusCode::FORBIDDEN => CanvasError::auth(format!("Forbidden: {}", message)),
            StatusCode::NOT_FOUND => CanvasError::not_found(message),
            StatusCode::TOO_MANY_REQUESTS => {
                CanvasError::RateLimit(format!("Rate limit exceeded: {}", message))
            }
            _ => CanvasError::api(status_code, message),
        }
    }

    /// Get the current user (useful for testing connection)
    pub async fn get_current_user(&self) -> Result<serde_json::Value> {
        self.get("/users/self").await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_building() {
        let config = Arc::new(CanvasConfig::new(
            "token".to_string(),
            "https://example.instructure.com/api/v1".to_string(),
        ));

        let client = CanvasClient::new(config).unwrap();

        assert_eq!(
            client.build_url("/courses"),
            "https://example.instructure.com/api/v1/courses"
        );

        assert_eq!(
            client.build_url("courses"),
            "https://example.instructure.com/api/v1/courses"
        );
    }
}

use crate::client::CanvasClient;
use crate::config::CanvasConfig;
use rmcp::model::{Implementation, ServerCapabilities, ServerInfo};
use rmcp::ServerHandler;
use std::sync::Arc;

/// Canvas MCP Server
///
/// This struct implements the MCP ServerHandler trait and provides
/// Canvas LMS integration tools via the Model Context Protocol.
#[derive(Clone)]
pub struct CanvasServer {
    /// Canvas API HTTP client
    pub client: Arc<CanvasClient>,

    /// Server configuration
    pub config: Arc<CanvasConfig>,
}

impl CanvasServer {
    /// Create a new Canvas MCP server
    pub fn new(config: CanvasConfig) -> anyhow::Result<Self> {
        let config = Arc::new(config);
        let client = Arc::new(CanvasClient::new(config.clone())?);

        Ok(Self { client, config })
    }
}

// Implement ServerHandler without tool_handler macro for now
// We'll add tools in Phase 3
impl ServerHandler for CanvasServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: rmcp::model::ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "rust-canvas-mcp".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
            instructions: Some(format!(
                "Canvas LMS MCP Server\n\
                 Institution: {}\n\
                 API URL: {}\n\
                 \n\
                 This server provides tools for interacting with Canvas LMS.\n\
                 Core tools will be implemented in Phase 3.",
                self.config
                    .institution_name
                    .as_deref()
                    .unwrap_or("Not specified"),
                self.config.api_url
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_creation() {
        let config = CanvasConfig::new(
            "test_token".to_string(),
            "https://test.instructure.com".to_string(),
        );

        let server = CanvasServer::new(config);
        assert!(server.is_ok());
    }

    #[test]
    fn test_server_info() {
        let config = CanvasConfig::new(
            "test_token".to_string(),
            "https://test.instructure.com".to_string(),
        );

        let server = CanvasServer::new(config).unwrap();
        let info = server.get_info();

        assert_eq!(info.server_info.name, "rust-canvas-mcp");
        assert_eq!(info.server_info.version, env!("CARGO_PKG_VERSION"));
    }
}

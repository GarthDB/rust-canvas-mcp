/// Canvas MCP Server Library
///
/// This library provides the core functionality for the Canvas MCP server,
/// including configuration, HTTP client, and Canvas API integrations.
pub mod client;
pub mod config;
pub mod error;
pub mod server;

// Re-export commonly used types
pub use client::CanvasClient;
pub use config::CanvasConfig;
pub use error::{CanvasError, Result};
pub use server::CanvasServer;

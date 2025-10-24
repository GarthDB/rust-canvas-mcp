use rust_canvas_mcp::{CanvasClient, CanvasConfig};
use std::env;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Check for --test flag
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && args[1] == "--test" {
        return run_connection_test().await;
    }

    println!("Rust Canvas MCP Server");
    println!("Version: {}", env!("CARGO_PKG_VERSION"));
    println!();
    println!("This is a Phase 2 implementation demonstrating:");
    println!("- Configuration loading from environment");
    println!("- Canvas API HTTP client");
    println!("- Connection testing");
    println!();
    println!("Usage:");
    println!("  cargo run -- --test    Test Canvas API connection");
    println!();
    println!("Full MCP server implementation coming in Phase 2 (Issue #4)");

    Ok(())
}

/// Run connection test
async fn run_connection_test() -> anyhow::Result<()> {
    println!("Testing Canvas API connection...");
    println!();

    // Load configuration
    let config = match CanvasConfig::from_env() {
        Ok(cfg) => {
            println!("✓ Configuration loaded");
            if let Some(ref inst) = cfg.institution_name {
                println!("  Institution: {}", inst);
            }
            println!("  API URL: {}", cfg.api_url);
            cfg
        }
        Err(e) => {
            eprintln!("✗ Configuration error: {}", e);
            eprintln!();
            eprintln!("Please ensure the following environment variables are set:");
            eprintln!("  CANVAS_API_TOKEN - Your Canvas API access token");
            eprintln!("  CANVAS_API_URL - Your Canvas API URL (e.g., https://institution.instructure.com/api/v1)");
            std::process::exit(1);
        }
    };

    // Create HTTP client
    let client = match CanvasClient::new(Arc::new(config)) {
        Ok(c) => {
            println!("✓ HTTP client created");
            c
        }
        Err(e) => {
            eprintln!("✗ Failed to create HTTP client: {}", e);
            std::process::exit(1);
        }
    };

    // Test connection by fetching current user
    print!("Testing API connection... ");
    match client.get_current_user().await {
        Ok(user) => {
            println!("✓");
            if let Some(name) = user.get("name").and_then(|v| v.as_str()) {
                println!("✓ Connected as: {}", name);
            }
            if let Some(id) = user.get("id").and_then(|v| v.as_u64()) {
                println!("  User ID: {}", id);
            }
            println!();
            println!("✓ All tests passed!");
        }
        Err(e) => {
            println!("✗");
            eprintln!("✗ API connection failed: {}", e);
            eprintln!();
            eprintln!("Please check:");
            eprintln!("  - Your API token is valid");
            eprintln!("  - Your API URL is correct");
            eprintln!("  - You have network access to Canvas");
            std::process::exit(1);
        }
    }

    Ok(())
}

use rmcp::ServerHandler;
use rust_canvas_mcp::{CanvasConfig, CanvasServer};
use std::env;
use std::fs;
use tracing_subscriber::fmt::writer::MakeWriterExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Check for --test flag
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && args[1] == "--test" {
        return run_connection_test().await;
    }

    // Setup logging to file ONLY (never stderr during normal operation)
    setup_logging();

    // Load configuration
    let config = CanvasConfig::from_env().map_err(|e| {
        // Configuration errors can go to stderr during startup
        eprintln!("Configuration error: {}", e);
        eprintln!();
        eprintln!("Required environment variables:");
        eprintln!("  CANVAS_API_TOKEN - Your Canvas API access token");
        eprintln!("  CANVAS_API_URL - Your Canvas API URL");
        e
    })?;

    // Create server
    let server = CanvasServer::new(config)?;

    // Serve via stdio - THIS MUST PRODUCE ZERO STDERR OUTPUT
    // Pass stdin/stdout as a tuple for the transport
    let io = (tokio::io::stdin(), tokio::io::stdout());
    rmcp::serve_server(server, io).await?;

    Ok(())
}

/// Setup logging to file only (never stderr)
fn setup_logging() {
    // Create log directory if it doesn't exist
    let log_dir = "/tmp/canvas-mcp";
    fs::create_dir_all(log_dir).ok();

    // Create daily rolling file appender
    let file_appender = tracing_appender::rolling::daily(log_dir, "server.log");

    // Only write to file, never stderr
    let file_writer = file_appender.with_max_level(tracing::Level::DEBUG);

    tracing_subscriber::fmt()
        .with_writer(file_writer)
        .with_ansi(false)
        .with_target(false)
        .init();
}

/// Run connection test
async fn run_connection_test() -> anyhow::Result<()> {
    use rust_canvas_mcp::CanvasClient;
    use std::sync::Arc;

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
    let client = match CanvasClient::new(Arc::new(config.clone())) {
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

    // Test MCP server creation
    print!("Testing MCP server creation... ");
    match CanvasServer::new(config) {
        Ok(server) => {
            println!("✓");
            let info = server.get_info();
            println!(
                "  Server: {} v{}",
                info.server_info.name, info.server_info.version
            );
            println!();
            println!("✓ All tests passed!");
        }
        Err(e) => {
            println!("✗");
            eprintln!("✗ Failed to create MCP server: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}

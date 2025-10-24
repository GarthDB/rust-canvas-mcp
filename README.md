# Rust Canvas MCP

A high-performance Rust-based MCP (Model Context Protocol) server for Canvas LMS API integration.

## Overview

This project implements a clean, efficient MCP server that provides seamless integration with Canvas LMS. It addresses the stderr pollution issues present in the Python implementation while providing all 71 tools for comprehensive Canvas functionality.

## Key Features

- **Zero stderr pollution** - Clean MCP protocol communication via stdio
- **71 Canvas API tools** - Complete coverage of Canvas functionality
- **Type-safe** - Leverages Rust's type system for robust parameter handling
- **High performance** - Fast startup and efficient request handling
- **Flexible ID handling** - Accepts both string and integer identifiers
- **Comprehensive error handling** - Proper JSON-RPC error responses

## Installation

### Prerequisites

- Rust 1.70+ (stable)
- Canvas API access token

### Building from Source

```bash
git clone https://github.com/GarthDB/rust-canvas-mcp.git
cd rust-canvas-mcp
cargo build --release
```

The compiled binary will be available at `target/release/rust-canvas-mcp`.

## Configuration

1. Copy the example environment file:
   ```bash
   cp env.example .env
   ```

2. Edit `.env` and configure your Canvas credentials:
   ```bash
   CANVAS_API_TOKEN=your_token_here
   CANVAS_API_URL=https://your-institution.instructure.com/api/v1
   ```

## Usage

### With Cursor IDE

Add to your `.cursor/mcp.json`:

```json
{
  "mcpServers": {
    "canvas-mcp": {
      "command": "/path/to/rust-canvas-mcp",
      "env": {
        "CANVAS_API_TOKEN": "your_token",
        "CANVAS_API_URL": "https://your-institution.instructure.com/api/v1"
      }
    }
  }
}
```

### Standalone Testing

Test your Canvas API connection:

```bash
cargo run -- --test
```

## Available Tools

The server provides 71 tools across 8 categories:

- **Course Tools** (8) - List and manage courses
- **Discussion Tools** (12) - Topics, entries, and replies
- **Assignment Tools** (10) - Assignments, submissions, analytics
- **Announcement Tools** (4) - Create and manage announcements
- **Page Tools** (6) - Course pages and content
- **User Tools** (5) - Users, enrollments, groups
- **Rubric Tools** (10) - Full CRUD for rubrics
- **Peer Review Tools** (8) - Reviews and analytics
- **Messaging Tools** (5) - Canvas conversations
- **Analytics Tools** (4) - Student performance tracking

## Development

### Running Tests

```bash
cargo test
```

### Code Coverage

```bash
cargo install cargo-llvm-cov
cargo llvm-cov --html
```

### Linting

```bash
cargo clippy --all-targets --all-features
cargo fmt --check
```

## Success Criteria

- ✅ Zero stderr output during normal operation
- ✅ All 71 tools functional
- ✅ Robust parameter handling (string/int IDs)
- ✅ Fast startup (<100ms)
- ✅ Clean JSON-RPC error handling
- ✅ Works seamlessly in Cursor
- ✅ >80% test coverage
- ✅ Comprehensive documentation

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Submit a pull request

All changes must go through PR review to the protected main branch.

## License

MIT License - see LICENSE file for details.

## Acknowledgments

- Inspired by the Python [canvas-mcp](https://github.com/vishalsachdev/canvas-mcp) implementation
- Built with the official Rust MCP SDK ([rmcp](https://github.com/modelcontextprotocol/rust-sdk))

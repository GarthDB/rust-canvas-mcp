# Canvas MCP Implementation Issues & Requirements for Rust Rewrite

## Summary
The Python-based Canvas MCP server (`canvas-mcp`) functionally works but has several implementation issues that interfere with proper MCP protocol communication and integration with Cursor IDE. This document outlines the problems and requirements for a Rust-based reimplementation.

## Current Status
- **Repository**: https://github.com/vishalsachdev/canvas-mcp
- **Language**: Python 3.10+ with FastMCP framework
- **Version**: 1.0.3
- **Functionality**: 71 tools successfully registered
- **Connection**: Successfully connects to Cursor and Canvas API
- **Core Issue**: MCP protocol communication is disrupted by stderr output

## Critical Issues

### 1. Stderr Output Pollution
**Problem**: The server prints diagnostic messages to stderr during normal operation, which interferes with MCP stdio protocol communication.

**Evidence from logs**:
```
[error] Starting Canvas MCP server with API URL: https://si.instructure.com/api/v1
[error] Institution: Seminary
[error] Use Ctrl+C to stop the server
[error] Registering Canvas MCP tools...
[error] Peer review comment analysis tools registered successfully!
[error] Canvas messaging tools registered successfully!
[error] All Canvas MCP tools registered successfully!
```

**Location**: `src/canvas_mcp/server.py` lines 135-138, 41, 50-58

**Code examples**:
```python
print(f"Starting Canvas MCP server with API URL: {config.canvas_api_url}", file=sys.stderr)
print(f"Institution: {config.institution_name}", file=sys.stderr)
print("Use Ctrl+C to stop the server", file=sys.stderr)
print("Registering Canvas MCP tools...", file=sys.stderr)
```

**Impact**: 
- MCP protocol uses stdio for JSON-RPC messages
- Any stderr output during initialization/operation can confuse the MCP client
- While Cursor treats these as `[error]` logs, they don't break functionality but indicate poor protocol hygiene
- May cause issues with other MCP clients that are stricter about stdio communication

### 2. Python Module Import Warnings
**Problem**: Running with `python -m canvas_mcp.server` triggers RuntimeWarnings about module import order.

**Evidence**:
```
/opt/local/Library/Frameworks/Python.framework/Versions/3.10/lib/python3.10/runpy.py:126: 
RuntimeWarning: 'canvas_mcp.server' found in sys.modules after import of package 'canvas_mcp', 
but prior to execution of 'canvas_mcp.server'; this may result in unpredictable behaviour
```

**Workaround**: Use direct command `canvas-mcp-server` instead of `python -m canvas_mcp.server`

**Impact**: Adds noise to stderr, potential for unpredictable behavior

### 3. Parameter Type Validation Issues
**Problem**: MCP tools experience parameter validation errors when called from Cursor's AI assistant interface.

**Evidence**:
```
Error calling tool: Parameter 'course_identifier' must be one of types [string, integer], got number
```

**Context**:
- Tools are properly registered and visible in Cursor
- Tools work from command-line tests
- Issue appears specific to AI assistant → MCP tool call path
- May be related to how FastMCP or Cursor serializes parameters

**Impact**: Reduces usability from AI assistant, requires manual API calls as workaround

### 4. Lack of Clean Stdio Mode
**Problem**: No way to suppress diagnostic output for pure MCP protocol communication.

**Current behavior**:
- Diagnostic messages always go to stderr
- No `--quiet` or `--stdio-only` flag
- Cannot disable startup messages

**Desired behavior**:
- When running as MCP server, only JSON-RPC messages on stdout
- No stderr output unless actual errors occur
- Optional verbose mode for debugging

## Successful Test Results

Despite issues, core functionality works:

### Working Features:
```bash
# API Connection Test
$ canvas-mcp-server --test
✓ API connection successful! Connected as: Garth David Braithwaite
✓ All tests passed!

# Server starts and connects
[info] Successfully connected to stdio server
[info] Storing stdio client user-canvas-mcp
[info] Found 71 tools, 1 prompts, and 0 resources

# Tools successfully list courses
Tool: list_courses, Result: Courses:
Code: NAC Idaho East Region Trimester | Doctrine and Covenants 2 (ID Braithwaite) | DC2 | Braithwaite 
Name: NAC Idaho East Region Trimester | Doctrine and Covenants 2 (ID Braithwaite) | DC2 | Braithwaite 
ID: 108367
```

### Manual API Test (bypass MCP):
```bash
curl -H "Authorization: Bearer <token>" \
  "https://si.instructure.com/api/v1/courses/108367/discussion_topics/13744606"
# Successfully returns full JSON response
```

## Architecture Analysis

### Current Python Stack:
- **MCP Framework**: FastMCP 2.12.5
- **HTTP Client**: httpx (async)
- **Environment**: python-dotenv
- **Validation**: pydantic 2.0+

### File Structure:
```
canvas-mcp/
├── pyproject.toml
├── src/canvas_mcp/
│   ├── __init__.py
│   ├── server.py              # Main entry, lots of stderr output
│   ├── core/
│   │   ├── client.py          # HTTP client
│   │   ├── config.py          # Environment config
│   │   ├── cache.py           # Request caching
│   │   └── validation.py      # Input validation
│   └── tools/                 # 71 tool implementations
│       ├── courses.py
│       ├── assignments.py
│       ├── discussions.py
│       └── ... (8 more files)
```

### Tool Categories (71 total):
1. **Student Tools** - Personal tracking, grades, TODO
2. **Course Tools** - List/manage courses
3. **Discussion Tools** - Topics, entries, replies
4. **Assignment Tools** - List, details, submissions, peer reviews
5. **Rubric Tools** - CRUD operations for rubrics
6. **User Tools** - Enrollments, users, groups
7. **Analytics Tools** - Student performance data
8. **Messaging Tools** - Canvas conversations

## Requirements for Rust Implementation

### 1. Clean MCP Protocol Communication
**CRITICAL**: Implement proper stdio protocol handling
- **Zero stderr output** during normal operation
- Stdout reserved exclusively for JSON-RPC messages
- Logging to file or disabled by default
- Optional verbose flag that redirects to log file (never stderr)

### 2. Proper MCP Server Implementation
Use official Rust MCP SDK or implement according to spec:
- Clean stdio transport
- Proper JSON-RPC 2.0 message handling
- Correct initialize handshake
- Tool registration without side effects

### 3. Type-Safe Parameter Handling
- Strong typing for all tool parameters
- Proper JSON schema generation
- Handle both string and integer for IDs (Canvas accepts both)
- Clear error messages on validation failures

### 4. Configuration
Minimal required env vars:
```bash
CANVAS_API_TOKEN=<token>
CANVAS_API_URL=https://si.instructure.com/api/v1
```

Optional:
```bash
INSTITUTION_NAME=Seminary
TIMEZONE=America/Boise
ENABLE_DATA_ANONYMIZATION=false
DEBUG=false
```

### 5. Core Functionality to Replicate

**Essential Canvas API Operations**:
```rust
// Courses
list_courses() -> Vec<Course>
get_course_details(course_id: CourseId) -> Course

// Discussions
list_discussion_topics(course_id: CourseId) -> Vec<Topic>
get_discussion_topic_details(course_id: CourseId, topic_id: TopicId) -> Topic

// Assignments  
list_assignments(course_id: CourseId) -> Vec<Assignment>
get_assignment_details(course_id: CourseId, assignment_id: AssignmentId) -> Assignment

// Users
list_users(course_id: CourseId) -> Vec<User>
```

Where `CourseId`, `TopicId`, etc. accept both `String` and `u64`

### 6. Error Handling
- Proper HTTP error handling (404, 401, 403, etc.)
- Clean error messages (no debug output to stderr)
- JSON-RPC error responses only

### 7. Testing Requirements
```bash
# Must pass without stderr pollution
cargo run -- --test
# Should output only: ✓ Connected as: <name>

# Must work with Cursor MCP
# No stderr output during normal operation
# All 71 tools should be discoverable
# Parameter validation must be robust
```

## MCP Configuration Example

### Global Config (`~/.cursor/mcp.json`):
```json
{
  "mcpServers": {
    "canvas-mcp-rust": {
      "command": "/path/to/canvas-mcp-rust",
      "env": {
        "CANVAS_API_TOKEN": "your_token",
        "CANVAS_API_URL": "https://si.instructure.com/api/v1",
        "INSTITUTION_NAME": "Seminary",
        "TIMEZONE": "America/Boise"
      }
    }
  }
}
```

### Workspace Config (`.cursor/mcp.json`):
```json
{
  "mcpServers": {
    "canvas-mcp-rust": {
      "command": "/path/to/canvas-mcp-rust",
      "env": {
        "CANVAS_API_TOKEN": "project_specific_token",
        "CANVAS_API_URL": "https://si.instructure.com/api/v1"
      }
    }
  }
}
```

## Success Criteria

A successful Rust implementation should:

1. ✅ **Zero stderr output** during normal MCP operation
2. ✅ **Clean stdio protocol** - only JSON-RPC on stdout
3. ✅ **All 71 tools** functional and discoverable
4. ✅ **Robust parameter handling** - accept string/int for IDs
5. ✅ **Fast startup** - Rust binary should start instantly
6. ✅ **No runtime warnings** - clean operation
7. ✅ **Proper error handling** - JSON-RPC errors only
8. ✅ **Works in Cursor** - tools callable from AI assistant
9. ✅ **Comprehensive tests** - API connection, tool calls
10. ✅ **Good documentation** - setup, usage, troubleshooting

## Additional Benefits of Rust Implementation

1. **Performance**: Faster startup, lower memory usage
2. **Type Safety**: Compile-time guarantees, fewer runtime errors
3. **Single Binary**: No Python environment required
4. **Cross-platform**: Easy distribution
5. **Clean MCP Protocol**: Proper separation of concerns
6. **Better Error Messages**: Compile-time validation

## References

- Python Implementation: https://github.com/vishalsachdev/canvas-mcp
- Canvas API Docs: https://canvas.instructure.com/doc/api/
- MCP Specification: https://modelcontextprotocol.io/
- FastMCP (Python): https://github.com/jlowin/fastmcp
- Current Issue Logs: See Cursor logs at `~/Library/Application Support/Cursor/logs/.../MCP user-canvas-mcp.log`

## Test Case: Fetch Today's Lesson

**Scenario**: Fetch discussion topic 13744606 from course 108367

**Current workaround** (manual API call):
```bash
curl -H "Authorization: Bearer <token>" \
  "https://si.instructure.com/api/v1/courses/108367/discussion_topics/13744606" \
  | python3 -m json.tool
```

**Expected Rust MCP behavior**:
```bash
# Via MCP tool call from Cursor
get_discussion_topic_details(course_identifier: 108367, topic_id: 13744606)
# Returns clean JSON response with no stderr pollution
```

## Conclusion

The Python Canvas MCP implementation is functionally complete but violates MCP protocol best practices by polluting stderr. A Rust reimplementation focused on clean stdio protocol communication would provide a more robust, faster, and maintainable solution while maintaining all 71 tools' functionality.

**Primary Goal**: Zero stderr output during normal operation to ensure clean MCP protocol communication.



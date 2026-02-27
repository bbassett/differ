# Differ

A desktop code review tool built with Tauri, SvelteKit, and Rust. Open a git repo, select two refs, browse file diffs, and submit review comments that are queued for processing via MCP.

## Development

```bash
make dev    # Start Tauri dev server
make check  # Type-check frontend + backend
```

## Architecture

```
Frontend (SvelteKit) → Tauri IPC → Rust Backend → CommentQueue
                                                        ↑
                                          MCP Server (HTTP :3100)
                                                        ↑
                                                   MCP Clients
```

The frontend lets you browse diffs and submit review comments. Comments are queued in-memory. An MCP server runs alongside the app on `127.0.0.1:3100`, exposing the queue to external tools.

### MCP Tools

| Tool | Description |
|------|-------------|
| `get_next_comment` | Dequeues and returns the next pending review comment |
| `get_queue_status` | Returns `{"pending": <count>}` |

## Connecting Claude Code to the MCP Server

The app runs a Streamable HTTP MCP server on port 3100. To connect Claude Code, add the following to `.mcp.json` in the project root:

```json
{
  "mcpServers": {
    "differ": {
      "command": "npx",
      "args": ["mcp-remote", "http://127.0.0.1:3100/mcp"]
    }
  }
}
```

Then restart Claude Code. The `get_next_comment` and `get_queue_status` tools will be available.

> **Note:** The Differ app must be running (`make dev`) for the MCP server to be reachable.

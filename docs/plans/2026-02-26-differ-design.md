# Differ — Local Pull Request Review App

## Overview

Desktop app for reviewing AI agent code changes locally. View side-by-side diffs between any two git refs, select lines, and send inline feedback directly to the agent via MCP.

**Stack:** Tauri (Rust) + Svelte + MCP

## Architecture

- **Tauri backend (Rust):** Git operations via `git2`, comment queue management, MCP server
- **Svelte frontend:** Diff rendering, line selection, comment input, queue status
- **MCP server:** Embedded in Tauri backend, exposes tools for agents to dequeue comments
- **No persistent database.** Comments live in-memory for the session.

## Git Integration

Uses `git2` crate (libgit2 bindings) — no shelling out to git CLI.

**Operations:**
- Repository discovery from CWD or user-selected directory
- Enumerate local branches, tags, and worktrees as comparison targets
- Generate structured diffs between any two refs
- Detect worktrees automatically

**Diff data model:**

```
DiffResult {
  base_ref: string
  compare_ref: string
  files: [{
    path: string
    status: added | modified | deleted | renamed
    old_path: string?
    hunks: [{
      old_start: u32, old_lines: u32
      new_start: u32, new_lines: u32
      lines: [{ type: add|delete|context, content: string, old_num: u32?, new_num: u32? }]
    }]
  }]
}
```

## Frontend

**Diff viewer:**
- File list sidebar — click to jump to file
- Main pane shows diff for selected file
- Toggle between side-by-side and unified view
- Syntax highlighting (Shiki or highlight.js)

**Line selection & commenting:**
- Click line number to select, shift+click for range
- Selected lines highlight in gutter
- Comment box appears anchored to selection — textarea + send button
- On send: comment is immediately dispatched to backend queue and sent to agent via MCP
- Inline indicator shows comment was sent
- No edit/delete after send — already dispatched

**Comment queue status:**
- Status bar shows queue depth and processing state (e.g. "3 pending, 1 in progress")

Each comment is an independent prompt — no batch review submission.

## MCP Server

Runs inside the Tauri Rust backend. Exposes two tools:

**`get_next_comment`** — Dequeues next pending comment. Returns:
```json
{
  "file": "src/main.rs",
  "start_line": 42,
  "end_line": 45,
  "code_context": "fn foo() {\n    let x = bad_thing();\n    ...\n}",
  "comment": "Use proper error handling here instead of unwrap"
}
```
Returns null if queue is empty.

**`get_queue_status`** — Returns count of pending comments.

**Transport:** stdio or SSE, whichever integrates best with Claude Code.

**Agent workflow:**
1. Agent connects to the app's MCP server
2. Calls `get_next_comment` to receive feedback
3. Reads the referenced file/lines, applies the feedback
4. Calls again until queue is empty

No response channel in v1. User sees results by refreshing the diff.

## Project Structure

```
differ/
├── src-tauri/
│   ├── src/
│   │   ├── main.rs          # Tauri entry, app setup
│   │   ├── git.rs           # git2 operations (diff, branches, worktrees)
│   │   ├── mcp.rs           # MCP server (tools, queue management)
│   │   └── commands.rs      # Tauri IPC command handlers
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/
│   ├── App.svelte           # Root layout
│   ├── lib/
│   │   ├── DiffViewer.svelte    # Side-by-side / unified renderer
│   │   ├── FileTree.svelte      # Changed files sidebar
│   │   ├── CommentBox.svelte    # Anchored comment input
│   │   ├── QueueStatus.svelte   # Pending comments indicator
│   │   └── RefSelector.svelte   # Branch/ref picker dropdowns
│   └── stores/
│       └── diff.ts              # Svelte stores for diff state
├── package.json
└── vite.config.ts
```

**Key crates:** `git2`, `serde`, `serde_json`, `tauri`

**Key npm packages:** `svelte`, `@tauri-apps/api`, syntax highlighter

## Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| App type | Tauri (desktop) | Native git access, self-contained, small binary |
| Frontend | Svelte | Lightweight, minimal boilerplate |
| Git access | `git2` crate | No CLI dependency, structured API |
| Agent integration | MCP server | Natural fit for Claude Code, future-proof |
| Diff comparison | Configurable (any two refs) | Flexible, defaults to main vs current branch |
| Commenting | Line + range level | Covers common review needs |
| Diff view | Toggle side-by-side / unified | User preference |
| Comment flow | Immediate dispatch, queued | Each comment is an independent agent prompt |
| Review rounds | Single pass (v1) | Simple, threaded rounds can be added later |
| Persistence | In-memory only | No DB needed for single-pass flow |

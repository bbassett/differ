# Differ Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a desktop app for reviewing AI agent code changes locally with inline commenting dispatched to agents via MCP.

**Architecture:** Tauri v2 (Rust) backend handles git operations via `git2`, manages a comment queue, and runs an embedded MCP server over SSE using `rmcp`. Svelte + TypeScript frontend renders diffs, handles line selection, and collects comments. Shared state via `Arc<Mutex<>>` bridges Tauri commands and MCP tools.

**Tech Stack:** Tauri v2, Svelte 5, TypeScript, Rust, git2, rmcp, Shiki

---

### Task 1: Scaffold Tauri + Svelte Project

**Files:**
- Create: entire project scaffold via CLI
- Modify: `src-tauri/Cargo.toml` (add dependencies)

**Step 1: Scaffold the project**

Run:
```bash
cd /Users/brandon/Projects/bbassett/differ
npm create tauri-app@latest . -- --template svelte-ts
```

If it prompts for identifier, use `com.differ.app`.

**Step 2: Install frontend dependencies**

Run:
```bash
npm install
```

**Step 3: Verify the scaffold builds**

Run:
```bash
npm run tauri dev
```

Expected: A Tauri window opens with the default Svelte template. Close it.

**Step 4: Add Rust dependencies to `src-tauri/Cargo.toml`**

Add under `[dependencies]`:
```toml
git2 = { version = "0.20", features = ["vendored-libgit2"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
rmcp = { version = "0.16", features = ["server", "transport-streamable-http-server"] }
tokio = { version = "1", features = ["full"] }
schemars = "0.8"
```

**Step 5: Verify Rust compilation**

Run:
```bash
cd src-tauri && cargo check
```

Expected: Compiles without errors (may take a while for first build).

**Step 6: Commit**

```bash
git add -A
git commit -m "Scaffold Tauri + Svelte project with dependencies"
```

---

### Task 2: Shared State & Data Types

**Files:**
- Create: `src-tauri/src/types.rs`
- Create: `src-tauri/src/state.rs`
- Modify: `src-tauri/src/lib.rs`

**Step 1: Create types module**

Create `src-tauri/src/types.rs`:
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiffResult {
    pub base_ref: String,
    pub compare_ref: String,
    pub files: Vec<DiffFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiffFile {
    pub path: String,
    pub status: FileStatus,
    pub old_path: Option<String>,
    pub hunks: Vec<DiffHunk>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileStatus {
    Added,
    Modified,
    Deleted,
    Renamed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiffHunk {
    pub old_start: u32,
    pub old_lines: u32,
    pub new_start: u32,
    pub new_lines: u32,
    pub lines: Vec<DiffLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiffLine {
    pub line_type: LineType,
    pub content: String,
    pub old_num: Option<u32>,
    pub new_num: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LineType {
    Add,
    Delete,
    Context,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewComment {
    pub id: u64,
    pub file: String,
    pub start_line: u32,
    pub end_line: u32,
    pub code_context: String,
    pub comment: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RefInfo {
    pub name: String,
    pub ref_type: RefType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RefType {
    Branch,
    Tag,
    Worktree,
}
```

**Step 2: Create state module**

Create `src-tauri/src/state.rs`:
```rust
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::types::ReviewComment;

#[derive(Debug)]
pub struct CommentQueue {
    queue: VecDeque<ReviewComment>,
    next_id: AtomicU64,
}

impl CommentQueue {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            next_id: AtomicU64::new(1),
        }
    }

    pub fn enqueue(&mut self, file: String, start_line: u32, end_line: u32, code_context: String, comment: String) -> u64 {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        self.queue.push_back(ReviewComment {
            id,
            file,
            start_line,
            end_line,
            code_context,
            comment,
        });
        id
    }

    pub fn dequeue(&mut self) -> Option<ReviewComment> {
        self.queue.pop_front()
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }
}

pub struct AppState {
    pub comment_queue: Arc<Mutex<CommentQueue>>,
    pub repo_path: Arc<Mutex<Option<String>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            comment_queue: Arc::new(Mutex::new(CommentQueue::new())),
            repo_path: Arc::new(Mutex::new(None)),
        }
    }
}
```

**Step 3: Wire up modules in `lib.rs`**

Replace contents of `src-tauri/src/lib.rs`:
```rust
mod state;
mod types;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::new())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

**Step 4: Verify compilation**

Run:
```bash
cd src-tauri && cargo check
```

Expected: Compiles without errors.

**Step 5: Write unit tests for CommentQueue**

Add to end of `src-tauri/src/state.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enqueue_dequeue() {
        let mut queue = CommentQueue::new();
        let id = queue.enqueue(
            "src/main.rs".into(), 10, 12,
            "let x = 1;".into(), "Use a constant".into(),
        );
        assert_eq!(id, 1);
        assert_eq!(queue.len(), 1);

        let comment = queue.dequeue().unwrap();
        assert_eq!(comment.id, 1);
        assert_eq!(comment.file, "src/main.rs");
        assert_eq!(comment.start_line, 10);
        assert_eq!(comment.end_line, 12);
        assert_eq!(comment.comment, "Use a constant");
        assert_eq!(queue.len(), 0);
    }

    #[test]
    fn test_fifo_order() {
        let mut queue = CommentQueue::new();
        queue.enqueue("a.rs".into(), 1, 1, "".into(), "first".into());
        queue.enqueue("b.rs".into(), 2, 2, "".into(), "second".into());

        assert_eq!(queue.dequeue().unwrap().comment, "first");
        assert_eq!(queue.dequeue().unwrap().comment, "second");
        assert!(queue.dequeue().is_none());
    }

    #[test]
    fn test_dequeue_empty() {
        let mut queue = CommentQueue::new();
        assert!(queue.dequeue().is_none());
        assert_eq!(queue.len(), 0);
    }

    #[test]
    fn test_ids_increment() {
        let mut queue = CommentQueue::new();
        let id1 = queue.enqueue("a.rs".into(), 1, 1, "".into(), "a".into());
        let id2 = queue.enqueue("b.rs".into(), 1, 1, "".into(), "b".into());
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
    }
}
```

**Step 6: Run tests**

Run:
```bash
cd src-tauri && cargo test
```

Expected: All 4 tests pass.

**Step 7: Commit**

```bash
git add src-tauri/src/types.rs src-tauri/src/state.rs src-tauri/src/lib.rs
git commit -m "Add shared data types, comment queue with tests"
```

---

### Task 3: Git Operations — Repo Discovery & Branch Listing

**Files:**
- Create: `src-tauri/src/git.rs`
- Modify: `src-tauri/src/lib.rs`

**Step 1: Write tests for git operations**

Create `src-tauri/src/git.rs`:
```rust
use git2::Repository;
use crate::types::{RefInfo, RefType};

pub fn discover_repo(path: &str) -> Result<Repository, String> {
    Repository::discover(path).map_err(|e| format!("Failed to discover repo: {}", e))
}

pub fn list_refs(repo: &Repository) -> Result<Vec<RefInfo>, String> {
    let mut refs = Vec::new();

    // Local branches
    let branches = repo.branches(Some(git2::BranchType::Local))
        .map_err(|e| format!("Failed to list branches: {}", e))?;

    for branch in branches {
        let (branch, _) = branch.map_err(|e| format!("Failed to read branch: {}", e))?;
        if let Some(name) = branch.name().map_err(|e| format!("Invalid branch name: {}", e))? {
            refs.push(RefInfo {
                name: name.to_string(),
                ref_type: RefType::Branch,
            });
        }
    }

    // Tags
    let tag_names = repo.tag_names(None)
        .map_err(|e| format!("Failed to list tags: {}", e))?;

    for tag_name in tag_names.iter().flatten() {
        refs.push(RefInfo {
            name: tag_name.to_string(),
            ref_type: RefType::Tag,
        });
    }

    Ok(refs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    fn setup_test_repo(path: &Path) -> Repository {
        let repo = Repository::init(path).unwrap();

        // Create an initial commit so branches exist
        let sig = git2::Signature::now("Test", "test@test.com").unwrap();
        let tree_id = {
            let mut index = repo.index().unwrap();
            let test_file = path.join("test.txt");
            fs::write(&test_file, "hello").unwrap();
            index.add_path(Path::new("test.txt")).unwrap();
            index.write_tree().unwrap()
        };
        let tree = repo.find_tree(tree_id).unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, "initial", &tree, &[]).unwrap();

        repo
    }

    #[test]
    fn test_discover_repo() {
        let dir = tempfile::tempdir().unwrap();
        Repository::init(dir.path()).unwrap();
        let repo = discover_repo(dir.path().to_str().unwrap());
        assert!(repo.is_ok());
    }

    #[test]
    fn test_discover_nonexistent() {
        let result = discover_repo("/nonexistent/path");
        assert!(result.is_err());
    }

    #[test]
    fn test_list_branches() {
        let dir = tempfile::tempdir().unwrap();
        let repo = setup_test_repo(dir.path());

        // Create another branch
        let head = repo.head().unwrap().peel_to_commit().unwrap();
        repo.branch("feature", &head, false).unwrap();

        let refs = list_refs(&repo).unwrap();
        let branch_names: Vec<&str> = refs.iter()
            .filter(|r| matches!(r.ref_type, RefType::Branch))
            .map(|r| r.name.as_str())
            .collect();

        assert!(branch_names.contains(&"main") || branch_names.contains(&"master"));
        assert!(branch_names.contains(&"feature"));
    }

    #[test]
    fn test_list_tags() {
        let dir = tempfile::tempdir().unwrap();
        let repo = setup_test_repo(dir.path());

        let head = repo.head().unwrap().peel_to_commit().unwrap();
        repo.tag_lightweight("v1.0", head.as_object(), false).unwrap();

        let refs = list_refs(&repo).unwrap();
        let tag_names: Vec<&str> = refs.iter()
            .filter(|r| matches!(r.ref_type, RefType::Tag))
            .map(|r| r.name.as_str())
            .collect();

        assert!(tag_names.contains(&"v1.0"));
    }
}
```

**Step 2: Add tempfile dev dependency**

Add to `src-tauri/Cargo.toml` under `[dev-dependencies]`:
```toml
[dev-dependencies]
tempfile = "3"
```

**Step 3: Wire up module in `lib.rs`**

Add to `src-tauri/src/lib.rs`:
```rust
mod git;
```

**Step 4: Run tests**

Run:
```bash
cd src-tauri && cargo test
```

Expected: All tests pass (queue tests + git tests).

**Step 5: Commit**

```bash
git add src-tauri/src/git.rs src-tauri/src/lib.rs src-tauri/Cargo.toml
git commit -m "Add git repo discovery and branch/tag listing with tests"
```

---

### Task 4: Git Operations — Diff Generation

**Files:**
- Modify: `src-tauri/src/git.rs`

**Step 1: Add diff test first**

Add to the `tests` module in `src-tauri/src/git.rs`:
```rust
    #[test]
    fn test_generate_diff_modified_file() {
        let dir = tempfile::tempdir().unwrap();
        let repo = setup_test_repo(dir.path());

        // Create a branch and modify a file
        let head = repo.head().unwrap().peel_to_commit().unwrap();
        repo.branch("feature", &head, false).unwrap();
        repo.set_head("refs/heads/feature").unwrap();
        repo.checkout_head(Some(git2::build::CheckoutBuilder::new().force())).unwrap();

        fs::write(dir.path().join("test.txt"), "hello\nworld\n").unwrap();
        let sig = git2::Signature::now("Test", "test@test.com").unwrap();
        let mut index = repo.index().unwrap();
        index.add_path(Path::new("test.txt")).unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        let parent = repo.head().unwrap().peel_to_commit().unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, "modify", &tree, &[&parent]).unwrap();

        let diff = generate_diff(&repo, "main", "feature").unwrap();
        assert_eq!(diff.base_ref, "main");
        assert_eq!(diff.compare_ref, "feature");
        assert_eq!(diff.files.len(), 1);
        assert_eq!(diff.files[0].path, "test.txt");
        assert!(matches!(diff.files[0].status, FileStatus::Modified));
        assert!(!diff.files[0].hunks.is_empty());
    }

    #[test]
    fn test_generate_diff_added_file() {
        let dir = tempfile::tempdir().unwrap();
        let repo = setup_test_repo(dir.path());

        let head = repo.head().unwrap().peel_to_commit().unwrap();
        repo.branch("feature", &head, false).unwrap();
        repo.set_head("refs/heads/feature").unwrap();
        repo.checkout_head(Some(git2::build::CheckoutBuilder::new().force())).unwrap();

        fs::write(dir.path().join("new.txt"), "new file\n").unwrap();
        let sig = git2::Signature::now("Test", "test@test.com").unwrap();
        let mut index = repo.index().unwrap();
        index.add_path(Path::new("new.txt")).unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        let parent = repo.head().unwrap().peel_to_commit().unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, "add file", &tree, &[&parent]).unwrap();

        let diff = generate_diff(&repo, "main", "feature").unwrap();
        let new_file = diff.files.iter().find(|f| f.path == "new.txt").unwrap();
        assert!(matches!(new_file.status, FileStatus::Added));
    }

    #[test]
    fn test_generate_diff_deleted_file() {
        let dir = tempfile::tempdir().unwrap();
        let repo = setup_test_repo(dir.path());

        let head = repo.head().unwrap().peel_to_commit().unwrap();
        repo.branch("feature", &head, false).unwrap();
        repo.set_head("refs/heads/feature").unwrap();
        repo.checkout_head(Some(git2::build::CheckoutBuilder::new().force())).unwrap();

        fs::remove_file(dir.path().join("test.txt")).unwrap();
        let sig = git2::Signature::now("Test", "test@test.com").unwrap();
        let mut index = repo.index().unwrap();
        index.remove_path(Path::new("test.txt")).unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        let parent = repo.head().unwrap().peel_to_commit().unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, "delete file", &tree, &[&parent]).unwrap();

        let diff = generate_diff(&repo, "main", "feature").unwrap();
        let deleted = diff.files.iter().find(|f| f.path == "test.txt").unwrap();
        assert!(matches!(deleted.status, FileStatus::Deleted));
    }
```

**Step 2: Run tests to verify they fail**

Run:
```bash
cd src-tauri && cargo test 2>&1 | head -5
```

Expected: FAIL — `generate_diff` function not found.

**Step 3: Implement `generate_diff`**

Add to `src-tauri/src/git.rs` (after `list_refs`):
```rust
use crate::types::{DiffResult, DiffFile, DiffHunk, DiffLine, FileStatus, LineType};

pub fn generate_diff(repo: &Repository, base: &str, compare: &str) -> Result<DiffResult, String> {
    let base_ref = format!("refs/heads/{}", base);
    let compare_ref = format!("refs/heads/{}", compare);

    let base_obj = repo.revparse_single(&base_ref)
        .map_err(|e| format!("Failed to resolve '{}': {}", base, e))?;
    let compare_obj = repo.revparse_single(&compare_ref)
        .map_err(|e| format!("Failed to resolve '{}': {}", compare, e))?;

    let base_tree = base_obj.peel_to_tree()
        .map_err(|e| format!("Failed to get tree for '{}': {}", base, e))?;
    let compare_tree = compare_obj.peel_to_tree()
        .map_err(|e| format!("Failed to get tree for '{}': {}", compare, e))?;

    let diff = repo.diff_tree_to_tree(Some(&base_tree), Some(&compare_tree), None)
        .map_err(|e| format!("Failed to generate diff: {}", e))?;

    let mut files = Vec::new();

    diff.foreach(
        &mut |delta, _| {
            let status = match delta.status() {
                git2::Delta::Added => FileStatus::Added,
                git2::Delta::Deleted => FileStatus::Deleted,
                git2::Delta::Modified => FileStatus::Modified,
                git2::Delta::Renamed => FileStatus::Renamed,
                _ => FileStatus::Modified,
            };

            let path = delta.new_file().path()
                .or_else(|| delta.old_file().path())
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();

            let old_path = if matches!(status, FileStatus::Renamed) {
                delta.old_file().path().map(|p| p.to_string_lossy().to_string())
            } else {
                None
            };

            files.push(DiffFile {
                path,
                status,
                old_path,
                hunks: Vec::new(),
            });
            true
        },
        None,
        Some(&mut |_delta, hunk| {
            if let Some(file) = files.last_mut() {
                file.hunks.push(DiffHunk {
                    old_start: hunk.old_start(),
                    old_lines: hunk.old_lines(),
                    new_start: hunk.new_start(),
                    new_lines: hunk.new_lines(),
                    lines: Vec::new(),
                });
            }
            true
        }),
        Some(&mut |_delta, _hunk, line| {
            if let Some(file) = files.last_mut() {
                if let Some(hunk) = file.hunks.last_mut() {
                    let line_type = match line.origin() {
                        '+' => LineType::Add,
                        '-' => LineType::Delete,
                        _ => LineType::Context,
                    };

                    let content = std::str::from_utf8(line.content())
                        .unwrap_or("")
                        .trim_end_matches('\n')
                        .to_string();

                    hunk.lines.push(DiffLine {
                        line_type,
                        content,
                        old_num: line.old_lineno(),
                        new_num: line.new_lineno(),
                    });
                }
            }
            true
        }),
    ).map_err(|e| format!("Failed to iterate diff: {}", e))?;

    Ok(DiffResult {
        base_ref: base.to_string(),
        compare_ref: compare.to_string(),
        files,
    })
}
```

**Step 4: Run tests**

Run:
```bash
cd src-tauri && cargo test
```

Expected: All tests pass.

**Step 5: Commit**

```bash
git add src-tauri/src/git.rs
git commit -m "Add diff generation between branches with tests"
```

---

### Task 5: Tauri IPC Commands

**Files:**
- Create: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/lib.rs`

**Step 1: Create commands module**

Create `src-tauri/src/commands.rs`:
```rust
use tauri::State;
use crate::git;
use crate::state::AppState;
use crate::types::{DiffResult, RefInfo};

#[tauri::command]
pub async fn open_repo(path: String, state: State<'_, AppState>) -> Result<Vec<RefInfo>, String> {
    let repo = git::discover_repo(&path)?;
    let refs = git::list_refs(&repo)?;
    let mut repo_path = state.repo_path.lock().await;
    *repo_path = Some(path);
    Ok(refs)
}

#[tauri::command]
pub async fn get_refs(state: State<'_, AppState>) -> Result<Vec<RefInfo>, String> {
    let repo_path = state.repo_path.lock().await;
    let path = repo_path.as_deref().ok_or("No repo opened")?;
    let repo = git::discover_repo(path)?;
    git::list_refs(&repo)
}

#[tauri::command]
pub async fn get_diff(base: String, compare: String, state: State<'_, AppState>) -> Result<DiffResult, String> {
    let repo_path = state.repo_path.lock().await;
    let path = repo_path.as_deref().ok_or("No repo opened")?;
    let repo = git::discover_repo(path)?;
    git::generate_diff(&repo, &base, &compare)
}

#[tauri::command]
pub async fn submit_comment(
    file: String,
    start_line: u32,
    end_line: u32,
    code_context: String,
    comment: String,
    state: State<'_, AppState>,
) -> Result<u64, String> {
    let mut queue = state.comment_queue.lock().await;
    let id = queue.enqueue(file, start_line, end_line, code_context, comment);
    Ok(id)
}

#[tauri::command]
pub async fn get_queue_length(state: State<'_, AppState>) -> Result<usize, String> {
    let queue = state.comment_queue.lock().await;
    Ok(queue.len())
}
```

**Step 2: Register commands in `lib.rs`**

Update `src-tauri/src/lib.rs`:
```rust
mod commands;
mod git;
mod state;
mod types;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::open_repo,
            commands::get_refs,
            commands::get_diff,
            commands::submit_comment,
            commands::get_queue_length,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

**Step 3: Verify compilation**

Run:
```bash
cd src-tauri && cargo check
```

Expected: Compiles without errors.

**Step 4: Commit**

```bash
git add src-tauri/src/commands.rs src-tauri/src/lib.rs
git commit -m "Add Tauri IPC commands for git ops and comment submission"
```

---

### Task 6: MCP Server

**Files:**
- Create: `src-tauri/src/mcp.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/state.rs`

**Step 1: Create MCP server module**

Create `src-tauri/src/mcp.rs`:
```rust
use rmcp::{
    ServerHandler, ServiceExt,
    handler::server::tool::ToolRouter,
    model::*,
    tool, tool_handler, tool_router,
    ErrorData as McpError,
};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::state::CommentQueue;
use crate::types::ReviewComment;

#[derive(Clone)]
pub struct DifferMcpServer {
    queue: Arc<Mutex<CommentQueue>>,
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl DifferMcpServer {
    pub fn new(queue: Arc<Mutex<CommentQueue>>) -> Self {
        Self {
            queue,
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "Get the next review comment from the queue. Returns the comment with file path, line range, code context, and the reviewer's feedback. Returns empty if no comments pending.")]
    async fn get_next_comment(&self) -> Result<CallToolResult, McpError> {
        let mut queue = self.queue.lock().await;
        match queue.dequeue() {
            Some(comment) => {
                let json = serde_json::to_string_pretty(&comment)
                    .unwrap_or_else(|_| "Failed to serialize comment".into());
                Ok(CallToolResult::success(vec![Content::text(json)]))
            }
            None => {
                Ok(CallToolResult::success(vec![Content::text(
                    "No comments pending.",
                )]))
            }
        }
    }

    #[tool(description = "Get the number of pending review comments in the queue.")]
    async fn get_queue_status(&self) -> Result<CallToolResult, McpError> {
        let queue = self.queue.lock().await;
        let count = queue.len();
        let msg = format!("{{\"pending\": {}}}", count);
        Ok(CallToolResult::success(vec![Content::text(msg)]))
    }
}

#[tool_handler]
impl ServerHandler for DifferMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "Differ review tool. Use get_next_comment to receive code review feedback from the user. \
                 Each comment includes a file path, line range, code context, and the reviewer's instruction. \
                 Process comments one at a time, applying the requested changes.".into()
            ),
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            ..Default::default()
        }
    }
}

pub async fn start_mcp_server(queue: Arc<Mutex<CommentQueue>>, port: u16) -> Result<(), String> {
    let server = DifferMcpServer::new(queue);

    let addr = format!("0.0.0.0:{}", port);
    eprintln!("Starting MCP server on {}", addr);

    // Use SSE transport
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| format!("Failed to bind MCP server: {}", e))?;

    let ct = rmcp::transport::StreamableHttpServerTransport::new(listener);
    let _service = server.serve(ct).await
        .map_err(|e| format!("Failed to start MCP server: {}", e))?;

    Ok(())
}
```

Note: The exact `rmcp` SSE/HTTP server API may need adjustment based on the actual crate API. The implementer should check `rmcp` docs and examples at https://github.com/modelcontextprotocol/rust-sdk if compilation fails. The key pattern is: create a `DifferMcpServer`, create a transport, call `.serve(transport)`.

**Step 2: Start MCP server alongside Tauri app**

Update `src-tauri/src/lib.rs`:
```rust
mod commands;
mod git;
mod mcp;
mod state;
mod types;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_state = AppState::new();
    let queue_for_mcp = app_state.comment_queue.clone();

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::open_repo,
            commands::get_refs,
            commands::get_diff,
            commands::submit_comment,
            commands::get_queue_length,
        ])
        .setup(|_app| {
            // Start MCP server on a background task
            tauri::async_runtime::spawn(async move {
                if let Err(e) = mcp::start_mcp_server(queue_for_mcp, 3100).await {
                    eprintln!("MCP server error: {}", e);
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

**Step 3: Verify compilation**

Run:
```bash
cd src-tauri && cargo check
```

Expected: Compiles. If `rmcp` API differs, adjust based on compiler errors and crate docs.

**Step 4: Commit**

```bash
git add src-tauri/src/mcp.rs src-tauri/src/lib.rs
git commit -m "Add MCP server with get_next_comment and get_queue_status tools"
```

---

### Task 7: Frontend — Layout & Ref Selector

**Files:**
- Modify: `src/App.svelte`
- Create: `src/lib/RefSelector.svelte`
- Modify: `src/styles.css`

**Step 1: Install Shiki for syntax highlighting**

Run:
```bash
npm install shiki
```

**Step 2: Create RefSelector component**

Create `src/lib/RefSelector.svelte`:
```svelte
<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';

  type RefInfo = {
    name: string;
    refType: 'branch' | 'tag' | 'worktree';
  };

  export let refs: RefInfo[] = [];
  export let selected: string = '';
  export let label: string = '';

  function handleChange(e: Event) {
    selected = (e.target as HTMLSelectElement).value;
  }
</script>

<div class="ref-selector">
  <label>{label}</label>
  <select value={selected} on:change={handleChange}>
    <option value="" disabled>Select ref...</option>
    {#each refs as ref}
      <option value={ref.name}>{ref.name} ({ref.refType})</option>
    {/each}
  </select>
</div>

<style>
  .ref-selector {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  label {
    font-weight: 600;
    font-size: 13px;
    color: #aaa;
  }
  select {
    background: #2a2a2a;
    color: #e0e0e0;
    border: 1px solid #444;
    border-radius: 4px;
    padding: 4px 8px;
    font-size: 13px;
  }
</style>
```

**Step 3: Update App.svelte with layout**

Replace contents of `src/App.svelte`:
```svelte
<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { open } from '@tauri-apps/plugin-dialog';
  import RefSelector from './lib/RefSelector.svelte';

  type RefInfo = { name: string; refType: string };
  type DiffResult = {
    baseRef: string;
    compareRef: string;
    files: DiffFile[];
  };
  type DiffFile = {
    path: string;
    status: string;
    oldPath: string | null;
    hunks: DiffHunk[];
  };
  type DiffHunk = {
    oldStart: number;
    oldLines: number;
    newStart: number;
    newLines: number;
    lines: DiffLine[];
  };
  type DiffLine = {
    lineType: string;
    content: string;
    oldNum: number | null;
    newNum: number | null;
  };

  let refs: RefInfo[] = [];
  let baseRef = '';
  let compareRef = '';
  let diff: DiffResult | null = null;
  let selectedFile: DiffFile | null = null;
  let repoPath = '';

  async function openRepo() {
    const selected = await open({ directory: true });
    if (selected) {
      repoPath = selected as string;
      refs = await invoke<RefInfo[]>('open_repo', { path: repoPath });
      // Default: base=main, compare=first other branch
      const main = refs.find(r => r.name === 'main' || r.name === 'master');
      if (main) baseRef = main.name;
    }
  }

  async function loadDiff() {
    if (!baseRef || !compareRef) return;
    diff = await invoke<DiffResult>('get_diff', { base: baseRef, compare: compareRef });
    if (diff && diff.files.length > 0) {
      selectedFile = diff.files[0];
    }
  }

  $: if (baseRef && compareRef) loadDiff();
</script>

<main>
  <header>
    <button on:click={openRepo}>Open Repo</button>
    {#if repoPath}
      <span class="repo-path">{repoPath}</span>
    {/if}
    <div class="ref-selectors">
      <RefSelector {refs} bind:selected={baseRef} label="Base" />
      <RefSelector {refs} bind:selected={compareRef} label="Compare" />
    </div>
  </header>

  <div class="workspace">
    {#if diff}
      <aside class="file-tree">
        {#each diff.files as file}
          <button
            class="file-entry"
            class:active={selectedFile === file}
            on:click={() => selectedFile = file}
          >
            <span class="status-badge {file.status}">{file.status[0].toUpperCase()}</span>
            {file.path}
          </button>
        {/each}
      </aside>

      <section class="diff-pane">
        {#if selectedFile}
          <h3>{selectedFile.path}</h3>
          <p>Diff viewer goes here (Task 8)</p>
        {:else}
          <p class="empty">Select a file to view diff</p>
        {/if}
      </section>
    {:else}
      <p class="empty">Open a repo and select two refs to compare.</p>
    {/if}
  </div>
</main>

<style>
  :global(body) {
    margin: 0;
    background: #1a1a1a;
    color: #e0e0e0;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
  }
  main {
    display: flex;
    flex-direction: column;
    height: 100vh;
  }
  header {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px 12px;
    background: #252525;
    border-bottom: 1px solid #333;
  }
  header button {
    background: #3a3a3a;
    color: #e0e0e0;
    border: 1px solid #555;
    border-radius: 4px;
    padding: 4px 12px;
    cursor: pointer;
    font-size: 13px;
  }
  header button:hover { background: #4a4a4a; }
  .repo-path {
    font-size: 12px;
    color: #888;
    max-width: 200px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .ref-selectors {
    display: flex;
    gap: 16px;
    margin-left: auto;
  }
  .workspace {
    display: flex;
    flex: 1;
    overflow: hidden;
  }
  .file-tree {
    width: 250px;
    border-right: 1px solid #333;
    overflow-y: auto;
    padding: 4px 0;
    background: #1e1e1e;
  }
  .file-entry {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 6px 12px;
    background: none;
    border: none;
    color: #ccc;
    font-size: 13px;
    cursor: pointer;
    text-align: left;
  }
  .file-entry:hover { background: #2a2a2a; }
  .file-entry.active { background: #333; color: #fff; }
  .status-badge {
    font-size: 11px;
    font-weight: 700;
    width: 16px;
    height: 16px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 3px;
  }
  .status-badge.added { background: #2ea04333; color: #3fb950; }
  .status-badge.modified { background: #d2992233; color: #d29922; }
  .status-badge.deleted { background: #f8514933; color: #f85149; }
  .diff-pane {
    flex: 1;
    overflow: auto;
    padding: 16px;
  }
  .empty {
    color: #666;
    text-align: center;
    margin-top: 40px;
  }
  h3 {
    margin: 0 0 12px;
    font-size: 14px;
    color: #aaa;
  }
</style>
```

**Step 4: Install the dialog plugin**

Run:
```bash
npm install @tauri-apps/plugin-dialog
```

Add to `src-tauri/Cargo.toml` dependencies:
```toml
tauri-plugin-dialog = "2"
```

Register in `src-tauri/src/lib.rs` (add `.plugin(tauri_plugin_dialog::init())` before `.run()`):
```rust
        .plugin(tauri_plugin_dialog::init())
```

Add dialog permission to `src-tauri/capabilities/default.json` — add `"dialog:default"` to the permissions array.

**Step 5: Verify it builds and renders**

Run:
```bash
npm run tauri dev
```

Expected: Window opens with header bar (Open Repo button, ref selectors) and empty workspace area.

**Step 6: Commit**

```bash
git add -A
git commit -m "Add app layout with file tree sidebar and ref selector"
```

---

### Task 8: Frontend — Diff Viewer

**Files:**
- Create: `src/lib/DiffViewer.svelte`
- Modify: `src/App.svelte`

**Step 1: Create DiffViewer component**

Create `src/lib/DiffViewer.svelte`:
```svelte
<script lang="ts">
  export let file: {
    path: string;
    status: string;
    hunks: {
      oldStart: number;
      oldLines: number;
      newStart: number;
      newLines: number;
      lines: { lineType: string; content: string; oldNum: number | null; newNum: number | null }[];
    }[];
  };
  export let viewMode: 'split' | 'unified' = 'split';
  export let onLineSelect: (file: string, startLine: number, endLine: number, codeContext: string) => void = () => {};

  let selectionStart: number | null = null;
  let selectionEnd: number | null = null;
  let selectionSide: 'old' | 'new' | null = null;

  function handleLineClick(lineNum: number, side: 'old' | 'new', e: MouseEvent) {
    if (e.shiftKey && selectionStart !== null && selectionSide === side) {
      selectionEnd = lineNum;
      emitSelection();
    } else {
      selectionStart = lineNum;
      selectionEnd = lineNum;
      selectionSide = side;
      emitSelection();
    }
  }

  function emitSelection() {
    if (selectionStart === null || selectionEnd === null) return;
    const start = Math.min(selectionStart, selectionEnd);
    const end = Math.max(selectionStart, selectionEnd);

    // Gather code context from selected lines
    const lines = file.hunks.flatMap(h => h.lines);
    const contextLines = lines.filter(l => {
      const num = selectionSide === 'new' ? l.newNum : l.oldNum;
      return num !== null && num >= start && num <= end;
    });
    const codeContext = contextLines.map(l => l.content).join('\n');

    onLineSelect(file.path, start, end, codeContext);
  }

  function isSelected(lineNum: number | null, side: 'old' | 'new'): boolean {
    if (lineNum === null || selectionStart === null || selectionEnd === null || selectionSide !== side) return false;
    const start = Math.min(selectionStart, selectionEnd);
    const end = Math.max(selectionStart, selectionEnd);
    return lineNum >= start && lineNum <= end;
  }

  export function clearSelection() {
    selectionStart = null;
    selectionEnd = null;
    selectionSide = null;
  }
</script>

{#if viewMode === 'split'}
  <div class="diff-split">
    {#each file.hunks as hunk}
      <div class="hunk-header">
        @@ -{hunk.oldStart},{hunk.oldLines} +{hunk.newStart},{hunk.newLines} @@
      </div>
      <div class="hunk-content-split">
        <div class="side old-side">
          {#each hunk.lines as line}
            {#if line.lineType !== 'add'}
              <div
                class="line {line.lineType}"
                class:selected={isSelected(line.oldNum, 'old')}
              >
                <span
                  class="line-num"
                  on:click={(e) => line.oldNum && handleLineClick(line.oldNum, 'old', e)}
                >{line.oldNum ?? ''}</span>
                <span class="line-content">{line.content}</span>
              </div>
            {:else}
              <div class="line filler"><span class="line-num"></span><span class="line-content"></span></div>
            {/if}
          {/each}
        </div>
        <div class="side new-side">
          {#each hunk.lines as line}
            {#if line.lineType !== 'delete'}
              <div
                class="line {line.lineType}"
                class:selected={isSelected(line.newNum, 'new')}
              >
                <span
                  class="line-num"
                  on:click={(e) => line.newNum && handleLineClick(line.newNum, 'new', e)}
                >{line.newNum ?? ''}</span>
                <span class="line-content">{line.content}</span>
              </div>
            {:else}
              <div class="line filler"><span class="line-num"></span><span class="line-content"></span></div>
            {/if}
          {/each}
        </div>
      </div>
    {/each}
  </div>
{:else}
  <div class="diff-unified">
    {#each file.hunks as hunk}
      <div class="hunk-header">
        @@ -{hunk.oldStart},{hunk.oldLines} +{hunk.newStart},{hunk.newLines} @@
      </div>
      {#each hunk.lines as line}
        <div
          class="line {line.lineType}"
          class:selected={isSelected(line.newNum ?? line.oldNum, 'new')}
        >
          <span class="line-num old">{line.oldNum ?? ''}</span>
          <span class="line-num new">{line.newNum ?? ''}</span>
          <span class="line-prefix">{line.lineType === 'add' ? '+' : line.lineType === 'delete' ? '-' : ' '}</span>
          <span
            class="line-content"
            on:click={(e) => {
              const num = line.newNum ?? line.oldNum;
              if (num) handleLineClick(num, 'new', e);
            }}
          >{line.content}</span>
        </div>
      {/each}
    {/each}
  </div>
{/if}

<style>
  .hunk-header {
    background: #1e3a5f;
    color: #79b8ff;
    padding: 4px 12px;
    font-size: 12px;
    font-family: monospace;
    border-top: 1px solid #333;
  }
  .hunk-content-split {
    display: flex;
  }
  .side {
    flex: 1;
    overflow-x: auto;
  }
  .old-side { border-right: 1px solid #333; }
  .line {
    display: flex;
    font-family: 'SF Mono', 'Fira Code', monospace;
    font-size: 13px;
    line-height: 20px;
  }
  .line.add { background: #12261e; }
  .line.delete { background: #2d1517; }
  .line.context { background: #1a1a1a; }
  .line.filler { background: #1a1a1a; opacity: 0.5; }
  .line.selected { background: #264f78 !important; }
  .line-num {
    min-width: 40px;
    padding: 0 8px;
    text-align: right;
    color: #555;
    user-select: none;
    cursor: pointer;
    flex-shrink: 0;
  }
  .line-num:hover { color: #aaa; background: #333; }
  .line-content {
    flex: 1;
    padding: 0 8px;
    white-space: pre;
  }
  .line-prefix {
    width: 16px;
    text-align: center;
    flex-shrink: 0;
    color: #666;
  }
  .diff-unified .line-num {
    min-width: 35px;
  }
  .diff-unified .line-num.old { border-right: none; }
  .diff-unified .line-num.new { border-right: 1px solid #333; }
</style>
```

**Step 2: Wire DiffViewer into App.svelte**

In `src/App.svelte`, replace the `{#if selectedFile}` block inside `.diff-pane` with:
```svelte
        {#if selectedFile}
          <DiffViewer
            file={selectedFile}
            {viewMode}
            onLineSelect={handleLineSelect}
          />
        {:else}
          <p class="empty">Select a file to view diff</p>
        {/if}
```

Add to the script section:
```typescript
  import DiffViewer from './lib/DiffViewer.svelte';

  let viewMode: 'split' | 'unified' = 'split';

  // Line selection state for comment box
  let selectionFile = '';
  let selectionStart = 0;
  let selectionEnd = 0;
  let selectionContext = '';
  let showCommentBox = false;

  function handleLineSelect(file: string, startLine: number, endLine: number, codeContext: string) {
    selectionFile = file;
    selectionStart = startLine;
    selectionEnd = endLine;
    selectionContext = codeContext;
    showCommentBox = true;
  }
```

Add a view mode toggle in the header:
```svelte
    <button on:click={() => viewMode = viewMode === 'split' ? 'unified' : 'split'}>
      {viewMode === 'split' ? 'Unified' : 'Split'}
    </button>
```

**Step 3: Verify it renders**

Run:
```bash
npm run tauri dev
```

Expected: Open a git repo with branches, select two refs, see file list and diff rendering.

**Step 4: Commit**

```bash
git add src/lib/DiffViewer.svelte src/App.svelte
git commit -m "Add diff viewer with split and unified modes, line selection"
```

---

### Task 9: Frontend — Comment Box

**Files:**
- Create: `src/lib/CommentBox.svelte`
- Modify: `src/App.svelte`

**Step 1: Create CommentBox component**

Create `src/lib/CommentBox.svelte`:
```svelte
<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';

  export let file: string;
  export let startLine: number;
  export let endLine: number;
  export let codeContext: string;
  export let onSubmit: () => void = () => {};
  export let onCancel: () => void = () => {};

  let comment = '';
  let submitting = false;

  async function submit() {
    if (!comment.trim()) return;
    submitting = true;
    try {
      await invoke('submit_comment', {
        file,
        startLine,
        endLine,
        codeContext,
        comment: comment.trim(),
      });
      comment = '';
      onSubmit();
    } catch (e) {
      console.error('Failed to submit comment:', e);
    } finally {
      submitting = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) {
      submit();
    }
    if (e.key === 'Escape') {
      onCancel();
    }
  }
</script>

<div class="comment-box">
  <div class="comment-header">
    <span class="comment-location">{file}:{startLine}{endLine !== startLine ? `-${endLine}` : ''}</span>
    <button class="close-btn" on:click={onCancel}>x</button>
  </div>
  <div class="code-preview">
    <pre>{codeContext}</pre>
  </div>
  <textarea
    bind:value={comment}
    on:keydown={handleKeydown}
    placeholder="Describe what should change..."
    rows="3"
    disabled={submitting}
  ></textarea>
  <div class="comment-actions">
    <span class="hint">Cmd+Enter to send</span>
    <button on:click={submit} disabled={submitting || !comment.trim()}>
      {submitting ? 'Sending...' : 'Send'}
    </button>
  </div>
</div>

<style>
  .comment-box {
    position: fixed;
    bottom: 16px;
    right: 16px;
    width: 400px;
    background: #2a2a2a;
    border: 1px solid #555;
    border-radius: 8px;
    box-shadow: 0 4px 24px rgba(0, 0, 0, 0.5);
    z-index: 100;
  }
  .comment-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 12px;
    border-bottom: 1px solid #444;
  }
  .comment-location {
    font-family: monospace;
    font-size: 12px;
    color: #79b8ff;
  }
  .close-btn {
    background: none;
    border: none;
    color: #888;
    cursor: pointer;
    font-size: 14px;
    padding: 0 4px;
  }
  .close-btn:hover { color: #fff; }
  .code-preview {
    max-height: 80px;
    overflow: auto;
    padding: 8px 12px;
    background: #1e1e1e;
    border-bottom: 1px solid #444;
  }
  .code-preview pre {
    margin: 0;
    font-size: 11px;
    font-family: 'SF Mono', 'Fira Code', monospace;
    color: #aaa;
    white-space: pre;
  }
  textarea {
    width: 100%;
    box-sizing: border-box;
    background: #1e1e1e;
    color: #e0e0e0;
    border: none;
    padding: 12px;
    font-size: 13px;
    font-family: inherit;
    resize: vertical;
  }
  textarea:focus { outline: none; }
  textarea::placeholder { color: #666; }
  .comment-actions {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 12px;
    border-top: 1px solid #444;
  }
  .hint {
    font-size: 11px;
    color: #666;
  }
  button {
    background: #347d39;
    color: #fff;
    border: none;
    border-radius: 4px;
    padding: 4px 16px;
    font-size: 13px;
    cursor: pointer;
  }
  button:hover { background: #3e8e41; }
  button:disabled { opacity: 0.5; cursor: not-allowed; }
</style>
```

**Step 2: Wire CommentBox into App.svelte**

Add to the script imports:
```typescript
  import CommentBox from './lib/CommentBox.svelte';
```

Add CommentBox at the bottom of the `<main>` element:
```svelte
  {#if showCommentBox}
    <CommentBox
      file={selectionFile}
      startLine={selectionStart}
      endLine={selectionEnd}
      codeContext={selectionContext}
      onSubmit={() => { showCommentBox = false; }}
      onCancel={() => { showCommentBox = false; }}
    />
  {/if}
```

**Step 3: Verify end-to-end**

Run:
```bash
npm run tauri dev
```

Expected: Click line numbers to select, comment box appears, type and send dispatches to backend queue.

**Step 4: Commit**

```bash
git add src/lib/CommentBox.svelte src/App.svelte
git commit -m "Add comment box with immediate dispatch to backend queue"
```

---

### Task 10: Frontend — Queue Status

**Files:**
- Create: `src/lib/QueueStatus.svelte`
- Modify: `src/App.svelte`

**Step 1: Create QueueStatus component**

Create `src/lib/QueueStatus.svelte`:
```svelte
<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount, onDestroy } from 'svelte';

  let pending = 0;
  let interval: ReturnType<typeof setInterval>;

  async function poll() {
    try {
      pending = await invoke<number>('get_queue_length');
    } catch {
      // ignore polling errors
    }
  }

  onMount(() => {
    poll();
    interval = setInterval(poll, 2000);
  });

  onDestroy(() => {
    clearInterval(interval);
  });
</script>

<div class="queue-status" class:has-items={pending > 0}>
  {#if pending > 0}
    {pending} comment{pending !== 1 ? 's' : ''} pending
  {:else}
    Queue empty
  {/if}
</div>

<style>
  .queue-status {
    font-size: 12px;
    color: #666;
    padding: 4px 8px;
    border-radius: 4px;
    background: #2a2a2a;
  }
  .queue-status.has-items {
    color: #d29922;
    background: #d2992215;
  }
</style>
```

**Step 2: Add to App.svelte header**

Add import:
```typescript
  import QueueStatus from './lib/QueueStatus.svelte';
```

Add in the header, before the ref selectors:
```svelte
    <QueueStatus />
```

**Step 3: Verify**

Run:
```bash
npm run tauri dev
```

Expected: Queue status shows in header. Send a comment and watch it increment.

**Step 4: Commit**

```bash
git add src/lib/QueueStatus.svelte src/App.svelte
git commit -m "Add queue status indicator with polling"
```

---

### Task 11: Integration Test — Full Flow

**Step 1: Manual integration test**

1. Run `npm run tauri dev`
2. Click "Open Repo" and select a git repo with at least two branches
3. Select base and compare refs
4. Verify file list appears in sidebar
5. Click a file, verify diff renders in split view
6. Toggle to unified, verify it switches
7. Click a line number, verify highlight
8. Shift+click another line, verify range selection
9. Verify comment box appears with code context
10. Type a comment and send
11. Verify queue status increments
12. In a separate terminal, verify MCP server is running: `curl http://localhost:3100` (should respond)

**Step 2: Test MCP tool via Claude Code**

Add to Claude Code's MCP config (`.claude.json` or similar):
```json
{
  "mcpServers": {
    "differ": {
      "type": "sse",
      "url": "http://localhost:3100/sse"
    }
  }
}
```

Run Claude Code and verify it can call `get_queue_status` and `get_next_comment`.

**Step 3: Fix any issues found during integration testing**

**Step 4: Commit any fixes**

```bash
git add -A
git commit -m "Integration test fixes"
```

---

### Task 12: Polish & README

**Step 1: Add a .gitignore**

Create `.gitignore`:
```
node_modules/
dist/
src-tauri/target/
```

**Step 2: Final build check**

Run:
```bash
npm run tauri build
```

Expected: Produces a runnable binary.

**Step 3: Commit**

```bash
git add .gitignore
git commit -m "Add .gitignore"
```

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
pub async fn get_diff(
    base: String,
    compare: String,
    state: State<'_, AppState>,
) -> Result<DiffResult, String> {
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

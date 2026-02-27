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
        .plugin(tauri_plugin_dialog::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::open_repo,
            commands::get_refs,
            commands::get_diff,
            commands::submit_comment,
            commands::get_queue_length,
        ])
        .setup(|_app| {
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

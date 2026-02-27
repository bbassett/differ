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

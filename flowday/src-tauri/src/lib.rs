mod commands;
mod db;
mod timer;

use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, LogicalSize, Manager, State};

use commands::{blocks, calendar, google, push};
use db::DbState;
use google::GoogleAuthState;
use timer::{Timer, TimerState};

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! Welcome to FlowDay.", name)
}

#[tauri::command]
async fn start_timer(
    app: AppHandle,
    timer: State<'_, Timer>,
    block_id: String,
    duration_secs: u64,
) -> Result<TimerState, String> {
    Ok(timer.start(app, block_id, duration_secs).await)
}

#[tauri::command]
async fn pause_timer(timer: State<'_, Timer>) -> Result<TimerState, String> {
    Ok(timer.pause().await)
}

#[tauri::command]
async fn resume_timer(app: AppHandle, timer: State<'_, Timer>) -> Result<TimerState, String> {
    Ok(timer.resume(app).await)
}

#[tauri::command]
async fn stop_timer(timer: State<'_, Timer>) -> Result<TimerState, String> {
    Ok(timer.stop().await)
}

#[tauri::command]
async fn extend_timer(timer: State<'_, Timer>, extra_secs: u64) -> Result<TimerState, String> {
    Ok(timer.extend(extra_secs).await)
}

#[tauri::command]
async fn get_timer_state(timer: State<'_, Timer>) -> Result<TimerState, String> {
    Ok(timer.get_state().await)
}

#[tauri::command]
async fn toggle_panel(app: AppHandle, expanded: bool) -> Result<(), String> {
    let window = app.get_webview_window("main").ok_or("Window not found")?;
    let height = if expanded { 600.0 } else { 60.0 };
    window
        .set_size(LogicalSize::new(420.0, height))
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn get_db_path(app: &tauri::App) -> PathBuf {
    let app_dir = app
        .path()
        .app_data_dir()
        .expect("Failed to resolve app data dir");
    std::fs::create_dir_all(&app_dir).expect("Failed to create app data dir");
    app_dir.join("flowday.db")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(Timer::new())
        .manage(GoogleAuthState::new())
        .setup(|app| {
            let db_path = get_db_path(app);
            println!("[flowday] Database path: {:?}", db_path);
            let conn =
                db::init_database(&db_path).expect("Failed to initialize database");
            println!("[flowday] Database initialized successfully");

            // Store connection as managed state for block commands
            app.manage(DbState(Mutex::new(conn)));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            toggle_panel,
            start_timer,
            pause_timer,
            resume_timer,
            stop_timer,
            extend_timer,
            get_timer_state,
            blocks::get_blocks,
            blocks::add_block,
            blocks::edit_block,
            blocks::delete_block,
            blocks::reorder_blocks,
            push::push_block_to_calendar,
            push::unpush_block_from_calendar,
            google::google_set_oauth_config,
            google::google_get_auth_url,
            google::google_exchange_code,
            google::google_list_accounts,
            google::google_remove_account,
            google::google_is_authenticated,
            google::google_fetch_events,
            google::google_create_event,
            calendar::calendar_sync,
            calendar::get_calendar_events,
            calendar::get_conflicts,
            calendar::get_last_sync_time,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

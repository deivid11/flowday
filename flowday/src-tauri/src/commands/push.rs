use serde::{Deserialize, Serialize};
use tauri::State;

use crate::db::DbState;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PushResult {
    pub block_id: String,
    pub calendar_event_id: String,
    pub pushed: bool,
}

/// Push a block to the calendar. Only DeepWork blocks are allowed.
/// In the current implementation, we generate a placeholder event ID
/// and mark the block as pushed. The actual Google Calendar integration
/// will use the google_tokens table when configured.
#[tauri::command]
pub fn push_block_to_calendar(
    db: State<'_, DbState>,
    block_id: String,
) -> Result<PushResult, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    // Fetch the block and validate it exists and is DeepWork
    let (block_type, already_pushed): (String, bool) = conn
        .query_row(
            "SELECT type, pushed_to_calendar FROM blocks WHERE id = ?1",
            [&block_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|_| format!("Block {} not found", block_id))?;

    if block_type != "DeepWork" {
        return Err("Only DeepWork blocks can be pushed to calendar".into());
    }

    if already_pushed {
        return Err("Block is already pushed to calendar".into());
    }

    // Generate a placeholder calendar event ID
    // Real implementation would call Google Calendar API here
    let calendar_event_id = format!("cal_{}", block_id);

    conn.execute(
        "UPDATE blocks SET pushed_to_calendar = 1, calendar_event_id = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
        rusqlite::params![calendar_event_id, block_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(PushResult {
        block_id,
        calendar_event_id,
        pushed: true,
    })
}

/// Remove a block from the calendar (undo push).
#[tauri::command]
pub fn unpush_block_from_calendar(
    db: State<'_, DbState>,
    block_id: String,
) -> Result<PushResult, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let (pushed, event_id): (bool, Option<String>) = conn
        .query_row(
            "SELECT pushed_to_calendar, calendar_event_id FROM blocks WHERE id = ?1",
            [&block_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|_| format!("Block {} not found", block_id))?;

    if !pushed {
        return Err("Block is not pushed to calendar".into());
    }

    // Real implementation would delete the Google Calendar event here
    // using the stored calendar_event_id

    conn.execute(
        "UPDATE blocks SET pushed_to_calendar = 0, calendar_event_id = NULL, updated_at = CURRENT_TIMESTAMP WHERE id = ?1",
        [&block_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(PushResult {
        block_id,
        calendar_event_id: event_id.unwrap_or_default(),
        pushed: false,
    })
}

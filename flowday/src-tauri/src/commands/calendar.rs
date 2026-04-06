use crate::db::DbState;
use serde::{Deserialize, Serialize};
use tauri::State;

/// A calendar event fetched from Google Calendar (or mock).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarEvent {
    pub id: String,
    pub google_event_id: String,
    pub summary: String,
    pub start_time: String, // HH:MM
    pub end_time: String,   // HH:MM
    pub date: String,       // YYYY-MM-DD
    pub all_day: bool,
    pub status: String,
}

/// A conflict between a block and a calendar event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Conflict {
    pub block_id: String,
    pub block_name: String,
    pub block_start: String,
    pub block_end: String,
    pub event_id: String,
    pub event_summary: String,
    pub event_start: String,
    pub event_end: String,
    pub overlap_minutes: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncResult {
    pub events_synced: usize,
    pub conflicts: Vec<Conflict>,
    pub last_synced_at: String,
}

fn today() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let days = now / 86400;
    let mut y = 1970i64;
    let mut remaining = days as i64;
    loop {
        let diy = if y % 4 == 0 && (y % 100 != 0 || y % 400 == 0) { 366 } else { 365 };
        if remaining < diy { break; }
        remaining -= diy;
        y += 1;
    }
    let leap = y % 4 == 0 && (y % 100 != 0 || y % 400 == 0);
    let md: [i64; 12] = [31, if leap { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut m = 0usize;
    for (i, &d) in md.iter().enumerate() {
        if remaining < d { m = i; break; }
        remaining -= d;
    }
    format!("{:04}-{:02}-{:02}", y, m + 1, remaining + 1)
}

fn now_timestamp() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    // ISO-ish timestamp for display
    let date = today();
    let day_secs = now % 86400;
    let h = day_secs / 3600;
    let m = (day_secs % 3600) / 60;
    let s = day_secs % 60;
    format!("{}T{:02}:{:02}:{:02}Z", date, h, m, s)
}

/// Parse HH:MM to minutes since midnight.
fn hhmm_to_minutes(t: &str) -> Option<i64> {
    if t.len() < 5 { return None; }
    let h: i64 = t[..2].parse().ok()?;
    let m: i64 = t[3..5].parse().ok()?;
    Some(h * 60 + m)
}

/// Detect overlaps between blocks and calendar events for today.
fn detect_conflicts(
    conn: &rusqlite::Connection,
    date: &str,
) -> Result<Vec<Conflict>, String> {
    // Get today's blocks
    let mut block_stmt = conn
        .prepare(
            "SELECT id, name, start_time, duration FROM blocks WHERE date = ?1 ORDER BY start_time",
        )
        .map_err(|e| e.to_string())?;

    let blocks: Vec<(String, String, String, i64)> = block_stmt
        .query_map([date], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    // Get today's calendar events (non-all-day, confirmed)
    let mut evt_stmt = conn
        .prepare(
            "SELECT id, summary, start_time, end_time FROM calendar_events WHERE date = ?1 AND all_day = 0 AND status = 'confirmed'",
        )
        .map_err(|e| e.to_string())?;

    let events: Vec<(String, String, String, String)> = evt_stmt
        .query_map([date], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut conflicts = Vec::new();

    for (block_id, block_name, block_start, block_duration) in &blocks {
        let Some(bs) = hhmm_to_minutes(block_start) else { continue };
        let be = bs + block_duration;

        for (evt_id, evt_summary, evt_start, evt_end) in &events {
            let Some(es) = hhmm_to_minutes(evt_start) else { continue };
            let Some(ee) = hhmm_to_minutes(evt_end) else { continue };

            // Overlap check: two intervals overlap if start1 < end2 && start2 < end1
            if bs < ee && es < be {
                let overlap_start = bs.max(es);
                let overlap_end = be.min(ee);
                let overlap_minutes = overlap_end - overlap_start;

                if overlap_minutes > 0 {
                    let block_end_str = format!(
                        "{:02}:{:02}",
                        be / 60,
                        be % 60
                    );
                    conflicts.push(Conflict {
                        block_id: block_id.clone(),
                        block_name: block_name.clone(),
                        block_start: block_start.clone(),
                        block_end: block_end_str,
                        event_id: evt_id.clone(),
                        event_summary: evt_summary.clone(),
                        event_start: evt_start.clone(),
                        event_end: evt_end.clone(),
                        overlap_minutes,
                    });
                }
            }
        }
    }

    Ok(conflicts)
}

/// Sync calendar events from Google Calendar.
/// Currently stores events passed from the frontend (which calls Google API via tokens).
/// When the Rust-side Google OAuth is ready, this will call the API directly.
#[tauri::command]
pub fn calendar_sync(
    db: State<'_, DbState>,
    events: Vec<CalendarEvent>,
) -> Result<SyncResult, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let date = today();

    // Clear today's cached events and re-insert
    conn.execute("DELETE FROM calendar_events WHERE date = ?1", [&date])
        .map_err(|e| e.to_string())?;

    for evt in &events {
        conn.execute(
            "INSERT OR REPLACE INTO calendar_events (id, google_event_id, summary, start_time, end_time, date, all_day, status, synced_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, CURRENT_TIMESTAMP)",
            rusqlite::params![
                &evt.id,
                &evt.google_event_id,
                &evt.summary,
                &evt.start_time,
                &evt.end_time,
                &date,
                evt.all_day,
                &evt.status,
            ],
        )
        .map_err(|e| e.to_string())?;
    }

    // Update last sync timestamp
    let ts = now_timestamp();
    conn.execute(
        "UPDATE settings SET value = ?1, updated_at = CURRENT_TIMESTAMP WHERE key = 'last_calendar_sync'",
        [&ts],
    )
    .map_err(|e| e.to_string())?;

    // Detect conflicts
    let conflicts = detect_conflicts(&conn, &date)?;

    Ok(SyncResult {
        events_synced: events.len(),
        conflicts,
        last_synced_at: ts,
    })
}

/// Get cached calendar events for today.
#[tauri::command]
pub fn get_calendar_events(db: State<'_, DbState>) -> Result<Vec<CalendarEvent>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let date = today();

    let mut stmt = conn
        .prepare(
            "SELECT id, google_event_id, summary, start_time, end_time, date, all_day, status
             FROM calendar_events WHERE date = ?1 ORDER BY start_time",
        )
        .map_err(|e| e.to_string())?;

    let events = stmt
        .query_map([&date], |row| {
            Ok(CalendarEvent {
                id: row.get(0)?,
                google_event_id: row.get(1)?,
                summary: row.get(2)?,
                start_time: row.get(3)?,
                end_time: row.get(4)?,
                date: row.get(5)?,
                all_day: row.get(6)?,
                status: row.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(events)
}

/// Detect conflicts between today's blocks and calendar events.
#[tauri::command]
pub fn get_conflicts(db: State<'_, DbState>) -> Result<Vec<Conflict>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let date = today();
    detect_conflicts(&conn, &date)
}

/// Get the last sync timestamp.
#[tauri::command]
pub fn get_last_sync_time(db: State<'_, DbState>) -> Result<String, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let ts: String = conn
        .query_row(
            "SELECT value FROM settings WHERE key = 'last_calendar_sync'",
            [],
            |row| row.get(0),
        )
        .unwrap_or_default();
    Ok(ts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hhmm_to_minutes() {
        assert_eq!(hhmm_to_minutes("00:00"), Some(0));
        assert_eq!(hhmm_to_minutes("08:30"), Some(510));
        assert_eq!(hhmm_to_minutes("23:59"), Some(1439));
        assert_eq!(hhmm_to_minutes("bad"), None);
    }

    #[test]
    fn test_today_format() {
        let d = today();
        assert_eq!(d.len(), 10);
        assert_eq!(&d[4..5], "-");
        assert_eq!(&d[7..8], "-");
    }

    #[test]
    fn test_overlap_logic() {
        // Block 09:00-10:30, Event 10:00-11:00 → 30min overlap
        let bs = 540i64; // 09:00
        let be = 630i64; // 10:30
        let es = 600i64; // 10:00
        let ee = 660i64; // 11:00

        assert!(bs < ee && es < be); // overlaps
        let overlap = be.min(ee) - bs.max(es);
        assert_eq!(overlap, 30);
    }

    #[test]
    fn test_no_overlap() {
        // Block 09:00-10:00, Event 10:00-11:00 → no overlap (adjacent)
        let bs = 540i64;
        let be = 600i64;
        let es = 600i64;
        let ee = 660i64;

        // bs < ee (540 < 660) = true, es < be (600 < 600) = false → no overlap
        assert!(!(bs < ee && es < be));
    }
}

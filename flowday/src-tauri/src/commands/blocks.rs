use crate::db::DbState;
use serde::{Deserialize, Serialize};
use tauri::State;

// ---------- Types ----------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub block_type: String,
    pub start_time: String,
    pub duration: i64,
    pub color: String,
    pub notes: Option<String>,
    pub pause_time: i64,
    pub interruption_count: i64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewBlock {
    pub name: String,
    #[serde(rename = "type")]
    pub block_type: String,
    pub start_time: String,
    pub duration: i64,
    pub color: String,
    pub notes: Option<String>,
    pub pause_time: i64,
    pub interruption_count: i64,
}

// ---------- Validation ----------

const VALID_BLOCK_TYPES: &[&str] = &["DeepWork", "Reactive", "Meeting", "Admin", "Break"];

fn validate_block_name(name: &str) -> Result<(), String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("Block name cannot be empty".into());
    }
    if trimmed.len() > 200 {
        return Err("Block name too long (max 200 chars)".into());
    }
    Ok(())
}

fn validate_duration(duration: i64) -> Result<(), String> {
    if duration <= 0 {
        return Err("Duration must be greater than 0".into());
    }
    if duration > 480 {
        return Err("Duration cannot exceed 480 minutes (8 hours)".into());
    }
    Ok(())
}

fn validate_start_time(time: &str) -> Result<(), String> {
    if time.len() != 5 || time.as_bytes()[2] != b':' {
        return Err("start_time must be in HH:MM format".into());
    }
    let hours: u32 = time[..2]
        .parse()
        .map_err(|_| "Invalid hours in start_time")?;
    let minutes: u32 = time[3..]
        .parse()
        .map_err(|_| "Invalid minutes in start_time")?;
    if hours > 23 {
        return Err("Hours must be 0-23".into());
    }
    if minutes > 59 {
        return Err("Minutes must be 0-59".into());
    }
    Ok(())
}

fn validate_block_type(block_type: &str) -> Result<(), String> {
    if !VALID_BLOCK_TYPES.contains(&block_type) {
        return Err(format!(
            "Invalid block type '{}'. Must be one of: {}",
            block_type,
            VALID_BLOCK_TYPES.join(", ")
        ));
    }
    Ok(())
}

fn validate_new_block(block: &NewBlock) -> Result<(), String> {
    validate_block_name(&block.name)?;
    validate_duration(block.duration)?;
    validate_start_time(&block.start_time)?;
    validate_block_type(&block.block_type)?;
    Ok(())
}

fn validate_edit_block(block: &Block) -> Result<(), String> {
    validate_block_name(&block.name)?;
    validate_duration(block.duration)?;
    validate_start_time(&block.start_time)?;
    validate_block_type(&block.block_type)?;
    Ok(())
}

fn today() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    // Convert to date string YYYY-MM-DD
    let days = now / 86400;
    let mut y = 1970i64;
    let mut remaining = days as i64;

    loop {
        let days_in_year = if y % 4 == 0 && (y % 100 != 0 || y % 400 == 0) {
            366
        } else {
            365
        };
        if remaining < days_in_year {
            break;
        }
        remaining -= days_in_year;
        y += 1;
    }

    let leap = y % 4 == 0 && (y % 100 != 0 || y % 400 == 0);
    let month_days: [i64; 12] = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];

    let mut m = 0usize;
    for (i, &d) in month_days.iter().enumerate() {
        if remaining < d {
            m = i;
            break;
        }
        remaining -= d;
    }

    format!("{:04}-{:02}-{:02}", y, m + 1, remaining + 1)
}

fn generate_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{:x}", ts)
}

// ---------- Commands ----------

#[tauri::command]
pub fn get_blocks(db: State<'_, DbState>) -> Result<Vec<Block>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let date = today();

    let mut stmt = conn
        .prepare(
            "SELECT id, name, type, start_time, duration, color, notes, pause_time, interruption_count
             FROM blocks
             WHERE date = ?1
             ORDER BY sort_order ASC, start_time ASC",
        )
        .map_err(|e| e.to_string())?;

    let blocks = stmt
        .query_map([&date], |row| {
            Ok(Block {
                id: row.get(0)?,
                name: row.get(1)?,
                block_type: row.get(2)?,
                start_time: row.get(3)?,
                duration: row.get(4)?,
                color: row.get(5)?,
                notes: row.get(6)?,
                pause_time: row.get(7)?,
                interruption_count: row.get(8)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(blocks)
}

#[tauri::command]
pub fn add_block(block: NewBlock, db: State<'_, DbState>) -> Result<Block, String> {
    validate_new_block(&block)?;

    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let id = generate_id();
    let date = today();

    // Get next sort_order
    let max_sort: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(sort_order), -1) FROM blocks WHERE date = ?1",
            [&date],
            |row| row.get(0),
        )
        .unwrap_or(-1);

    conn.execute(
        "INSERT INTO blocks (id, date, name, type, start_time, duration, color, notes, pause_time, interruption_count, sort_order)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        rusqlite::params![
            &id,
            &date,
            block.name.trim(),
            &block.block_type,
            &block.start_time,
            block.duration,
            &block.color,
            &block.notes,
            block.pause_time,
            block.interruption_count,
            max_sort + 1,
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(Block {
        id,
        name: block.name.trim().to_string(),
        block_type: block.block_type,
        start_time: block.start_time,
        duration: block.duration,
        color: block.color,
        notes: block.notes,
        pause_time: block.pause_time,
        interruption_count: block.interruption_count,
    })
}

#[tauri::command]
pub fn edit_block(block: Block, db: State<'_, DbState>) -> Result<Block, String> {
    validate_edit_block(&block)?;

    let conn = db.0.lock().map_err(|e| e.to_string())?;

    let rows = conn
        .execute(
            "UPDATE blocks SET name = ?1, type = ?2, start_time = ?3, duration = ?4, color = ?5, notes = ?6, pause_time = ?7, interruption_count = ?8, updated_at = CURRENT_TIMESTAMP
             WHERE id = ?9",
            rusqlite::params![
                block.name.trim(),
                &block.block_type,
                &block.start_time,
                block.duration,
                &block.color,
                &block.notes,
                block.pause_time,
                block.interruption_count,
                &block.id,
            ],
        )
        .map_err(|e| e.to_string())?;

    if rows == 0 {
        return Err(format!("Block '{}' not found", block.id));
    }

    Ok(Block {
        name: block.name.trim().to_string(),
        ..block
    })
}

#[tauri::command]
pub fn delete_block(id: String, db: State<'_, DbState>) -> Result<(), String> {
    if id.trim().is_empty() {
        return Err("Block id cannot be empty".into());
    }

    let conn = db.0.lock().map_err(|e| e.to_string())?;

    // Delete related interruptions first (foreign key)
    conn.execute("DELETE FROM interruptions WHERE block_id = ?1", [&id])
        .map_err(|e| e.to_string())?;

    // Delete related energy ratings
    conn.execute("DELETE FROM energy_ratings WHERE block_id = ?1", [&id])
        .map_err(|e| e.to_string())?;

    let rows = conn
        .execute("DELETE FROM blocks WHERE id = ?1", [&id])
        .map_err(|e| e.to_string())?;

    if rows == 0 {
        return Err(format!("Block '{}' not found", id));
    }

    Ok(())
}

#[tauri::command]
pub fn reorder_blocks(ids: Vec<String>, db: State<'_, DbState>) -> Result<(), String> {
    if ids.is_empty() {
        return Ok(());
    }

    let conn = db.0.lock().map_err(|e| e.to_string())?;

    for (i, id) in ids.iter().enumerate() {
        conn.execute(
            "UPDATE blocks SET sort_order = ?1 WHERE id = ?2",
            rusqlite::params![i as i64, id],
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_block_name() {
        assert!(validate_block_name("").is_err());
        assert!(validate_block_name("   ").is_err());
        assert!(validate_block_name("Focus").is_ok());
        assert!(validate_block_name(&"x".repeat(201)).is_err());
    }

    #[test]
    fn test_validate_duration() {
        assert!(validate_duration(0).is_err());
        assert!(validate_duration(-5).is_err());
        assert!(validate_duration(90).is_ok());
        assert!(validate_duration(481).is_err());
    }

    #[test]
    fn test_validate_start_time() {
        assert!(validate_start_time("08:00").is_ok());
        assert!(validate_start_time("23:59").is_ok());
        assert!(validate_start_time("00:00").is_ok());
        assert!(validate_start_time("24:00").is_err());
        assert!(validate_start_time("12:60").is_err());
        assert!(validate_start_time("abc").is_err());
        assert!(validate_start_time("1:00").is_err());
    }

    #[test]
    fn test_validate_block_type() {
        assert!(validate_block_type("DeepWork").is_ok());
        assert!(validate_block_type("Reactive").is_ok());
        assert!(validate_block_type("Meeting").is_ok());
        assert!(validate_block_type("Admin").is_ok());
        assert!(validate_block_type("Break").is_ok());
        assert!(validate_block_type("invalid").is_err());
        assert!(validate_block_type("").is_err());
    }

    #[test]
    fn test_today() {
        let d = today();
        assert_eq!(d.len(), 10);
        assert_eq!(&d[4..5], "-");
        assert_eq!(&d[7..8], "-");
    }
}

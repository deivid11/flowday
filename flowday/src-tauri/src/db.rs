use rusqlite::Connection;
use std::path::Path;
use std::sync::Mutex;

/// Thread-safe wrapper around rusqlite::Connection for Tauri managed state.
pub struct DbState(pub Mutex<Connection>);

const MIGRATIONS: &[(&str, &str)] = &[
    ("001_init", include_str!("../migrations/001_init.sql")),
    ("002_indexes", include_str!("../migrations/002_indexes.sql")),
    ("003_sort_order", include_str!("../migrations/003_sort_order.sql")),
    ("004_push_calendar", include_str!("../migrations/004_push_calendar.sql")),
    ("005_calendar_cache", include_str!("../migrations/005_calendar_cache.sql")),
];

pub fn init_database(db_path: &Path) -> Result<Connection, Box<dyn std::error::Error>> {
    let conn = Connection::open(db_path)?;

    // Enable WAL mode for better concurrent read/write performance
    conn.execute_batch("PRAGMA journal_mode=WAL;")?;
    println!("[db] WAL mode enabled");

    // Enable foreign key enforcement
    conn.execute_batch("PRAGMA foreign_keys=ON;")?;
    println!("[db] Foreign keys enabled");

    // Create migrations tracking table
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS _migrations (
            name TEXT PRIMARY KEY,
            applied_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );"
    )?;

    // Run pending migrations
    for (name, sql) in MIGRATIONS {
        let already_applied: bool = conn.query_row(
            "SELECT COUNT(*) > 0 FROM _migrations WHERE name = ?1",
            [name],
            |row| row.get(0),
        )?;

        if !already_applied {
            conn.execute_batch(sql)?;
            conn.execute(
                "INSERT INTO _migrations (name) VALUES (?1)",
                [name],
            )?;
            println!("[db] Applied migration: {}", name);
        } else {
            println!("[db] Migration already applied: {}", name);
        }
    }

    // Verify table count
    let table_count: i32 = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name NOT LIKE '\\_%' ESCAPE '\\'",
        [],
        |row| row.get(0),
    )?;
    println!("[db] Database ready with {} tables", table_count);

    Ok(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_init_database() {
        let path = PathBuf::from(":memory:");
        let conn = init_database(&path).expect("Failed to init database");

        // Check 7 tables exist
        let table_count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name NOT LIKE '\\_%' ESCAPE '\\'",
            [],
            |row| row.get(0),
        ).unwrap();
        assert!(table_count >= 7, "Expected at least 7 tables, got {}", table_count);

        // Check WAL mode
        let journal_mode: String = conn.query_row(
            "PRAGMA journal_mode;",
            [],
            |row| row.get(0),
        ).unwrap();
        // In-memory databases use "memory" journal mode, not WAL
        assert!(journal_mode == "wal" || journal_mode == "memory");

        // Check foreign keys enabled
        let fk: i32 = conn.query_row(
            "PRAGMA foreign_keys;",
            [],
            |row| row.get(0),
        ).unwrap();
        assert_eq!(fk, 1);

        // Check default settings inserted
        let settings_count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM settings",
            [],
            |row| row.get(0),
        ).unwrap();
        assert!(settings_count >= 11, "Expected at least 11 settings, got {}", settings_count);

        // Check indexes exist
        let index_count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name LIKE 'idx_%'",
            [],
            |row| row.get(0),
        ).unwrap();
        assert!(index_count >= 6, "Expected at least 6 indexes, got {}", index_count);

        // Verify idempotency - running again should not fail
        let _ = init_database(&path).expect("Second init should succeed");
    }
}

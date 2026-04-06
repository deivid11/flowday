-- Cache for Google Calendar events pulled during sync
CREATE TABLE IF NOT EXISTS calendar_events (
  id TEXT PRIMARY KEY,
  google_event_id TEXT NOT NULL UNIQUE,
  summary TEXT NOT NULL,
  start_time TEXT NOT NULL,
  end_time TEXT NOT NULL,
  date TEXT NOT NULL,
  all_day BOOLEAN DEFAULT 0,
  status TEXT DEFAULT 'confirmed',
  synced_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_calendar_events_date ON calendar_events(date);
CREATE INDEX IF NOT EXISTS idx_calendar_events_google_id ON calendar_events(google_event_id);

-- Track last sync timestamp
INSERT OR IGNORE INTO settings (key, value) VALUES ('last_calendar_sync', '');

-- FlowDay Schema v1
-- 7 tables: blocks, templates, interruptions, energy_ratings, daily_stats, settings, google_tokens

CREATE TABLE IF NOT EXISTS blocks (
  id TEXT PRIMARY KEY,
  date TEXT NOT NULL,
  name TEXT NOT NULL,
  type TEXT NOT NULL,
  start_time TEXT NOT NULL,
  duration INTEGER NOT NULL,
  color TEXT NOT NULL,
  notes TEXT,
  pause_time INTEGER DEFAULT 0,
  interruption_count INTEGER DEFAULT 0,
  completed BOOLEAN DEFAULT 0,
  interrupted BOOLEAN DEFAULT 0,
  skipped BOOLEAN DEFAULT 0,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS templates (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL UNIQUE,
  blocks_json TEXT NOT NULL,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  is_builtin BOOLEAN DEFAULT 0
);

CREATE TABLE IF NOT EXISTS interruptions (
  id TEXT PRIMARY KEY,
  block_id TEXT NOT NULL,
  timestamp TIMESTAMP NOT NULL,
  reason TEXT,
  FOREIGN KEY(block_id) REFERENCES blocks(id)
);

CREATE TABLE IF NOT EXISTS energy_ratings (
  id TEXT PRIMARY KEY,
  block_id TEXT NOT NULL UNIQUE,
  rating TEXT NOT NULL,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY(block_id) REFERENCES blocks(id)
);

CREATE TABLE IF NOT EXISTS daily_stats (
  id TEXT PRIMARY KEY,
  date TEXT NOT NULL UNIQUE,
  deep_work_minutes INTEGER DEFAULT 0,
  meeting_minutes INTEGER DEFAULT 0,
  reactive_minutes INTEGER DEFAULT 0,
  total_blocks INTEGER DEFAULT 0,
  completed_blocks INTEGER DEFAULT 0,
  interrupted_blocks INTEGER DEFAULT 0,
  skipped_blocks INTEGER DEFAULT 0,
  total_interruptions INTEGER DEFAULT 0,
  avg_energy TEXT,
  shutdown_note TEXT,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS settings (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL,
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

INSERT OR IGNORE INTO settings (key, value) VALUES ('deep_work_duration', '90');
INSERT OR IGNORE INTO settings (key, value) VALUES ('reactive_duration', '30');
INSERT OR IGNORE INTO settings (key, value) VALUES ('break_duration', '10');
INSERT OR IGNORE INTO settings (key, value) VALUES ('alarm_sound', 'system_default');
INSERT OR IGNORE INTO settings (key, value) VALUES ('five_min_warning', '1');
INSERT OR IGNORE INTO settings (key, value) VALUES ('auto_start_next', '0');
INSERT OR IGNORE INTO settings (key, value) VALUES ('push_to_calendar', '0');
INSERT OR IGNORE INTO settings (key, value) VALUES ('shutdown_time', '17:00');
INSERT OR IGNORE INTO settings (key, value) VALUES ('morning_start_time', '08:00');
INSERT OR IGNORE INTO settings (key, value) VALUES ('lunch_window_start', '13:00');
INSERT OR IGNORE INTO settings (key, value) VALUES ('lunch_window_end', '14:00');

CREATE TABLE IF NOT EXISTS google_tokens (
  id TEXT PRIMARY KEY,
  account_email TEXT NOT NULL UNIQUE,
  access_token TEXT NOT NULL,
  refresh_token TEXT,
  expires_at INTEGER,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

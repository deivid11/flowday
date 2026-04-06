-- Add calendar push tracking to blocks
ALTER TABLE blocks ADD COLUMN pushed_to_calendar BOOLEAN DEFAULT 0;
ALTER TABLE blocks ADD COLUMN calendar_event_id TEXT;

-- Performance indexes for FlowDay

CREATE INDEX IF NOT EXISTS idx_blocks_date ON blocks(date);
CREATE INDEX IF NOT EXISTS idx_blocks_date_type ON blocks(date, type);
CREATE INDEX IF NOT EXISTS idx_interruptions_block_id ON interruptions(block_id);
CREATE INDEX IF NOT EXISTS idx_interruptions_timestamp ON interruptions(timestamp);
CREATE INDEX IF NOT EXISTS idx_daily_stats_date ON daily_stats(date);
CREATE INDEX IF NOT EXISTS idx_energy_block_id ON energy_ratings(block_id);

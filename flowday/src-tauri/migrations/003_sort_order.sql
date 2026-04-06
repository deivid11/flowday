-- Add sort_order column for block reordering
ALTER TABLE blocks ADD COLUMN sort_order INTEGER DEFAULT 0;

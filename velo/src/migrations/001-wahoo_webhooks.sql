CREATE TABLE IF NOT EXISTS wahoo_webhooks (
  data TEXT NOT NULL, -- JSON
  created_at TEXT NOT NULL -- ISO8601 (select datetime('now', 'utc')), UTC
) strict;

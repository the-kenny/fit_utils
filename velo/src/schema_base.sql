CREATE TABLE IF NOT EXISTS migrations (
  sequence INTEGER PRIMARY KEY,
  name TEXT NOT NULL,
  executed TEXT NOT NULL -- ISO8601 (select datetime('now', 'utc')), UTC
) strict;

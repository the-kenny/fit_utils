create table fit_files (
  file BLOB NOT NULL,
  sha256 BLOB NOT NULL,
  source_url TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
) strict;
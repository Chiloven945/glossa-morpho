-- Initial draft schema for glossa-morpho.
-- Replace with your actual SQLite schema once you wire the database.

CREATE TABLE IF NOT EXISTS resource_files (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL,
  logical_path TEXT NOT NULL,
  format TEXT NOT NULL,
  locale TEXT NOT NULL,
  role TEXT NOT NULL,
  raw_path TEXT,
  checksum TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS translation_units (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL,
  file_id TEXT NOT NULL,
  canonical_key TEXT NOT NULL,
  display_key TEXT NOT NULL,
  namespace TEXT,
  key_style TEXT,
  order_index INTEGER DEFAULT 0,
  context TEXT,
  is_deleted INTEGER DEFAULT 0
);

CREATE TABLE IF NOT EXISTS translations (
  id TEXT PRIMARY KEY,
  unit_id TEXT NOT NULL,
  locale TEXT NOT NULL,
  value TEXT NOT NULL,
  status TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

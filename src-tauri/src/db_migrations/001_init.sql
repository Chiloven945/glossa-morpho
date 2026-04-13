PRAGMA
journal_mode = WAL;

CREATE TABLE IF NOT EXISTS project_meta
(
    key
    TEXT
    PRIMARY
    KEY,
    value
    TEXT
    NOT
    NULL
);

CREATE TABLE IF NOT EXISTS locale_graph
(
    code
    TEXT
    PRIMARY
    KEY,
    label
    TEXT
    NOT
    NULL,
    parent_code
    TEXT
);

CREATE TABLE IF NOT EXISTS resource_files
(
    id
    TEXT
    PRIMARY
    KEY,
    sort_order
    INTEGER
    NOT
    NULL
    DEFAULT
    0,
    name
    TEXT
    NOT
    NULL,
    logical_path
    TEXT
    NOT
    NULL,
    format
    TEXT
    NOT
    NULL,
    locale
    TEXT
    NOT
    NULL,
    based_on_locale
    TEXT,
    raw_relative_path
    TEXT
);

CREATE TABLE IF NOT EXISTS entries
(
    id
    TEXT
    PRIMARY
    KEY,
    sort_order
    INTEGER
    NOT
    NULL
    DEFAULT
    0,
    file_id
    TEXT
    NOT
    NULL,
    key_name
    TEXT
    NOT
    NULL,
    source_value
    TEXT
    NOT
    NULL,
    target_value
    TEXT
    NOT
    NULL,
    status
    TEXT
    NOT
    NULL,
    updated_at
    TEXT
    NOT
    NULL,
    file_path
    TEXT
    NOT
    NULL,
    source_locale
    TEXT
    NOT
    NULL,
    target_locale
    TEXT
    NOT
    NULL,
    note
    TEXT
    NOT
    NULL
    DEFAULT
    ''
);

CREATE TABLE IF NOT EXISTS entry_issues
(
    id
    TEXT
    PRIMARY
    KEY,
    entry_id
    TEXT
    NOT
    NULL,
    level
    TEXT
    NOT
    NULL,
    message
    TEXT
    NOT
    NULL
);

CREATE TABLE IF NOT EXISTS entry_candidates
(
    id
    TEXT
    PRIMARY
    KEY,
    entry_id
    TEXT
    NOT
    NULL,
    source
    TEXT
    NOT
    NULL,
    value
    TEXT
    NOT
    NULL,
    score
    REAL
    NOT
    NULL
    DEFAULT
    0
);

CREATE TABLE IF NOT EXISTS entry_history
(
    id
    TEXT
    PRIMARY
    KEY,
    entry_id
    TEXT
    NOT
    NULL,
    action
    TEXT
    NOT
    NULL,
    before_value
    TEXT
    NOT
    NULL,
    after_value
    TEXT
    NOT
    NULL,
    operator
    TEXT
    NOT
    NULL,
    created_at
    TEXT
    NOT
    NULL
);

CREATE INDEX IF NOT EXISTS idx_entries_file_id ON entries(file_id);
CREATE INDEX IF NOT EXISTS idx_entries_file_key ON entries(file_id, key_name);
CREATE INDEX IF NOT EXISTS idx_entry_issues_entry_id ON entry_issues(entry_id);
CREATE INDEX IF NOT EXISTS idx_entry_candidates_entry_id ON entry_candidates(entry_id);
CREATE INDEX IF NOT EXISTS idx_entry_history_entry_id ON entry_history(entry_id);

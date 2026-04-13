use rusqlite::Connection;

pub fn apply_schema(connection: &Connection) -> Result<(), String> {
    connection
        .execute_batch(include_str!("../db_migrations/001_init.sql"))
        .map_err(|error| format!("failed to apply SQLite schema: {error}"))
}

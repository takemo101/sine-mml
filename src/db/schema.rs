use crate::db::DbError;
use rusqlite::Connection;

pub const CURRENT_VERSION: i64 = 1;

/// Initializes the database schema.
///
/// # Errors
///
/// Returns `DbError` if SQL execution fails.
pub fn initialize(conn: &Connection) -> Result<(), DbError> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            mml TEXT NOT NULL,
            waveform TEXT NOT NULL CHECK(waveform IN ('sine', 'sawtooth', 'square')),
            volume REAL NOT NULL CHECK(volume >= 0.0 AND volume <= 1.0),
            bpm INTEGER NOT NULL CHECK(bpm >= 30 AND bpm <= 300),
            created_at TEXT NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_history_created_at ON history(created_at DESC)",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER PRIMARY KEY
        )",
        [],
    )?;

    let count: i64 = conn
        .query_row("SELECT count(*) FROM schema_version", [], |row| row.get(0))
        .unwrap_or(0);

    if count == 0 {
        conn.execute(
            "INSERT INTO schema_version (version) VALUES (?)",
            [CURRENT_VERSION],
        )?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_schema_creates_history_table() {
        let conn = Connection::open_in_memory().unwrap();
        initialize(&conn).unwrap();

        let count: i32 = conn
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='history'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);

        conn.execute("INSERT INTO history (mml, waveform, volume, bpm, created_at) VALUES ('CDE', 'sine', 0.5, 120, '2023-01-01')", []).unwrap();
    }

    #[test]
    fn test_schema_creates_schema_version_table() {
        let conn = Connection::open_in_memory().unwrap();
        initialize(&conn).unwrap();

        let count: i32 = conn
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='schema_version'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);

        let version: i64 = conn
            .query_row("SELECT version FROM schema_version", [], |row| row.get(0))
            .unwrap();
        assert_eq!(version, CURRENT_VERSION);
    }
}

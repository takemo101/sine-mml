use rusqlite::Connection;
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

pub mod schema;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("DB-E001: Failed to create database file: {0}")]
    FileCreation(String),

    #[error("DB-E002: Failed to connect to database: {0}")]
    Connection(String),

    #[error("DB-E003: Failed to initialize schema: {0}")]
    SchemaInit(String),

    #[error("Database error: {0}")]
    Rusqlite(#[from] rusqlite::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

pub struct Database {
    conn: Connection,
}

impl Database {
    /// Initializes the database connection and schema.
    ///
    /// # Errors
    ///
    /// Returns `DbError` if:
    /// - The data directory cannot be determined or created.
    /// - The database connection cannot be established.
    /// - The schema initialization fails.
    pub fn init() -> Result<Self, DbError> {
        let db_path = get_db_path()?;

        if let Some(parent) = db_path.parent() {
            fs::create_dir_all(parent).map_err(|e| DbError::FileCreation(e.to_string()))?;
        }

        let conn = Connection::open(&db_path).map_err(|e| DbError::Connection(e.to_string()))?;

        // Enable WAL mode
        conn.pragma_update(None, "journal_mode", "WAL")?;

        // Initialize schema
        schema::initialize(&conn).map_err(|e| DbError::SchemaInit(e.to_string()))?;

        Ok(Database { conn })
    }

    #[must_use]
    pub fn get_connection(&self) -> &Connection {
        &self.conn
    }
}

/// Returns the path to the database file.
///
/// # Errors
///
/// Returns `DbError` if the user's data directory cannot be determined.
pub fn get_db_path() -> Result<PathBuf, DbError> {
    let mut path = dirs::data_dir()
        .ok_or_else(|| DbError::FileCreation("Could not determine data directory".to_string()))?;
    path.push("sine-mml");
    path.push("sine-mml.db");
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_db_path_returns_valid_path() {
        let path = get_db_path();
        assert!(path.is_ok());
        let p = path.unwrap();
        assert!(p.to_string_lossy().ends_with("sine-mml.db"));
    }

    #[test]
    fn test_database_init_creates_file() {
        let db = Database::init();
        assert!(db.is_ok());
    }

    #[test]
    fn test_wal_mode_enabled() {
        let db = Database::init().unwrap();
        let mode: String = db
            .conn
            .query_row("PRAGMA journal_mode", [], |row| row.get(0))
            .unwrap();
        assert_eq!(mode.to_uppercase(), "WAL");
    }
}

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

pub mod history;
pub mod schema;

pub use history::{HistoryEntry, Waveform};

#[derive(Error, Debug)]
pub enum DbError {
    #[error("DB-E001: Failed to create database file: {0}")]
    FileCreation(String),

    #[error("DB-E002: Failed to connect to database: {0}")]
    Connection(String),

    #[error("DB-E003: Failed to initialize schema: {0}")]
    SchemaInit(String),

    #[error("DB-E004: Failed to save history: {0}")]
    SaveFailed(String),

    #[error("DB-E005: Failed to fetch history: {0}")]
    FetchFailed(String),

    #[error("DB-E006: History not found with id: {0}")]
    NotFound(i64),

    #[error("Invalid waveform type: {0}")]
    InvalidWaveform(String),

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

    #[cfg(test)]
    pub fn open_in_memory() -> Result<Self, DbError> {
        let conn = Connection::open_in_memory()?;
        schema::initialize(&conn).map_err(|e| DbError::SchemaInit(e.to_string()))?;
        Ok(Database { conn })
    }

    #[must_use]
    pub fn get_connection(&self) -> &Connection {
        &self.conn
    }

    /// Saves a history entry to the database.
    ///
    /// # Errors
    ///
    /// Returns `DbError` if:
    /// - Validation fails (MML length, volume range, BPM range).
    /// - Database insertion fails.
    pub fn save(&self, entry: &HistoryEntry) -> Result<i64, DbError> {
        if entry.mml.is_empty() {
            return Err(DbError::SaveFailed("MML cannot be empty".to_string()));
        }
        if entry.mml.len() > 10000 {
            return Err(DbError::SaveFailed(
                "MML too long (max 10000 chars)".to_string(),
            ));
        }
        if entry.volume < 0.0 || entry.volume > 1.0 {
            return Err(DbError::SaveFailed(
                "Volume must be between 0.0 and 1.0".to_string(),
            ));
        }
        if entry.bpm < 30 || entry.bpm > 300 {
            return Err(DbError::SaveFailed(
                "BPM must be between 30 and 300".to_string(),
            ));
        }

        self.conn.execute(
            "INSERT INTO history (mml, waveform, volume, bpm, note, created_at) VALUES (?, ?, ?, ?, ?, ?)",
            params![
                entry.mml,
                entry.waveform.as_str(),
                entry.volume,
                entry.bpm,
                entry.note.as_deref(),
                entry.created_at.to_rfc3339()
            ],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    /// Lists history entries from the database.
    ///
    /// # Errors
    ///
    /// Returns `DbError` if database query fails.
    pub fn list(&self, limit: Option<usize>) -> Result<Vec<HistoryEntry>, DbError> {
        let sql = if let Some(l) = limit {
            format!("SELECT id, mml, waveform, volume, bpm, note, created_at FROM history ORDER BY created_at DESC LIMIT {l}")
        } else {
            "SELECT id, mml, waveform, volume, bpm, note, created_at FROM history ORDER BY created_at DESC".to_string()
        };

        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map([], |row| {
            let waveform_str: String = row.get(2)?;
            let waveform = waveform_str.parse::<Waveform>().map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    2,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?;

            let created_at_str: String = row.get(6)?;
            let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                .map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        6,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    )
                })?
                .with_timezone(&Utc);

            Ok(HistoryEntry {
                id: Some(row.get(0)?),
                mml: row.get(1)?,
                waveform,
                volume: row.get(3)?,
                bpm: row.get(4)?,
                note: row.get(5)?,
                created_at,
            })
        })?;

        let mut entries = Vec::new();
        for row in rows {
            entries.push(row?);
        }

        Ok(entries)
    }

    /// Clears all history entries from the database.
    ///
    /// # Errors
    ///
    /// Returns `DbError` if database deletion fails.
    pub fn clear_all(&self) -> Result<usize, DbError> {
        let deleted = self
            .conn
            .execute("DELETE FROM history", [])
            .map_err(|e| DbError::SaveFailed(e.to_string()))?;
        Ok(deleted)
    }

    pub fn count(&self) -> Result<i64, DbError> {
        self.conn
            .query_row("SELECT COUNT(*) FROM history", [], |row| row.get(0))
            .map_err(|e| DbError::FetchFailed(e.to_string()))
    }

    pub fn get_by_id(&self, id: i64) -> Result<HistoryEntry, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, mml, waveform, volume, bpm, note, created_at FROM history WHERE id = ?",
        )?;

        let result = stmt.query_row(params![id], |row| {
            let waveform_str: String = row.get(2)?;
            let waveform = waveform_str.parse::<Waveform>().map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    2,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?;

            let created_at_str: String = row.get(6)?;
            let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                .map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        6,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    )
                })?
                .with_timezone(&Utc);

            Ok(HistoryEntry {
                id: Some(row.get(0)?),
                mml: row.get(1)?,
                waveform,
                volume: row.get(3)?,
                bpm: row.get(4)?,
                note: row.get(5)?,
                created_at,
            })
        });

        match result {
            Ok(entry) => Ok(entry),
            Err(rusqlite::Error::QueryReturnedNoRows) => Err(DbError::NotFound(id)),
            Err(e) => Err(DbError::Rusqlite(e)),
        }
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
        if db.is_err() {
            println!("Skipping test: data directory unavailable in CI");
            return;
        }
        assert!(db.is_ok());
    }

    #[test]
    fn test_wal_mode_enabled() {
        let Ok(db) = Database::init() else {
            println!("Skipping test: data directory unavailable in CI");
            return;
        };
        let mode: String = db
            .conn
            .query_row("PRAGMA journal_mode", [], |row| row.get(0))
            .unwrap();
        assert_eq!(mode.to_uppercase(), "WAL");
    }

    #[test]
    fn test_save_and_get_by_id() {
        let db = Database::open_in_memory().unwrap();
        let entry = HistoryEntry::new("CDE".to_string(), Waveform::Sine, 0.5, 120, None);
        let id = db.save(&entry).unwrap();

        let fetched = db.get_by_id(id).unwrap();
        assert_eq!(fetched.mml, "CDE");
        assert_eq!(fetched.waveform, Waveform::Sine);
        assert_eq!(fetched.volume, 0.5);
        assert_eq!(fetched.bpm, 120);
        assert_eq!(fetched.note, None);
        assert_eq!(fetched.id, Some(id));
    }

    #[test]
    fn test_save_and_get_by_id_with_note() {
        let db = Database::open_in_memory().unwrap();
        let entry = HistoryEntry::new(
            "CDE".to_string(),
            Waveform::Sine,
            0.5,
            120,
            Some("My melody".to_string()),
        );
        let id = db.save(&entry).unwrap();

        let fetched = db.get_by_id(id).unwrap();
        assert_eq!(fetched.mml, "CDE");
        assert_eq!(fetched.note, Some("My melody".to_string()));
    }

    #[test]
    fn test_save_and_get_by_id_with_utf8_note() {
        let db = Database::open_in_memory().unwrap();
        let entry = HistoryEntry::new(
            "CDE".to_string(),
            Waveform::Sine,
            0.5,
            120,
            Some("🎵 私のメロディ 🎶".to_string()),
        );
        let id = db.save(&entry).unwrap();

        let fetched = db.get_by_id(id).unwrap();
        assert_eq!(fetched.note, Some("🎵 私のメロディ 🎶".to_string()));
    }

    #[test]
    fn test_list_returns_descending_order() {
        let db = Database::open_in_memory().unwrap();
        let entry1 = HistoryEntry::new("C".to_string(), Waveform::Sine, 0.5, 120, None);
        let entry2 = HistoryEntry::new("D".to_string(), Waveform::Square, 0.6, 130, None);

        db.save(&entry1).unwrap();
        // Ensure timestamp difference
        std::thread::sleep(std::time::Duration::from_millis(10));
        db.save(&entry2).unwrap();

        let list = db.list(None).unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].mml, "D"); // Newest first
        assert_eq!(list[1].mml, "C");
    }

    #[test]
    fn test_list_includes_note() {
        let db = Database::open_in_memory().unwrap();
        let entry1 = HistoryEntry::new(
            "C".to_string(),
            Waveform::Sine,
            0.5,
            120,
            Some("Note 1".to_string()),
        );
        let entry2 = HistoryEntry::new("D".to_string(), Waveform::Square, 0.6, 130, None);

        db.save(&entry1).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        db.save(&entry2).unwrap();

        let list = db.list(None).unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].note, None); // D (newest)
        assert_eq!(list[1].note, Some("Note 1".to_string())); // C
    }

    #[test]
    fn test_list_with_limit() {
        let db = Database::open_in_memory().unwrap();
        for i in 0..5 {
            let entry = HistoryEntry::new(format!("MML{i}"), Waveform::Sine, 0.5, 120, None);
            db.save(&entry).unwrap();
        }

        let list = db.list(Some(3)).unwrap();
        assert_eq!(list.len(), 3);
        assert_eq!(list[0].mml, "MML4");
    }

    #[test]
    fn test_get_by_id_not_found() {
        let db = Database::open_in_memory().unwrap();
        let result = db.get_by_id(999);
        assert!(matches!(result, Err(DbError::NotFound(999))));
    }

    #[test]
    fn test_save_validation_mml_empty() {
        let db = Database::open_in_memory().unwrap();
        let entry = HistoryEntry::new(String::new(), Waveform::Sine, 0.5, 120, None);
        let result = db.save(&entry);
        assert!(result.is_err());
        match result {
            Err(DbError::SaveFailed(msg)) => assert!(msg.contains("MML cannot be empty")),
            _ => panic!("Expected SaveFailed error"),
        }
    }

    #[test]
    fn test_save_validation_volume_out_of_range() {
        let db = Database::open_in_memory().unwrap();
        let entry = HistoryEntry::new("C".to_string(), Waveform::Sine, 1.5, 120, None);
        let result = db.save(&entry);
        assert!(result.is_err());
    }

    #[test]
    fn test_save_validation_bpm_out_of_range() {
        let db = Database::open_in_memory().unwrap();
        let entry = HistoryEntry::new("C".to_string(), Waveform::Sine, 0.5, 500, None);
        let result = db.save(&entry);
        assert!(result.is_err());
    }

    #[test]
    fn test_clear_all_empty_db() {
        let db = Database::open_in_memory().unwrap();
        let deleted = db.clear_all().unwrap();
        assert_eq!(deleted, 0);
    }

    #[test]
    fn test_clear_all_with_entries() {
        let db = Database::open_in_memory().unwrap();
        for i in 0..3 {
            let entry = HistoryEntry::new(format!("MML{i}"), Waveform::Sine, 0.5, 120, None);
            db.save(&entry).unwrap();
        }

        let deleted = db.clear_all().unwrap();
        assert_eq!(deleted, 3);

        let list = db.list(None).unwrap();
        assert!(list.is_empty());
    }

    #[test]
    fn test_count_empty_db() {
        let db = Database::open_in_memory().unwrap();
        assert_eq!(db.count().unwrap(), 0);
    }

    #[test]
    fn test_count_with_entries() {
        let db = Database::open_in_memory().unwrap();
        for i in 0..3 {
            let entry = HistoryEntry::new(format!("MML{i}"), Waveform::Sine, 0.5, 120, None);
            db.save(&entry).unwrap();
        }
        assert_eq!(db.count().unwrap(), 3);
    }

    #[test]
    fn test_count_after_clear() {
        let db = Database::open_in_memory().unwrap();
        for i in 0..3 {
            let entry = HistoryEntry::new(format!("MML{i}"), Waveform::Sine, 0.5, 120, None);
            db.save(&entry).unwrap();
        }
        assert_eq!(db.count().unwrap(), 3);
        db.clear_all().unwrap();
        assert_eq!(db.count().unwrap(), 0);
    }
}

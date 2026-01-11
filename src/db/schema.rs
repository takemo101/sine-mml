use crate::db::DbError;
use rusqlite::Connection;

pub const CURRENT_VERSION: i64 = 2;

/// Initializes the database schema.
///
/// # Errors
///
/// Returns `DbError` if SQL execution fails.
pub fn initialize(conn: &Connection) -> Result<(), DbError> {
    // schema_version テーブル作成（先に作成）
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER PRIMARY KEY
        )",
        [],
    )?;

    // 新規DBかどうかを判定（history テーブルが存在しない）
    let is_new_db = conn
        .query_row(
            "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='history'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or(0)
        == 0;

    if is_new_db {
        // 新規DB: v2 スキーマで作成し、即座にバージョンを2に設定
        conn.execute(
            "CREATE TABLE IF NOT EXISTS history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                mml TEXT NOT NULL,
                waveform TEXT NOT NULL CHECK(waveform IN ('sine', 'sawtooth', 'square')),
                volume REAL NOT NULL CHECK(volume >= 0.0 AND volume <= 1.0),
                bpm INTEGER NOT NULL CHECK(bpm >= 30 AND bpm <= 300),
                note TEXT NULL CHECK(note IS NULL OR length(note) <= 500),
                created_at TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_history_created_at ON history(created_at DESC)",
            [],
        )?;

        // 新規DBはバージョン2を強制設定（不整合状態からの復旧にも対応）
        conn.execute("DELETE FROM schema_version", [])?;
        conn.execute(
            "INSERT INTO schema_version (version) VALUES (?)",
            [CURRENT_VERSION],
        )?;
    } else {
        // 既存DB: マイグレーション実行
        migrate(conn)?;
    }

    Ok(())
}

/// Gets the current schema version.
///
/// # Returns
///
/// - `Ok(version)` - Current version (1 if `schema_version` table is empty or doesn't exist)
/// - `Err(DbError)` - Serious database error (I/O error, lock error, etc.)
fn get_current_version(conn: &Connection) -> Result<i64, DbError> {
    let version = conn.query_row("SELECT version FROM schema_version", [], |row| {
        row.get::<_, i64>(0)
    });

    match version {
        Ok(v) => Ok(v),
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            // テーブルは存在するが空 = v1相当（初期化処理が不完全）
            Ok(1)
        }
        Err(e) => {
            // エラーメッセージで "no such table" を確認
            let err_msg = e.to_string();
            if err_msg.contains("no such table") {
                // テーブルが存在しない = v1（マイグレーション前の古いDB）
                Ok(1)
            } else {
                // その他のエラー（I/O、ロック等）は上位に伝播
                Err(DbError::from(e))
            }
        }
    }
}

/// Checks if a column exists in a table.
fn column_exists(conn: &Connection, table: &str, column: &str) -> bool {
    let count: i32 = conn
        .query_row(
            &format!("SELECT count(*) FROM pragma_table_info('{table}') WHERE name='{column}'"),
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    count > 0
}

/// Migrates the database schema to the current version.
///
/// # Errors
///
/// Returns `DbError` if migration fails.
pub fn migrate(conn: &Connection) -> Result<(), DbError> {
    let version = get_current_version(conn)?;

    if version < 2 {
        // v1 → v2 マイグレーション
        let tx = conn.unchecked_transaction()?;

        // note カラムが既に存在するかチェック（冪等性のため）
        if !column_exists(&tx, "history", "note") {
            tx.execute(
                "ALTER TABLE history ADD COLUMN note TEXT NULL CHECK(note IS NULL OR length(note) <= 500)",
                [],
            )?;
        }

        // バージョン更新
        let row_count: i64 = tx
            .query_row("SELECT count(*) FROM schema_version", [], |row| row.get(0))
            .unwrap_or(0);

        if row_count > 0 {
            tx.execute("UPDATE schema_version SET version = ?", [CURRENT_VERSION])?;
        } else {
            tx.execute(
                "INSERT INTO schema_version (version) VALUES (?)",
                [CURRENT_VERSION],
            )?;
        }

        tx.commit()?;
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

        conn.execute(
            "INSERT INTO history (mml, waveform, volume, bpm, created_at) VALUES ('CDE', 'sine', 0.5, 120, '2023-01-01')",
            [],
        )
        .unwrap();
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

    #[test]
    fn test_migrate_v1_to_v2() {
        let conn = Connection::open_in_memory().unwrap();

        // v1 スキーマで DB 作成
        conn.execute(
            "CREATE TABLE history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                mml TEXT NOT NULL,
                waveform TEXT NOT NULL,
                volume REAL NOT NULL,
                bpm INTEGER NOT NULL,
                created_at TEXT NOT NULL
            )",
            [],
        )
        .unwrap();

        // 既存データ挿入
        conn.execute(
            "INSERT INTO history (mml, waveform, volume, bpm, created_at) 
             VALUES ('CDE', 'sine', 0.5, 120, '2023-01-01')",
            [],
        )
        .unwrap();

        // schema_version テーブル作成
        conn.execute(
            "CREATE TABLE schema_version (version INTEGER PRIMARY KEY)",
            [],
        )
        .unwrap();
        conn.execute("INSERT INTO schema_version (version) VALUES (1)", [])
            .unwrap();

        // マイグレーション実行
        migrate(&conn).unwrap();

        // note カラムが追加されたことを確認
        let count: i32 = conn
            .query_row(
                "SELECT count(*) FROM pragma_table_info('history') WHERE name='note'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);

        // 既存データが保持されていることを確認
        let mml: String = conn
            .query_row("SELECT mml FROM history WHERE id=1", [], |row| row.get(0))
            .unwrap();
        assert_eq!(mml, "CDE");

        // バージョンが更新されたことを確認
        let version: i64 = conn
            .query_row("SELECT version FROM schema_version", [], |row| row.get(0))
            .unwrap();
        assert_eq!(version, 2);
    }

    #[test]
    fn test_new_db_no_migration() {
        let conn = Connection::open_in_memory().unwrap();

        // 新規DBで初期化（history テーブルがない状態）
        initialize(&conn).unwrap();

        // note カラムが存在することを確認（v2スキーマで作成された）
        let count: i32 = conn
            .query_row(
                "SELECT count(*) FROM pragma_table_info('history') WHERE name='note'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);

        // バージョンが2であることを確認
        let version: i64 = conn
            .query_row("SELECT version FROM schema_version", [], |row| row.get(0))
            .unwrap();
        assert_eq!(version, 2);
    }

    #[test]
    fn test_migrate_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        initialize(&conn).unwrap();

        // 2回目の初期化（冪等性確認）
        let result = initialize(&conn);
        assert!(result.is_ok());

        // 3回目のマイグレーション呼び出し
        let result = migrate(&conn);
        assert!(result.is_ok());

        // バージョンが変わらないことを確認
        let version: i64 = conn
            .query_row("SELECT version FROM schema_version", [], |row| row.get(0))
            .unwrap();
        assert_eq!(version, 2);
    }

    #[test]
    fn test_note_length_constraint() {
        let conn = Connection::open_in_memory().unwrap();
        initialize(&conn).unwrap();

        // 500文字以下: 成功
        let result = conn.execute(
            "INSERT INTO history (mml, waveform, volume, bpm, note, created_at) 
             VALUES ('CDE', 'sine', 0.5, 120, ?, '2023-01-01')",
            [&"a".repeat(500)],
        );
        assert!(result.is_ok());

        // 501文字: エラー
        let result = conn.execute(
            "INSERT INTO history (mml, waveform, volume, bpm, note, created_at) 
             VALUES ('CDE', 'sine', 0.5, 120, ?, '2023-01-01')",
            [&"a".repeat(501)],
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_note_null_allowed() {
        let conn = Connection::open_in_memory().unwrap();
        initialize(&conn).unwrap();

        // NULL: 成功
        let result = conn.execute(
            "INSERT INTO history (mml, waveform, volume, bpm, note, created_at) 
             VALUES ('CDE', 'sine', 0.5, 120, NULL, '2023-01-01')",
            [],
        );
        assert!(result.is_ok());

        // NULLの取得確認
        let note: Option<String> = conn
            .query_row("SELECT note FROM history WHERE id=1", [], |row| row.get(0))
            .unwrap();
        assert_eq!(note, None);
    }
}

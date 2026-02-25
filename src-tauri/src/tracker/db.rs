use rusqlite::{Connection, Result};
use std::path::Path;

/// Open a SQLite connection with WAL mode, busy timeout, and foreign keys enabled.
pub fn open_connection(db_path: &Path) -> Result<Connection> {
    let conn = Connection::open(db_path)?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "busy_timeout", 5000)?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    Ok(conn)
}

/// Ensure the schema is up-to-date, running migrations as needed.
pub fn ensure_schema(conn: &Connection) -> Result<()> {
    // Create the schema_version table if it doesn't exist
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER NOT NULL,
            applied_at TEXT NOT NULL DEFAULT (datetime('now'))
        );"
    )?;

    let current_version: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_version",
            [],
            |row| row.get(0),
        )?;

    if current_version < 1 {
        migrate_v1(conn)?;
    }

    Ok(())
}

/// Schema version 1: core tables for issues, comments, labels, dependencies, and FTS5 search.
fn migrate_v1(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "BEGIN;

        CREATE TABLE IF NOT EXISTS issues (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            body TEXT NOT NULL DEFAULT '',
            status TEXT NOT NULL DEFAULT 'open',
            priority TEXT NOT NULL DEFAULT 'medium',
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now')),
            closed_at TEXT,
            author TEXT NOT NULL DEFAULT ''
        );

        CREATE TABLE IF NOT EXISTS comments (
            id TEXT PRIMARY KEY,
            issue_id TEXT NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
            body TEXT NOT NULL,
            author TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE INDEX IF NOT EXISTS idx_comments_issue_id ON comments(issue_id);

        CREATE TABLE IF NOT EXISTS labels (
            issue_id TEXT NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
            label TEXT NOT NULL,
            PRIMARY KEY (issue_id, label)
        );
        CREATE INDEX IF NOT EXISTS idx_labels_label ON labels(label);

        CREATE TABLE IF NOT EXISTS dependencies (
            from_id TEXT NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
            to_id TEXT NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
            dep_type TEXT NOT NULL DEFAULT 'blocks',
            PRIMARY KEY (from_id, to_id)
        );

        CREATE VIRTUAL TABLE IF NOT EXISTS issues_fts USING fts5(
            title,
            body,
            content='issues',
            content_rowid='rowid'
        );

        INSERT INTO schema_version (version) VALUES (1);

        COMMIT;"
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_open_connection_in_memory() {
        // Use an in-memory database for testing
        let conn = Connection::open_in_memory().unwrap();
        conn.pragma_update(None, "journal_mode", "WAL").unwrap();
        conn.pragma_update(None, "busy_timeout", 5000).unwrap();
        conn.pragma_update(None, "foreign_keys", "ON").unwrap();

        let fk: i64 = conn
            .pragma_query_value(None, "foreign_keys", |row| row.get(0))
            .unwrap();
        assert_eq!(fk, 1);
    }

    #[test]
    fn test_ensure_schema_creates_tables() {
        let conn = Connection::open_in_memory().unwrap();
        ensure_schema(&conn).unwrap();

        // Verify schema_version
        let version: i64 = conn
            .query_row("SELECT MAX(version) FROM schema_version", [], |row| row.get(0))
            .unwrap();
        assert_eq!(version, 1);

        // Verify issues table exists
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM issues", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);

        // Verify comments table exists
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM comments", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_ensure_schema_is_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        ensure_schema(&conn).unwrap();
        ensure_schema(&conn).unwrap(); // Should not error on second call

        let version: i64 = conn
            .query_row("SELECT MAX(version) FROM schema_version", [], |row| row.get(0))
            .unwrap();
        assert_eq!(version, 1);
    }
}

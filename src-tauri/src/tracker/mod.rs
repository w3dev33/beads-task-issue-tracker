mod config;
mod db;
mod ids;

pub use ids::generate_id;

pub use config::ProjectConfig;

use rusqlite::Connection;
use std::fs;
use std::path::{Path, PathBuf};

const GITIGNORE_CONTENT: &str = "\
# SQLite database (local cache, rebuilt from JSONL)
*.db
*.db-wal
*.db-shm
*.db-journal
";

/// The built-in tracker engine. Manages a SQLite database inside a project's
/// `.tracker/` directory.
pub struct Engine {
    #[allow(dead_code)]
    conn: Connection,
    #[allow(dead_code)]
    config: ProjectConfig,
}

impl Engine {
    /// Open an existing tracker database at `project_path/.tracker/tracker.db`.
    /// Runs schema migrations if needed.
    pub fn open(project_path: &Path) -> rusqlite::Result<Self> {
        let config = ProjectConfig::load(project_path);
        let db_path = Self::db_path(project_path, &config);

        let conn = db::open_connection(&db_path)?;
        db::ensure_schema(&conn)?;

        Ok(Self { conn, config })
    }

    /// Initialize a new tracker in `project_path/.tracker/`.
    /// Creates the directory, `.gitignore`, and an empty database with schema.
    pub fn init(project_path: &Path, config: ProjectConfig) -> rusqlite::Result<Self> {
        let tracker_dir = project_path.join(&config.folder_name);
        fs::create_dir_all(&tracker_dir).map_err(|e| {
            rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_CANTOPEN),
                Some(format!("Failed to create tracker directory: {}", e)),
            )
        })?;

        // Write .gitignore
        let gitignore_path = tracker_dir.join(".gitignore");
        if !gitignore_path.exists() {
            let _ = fs::write(&gitignore_path, GITIGNORE_CONTENT);
        }

        let db_path = tracker_dir.join("tracker.db");
        let conn = db::open_connection(&db_path)?;
        db::ensure_schema(&conn)?;

        Ok(Self { conn, config })
    }

    fn db_path(project_path: &Path, config: &ProjectConfig) -> PathBuf {
        project_path.join(&config.folder_name).join("tracker.db")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_init_creates_directory_and_db() {
        let tmp = tempfile::tempdir().unwrap();
        let project_path = tmp.path();

        let config = ProjectConfig {
            folder_name: ".tracker".to_string(),
            issue_prefix: "test".to_string(),
            actor: "tester".to_string(),
        };

        let _engine = Engine::init(project_path, config).unwrap();

        // Verify .tracker directory was created
        assert!(project_path.join(".tracker").is_dir());

        // Verify .gitignore was created with expected content
        let gitignore = fs::read_to_string(project_path.join(".tracker/.gitignore")).unwrap();
        assert!(gitignore.contains("*.db"));
        assert!(gitignore.contains("*.db-wal"));

        // Verify database file was created
        assert!(project_path.join(".tracker/tracker.db").exists());
    }

    #[test]
    fn test_open_existing_tracker() {
        let tmp = tempfile::tempdir().unwrap();
        let project_path = tmp.path();

        let config = ProjectConfig {
            folder_name: ".tracker".to_string(),
            issue_prefix: "test".to_string(),
            actor: "tester".to_string(),
        };

        // Init first
        drop(Engine::init(project_path, config).unwrap());

        // Then open
        let _engine = Engine::open(project_path).unwrap();
    }
}

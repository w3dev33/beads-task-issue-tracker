mod config;
pub mod convert;
mod db;
mod export;
mod ids;
mod issues;
mod search;

pub use ids::generate_id;

pub use config::ProjectConfig;
pub use issues::{
    CreateIssueParams, TrackerComment, TrackerIssue, TrackerRelation, UpdateIssueParams,
};
pub use search::SearchResult;

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

const AGENTS_MD_CONTENT: &str = include_str!("agents_template.md");

/// The built-in tracker engine. Manages a SQLite database inside a project's
/// `.tracker/` directory.
pub struct Engine {
    conn: Connection,
    config: ProjectConfig,
    project_path: PathBuf,
}

impl Engine {
    /// Open an existing tracker database at `project_path/.tracker/tracker.db`.
    /// Runs schema migrations if needed.
    pub fn open(project_path: &Path) -> rusqlite::Result<Self> {
        let config = ProjectConfig::load(project_path);
        let db_path = Self::db_path(project_path, &config);

        let conn = db::open_connection(&db_path)?;
        db::ensure_schema(&conn)?;

        Ok(Self {
            conn,
            config,
            project_path: project_path.to_path_buf(),
        })
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

        // Write AGENTS.md (CLI reference for AI agents)
        let agents_path = tracker_dir.join("AGENTS.md");
        if !agents_path.exists() {
            let _ = fs::write(&agents_path, AGENTS_MD_CONTENT);
        }

        let db_path = tracker_dir.join("tracker.db");
        let conn = db::open_connection(&db_path)?;
        db::ensure_schema(&conn)?;

        Ok(Self {
            conn,
            config,
            project_path: project_path.to_path_buf(),
        })
    }

    /// Best-effort JSONL export after every write mutation.
    fn export_jsonl(&self) {
        if let Err(e) = export::export_all(&self.conn, &self.config, &self.project_path) {
            log::warn!("[tracker] JSONL export failed: {}", e);
        }
    }

    /// List issues, optionally filtered by status ("open", "closed", "all").
    pub fn list_issues(&self, status_filter: Option<&str>) -> rusqlite::Result<Vec<TrackerIssue>> {
        issues::list_issues(&self.conn, status_filter)
    }

    /// Get a single issue by ID with full details (comments, labels, deps).
    pub fn get_issue(&self, id: &str) -> rusqlite::Result<TrackerIssue> {
        issues::get_issue(&self.conn, id)
    }

    /// Create a new issue. ID is auto-generated from the config prefix.
    pub fn create_issue(&self, params: CreateIssueParams) -> rusqlite::Result<TrackerIssue> {
        let result = issues::create_issue(&self.conn, &self.config.issue_prefix, params)?;
        self.export_jsonl();
        Ok(result)
    }

    /// Update an existing issue. Only provided fields are modified.
    pub fn update_issue(&self, id: &str, params: UpdateIssueParams) -> rusqlite::Result<TrackerIssue> {
        let result = issues::update_issue(&self.conn, id, params)?;
        self.export_jsonl();
        Ok(result)
    }

    /// Close an issue (set status=closed, closed_at=now).
    pub fn close_issue(&self, id: &str) -> rusqlite::Result<TrackerIssue> {
        let result = issues::close_issue(&self.conn, id)?;
        self.export_jsonl();
        Ok(result)
    }

    /// Delete an issue. Hard delete removes the row; soft delete sets status=tombstone.
    pub fn delete_issue(&self, id: &str, hard: bool) -> rusqlite::Result<()> {
        issues::delete_issue(&self.conn, id, hard)?;
        self.export_jsonl();
        Ok(())
    }

    /// Add a comment to an issue.
    pub fn add_comment(
        &self,
        issue_id: &str,
        author: &str,
        body: &str,
    ) -> rusqlite::Result<TrackerComment> {
        let result = issues::add_comment(&self.conn, issue_id, author, body)?;
        self.export_jsonl();
        Ok(result)
    }

    /// Delete a comment by ID.
    pub fn delete_comment(&self, comment_id: &str) -> rusqlite::Result<()> {
        issues::delete_comment(&self.conn, comment_id)?;
        self.export_jsonl();
        Ok(())
    }

    /// Add a label to an issue (idempotent).
    pub fn add_label(&self, issue_id: &str, label: &str) -> rusqlite::Result<()> {
        issues::add_label(&self.conn, issue_id, label)?;
        self.export_jsonl();
        Ok(())
    }

    /// Remove a label from an issue.
    pub fn remove_label(&self, issue_id: &str, label: &str) -> rusqlite::Result<()> {
        issues::remove_label(&self.conn, issue_id, label)?;
        self.export_jsonl();
        Ok(())
    }

    /// Add a dependency/relation between two issues.
    pub fn add_dependency(
        &self,
        from_id: &str,
        to_id: &str,
        dep_type: &str,
    ) -> rusqlite::Result<()> {
        issues::add_dependency(&self.conn, from_id, to_id, dep_type)?;
        self.export_jsonl();
        Ok(())
    }

    /// Remove a dependency between two issues.
    pub fn remove_dependency(&self, from_id: &str, to_id: &str) -> rusqlite::Result<()> {
        issues::remove_dependency(&self.conn, from_id, to_id)?;
        self.export_jsonl();
        Ok(())
    }

    /// Full-text search across issue titles, bodies, and notes.
    pub fn search(&self, query: &str, limit: Option<usize>) -> rusqlite::Result<Vec<SearchResult>> {
        search::search(&self.conn, query, limit.unwrap_or(50))
    }

    /// List "ready" issues: open/in_progress issues not blocked by any open issue.
    pub fn list_ready_issues(&self) -> rusqlite::Result<Vec<TrackerIssue>> {
        issues::list_ready_issues(&self.conn)
    }

    /// List child issues of a parent issue.
    pub fn list_children(&self, parent_id: &str) -> rusqlite::Result<Vec<TrackerIssue>> {
        issues::list_children(&self.conn, parent_id)
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

        // Verify AGENTS.md was created
        let agents = fs::read_to_string(project_path.join(".tracker/AGENTS.md")).unwrap();
        assert!(agents.contains("beads-tracker"));

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

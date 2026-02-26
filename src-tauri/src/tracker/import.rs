use rusqlite::{params, Connection, Result};
use std::io::{BufRead, BufReader};
use std::path::Path;

use super::export::{int_to_priority, JsonlComment, JsonlDependency, JsonlIssue};
use super::ids::generate_id;
use super::issues::{fts_delete, fts_insert};

/// Result of a JSONL import operation.
#[derive(Debug, Default)]
pub struct ImportResult {
    pub inserted: usize,
    pub updated: usize,
    pub skipped: usize,
    pub errors: usize,
}

/// Import all issues from a JSONL file into the database.
///
/// Strategy: last-write-wins by `updated_at` per issue.
/// - If issue doesn't exist → INSERT
/// - If issue exists and JSONL is newer → UPDATE
/// - If issue exists and DB is same/newer → merge comments only (append-only)
///
/// Comments use append-only merge (by issue_id, author, created_at composite key).
/// Labels and dependencies are fully replaced from JSONL when the issue is inserted/updated.
/// Malformed lines are logged and skipped.
pub fn import_all(conn: &Connection, jsonl_path: &Path) -> Result<ImportResult> {
    let file = std::fs::File::open(jsonl_path).map_err(|e| {
        rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_CANTOPEN),
            Some(format!("Failed to open JSONL file: {}", e)),
        )
    })?;

    let reader = BufReader::new(file);
    let mut result = ImportResult::default();

    let tx = conn.unchecked_transaction()?;
    tx.execute_batch("PRAGMA defer_foreign_keys = ON")?;

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => {
                result.errors += 1;
                continue;
            }
        };

        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let issue: JsonlIssue = match serde_json::from_str(line) {
            Ok(i) => i,
            Err(e) => {
                log::warn!("[tracker/import] Malformed line, skipping: {}", e);
                result.errors += 1;
                continue;
            }
        };

        match import_one_issue(&tx, &issue)? {
            ImportAction::Inserted => result.inserted += 1,
            ImportAction::Updated => result.updated += 1,
            ImportAction::Skipped => result.skipped += 1,
        }
    }

    tx.commit()?;
    Ok(result)
}

enum ImportAction {
    Inserted,
    Updated,
    Skipped,
}

fn import_one_issue(conn: &Connection, issue: &JsonlIssue) -> Result<ImportAction> {
    let existing: Option<String> = conn
        .query_row(
            "SELECT updated_at FROM issues WHERE id = ?1",
            params![issue.id],
            |row| row.get(0),
        )
        .ok();

    match existing {
        None => {
            // Issue doesn't exist → INSERT
            upsert_issue(conn, issue)?;
            replace_labels(conn, &issue.id, &issue.labels)?;
            replace_dependencies(conn, &issue.id, &issue.dependencies)?;
            merge_comments(conn, &issue.id, &issue.comments)?;
            fts_insert(conn, &issue.id)?;
            Ok(ImportAction::Inserted)
        }
        Some(db_updated_at) => {
            if issue.updated_at > db_updated_at {
                // JSONL is newer → UPDATE
                update_issue_from_jsonl(conn, issue)?;
                replace_labels(conn, &issue.id, &issue.labels)?;
                replace_dependencies(conn, &issue.id, &issue.dependencies)?;
                merge_comments(conn, &issue.id, &issue.comments)?;
                // Re-sync FTS
                fts_delete(conn, &issue.id)?;
                fts_insert(conn, &issue.id)?;
                Ok(ImportAction::Updated)
            } else {
                // DB is same/newer → merge comments only
                merge_comments(conn, &issue.id, &issue.comments)?;
                Ok(ImportAction::Skipped)
            }
        }
    }
}

fn upsert_issue(conn: &Connection, issue: &JsonlIssue) -> Result<()> {
    let priority = int_to_priority(issue.priority);
    let assignee = issue.owner.as_deref();
    let author = if issue.created_by.is_empty() {
        "unknown"
    } else {
        &issue.created_by
    };

    conn.execute(
        "INSERT INTO issues (
            id, title, body, issue_type, status, priority,
            assignee, author, created_at, updated_at, closed_at,
            external_ref, estimate_minutes, design, acceptance_criteria,
            notes, parent, metadata, spec_id
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6,
            ?7, ?8, ?9, ?10, ?11,
            ?12, ?13, ?14, ?15,
            ?16, ?17, ?18, ?19
        )",
        params![
            issue.id,
            issue.title,
            issue.description,
            issue.issue_type,
            issue.status,
            priority,
            assignee,
            author,
            issue.created_at,
            issue.updated_at,
            issue.closed_at,
            issue.external_ref,
            issue.estimate_minutes,
            issue.design,
            issue.acceptance_criteria,
            issue.notes,
            issue.parent,
            issue.metadata,
            issue.spec_id,
        ],
    )?;
    Ok(())
}

fn update_issue_from_jsonl(conn: &Connection, issue: &JsonlIssue) -> Result<()> {
    let priority = int_to_priority(issue.priority);
    let assignee = issue.owner.as_deref();
    let author = if issue.created_by.is_empty() {
        "unknown"
    } else {
        &issue.created_by
    };

    conn.execute(
        "UPDATE issues SET
            title = ?2, body = ?3, issue_type = ?4, status = ?5, priority = ?6,
            assignee = ?7, author = ?8, created_at = ?9, updated_at = ?10, closed_at = ?11,
            external_ref = ?12, estimate_minutes = ?13, design = ?14, acceptance_criteria = ?15,
            notes = ?16, parent = ?17, metadata = ?18, spec_id = ?19
        WHERE id = ?1",
        params![
            issue.id,
            issue.title,
            issue.description,
            issue.issue_type,
            issue.status,
            priority,
            assignee,
            author,
            issue.created_at,
            issue.updated_at,
            issue.closed_at,
            issue.external_ref,
            issue.estimate_minutes,
            issue.design,
            issue.acceptance_criteria,
            issue.notes,
            issue.parent,
            issue.metadata,
            issue.spec_id,
        ],
    )?;
    Ok(())
}

/// Append-only merge: insert comments that don't already exist (by issue_id, author, created_at).
fn merge_comments(conn: &Connection, issue_id: &str, comments: &[JsonlComment]) -> Result<()> {
    for comment in comments {
        let exists: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM comments WHERE issue_id = ?1 AND author = ?2 AND created_at = ?3)",
            params![issue_id, comment.author, comment.created_at],
            |row| row.get(0),
        )?;

        if !exists {
            // Generate a new comment ID if the JSONL one is empty
            let comment_id = if comment.id.is_empty() {
                generate_id(conn, "c").unwrap_or_else(|_| format!("c-{}", comment.created_at))
            } else {
                comment.id.clone()
            };

            conn.execute(
                "INSERT OR IGNORE INTO comments (id, issue_id, author, body, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![comment_id, issue_id, comment.author, comment.text, comment.created_at],
            )?;
        }
    }
    Ok(())
}

/// Full replace: delete all labels for the issue, then insert from JSONL.
fn replace_labels(conn: &Connection, issue_id: &str, labels: &[String]) -> Result<()> {
    conn.execute("DELETE FROM labels WHERE issue_id = ?1", params![issue_id])?;
    for label in labels {
        conn.execute(
            "INSERT INTO labels (issue_id, label) VALUES (?1, ?2)",
            params![issue_id, label],
        )?;
    }
    Ok(())
}

/// Full replace: delete all dependencies where from_id = issue_id, then insert from JSONL.
fn replace_dependencies(
    conn: &Connection,
    issue_id: &str,
    deps: &[JsonlDependency],
) -> Result<()> {
    conn.execute(
        "DELETE FROM dependencies WHERE from_id = ?1",
        params![issue_id],
    )?;
    for dep in deps {
        conn.execute(
            "INSERT INTO dependencies (from_id, to_id, dep_type) VALUES (?1, ?2, ?3)",
            params![dep.issue_id, dep.depends_on_id, dep.dep_type],
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tracker::config::ProjectConfig;
    use crate::tracker::db;
    use crate::tracker::export::export_all;
    use crate::tracker::issues::{self, CreateIssueParams};
    use std::fs;

    fn setup() -> (tempfile::TempDir, Connection) {
        let tmp = tempfile::tempdir().unwrap();
        let tracker_dir = tmp.path().join(".tracker");
        fs::create_dir_all(&tracker_dir).unwrap();

        let db_path = tracker_dir.join("tracker.db");
        let conn = db::open_connection(&db_path).unwrap();
        db::ensure_schema(&conn).unwrap();
        (tmp, conn)
    }

    fn default_config() -> ProjectConfig {
        ProjectConfig {
            folder_name: ".tracker".to_string(),
            issue_prefix: "test".to_string(),
            actor: "tester".to_string(),
        }
    }

    fn write_jsonl(tmp: &tempfile::TempDir, content: &str) -> std::path::PathBuf {
        let path = tmp.path().join("import.jsonl");
        fs::write(&path, content).unwrap();
        path
    }

    fn make_jsonl_line(id: &str, title: &str, updated_at: &str) -> String {
        serde_json::json!({
            "id": id,
            "title": title,
            "description": "",
            "status": "open",
            "priority": 2,
            "issue_type": "task",
            "created_at": "2026-01-01T00:00:00Z",
            "created_by": "tester",
            "updated_at": updated_at,
            "source_repo": ".",
            "compaction_level": 0,
            "original_size": 0,
        })
        .to_string()
    }

    // 1. test_import_new_issue — single issue into empty DB
    #[test]
    fn test_import_new_issue() {
        let (tmp, conn) = setup();
        let line = make_jsonl_line("test-0001", "New issue", "2026-01-01T00:00:00Z");
        let path = write_jsonl(&tmp, &line);

        let result = import_all(&conn, &path).unwrap();
        assert_eq!(result.inserted, 1);
        assert_eq!(result.updated, 0);
        assert_eq!(result.skipped, 0);
        assert_eq!(result.errors, 0);

        // Verify it's in the DB
        let title: String = conn
            .query_row("SELECT title FROM issues WHERE id = 'test-0001'", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(title, "New issue");
    }

    // 2. test_import_with_labels — labels array populated
    #[test]
    fn test_import_with_labels() {
        let (tmp, conn) = setup();
        let line = serde_json::json!({
            "id": "test-0001",
            "title": "Labeled issue",
            "status": "open",
            "priority": 1,
            "issue_type": "bug",
            "created_at": "2026-01-01T00:00:00Z",
            "created_by": "tester",
            "updated_at": "2026-01-01T00:00:00Z",
            "source_repo": ".",
            "compaction_level": 0,
            "original_size": 0,
            "labels": ["bug", "urgent"]
        })
        .to_string();
        let path = write_jsonl(&tmp, &line);

        let result = import_all(&conn, &path).unwrap();
        assert_eq!(result.inserted, 1);

        let labels: Vec<String> = {
            let mut stmt = conn
                .prepare("SELECT label FROM labels WHERE issue_id = 'test-0001' ORDER BY label")
                .unwrap();
            stmt.query_map([], |r| r.get(0))
                .unwrap()
                .map(|r| r.unwrap())
                .collect()
        };
        assert_eq!(labels, vec!["bug", "urgent"]);
    }

    // 3. test_import_with_comments — comments with field mapping
    #[test]
    fn test_import_with_comments() {
        let (tmp, conn) = setup();
        let line = serde_json::json!({
            "id": "test-0001",
            "title": "Commented issue",
            "status": "open",
            "priority": 2,
            "issue_type": "task",
            "created_at": "2026-01-01T00:00:00Z",
            "created_by": "tester",
            "updated_at": "2026-01-01T00:00:00Z",
            "source_repo": ".",
            "compaction_level": 0,
            "original_size": 0,
            "comments": [
                {
                    "id": "c-001",
                    "issue_id": "test-0001",
                    "author": "alice",
                    "text": "First comment",
                    "created_at": "2026-01-01T01:00:00Z"
                },
                {
                    "id": "c-002",
                    "issue_id": "test-0001",
                    "author": "bob",
                    "text": "Second comment",
                    "created_at": "2026-01-01T02:00:00Z"
                }
            ]
        })
        .to_string();
        let path = write_jsonl(&tmp, &line);

        import_all(&conn, &path).unwrap();

        let count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM comments WHERE issue_id = 'test-0001'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 2);

        // Verify field mapping: text → body
        let body: String = conn
            .query_row(
                "SELECT body FROM comments WHERE id = 'c-001'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(body, "First comment");
    }

    // 4. test_import_with_dependencies — two issues with dep
    #[test]
    fn test_import_with_dependencies() {
        let (tmp, conn) = setup();
        let line_a = serde_json::json!({
            "id": "test-a",
            "title": "Issue A",
            "status": "open",
            "priority": 2,
            "issue_type": "task",
            "created_at": "2026-01-01T00:00:00Z",
            "created_by": "tester",
            "updated_at": "2026-01-01T00:00:00Z",
            "source_repo": ".",
            "compaction_level": 0,
            "original_size": 0,
            "dependencies": [{"issue_id": "test-a", "depends_on_id": "test-b", "type": "blocks"}]
        })
        .to_string();
        let line_b = make_jsonl_line("test-b", "Issue B", "2026-01-01T00:00:00Z");
        let content = format!("{}\n{}\n", line_a, line_b);
        let path = write_jsonl(&tmp, &content);

        let result = import_all(&conn, &path).unwrap();
        assert_eq!(result.inserted, 2);

        let dep_count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM dependencies WHERE from_id = 'test-a' AND to_id = 'test-b'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(dep_count, 1);
    }

    // 5. test_import_update_newer — JSONL newer → updates DB
    #[test]
    fn test_import_update_newer() {
        let (tmp, conn) = setup();

        // Create issue in DB with old timestamp
        conn.execute(
            "INSERT INTO issues (id, title, body, author, created_at, updated_at)
             VALUES ('test-0001', 'Old title', '', 'tester', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
            [],
        ).unwrap();
        fts_insert(&conn, "test-0001").unwrap();

        // Import with newer timestamp
        let line = make_jsonl_line("test-0001", "New title", "2026-01-02T00:00:00Z");
        let path = write_jsonl(&tmp, &line);

        let result = import_all(&conn, &path).unwrap();
        assert_eq!(result.updated, 1);
        assert_eq!(result.inserted, 0);
        assert_eq!(result.skipped, 0);

        let title: String = conn
            .query_row("SELECT title FROM issues WHERE id = 'test-0001'", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(title, "New title");
    }

    // 6. test_import_skip_older — JSONL older → keeps DB version
    #[test]
    fn test_import_skip_older() {
        let (tmp, conn) = setup();

        // Create issue in DB with newer timestamp
        conn.execute(
            "INSERT INTO issues (id, title, body, author, created_at, updated_at)
             VALUES ('test-0001', 'DB title', '', 'tester', '2026-01-01T00:00:00Z', '2026-01-02T00:00:00Z')",
            [],
        ).unwrap();

        // Import with older timestamp
        let line = make_jsonl_line("test-0001", "JSONL title", "2026-01-01T00:00:00Z");
        let path = write_jsonl(&tmp, &line);

        let result = import_all(&conn, &path).unwrap();
        assert_eq!(result.skipped, 1);
        assert_eq!(result.inserted, 0);
        assert_eq!(result.updated, 0);

        let title: String = conn
            .query_row("SELECT title FROM issues WHERE id = 'test-0001'", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(title, "DB title");
    }

    // 7. test_import_comment_append_only — no duplicate, adds new
    #[test]
    fn test_import_comment_append_only() {
        let (tmp, conn) = setup();

        // Create issue and existing comment
        conn.execute(
            "INSERT INTO issues (id, title, body, author, created_at, updated_at)
             VALUES ('test-0001', 'Issue', '', 'tester', '2026-01-01T00:00:00Z', '2026-01-02T00:00:00Z')",
            [],
        ).unwrap();
        conn.execute(
            "INSERT INTO comments (id, issue_id, author, body, created_at)
             VALUES ('c-existing', 'test-0001', 'alice', 'Existing', '2026-01-01T01:00:00Z')",
            [],
        ).unwrap();

        // Import with same issue (older, so issue skipped) but with existing + new comment
        let line = serde_json::json!({
            "id": "test-0001",
            "title": "Issue",
            "status": "open",
            "priority": 2,
            "issue_type": "task",
            "created_at": "2026-01-01T00:00:00Z",
            "created_by": "tester",
            "updated_at": "2026-01-01T00:00:00Z",
            "source_repo": ".",
            "compaction_level": 0,
            "original_size": 0,
            "comments": [
                {"id": "c-existing", "issue_id": "test-0001", "author": "alice", "text": "Existing", "created_at": "2026-01-01T01:00:00Z"},
                {"id": "c-new", "issue_id": "test-0001", "author": "bob", "text": "New comment", "created_at": "2026-01-01T02:00:00Z"}
            ]
        }).to_string();
        let path = write_jsonl(&tmp, &line);

        let result = import_all(&conn, &path).unwrap();
        assert_eq!(result.skipped, 1); // issue skipped (older)

        let count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM comments WHERE issue_id = 'test-0001'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 2); // existing + new
    }

    // 8. test_import_labels_full_replace — old labels replaced
    #[test]
    fn test_import_labels_full_replace() {
        let (tmp, conn) = setup();

        // Create issue with old labels
        conn.execute(
            "INSERT INTO issues (id, title, body, author, created_at, updated_at)
             VALUES ('test-0001', 'Issue', '', 'tester', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
            [],
        ).unwrap();
        fts_insert(&conn, "test-0001").unwrap();
        conn.execute(
            "INSERT INTO labels (issue_id, label) VALUES ('test-0001', 'old-label')",
            [],
        ).unwrap();

        // Import with newer timestamp and different labels
        let line = serde_json::json!({
            "id": "test-0001",
            "title": "Issue",
            "status": "open",
            "priority": 2,
            "issue_type": "task",
            "created_at": "2026-01-01T00:00:00Z",
            "created_by": "tester",
            "updated_at": "2026-01-02T00:00:00Z",
            "source_repo": ".",
            "compaction_level": 0,
            "original_size": 0,
            "labels": ["new-label-1", "new-label-2"]
        }).to_string();
        let path = write_jsonl(&tmp, &line);

        import_all(&conn, &path).unwrap();

        let labels: Vec<String> = {
            let mut stmt = conn
                .prepare("SELECT label FROM labels WHERE issue_id = 'test-0001' ORDER BY label")
                .unwrap();
            stmt.query_map([], |r| r.get(0))
                .unwrap()
                .map(|r| r.unwrap())
                .collect()
        };
        assert_eq!(labels, vec!["new-label-1", "new-label-2"]);
    }

    // 9. test_import_malformed_line_skipped — errors counted, valid lines imported
    #[test]
    fn test_import_malformed_line_skipped() {
        let (tmp, conn) = setup();
        let valid = make_jsonl_line("test-0001", "Valid", "2026-01-01T00:00:00Z");
        let content = format!("{{bad json\n{}\nnot json at all\n", valid);
        let path = write_jsonl(&tmp, &content);

        let result = import_all(&conn, &path).unwrap();
        assert_eq!(result.inserted, 1);
        assert_eq!(result.errors, 2);
    }

    // 10. test_import_empty_file — all counts 0
    #[test]
    fn test_import_empty_file() {
        let (tmp, conn) = setup();
        let path = write_jsonl(&tmp, "");

        let result = import_all(&conn, &path).unwrap();
        assert_eq!(result.inserted, 0);
        assert_eq!(result.updated, 0);
        assert_eq!(result.skipped, 0);
        assert_eq!(result.errors, 0);
    }

    // 11. test_import_preserves_local_only — DB-only issues untouched
    #[test]
    fn test_import_preserves_local_only() {
        let (tmp, conn) = setup();

        // Create a local-only issue
        conn.execute(
            "INSERT INTO issues (id, title, body, author, created_at, updated_at)
             VALUES ('local-001', 'Local only', '', 'tester', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
            [],
        ).unwrap();

        // Import a different issue
        let line = make_jsonl_line("test-0001", "Imported", "2026-01-01T00:00:00Z");
        let path = write_jsonl(&tmp, &line);

        import_all(&conn, &path).unwrap();

        // Local issue must still exist
        let title: String = conn
            .query_row("SELECT title FROM issues WHERE id = 'local-001'", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(title, "Local only");

        // Both issues exist
        let count: i32 = conn
            .query_row("SELECT COUNT(*) FROM issues", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 2);
    }

    // 12. test_import_roundtrip — create → export → clear → import → verify
    #[test]
    fn test_import_roundtrip() {
        let (tmp, conn) = setup();
        let config = default_config();

        // Create issues with various data
        let id1 = issues::create_issue(
            &conn,
            "test",
            CreateIssueParams {
                title: "Issue One".to_string(),
                body: Some("Body one".to_string()),
                issue_type: None,
                status: None,
                priority: Some("p1".to_string()),
                assignee: Some("alice".to_string()),
                author: Some("bob".to_string()),
                labels: Some(vec!["bug".to_string()]),
                external_ref: None,
                estimate_minutes: Some(60),
                design: None,
                acceptance_criteria: None,
                notes: Some("Note one".to_string()),
                parent: None,
                metadata: None,
                spec_id: None,
            },
        )
        .unwrap()
        .id;
        issues::add_comment(&conn, &id1, "alice", "A comment").unwrap();

        let id2 = issues::create_issue(
            &conn,
            "test",
            CreateIssueParams {
                title: "Issue Two".to_string(),
                body: None,
                issue_type: None,
                status: None,
                priority: None,
                assignee: None,
                author: None,
                labels: None,
                external_ref: None,
                estimate_minutes: None,
                design: None,
                acceptance_criteria: None,
                notes: None,
                parent: None,
                metadata: None,
                spec_id: None,
            },
        )
        .unwrap()
        .id;
        issues::add_dependency(&conn, &id1, &id2, "blocks").unwrap();

        // Export
        export_all(&conn, &config, tmp.path()).unwrap();
        let jsonl_path = tmp.path().join(".tracker/issues.jsonl");

        // Clear DB
        conn.execute_batch(
            "DELETE FROM dependencies; DELETE FROM labels; DELETE FROM comments;
             DELETE FROM issues_fts; DELETE FROM issues;",
        )
        .unwrap();

        // Import
        let result = import_all(&conn, &jsonl_path).unwrap();
        assert_eq!(result.inserted, 2);
        assert_eq!(result.errors, 0);

        // Verify issue 1
        let title: String = conn
            .query_row(
                "SELECT title FROM issues WHERE id = ?1",
                params![id1],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(title, "Issue One");

        let priority: String = conn
            .query_row(
                "SELECT priority FROM issues WHERE id = ?1",
                params![id1],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(priority, "p1");

        let notes: Option<String> = conn
            .query_row(
                "SELECT notes FROM issues WHERE id = ?1",
                params![id1],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(notes, Some("Note one".to_string()));

        // Verify comment
        let comment_count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM comments WHERE issue_id = ?1",
                params![id1],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(comment_count, 1);

        // Verify label
        let label: String = conn
            .query_row(
                "SELECT label FROM labels WHERE issue_id = ?1",
                params![id1],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(label, "bug");

        // Verify dependency
        let dep_count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM dependencies WHERE from_id = ?1 AND to_id = ?2",
                params![id1, id2],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(dep_count, 1);
    }

    // 13. test_import_deferred_foreign_keys — dep referencing later issue
    #[test]
    fn test_import_deferred_foreign_keys() {
        let (tmp, conn) = setup();
        // Issue A depends on Issue B, but A appears first in the file
        let line_a = serde_json::json!({
            "id": "test-a",
            "title": "Issue A",
            "status": "open",
            "priority": 2,
            "issue_type": "task",
            "created_at": "2026-01-01T00:00:00Z",
            "created_by": "tester",
            "updated_at": "2026-01-01T00:00:00Z",
            "source_repo": ".",
            "compaction_level": 0,
            "original_size": 0,
            "dependencies": [{"issue_id": "test-a", "depends_on_id": "test-b", "type": "blocks"}]
        })
        .to_string();
        let line_b = make_jsonl_line("test-b", "Issue B", "2026-01-01T00:00:00Z");
        // A before B: dependency references issue not yet inserted
        let content = format!("{}\n{}\n", line_a, line_b);
        let path = write_jsonl(&tmp, &content);

        // This should succeed thanks to PRAGMA defer_foreign_keys
        let result = import_all(&conn, &path).unwrap();
        assert_eq!(result.inserted, 2);
        assert_eq!(result.errors, 0);
    }
}

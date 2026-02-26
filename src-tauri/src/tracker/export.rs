use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use super::config::ProjectConfig;

/// JSONL representation of an issue, matching bd/br format.
#[derive(Serialize, Deserialize)]
pub(crate) struct JsonlIssue {
    pub(crate) id: String,
    pub(crate) title: String,
    #[serde(default)]
    #[serde(skip_serializing_if = "String::is_empty")]
    pub(crate) description: String,
    pub(crate) status: String,
    #[serde(default = "default_priority")]
    pub(crate) priority: i32,
    #[serde(default = "default_issue_type")]
    pub(crate) issue_type: String,
    pub(crate) created_at: String,
    #[serde(default)]
    pub(crate) created_by: String,
    pub(crate) updated_at: String,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) closed_at: Option<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) close_reason: Option<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) owner: Option<String>,
    #[serde(default)]
    pub(crate) source_repo: String,
    #[serde(default)]
    pub(crate) compaction_level: i32,
    #[serde(default)]
    pub(crate) original_size: i32,
    // Extra tracker fields (bd/br silently ignores them)
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) external_ref: Option<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) estimate_minutes: Option<i32>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) design: Option<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) acceptance_criteria: Option<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) notes: Option<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) parent: Option<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) metadata: Option<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) spec_id: Option<String>,
    // Inlined collections
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) labels: Vec<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) comments: Vec<JsonlComment>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) dependencies: Vec<JsonlDependency>,
}

fn default_priority() -> i32 {
    2
}

fn default_issue_type() -> String {
    "task".to_string()
}

/// JSONL representation of a comment.
#[derive(Serialize, Deserialize)]
pub(crate) struct JsonlComment {
    #[serde(default)]
    pub(crate) id: String,
    pub(crate) issue_id: String,
    pub(crate) author: String,
    pub(crate) text: String,
    pub(crate) created_at: String,
}

/// JSONL representation of a dependency.
#[derive(Serialize, Deserialize)]
pub(crate) struct JsonlDependency {
    pub(crate) issue_id: String,
    pub(crate) depends_on_id: String,
    #[serde(rename = "type")]
    pub(crate) dep_type: String,
}

/// Convert priority string ("p0"-"p4") to integer (0-4). Defaults to 2 (medium).
pub(crate) fn priority_to_int(priority: &str) -> i32 {
    match priority {
        "p0" => 0,
        "p1" => 1,
        "p2" => 2,
        "p3" => 3,
        "p4" => 4,
        other => other.parse().unwrap_or(2),
    }
}

/// Convert priority integer (0-4) back to string ("p0"-"p4"). Defaults to "p2".
pub(crate) fn int_to_priority(p: i32) -> String {
    match p {
        0 => "p0".to_string(),
        1 => "p1".to_string(),
        2 => "p2".to_string(),
        3 => "p3".to_string(),
        4 => "p4".to_string(),
        _ => "p2".to_string(),
    }
}

/// Bulk-fetch all comments, grouped by issue_id.
fn fetch_all_comments(conn: &Connection) -> Result<HashMap<String, Vec<JsonlComment>>> {
    let mut stmt = conn.prepare(
        "SELECT id, issue_id, author, body, created_at FROM comments ORDER BY created_at",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(JsonlComment {
            id: row.get(0)?,
            issue_id: row.get(1)?,
            author: row.get(2)?,
            text: row.get(3)?,
            created_at: row.get(4)?,
        })
    })?;

    let mut map: HashMap<String, Vec<JsonlComment>> = HashMap::new();
    for row in rows {
        let comment = row?;
        map.entry(comment.issue_id.clone()).or_default().push(comment);
    }
    Ok(map)
}

/// Bulk-fetch all dependencies, grouped by from_id (the issue that has the dependency).
fn fetch_all_dependencies(conn: &Connection) -> Result<HashMap<String, Vec<JsonlDependency>>> {
    let mut stmt = conn.prepare("SELECT from_id, to_id, dep_type FROM dependencies")?;
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
        ))
    })?;

    let mut map: HashMap<String, Vec<JsonlDependency>> = HashMap::new();
    for row in rows {
        let (from_id, to_id, dep_type) = row?;
        map.entry(from_id.clone())
            .or_default()
            .push(JsonlDependency {
                issue_id: from_id,
                depends_on_id: to_id,
                dep_type,
            });
    }
    Ok(map)
}

/// Bulk-fetch all labels, grouped by issue_id.
fn fetch_all_labels(conn: &Connection) -> Result<HashMap<String, Vec<String>>> {
    let mut stmt = conn.prepare("SELECT issue_id, label FROM labels ORDER BY label")?;
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;

    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    for row in rows {
        let (issue_id, label) = row?;
        map.entry(issue_id).or_default().push(label);
    }
    Ok(map)
}

/// Export all issues (including tombstones) to `.tracker/issues.jsonl`.
/// Uses atomic write (write to `.tmp`, then rename) to prevent corruption.
pub fn export_all(conn: &Connection, config: &ProjectConfig, project_path: &Path) -> Result<()> {
    let tracker_dir = project_path.join(&config.folder_name);
    let jsonl_path = tracker_dir.join("issues.jsonl");
    let tmp_path = tracker_dir.join("issues.jsonl.tmp");

    // Bulk-fetch all related data
    let mut all_comments = fetch_all_comments(conn)?;
    let mut all_deps = fetch_all_dependencies(conn)?;
    let mut all_labels = fetch_all_labels(conn)?;

    // Fetch all issues (including tombstones — they're part of the sync state)
    let mut stmt = conn.prepare(
        "SELECT id, title, body, issue_type, status, priority,
                assignee, author, created_at, updated_at, closed_at,
                external_ref, estimate_minutes, design, acceptance_criteria,
                notes, parent, metadata, spec_id
         FROM issues
         ORDER BY created_at"
    )?;

    let mut lines: Vec<String> = Vec::new();
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,  // id
            row.get::<_, String>(1)?,  // title
            row.get::<_, String>(2)?,  // body
            row.get::<_, String>(3)?,  // issue_type
            row.get::<_, String>(4)?,  // status
            row.get::<_, String>(5)?,  // priority
            row.get::<_, Option<String>>(6)?,  // assignee
            row.get::<_, String>(7)?,  // author
            row.get::<_, String>(8)?,  // created_at
            row.get::<_, String>(9)?,  // updated_at
            row.get::<_, Option<String>>(10)?, // closed_at
            row.get::<_, Option<String>>(11)?, // external_ref
            row.get::<_, Option<i32>>(12)?,    // estimate_minutes
            row.get::<_, Option<String>>(13)?, // design
            row.get::<_, Option<String>>(14)?, // acceptance_criteria
            row.get::<_, Option<String>>(15)?, // notes
            row.get::<_, Option<String>>(16)?, // parent
            row.get::<_, Option<String>>(17)?, // metadata
            row.get::<_, Option<String>>(18)?, // spec_id
        ))
    })?;

    for row in rows {
        let (
            id, title, body, issue_type, status, priority,
            assignee, author, created_at, updated_at, closed_at,
            external_ref, estimate_minutes, design, acceptance_criteria,
            notes, parent, metadata, spec_id,
        ) = row?;

        let close_reason = if status == "closed" {
            Some("done".to_string())
        } else {
            None
        };

        let comments = all_comments.remove(&id).unwrap_or_default();
        let deps = all_deps.remove(&id).unwrap_or_default();
        let labels = all_labels.remove(&id).unwrap_or_default();

        let jsonl_issue = JsonlIssue {
            id,
            title,
            description: body,
            status,
            priority: priority_to_int(&priority),
            issue_type,
            created_at,
            created_by: author,
            updated_at,
            closed_at,
            close_reason,
            owner: assignee,
            source_repo: ".".to_string(),
            compaction_level: 0,
            original_size: 0,
            external_ref,
            estimate_minutes,
            design,
            acceptance_criteria,
            notes,
            parent,
            metadata,
            spec_id,
            labels,
            comments,
            dependencies: deps,
        };

        let line = serde_json::to_string(&jsonl_issue).map_err(|e| {
            rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_ERROR),
                Some(format!("JSON serialization error: {}", e)),
            )
        })?;
        lines.push(line);
    }

    // Atomic write: tmp file + rename
    let content = if lines.is_empty() {
        String::new()
    } else {
        lines.join("\n") + "\n"
    };

    fs::write(&tmp_path, &content).map_err(|e| {
        rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_CANTOPEN),
            Some(format!("Failed to write tmp JSONL: {}", e)),
        )
    })?;

    fs::rename(&tmp_path, &jsonl_path).map_err(|e| {
        rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_CANTOPEN),
            Some(format!("Failed to rename JSONL: {}", e)),
        )
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tracker::db;
    use crate::tracker::issues::{self, CreateIssueParams};
    use rusqlite::Connection;

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

    fn make_issue(conn: &Connection, title: &str) -> String {
        issues::create_issue(
            conn,
            "test",
            CreateIssueParams {
                title: title.to_string(),
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
        .id
    }

    #[test]
    fn test_export_empty_db() {
        let (tmp, conn) = setup();
        let config = default_config();
        export_all(&conn, &config, tmp.path()).unwrap();

        let content = fs::read_to_string(tmp.path().join(".tracker/issues.jsonl")).unwrap();
        assert!(content.is_empty());
    }

    #[test]
    fn test_export_basic_issue() {
        let (tmp, conn) = setup();
        let config = default_config();

        issues::create_issue(
            &conn,
            "test",
            CreateIssueParams {
                title: "Test issue".to_string(),
                body: Some("A body".to_string()),
                issue_type: None,
                status: None,
                priority: Some("p1".to_string()),
                assignee: Some("alice".to_string()),
                author: Some("bob".to_string()),
                labels: None,
                external_ref: None,
                estimate_minutes: Some(30),
                design: None,
                acceptance_criteria: None,
                notes: Some("Some notes".to_string()),
                parent: None,
                metadata: None,
                spec_id: None,
            },
        )
        .unwrap();

        export_all(&conn, &config, tmp.path()).unwrap();

        let content = fs::read_to_string(tmp.path().join(".tracker/issues.jsonl")).unwrap();
        let obj: serde_json::Value = serde_json::from_str(content.trim()).unwrap();

        assert_eq!(obj["title"], "Test issue");
        assert_eq!(obj["description"], "A body");
        assert_eq!(obj["priority"], 1);
        assert_eq!(obj["created_by"], "bob");
        assert_eq!(obj["owner"], "alice");
        assert_eq!(obj["source_repo"], ".");
        assert_eq!(obj["estimate_minutes"], 30);
        assert_eq!(obj["notes"], "Some notes");
        assert_eq!(obj["issue_type"], "task");
        assert_eq!(obj["status"], "open");
    }

    #[test]
    fn test_priority_mapping() {
        assert_eq!(priority_to_int("p0"), 0);
        assert_eq!(priority_to_int("p1"), 1);
        assert_eq!(priority_to_int("p2"), 2);
        assert_eq!(priority_to_int("p3"), 3);
        assert_eq!(priority_to_int("p4"), 4);
        assert_eq!(priority_to_int("unknown"), 2); // default
    }

    #[test]
    fn test_export_with_labels() {
        let (tmp, conn) = setup();
        let config = default_config();

        issues::create_issue(
            &conn,
            "test",
            CreateIssueParams {
                title: "Labeled".to_string(),
                body: None,
                issue_type: None,
                status: None,
                priority: None,
                assignee: None,
                author: None,
                labels: Some(vec!["bug".to_string(), "urgent".to_string()]),
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
        .unwrap();

        export_all(&conn, &config, tmp.path()).unwrap();

        let content = fs::read_to_string(tmp.path().join(".tracker/issues.jsonl")).unwrap();
        let obj: serde_json::Value = serde_json::from_str(content.trim()).unwrap();

        let labels = obj["labels"].as_array().unwrap();
        assert_eq!(labels.len(), 2);
        assert!(labels.contains(&serde_json::json!("bug")));
        assert!(labels.contains(&serde_json::json!("urgent")));
    }

    #[test]
    fn test_export_with_comments() {
        let (tmp, conn) = setup();
        let config = default_config();

        let id = make_issue(&conn, "With comments");
        issues::add_comment(&conn, &id, "alice", "First comment").unwrap();
        issues::add_comment(&conn, &id, "bob", "Second comment").unwrap();

        export_all(&conn, &config, tmp.path()).unwrap();

        let content = fs::read_to_string(tmp.path().join(".tracker/issues.jsonl")).unwrap();
        let obj: serde_json::Value = serde_json::from_str(content.trim()).unwrap();

        let comments = obj["comments"].as_array().unwrap();
        assert_eq!(comments.len(), 2);
        assert_eq!(comments[0]["text"], "First comment");
        assert_eq!(comments[0]["author"], "alice");
        assert_eq!(comments[1]["text"], "Second comment");
        assert_eq!(comments[1]["author"], "bob");
        // Comments have issue_id field
        assert_eq!(comments[0]["issue_id"], id);
    }

    #[test]
    fn test_export_with_dependencies() {
        let (tmp, conn) = setup();
        let config = default_config();

        let a = make_issue(&conn, "Issue A");
        let b = make_issue(&conn, "Issue B");

        issues::add_dependency(&conn, &a, &b, "blocks").unwrap();

        export_all(&conn, &config, tmp.path()).unwrap();

        let content = fs::read_to_string(tmp.path().join(".tracker/issues.jsonl")).unwrap();
        // Find issue A's line
        for line in content.lines() {
            let obj: serde_json::Value = serde_json::from_str(line).unwrap();
            if obj["id"] == a {
                let deps = obj["dependencies"].as_array().unwrap();
                assert_eq!(deps.len(), 1);
                assert_eq!(deps[0]["issue_id"], a);
                assert_eq!(deps[0]["depends_on_id"], b);
                assert_eq!(deps[0]["type"], "blocks");
                return;
            }
        }
        panic!("Issue A not found in export");
    }

    #[test]
    fn test_export_null_fields_omitted() {
        let (tmp, conn) = setup();
        let config = default_config();

        make_issue(&conn, "Minimal issue");

        export_all(&conn, &config, tmp.path()).unwrap();

        let content = fs::read_to_string(tmp.path().join(".tracker/issues.jsonl")).unwrap();
        let obj: serde_json::Value = serde_json::from_str(content.trim()).unwrap();

        // Optional fields should not be present
        assert!(obj.get("closed_at").is_none());
        assert!(obj.get("close_reason").is_none());
        assert!(obj.get("owner").is_none());
        assert!(obj.get("design").is_none());
        assert!(obj.get("estimate_minutes").is_none());
        // Empty collections should not be present
        assert!(obj.get("labels").is_none());
        assert!(obj.get("comments").is_none());
        assert!(obj.get("dependencies").is_none());
    }

    #[test]
    fn test_export_closed_issue_has_close_reason() {
        let (tmp, conn) = setup();
        let config = default_config();

        let id = make_issue(&conn, "Will close");
        issues::close_issue(&conn, &id).unwrap();

        export_all(&conn, &config, tmp.path()).unwrap();

        let content = fs::read_to_string(tmp.path().join(".tracker/issues.jsonl")).unwrap();
        let obj: serde_json::Value = serde_json::from_str(content.trim()).unwrap();

        assert_eq!(obj["status"], "closed");
        assert_eq!(obj["close_reason"], "done");
        assert!(obj["closed_at"].is_string());
    }

    #[test]
    fn test_export_tombstone_included() {
        let (tmp, conn) = setup();
        let config = default_config();

        let id = make_issue(&conn, "Will soft-delete");
        issues::delete_issue(&conn, &id, false).unwrap();

        export_all(&conn, &config, tmp.path()).unwrap();

        let content = fs::read_to_string(tmp.path().join(".tracker/issues.jsonl")).unwrap();
        assert!(!content.is_empty());
        let obj: serde_json::Value = serde_json::from_str(content.trim()).unwrap();
        assert_eq!(obj["status"], "tombstone");
    }

    #[test]
    fn test_export_overwrites_previous() {
        let (tmp, conn) = setup();
        let config = default_config();

        make_issue(&conn, "First");
        export_all(&conn, &config, tmp.path()).unwrap();

        let content1 = fs::read_to_string(tmp.path().join(".tracker/issues.jsonl")).unwrap();
        assert_eq!(content1.lines().count(), 1);

        make_issue(&conn, "Second");
        export_all(&conn, &config, tmp.path()).unwrap();

        let content2 = fs::read_to_string(tmp.path().join(".tracker/issues.jsonl")).unwrap();
        assert_eq!(content2.lines().count(), 2);
    }

    #[test]
    fn test_export_multiple_issues() {
        let (tmp, conn) = setup();
        let config = default_config();

        for i in 0..5 {
            make_issue(&conn, &format!("Issue {}", i));
        }

        export_all(&conn, &config, tmp.path()).unwrap();

        let content = fs::read_to_string(tmp.path().join(".tracker/issues.jsonl")).unwrap();
        assert_eq!(content.lines().count(), 5);

        // Each line should be valid JSON
        for line in content.lines() {
            let obj: serde_json::Value = serde_json::from_str(line).unwrap();
            assert!(obj["id"].is_string());
            assert!(obj["title"].is_string());
        }
    }
}

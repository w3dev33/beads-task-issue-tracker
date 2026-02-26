use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

use super::export::JsonlIssue;
use super::issues::{fts_delete, fts_insert};

/// A stored sync conflict between local and remote versions of an issue.
#[derive(Debug, Serialize, Deserialize)]
pub struct ConflictRecord {
    pub id: i64,
    pub issue_id: String,
    pub local_json: String,
    pub remote_json: String,
    pub detected_at: String,
}

/// List all unresolved conflicts.
pub fn get_conflicts(conn: &Connection) -> Result<Vec<ConflictRecord>> {
    let mut stmt = conn.prepare(
        "SELECT id, issue_id, local_json, remote_json, detected_at FROM conflicts ORDER BY detected_at DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(ConflictRecord {
            id: row.get(0)?,
            issue_id: row.get(1)?,
            local_json: row.get(2)?,
            remote_json: row.get(3)?,
            detected_at: row.get(4)?,
        })
    })?;
    rows.collect()
}

/// Count unresolved conflicts.
pub fn count_conflicts(conn: &Connection) -> Result<i64> {
    conn.query_row("SELECT COUNT(*) FROM conflicts", [], |row| row.get(0))
}

/// Resolve a conflict by applying the chosen version ("local" or "remote").
/// - "local": keep current DB state, just delete the conflict record
/// - "remote": apply the remote JSONL version to the DB, then delete the conflict
pub fn resolve_conflict(conn: &Connection, conflict_id: i64, resolution: &str) -> Result<()> {
    let record: ConflictRecord = conn.query_row(
        "SELECT id, issue_id, local_json, remote_json, detected_at FROM conflicts WHERE id = ?1",
        params![conflict_id],
        |row| {
            Ok(ConflictRecord {
                id: row.get(0)?,
                issue_id: row.get(1)?,
                local_json: row.get(2)?,
                remote_json: row.get(3)?,
                detected_at: row.get(4)?,
            })
        },
    )?;

    if resolution == "remote" {
        // Parse the remote JSON and apply it
        let issue: JsonlIssue = serde_json::from_str(&record.remote_json).map_err(|e| {
            rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_ERROR),
                Some(format!("Failed to parse remote JSON: {}", e)),
            )
        })?;
        apply_issue(conn, &issue)?;
    }
    // For "local" resolution, we keep the DB as-is

    // Update synced_at so future syncs don't re-flag this issue
    conn.execute(
        "UPDATE issues SET synced_at = updated_at WHERE id = ?1",
        params![record.issue_id],
    )?;

    // Delete the conflict record
    conn.execute("DELETE FROM conflicts WHERE id = ?1", params![conflict_id])?;

    Ok(())
}

/// Dismiss a conflict without changing the issue (keep current DB state).
pub fn dismiss_conflict(conn: &Connection, conflict_id: i64) -> Result<()> {
    // Get issue_id before deleting
    let issue_id: String = conn.query_row(
        "SELECT issue_id FROM conflicts WHERE id = ?1",
        params![conflict_id],
        |row| row.get(0),
    )?;

    // Update synced_at so future syncs don't re-flag
    conn.execute(
        "UPDATE issues SET synced_at = updated_at WHERE id = ?1",
        params![issue_id],
    )?;

    conn.execute("DELETE FROM conflicts WHERE id = ?1", params![conflict_id])?;
    Ok(())
}

/// Apply a JsonlIssue to the database (full replacement of issue fields, labels, deps).
fn apply_issue(conn: &Connection, issue: &JsonlIssue) -> Result<()> {
    use super::export::int_to_priority;

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

    // Replace labels
    conn.execute(
        "DELETE FROM labels WHERE issue_id = ?1",
        params![issue.id],
    )?;
    for label in &issue.labels {
        conn.execute(
            "INSERT INTO labels (issue_id, label) VALUES (?1, ?2)",
            params![issue.id, label],
        )?;
    }

    // Replace dependencies
    conn.execute(
        "DELETE FROM dependencies WHERE from_id = ?1",
        params![issue.id],
    )?;
    for dep in &issue.dependencies {
        conn.execute(
            "INSERT INTO dependencies (from_id, to_id, dep_type) VALUES (?1, ?2, ?3)",
            params![dep.issue_id, dep.depends_on_id, dep.dep_type],
        )?;
    }

    // Re-sync FTS
    fts_delete(conn, &issue.id)?;
    fts_insert(conn, &issue.id)?;

    Ok(())
}

use rusqlite::{params, Connection, Result};

use super::ids::generate_id;

/// A tracker issue with all fields, matching the frontend Issue struct.
#[derive(Debug, Clone)]
pub struct TrackerIssue {
    pub id: String,
    pub title: String,
    pub body: String,
    pub issue_type: String,
    pub status: String,
    pub priority: String,
    pub assignee: Option<String>,
    pub author: String,
    pub created_at: String,
    pub updated_at: String,
    pub closed_at: Option<String>,
    pub external_ref: Option<String>,
    pub estimate_minutes: Option<i32>,
    pub design: Option<String>,
    pub acceptance_criteria: Option<String>,
    pub notes: Option<String>,
    pub parent: Option<String>,
    pub metadata: Option<String>,
    pub spec_id: Option<String>,
    // Populated by get_issue (full) or counts by list_issues
    pub labels: Vec<String>,
    pub comments: Vec<TrackerComment>,
    pub blocked_by: Vec<String>,
    pub blocks: Vec<String>,
    pub comment_count: i32,
    pub dependency_count: i32,
    pub dependent_count: i32,
}

/// A comment on an issue.
#[derive(Debug, Clone)]
pub struct TrackerComment {
    pub id: String,
    pub body: String,
    pub author: String,
    pub created_at: String,
}

/// Parameters for creating a new issue.
pub struct CreateIssueParams {
    pub title: String,
    pub body: Option<String>,
    pub issue_type: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub assignee: Option<String>,
    pub author: Option<String>,
    pub labels: Option<Vec<String>>,
    pub external_ref: Option<String>,
    pub estimate_minutes: Option<i32>,
    pub design: Option<String>,
    pub acceptance_criteria: Option<String>,
    pub notes: Option<String>,
    pub parent: Option<String>,
    pub metadata: Option<String>,
    pub spec_id: Option<String>,
}

/// Parameters for updating an existing issue. Only provided fields are updated.
pub struct UpdateIssueParams {
    pub title: Option<String>,
    pub body: Option<String>,
    pub issue_type: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub assignee: Option<Option<String>>,
    pub labels: Option<Vec<String>>,
    pub external_ref: Option<Option<String>>,
    pub estimate_minutes: Option<Option<i32>>,
    pub design: Option<Option<String>>,
    pub acceptance_criteria: Option<Option<String>>,
    pub notes: Option<Option<String>>,
    pub parent: Option<Option<String>>,
    pub metadata: Option<Option<String>>,
    pub spec_id: Option<Option<String>>,
}

/// List issues, optionally filtered by status ("open", "closed", or "all").
/// Returns lightweight issues with counts instead of inlined comments/deps.
pub fn list_issues(conn: &Connection, status_filter: Option<&str>) -> Result<Vec<TrackerIssue>> {
    let (where_clause, filter_value) = match status_filter {
        Some("all") | None => (String::new(), None),
        Some(s) => ("WHERE i.status = ?1".to_string(), Some(s.to_string())),
    };

    let sql = format!(
        "SELECT i.id, i.title, i.body, i.issue_type, i.status, i.priority,
                i.assignee, i.author, i.created_at, i.updated_at, i.closed_at,
                i.external_ref, i.estimate_minutes, i.design, i.acceptance_criteria,
                i.notes, i.parent, i.metadata, i.spec_id,
                (SELECT COUNT(*) FROM comments c WHERE c.issue_id = i.id) AS comment_count,
                (SELECT COUNT(*) FROM dependencies d WHERE d.to_id = i.id AND d.dep_type = 'blocks') AS dependency_count,
                (SELECT COUNT(*) FROM dependencies d WHERE d.from_id = i.id AND d.dep_type = 'blocks') AS dependent_count
         FROM issues i
         {}
         ORDER BY i.created_at DESC",
        where_clause
    );

    let mut stmt = conn.prepare(&sql)?;

    // Collect into Vec first to avoid closure type mismatch
    let raw_issues: Vec<TrackerIssue> = if let Some(ref val) = filter_value {
        let rows = stmt.query_map([val], |row| row_to_issue(row, false))?;
        rows.collect::<Result<Vec<_>>>()?
    } else {
        let rows = stmt.query_map([], |row| row_to_issue(row, false))?;
        rows.collect::<Result<Vec<_>>>()?
    };

    let mut issues = Vec::with_capacity(raw_issues.len());
    for mut issue in raw_issues {
        issue.labels = fetch_labels(conn, &issue.id)?;
        issues.push(issue);
    }

    Ok(issues)
}

/// Get a single issue by ID, with comments, labels, and dependencies inlined.
pub fn get_issue(conn: &Connection, id: &str) -> Result<TrackerIssue> {
    let mut stmt = conn.prepare(
        "SELECT i.id, i.title, i.body, i.issue_type, i.status, i.priority,
                i.assignee, i.author, i.created_at, i.updated_at, i.closed_at,
                i.external_ref, i.estimate_minutes, i.design, i.acceptance_criteria,
                i.notes, i.parent, i.metadata, i.spec_id,
                (SELECT COUNT(*) FROM comments c WHERE c.issue_id = i.id) AS comment_count,
                (SELECT COUNT(*) FROM dependencies d WHERE d.to_id = i.id AND d.dep_type = 'blocks') AS dependency_count,
                (SELECT COUNT(*) FROM dependencies d WHERE d.from_id = i.id AND d.dep_type = 'blocks') AS dependent_count
         FROM issues i
         WHERE i.id = ?1"
    )?;

    let mut issue = stmt.query_row([id], |row| row_to_issue(row, false))?;

    // Fetch related data
    issue.labels = fetch_labels(conn, id)?;
    issue.comments = fetch_comments(conn, id)?;
    issue.blocked_by = fetch_blocked_by(conn, id)?;
    issue.blocks = fetch_blocks(conn, id)?;

    Ok(issue)
}

/// Create a new issue. Generates an ID, inserts the row, syncs FTS, and returns the created issue.
pub fn create_issue(
    conn: &Connection,
    prefix: &str,
    params: CreateIssueParams,
) -> Result<TrackerIssue> {
    let id = generate_id(conn, prefix)?;
    let body = params.body.unwrap_or_default();
    let issue_type = params.issue_type.unwrap_or_else(|| "task".to_string());
    let status = params.status.unwrap_or_else(|| "open".to_string());
    let priority = params.priority.unwrap_or_else(|| "p2".to_string());
    let author = params.author.unwrap_or_default();

    conn.execute(
        "INSERT INTO issues (id, title, body, issue_type, status, priority, assignee, author,
                             external_ref, estimate_minutes, design, acceptance_criteria,
                             notes, parent, metadata, spec_id)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
        params![
            id,
            params.title,
            body,
            issue_type,
            status,
            priority,
            params.assignee,
            author,
            params.external_ref,
            params.estimate_minutes,
            params.design,
            params.acceptance_criteria,
            params.notes,
            params.parent,
            params.metadata,
            params.spec_id,
        ],
    )?;

    // Insert labels
    if let Some(labels) = params.labels {
        insert_labels(conn, &id, &labels)?;
    }

    // Sync FTS
    fts_insert(conn, &id)?;

    get_issue(conn, &id)
}

/// Update an existing issue. Only provided fields are modified.
pub fn update_issue(conn: &Connection, id: &str, params: UpdateIssueParams) -> Result<TrackerIssue> {
    // Build dynamic SET clause
    let mut sets: Vec<String> = Vec::new();
    let mut values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
    let mut idx = 1usize;

    macro_rules! set_field {
        ($field:expr, $val:expr) => {
            if let Some(v) = $val {
                sets.push(format!("{} = ?{}", $field, idx));
                values.push(Box::new(v));
                idx += 1;
            }
        };
    }

    macro_rules! set_nullable {
        ($field:expr, $val:expr) => {
            if let Some(v) = $val {
                sets.push(format!("{} = ?{}", $field, idx));
                values.push(Box::new(v));
                idx += 1;
            }
        };
    }

    set_field!("title", params.title);
    set_field!("body", params.body);
    set_field!("issue_type", params.issue_type);
    set_field!("status", params.status);
    set_field!("priority", params.priority);
    set_nullable!("assignee", params.assignee);
    set_nullable!("external_ref", params.external_ref);
    set_nullable!("estimate_minutes", params.estimate_minutes);
    set_nullable!("design", params.design);
    set_nullable!("acceptance_criteria", params.acceptance_criteria);
    set_nullable!("notes", params.notes);
    set_nullable!("parent", params.parent);
    set_nullable!("metadata", params.metadata);
    set_nullable!("spec_id", params.spec_id);

    if !sets.is_empty() {
        sets.push(format!("updated_at = datetime('now')"));
        let sql = format!("UPDATE issues SET {} WHERE id = ?{}", sets.join(", "), idx);
        values.push(Box::new(id.to_string()));

        let param_refs: Vec<&dyn rusqlite::types::ToSql> = values.iter().map(|v| v.as_ref()).collect();
        conn.execute(&sql, param_refs.as_slice())?;

        // Sync FTS if title or body changed
        fts_delete(conn, id)?;
        fts_insert(conn, id)?;
    }

    // Update labels if provided
    if let Some(labels) = params.labels {
        conn.execute("DELETE FROM labels WHERE issue_id = ?1", [id])?;
        insert_labels(conn, id, &labels)?;
    }

    get_issue(conn, id)
}

/// Close an issue: set status=closed, closed_at=now, updated_at=now.
pub fn close_issue(conn: &Connection, id: &str) -> Result<TrackerIssue> {
    conn.execute(
        "UPDATE issues SET status = 'closed', closed_at = datetime('now'), updated_at = datetime('now') WHERE id = ?1",
        [id],
    )?;
    get_issue(conn, id)
}

/// Delete an issue. If `hard` is true, DELETE the row (cascades). Otherwise, set status=tombstone.
pub fn delete_issue(conn: &Connection, id: &str, hard: bool) -> Result<()> {
    if hard {
        // FTS cleanup before delete
        fts_delete(conn, id)?;
        conn.execute("DELETE FROM issues WHERE id = ?1", [id])?;
    } else {
        conn.execute(
            "UPDATE issues SET status = 'tombstone', updated_at = datetime('now') WHERE id = ?1",
            [id],
        )?;
    }
    Ok(())
}

// --- Internal helpers ---

fn row_to_issue(row: &rusqlite::Row, _full: bool) -> rusqlite::Result<TrackerIssue> {
    Ok(TrackerIssue {
        id: row.get(0)?,
        title: row.get(1)?,
        body: row.get(2)?,
        issue_type: row.get(3)?,
        status: row.get(4)?,
        priority: row.get(5)?,
        assignee: row.get(6)?,
        author: row.get(7)?,
        created_at: row.get(8)?,
        updated_at: row.get(9)?,
        closed_at: row.get(10)?,
        external_ref: row.get(11)?,
        estimate_minutes: row.get(12)?,
        design: row.get(13)?,
        acceptance_criteria: row.get(14)?,
        notes: row.get(15)?,
        parent: row.get(16)?,
        metadata: row.get(17)?,
        spec_id: row.get(18)?,
        comment_count: row.get(19)?,
        dependency_count: row.get(20)?,
        dependent_count: row.get(21)?,
        // Populated separately
        labels: Vec::new(),
        comments: Vec::new(),
        blocked_by: Vec::new(),
        blocks: Vec::new(),
    })
}

fn fetch_labels(conn: &Connection, issue_id: &str) -> Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT label FROM labels WHERE issue_id = ?1 ORDER BY label")?;
    let rows = stmt.query_map([issue_id], |row| row.get(0))?;
    rows.collect()
}

fn fetch_comments(conn: &Connection, issue_id: &str) -> Result<Vec<TrackerComment>> {
    let mut stmt = conn.prepare(
        "SELECT id, body, author, created_at FROM comments WHERE issue_id = ?1 ORDER BY created_at",
    )?;
    let rows = stmt.query_map([issue_id], |row| {
        Ok(TrackerComment {
            id: row.get(0)?,
            body: row.get(1)?,
            author: row.get(2)?,
            created_at: row.get(3)?,
        })
    })?;
    rows.collect()
}

fn fetch_blocked_by(conn: &Connection, issue_id: &str) -> Result<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT from_id FROM dependencies WHERE to_id = ?1 AND dep_type = 'blocks'",
    )?;
    let rows = stmt.query_map([issue_id], |row| row.get(0))?;
    rows.collect()
}

fn fetch_blocks(conn: &Connection, issue_id: &str) -> Result<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT to_id FROM dependencies WHERE from_id = ?1 AND dep_type = 'blocks'",
    )?;
    let rows = stmt.query_map([issue_id], |row| row.get(0))?;
    rows.collect()
}

fn insert_labels(conn: &Connection, issue_id: &str, labels: &[String]) -> Result<()> {
    let mut stmt = conn.prepare("INSERT OR IGNORE INTO labels (issue_id, label) VALUES (?1, ?2)")?;
    for label in labels {
        stmt.execute(params![issue_id, label])?;
    }
    Ok(())
}

/// Insert issue into FTS5 index (standalone table with issue_id column).
fn fts_insert(conn: &Connection, issue_id: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO issues_fts(issue_id, title, body)
         SELECT id, title, body FROM issues WHERE id = ?1",
        [issue_id],
    )?;
    Ok(())
}

/// Delete issue from FTS5 index.
fn fts_delete(conn: &Connection, issue_id: &str) -> Result<()> {
    conn.execute(
        "DELETE FROM issues_fts WHERE issue_id = ?1",
        [issue_id],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tracker::db;

    fn setup_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        db::ensure_schema(&conn).unwrap();
        conn
    }

    #[test]
    fn test_create_and_get_issue() {
        let conn = setup_db();
        let issue = create_issue(
            &conn,
            "test",
            CreateIssueParams {
                title: "My first issue".to_string(),
                body: Some("Description here".to_string()),
                issue_type: None,
                status: None,
                priority: Some("p1".to_string()),
                assignee: Some("alice".to_string()),
                author: Some("bob".to_string()),
                labels: None,
                external_ref: None,
                estimate_minutes: Some(60),
                design: None,
                acceptance_criteria: Some("It works".to_string()),
                notes: None,
                parent: None,
                metadata: None,
                spec_id: None,
            },
        )
        .unwrap();

        assert!(issue.id.starts_with("test-"));
        assert_eq!(issue.title, "My first issue");
        assert_eq!(issue.body, "Description here");
        assert_eq!(issue.issue_type, "task");
        assert_eq!(issue.status, "open");
        assert_eq!(issue.priority, "p1");
        assert_eq!(issue.assignee, Some("alice".to_string()));
        assert_eq!(issue.author, "bob");
        assert_eq!(issue.estimate_minutes, Some(60));
        assert_eq!(issue.acceptance_criteria, Some("It works".to_string()));

        // Get same issue
        let fetched = get_issue(&conn, &issue.id).unwrap();
        assert_eq!(fetched.id, issue.id);
        assert_eq!(fetched.title, "My first issue");
    }

    #[test]
    fn test_list_issues_filters() {
        let conn = setup_db();
        create_issue(
            &conn,
            "test",
            CreateIssueParams {
                title: "Open issue".to_string(),
                body: None,
                issue_type: None,
                status: Some("open".to_string()),
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
        .unwrap();

        let closed = create_issue(
            &conn,
            "test",
            CreateIssueParams {
                title: "Closed issue".to_string(),
                body: None,
                issue_type: None,
                status: Some("closed".to_string()),
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
        .unwrap();
        // Also set closed_at for realism
        conn.execute(
            "UPDATE issues SET closed_at = datetime('now') WHERE id = ?1",
            [&closed.id],
        )
        .unwrap();

        let all = list_issues(&conn, Some("all")).unwrap();
        assert_eq!(all.len(), 2);

        let open = list_issues(&conn, Some("open")).unwrap();
        assert_eq!(open.len(), 1);
        assert_eq!(open[0].title, "Open issue");

        let closed_list = list_issues(&conn, Some("closed")).unwrap();
        assert_eq!(closed_list.len(), 1);
        assert_eq!(closed_list[0].title, "Closed issue");
    }

    #[test]
    fn test_update_partial_fields() {
        let conn = setup_db();
        let issue = create_issue(
            &conn,
            "test",
            CreateIssueParams {
                title: "Original".to_string(),
                body: Some("Original body".to_string()),
                issue_type: None,
                status: None,
                priority: Some("p2".to_string()),
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
        .unwrap();

        // Update only title
        let updated = update_issue(
            &conn,
            &issue.id,
            UpdateIssueParams {
                title: Some("Updated title".to_string()),
                body: None,
                issue_type: None,
                status: None,
                priority: None,
                assignee: None,
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
        .unwrap();

        assert_eq!(updated.title, "Updated title");
        assert_eq!(updated.body, "Original body"); // unchanged
        assert_eq!(updated.priority, "p2"); // unchanged
    }

    #[test]
    fn test_close_issue() {
        let conn = setup_db();
        let issue = create_issue(
            &conn,
            "test",
            CreateIssueParams {
                title: "To close".to_string(),
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
        .unwrap();

        let closed = close_issue(&conn, &issue.id).unwrap();
        assert_eq!(closed.status, "closed");
        assert!(closed.closed_at.is_some());
    }

    #[test]
    fn test_delete_hard() {
        let conn = setup_db();
        let issue = create_issue(
            &conn,
            "test",
            CreateIssueParams {
                title: "To delete".to_string(),
                body: None,
                issue_type: None,
                status: None,
                priority: None,
                assignee: None,
                author: None,
                labels: Some(vec!["bug".to_string()]),
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

        // Add a comment
        conn.execute(
            "INSERT INTO comments (id, issue_id, body, author) VALUES ('c1', ?1, 'test comment', 'tester')",
            [&issue.id],
        )
        .unwrap();

        delete_issue(&conn, &issue.id, true).unwrap();

        // Issue should be gone
        let result = get_issue(&conn, &issue.id);
        assert!(result.is_err());

        // Labels should be gone (CASCADE)
        let label_count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM labels WHERE issue_id = ?1",
                [&issue.id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(label_count, 0);

        // Comments should be gone (CASCADE)
        let comment_count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM comments WHERE issue_id = ?1",
                [&issue.id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(comment_count, 0);
    }

    #[test]
    fn test_delete_soft() {
        let conn = setup_db();
        let issue = create_issue(
            &conn,
            "test",
            CreateIssueParams {
                title: "To soft-delete".to_string(),
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
        .unwrap();

        delete_issue(&conn, &issue.id, false).unwrap();

        let fetched = get_issue(&conn, &issue.id).unwrap();
        assert_eq!(fetched.status, "tombstone");
    }

    #[test]
    fn test_create_with_labels() {
        let conn = setup_db();
        let issue = create_issue(
            &conn,
            "test",
            CreateIssueParams {
                title: "Labeled issue".to_string(),
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

        assert_eq!(issue.labels.len(), 2);
        assert!(issue.labels.contains(&"bug".to_string()));
        assert!(issue.labels.contains(&"urgent".to_string()));

        // Round-trip via get_issue
        let fetched = get_issue(&conn, &issue.id).unwrap();
        assert_eq!(fetched.labels.len(), 2);
    }

    #[test]
    fn test_issue_with_comments_and_deps() {
        let conn = setup_db();

        let issue_a = create_issue(
            &conn,
            "test",
            CreateIssueParams {
                title: "Issue A".to_string(),
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
        .unwrap();

        let issue_b = create_issue(
            &conn,
            "test",
            CreateIssueParams {
                title: "Issue B".to_string(),
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
        .unwrap();

        // Add a comment to issue A
        conn.execute(
            "INSERT INTO comments (id, issue_id, body, author) VALUES ('c1', ?1, 'A comment', 'tester')",
            [&issue_a.id],
        )
        .unwrap();

        // A blocks B: from_id=A, to_id=B
        conn.execute(
            "INSERT INTO dependencies (from_id, to_id, dep_type) VALUES (?1, ?2, 'blocks')",
            params![issue_a.id, issue_b.id],
        )
        .unwrap();

        // Fetch A: should have 1 comment, blocks=[B], blocked_by=[]
        let a = get_issue(&conn, &issue_a.id).unwrap();
        assert_eq!(a.comments.len(), 1);
        assert_eq!(a.comments[0].body, "A comment");
        assert_eq!(a.blocks, vec![issue_b.id.clone()]);
        assert!(a.blocked_by.is_empty());
        assert_eq!(a.comment_count, 1);
        assert_eq!(a.dependent_count, 1); // A blocks 1 issue

        // Fetch B: should have blocked_by=[A]
        let b = get_issue(&conn, &issue_b.id).unwrap();
        assert_eq!(b.blocked_by, vec![issue_a.id.clone()]);
        assert!(b.blocks.is_empty());
        assert_eq!(b.dependency_count, 1); // B is blocked by 1 issue
    }
}

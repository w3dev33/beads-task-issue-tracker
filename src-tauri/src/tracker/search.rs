use rusqlite::{params, Connection, Result};

/// A search result from the FTS5 full-text index.
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub issue_id: String,
    pub title: String,
    pub snippet: String,
    pub rank: f64,
    pub status: String,
}

/// Search issues using FTS5 full-text search.
///
/// The query is sanitized to strip FTS5 operators, then matched against
/// the `issues_fts` index (title, body, notes). Results are ranked by BM25
/// relevance and include a highlighted snippet from the body.
pub fn search(conn: &Connection, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
    let sanitized = sanitize_fts_query(query);
    if sanitized.is_empty() {
        return Ok(Vec::new());
    }

    // Append * to each term for prefix matching (e.g., "auth" matches "authentication")
    let prefix_query: String = sanitized
        .split_whitespace()
        .map(|term| format!("{}*", term))
        .collect::<Vec<_>>()
        .join(" ");

    let mut stmt = conn.prepare(
        "SELECT f.issue_id, i.title,
                snippet(issues_fts, 2, '<b>', '</b>', '...', 32) AS snippet,
                rank,
                i.status
         FROM issues_fts f
         JOIN issues i ON i.id = f.issue_id
         WHERE issues_fts MATCH ?1
         ORDER BY rank
         LIMIT ?2"
    )?;

    let rows = stmt.query_map(params![prefix_query, limit as i64], |row| {
        Ok(SearchResult {
            issue_id: row.get(0)?,
            title: row.get(1)?,
            snippet: row.get(2)?,
            rank: row.get(3)?,
            status: row.get(4)?,
        })
    })?;

    rows.collect()
}

/// Strip FTS5 special operators from user input to prevent syntax errors.
fn sanitize_fts_query(query: &str) -> String {
    // Remove FTS5 special characters
    let cleaned: String = query
        .chars()
        .map(|c| match c {
            '"' | '*' | '(' | ')' | '+' | '-' | '^' | ':' => ' ',
            _ => c,
        })
        .collect();

    // Remove FTS5 keyword operators (AND, OR, NOT, NEAR)
    cleaned
        .split_whitespace()
        .filter(|word| {
            let upper = word.to_uppercase();
            upper != "AND" && upper != "OR" && upper != "NOT" && upper != "NEAR"
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tracker::db;
    use crate::tracker::issues::{self, CreateIssueParams};

    fn setup_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        db::ensure_schema(&conn).unwrap();
        conn
    }

    fn make_issue(conn: &Connection, title: &str, body: &str, notes: Option<&str>) -> String {
        let issue = issues::create_issue(
            conn,
            "test",
            CreateIssueParams {
                title: title.to_string(),
                body: Some(body.to_string()),
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
                notes: notes.map(|s| s.to_string()),
                parent: None,
                metadata: None,
                spec_id: None,
            },
        )
        .unwrap();
        issue.id
    }

    #[test]
    fn test_basic_search() {
        let conn = setup_db();
        make_issue(&conn, "Fix authentication bug", "Login fails on Safari", None);
        make_issue(&conn, "Add dashboard widget", "New analytics widget", None);
        make_issue(&conn, "Update authentication flow", "OAuth2 migration", None);

        let results = search(&conn, "authentication", 50).unwrap();
        assert_eq!(results.len(), 2);
        // Both authentication issues found
        assert!(results.iter().all(|r| r.title.contains("authentication") || r.title.contains("authentication")));
    }

    #[test]
    fn test_search_body() {
        let conn = setup_db();
        make_issue(&conn, "Bug report", "The postgresql database crashes under load", None);
        make_issue(&conn, "Feature request", "Add support for redis caching", None);

        let results = search(&conn, "postgresql", 50).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Bug report");
    }

    #[test]
    fn test_search_notes() {
        let conn = setup_db();
        make_issue(&conn, "Refactor API", "Clean up endpoints", Some("Consider graphql migration"));
        make_issue(&conn, "Fix tests", "Flaky integration tests", None);

        let results = search(&conn, "graphql", 50).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Refactor API");
    }

    #[test]
    fn test_search_no_results() {
        let conn = setup_db();
        make_issue(&conn, "Some issue", "Some body", None);

        let results = search(&conn, "nonexistentterm", 50).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_special_chars() {
        let conn = setup_db();
        make_issue(&conn, "Test issue", "Test body", None);

        // These should not cause FTS5 syntax errors
        assert!(search(&conn, "\"quoted\"", 50).is_ok());
        assert!(search(&conn, "test*", 50).is_ok());
        assert!(search(&conn, "(parentheses)", 50).is_ok());
        assert!(search(&conn, "a + b - c", 50).is_ok());
        assert!(search(&conn, "NOT AND OR", 50).is_ok());
    }

    #[test]
    fn test_search_partial_words() {
        let conn = setup_db();
        make_issue(&conn, "Fix authentication system", "OAuth implementation", None);

        let results = search(&conn, "auth", 50).unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].title.contains("authentication"));
    }

    #[test]
    fn test_search_respects_limit() {
        let conn = setup_db();
        for i in 0..5 {
            make_issue(&conn, &format!("Issue about widgets {}", i), "Widget details", None);
        }

        let results = search(&conn, "widgets", 2).unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_search_empty_query() {
        let conn = setup_db();
        make_issue(&conn, "Test", "Body", None);

        let results = search(&conn, "", 50).unwrap();
        assert!(results.is_empty());

        let results = search(&conn, "   ", 50).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_returns_status() {
        let conn = setup_db();
        let id = make_issue(&conn, "Closeable issue", "Body text", None);
        issues::close_issue(&conn, &id).unwrap();

        let results = search(&conn, "closeable", 50).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].status, "closed");
    }
}

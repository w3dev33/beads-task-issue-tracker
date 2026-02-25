use rand::Rng;
use rusqlite::{Connection, Result};

const SUFFIX_LEN: usize = 4;
const BASE36_CHARS: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";
const MAX_RETRIES: usize = 10;

/// Generate a unique issue ID in `{prefix}-{base36_suffix}` format.
///
/// The suffix is a 4-character random base36 string (~1.6M combinations).
/// Checks the `issues` table for collisions, retrying up to 10 times.
pub fn generate_id(conn: &Connection, prefix: &str) -> Result<String> {
    let mut rng = rand::thread_rng();

    for _ in 0..MAX_RETRIES {
        let suffix: String = (0..SUFFIX_LEN)
            .map(|_| {
                let idx = rng.gen_range(0..BASE36_CHARS.len());
                BASE36_CHARS[idx] as char
            })
            .collect();

        let id = format!("{}-{}", prefix, suffix);

        let exists: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM issues WHERE id = ?1)",
            [&id],
            |row| row.get(0),
        )?;

        if !exists {
            return Ok(id);
        }
    }

    Err(rusqlite::Error::SqliteFailure(
        rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_FULL),
        Some(format!(
            "Failed to generate unique ID after {} attempts",
            MAX_RETRIES
        )),
    ))
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
    fn test_generate_id_format() {
        let conn = setup_db();
        let id = generate_id(&conn, "tracker").unwrap();

        // Format: prefix-XXXX
        let parts: Vec<&str> = id.splitn(2, '-').collect();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0], "tracker");
        assert_eq!(parts[1].len(), SUFFIX_LEN);

        // All chars are base36
        for c in parts[1].chars() {
            assert!(
                c.is_ascii_digit() || (c.is_ascii_lowercase() && c <= 'z'),
                "unexpected char: {}",
                c
            );
        }
    }

    #[test]
    fn test_generate_id_no_collision() {
        let conn = setup_db();

        let mut ids = std::collections::HashSet::new();
        for _ in 0..50 {
            let id = generate_id(&conn, "test").unwrap();
            assert!(ids.insert(id), "generated duplicate ID");
        }
    }

    #[test]
    fn test_generate_id_with_existing() {
        let conn = setup_db();

        // Insert an existing issue
        conn.execute(
            "INSERT INTO issues (id, title, author) VALUES (?1, ?2, ?3)",
            ["test-aaaa", "Existing issue", "tester"],
        )
        .unwrap();

        // Generate a new ID — should not collide with existing
        let id = generate_id(&conn, "test").unwrap();
        assert_ne!(id, "test-aaaa");
    }
}

use rusqlite::Connection;
use serde::Serialize;
use std::fs;
use std::path::Path;

use super::export::export_all;
use super::import::{import_all, ImportResult};
use super::config::ProjectConfig;

/// Result of a migration from `.beads/` to `.tracker/`.
#[derive(Debug, Serialize)]
pub struct MigrationResult {
    pub issues: ImportResultSerialized,
    pub attachments_copied: usize,
    pub attachments_skipped: usize,
    pub config_migrated: bool,
    pub warnings: Vec<String>,
}

/// Serializable version of ImportResult for Tauri.
#[derive(Debug, Serialize)]
pub struct ImportResultSerialized {
    pub inserted: usize,
    pub updated: usize,
    pub skipped: usize,
    pub errors: usize,
}

impl From<ImportResult> for ImportResultSerialized {
    fn from(r: ImportResult) -> Self {
        Self {
            inserted: r.inserted,
            updated: r.updated,
            skipped: r.skipped,
            errors: r.errors,
        }
    }
}

/// Info about available `.beads/` data for migration.
#[derive(Debug, Serialize)]
pub struct BeadsSourceInfo {
    pub has_jsonl: bool,
    pub issue_count: usize,
}

/// Check if a `.beads/` directory has data available for migration.
pub fn check_beads_source(project_path: &Path) -> BeadsSourceInfo {
    let beads_dir = project_path.join(".beads");
    let jsonl_path = beads_dir.join("issues.jsonl");

    if !jsonl_path.exists() {
        return BeadsSourceInfo {
            has_jsonl: false,
            issue_count: 0,
        };
    }

    // Count non-empty, non-tombstone lines
    let count = match fs::read_to_string(&jsonl_path) {
        Ok(content) => content
            .lines()
            .filter(|line| {
                let line = line.trim();
                if line.is_empty() {
                    return false;
                }
                // Exclude tombstone issues from the count
                !line.contains("\"status\":\"tombstone\"")
            })
            .count(),
        Err(_) => 0,
    };

    BeadsSourceInfo {
        has_jsonl: true,
        issue_count: count,
    }
}

/// Migrate data from `.beads/` into the tracker database.
///
/// Steps:
/// 1. Validate source (.beads/issues.jsonl must exist)
/// 2. Ensure tracker DB is initialized
/// 3. Import JSONL into tracker DB (reuses import::import_all)
/// 4. Copy attachments (.beads/attachments/ → .tracker/attachments/)
/// 5. Extract config (issue-prefix from .beads/config.yaml)
/// 6. Re-export to normalize .tracker/issues.jsonl
///
/// Non-destructive: never modifies or deletes .beads/.
pub fn migrate_from_beads(
    conn: &Connection,
    config: &ProjectConfig,
    project_path: &Path,
) -> Result<MigrationResult, String> {
    let beads_dir = project_path.join(".beads");
    let tracker_dir = project_path.join(&config.folder_name);
    let mut warnings = Vec::new();

    // 1. Validate source
    let jsonl_path = beads_dir.join("issues.jsonl");
    if !jsonl_path.exists() {
        return Err("No .beads/issues.jsonl found — nothing to migrate".to_string());
    }

    // 2. Ensure tracker dir exists
    if !tracker_dir.exists() {
        fs::create_dir_all(&tracker_dir)
            .map_err(|e| format!("Failed to create tracker directory: {}", e))?;
    }

    // 3. Import JSONL
    let import_result = import_all(conn, &jsonl_path)
        .map_err(|e| format!("JSONL import failed: {}", e))?;

    log::info!(
        "[tracker/migrate] JSONL import: {} inserted, {} updated, {} skipped, {} errors",
        import_result.inserted,
        import_result.updated,
        import_result.skipped,
        import_result.errors
    );

    if import_result.errors > 0 {
        warnings.push(format!(
            "{} malformed lines skipped during import",
            import_result.errors
        ));
    }

    // 4. Copy attachments
    let (copied, skipped) = copy_attachments(&beads_dir, &tracker_dir, &mut warnings);

    // 5. Extract config
    let config_migrated = extract_config(&beads_dir, &tracker_dir, &mut warnings);

    // 6. Re-export to normalize
    if let Err(e) = export_all(conn, config, project_path) {
        warnings.push(format!("Post-migration JSONL export failed: {}", e));
    }

    Ok(MigrationResult {
        issues: import_result.into(),
        attachments_copied: copied,
        attachments_skipped: skipped,
        config_migrated,
        warnings,
    })
}

/// Recursively copy attachments from `.beads/attachments/` to `.tracker/attachments/`.
/// Skips files that already exist at the destination.
/// Returns (copied, skipped) counts.
fn copy_attachments(
    beads_dir: &Path,
    tracker_dir: &Path,
    warnings: &mut Vec<String>,
) -> (usize, usize) {
    let src = beads_dir.join("attachments");
    let dst = tracker_dir.join("attachments");

    if !src.exists() || !src.is_dir() {
        return (0, 0);
    }

    let mut copied = 0usize;
    let mut skipped = 0usize;

    if let Err(e) = copy_dir_recursive(&src, &dst, &mut copied, &mut skipped, warnings) {
        warnings.push(format!("Attachment copy error: {}", e));
    }

    log::info!(
        "[tracker/migrate] Attachments: {} copied, {} skipped (already exist)",
        copied,
        skipped
    );

    (copied, skipped)
}

fn copy_dir_recursive(
    src: &Path,
    dst: &Path,
    copied: &mut usize,
    skipped: &mut usize,
    warnings: &mut Vec<String>,
) -> Result<(), String> {
    fs::create_dir_all(dst).map_err(|e| format!("Failed to create {}: {}", dst.display(), e))?;

    let entries =
        fs::read_dir(src).map_err(|e| format!("Failed to read {}: {}", src.display(), e))?;

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                warnings.push(format!("Failed to read entry in {}: {}", src.display(), e));
                continue;
            }
        };

        let src_path = entry.path();
        let file_name = entry.file_name();
        let dst_path = dst.join(&file_name);

        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path, copied, skipped, warnings)?;
        } else if dst_path.exists() {
            *skipped += 1;
        } else {
            match fs::copy(&src_path, &dst_path) {
                Ok(_) => *copied += 1,
                Err(e) => {
                    warnings.push(format!(
                        "Failed to copy {}: {}",
                        src_path.display(),
                        e
                    ));
                }
            }
        }
    }

    Ok(())
}

/// Extract issue-prefix from `.beads/config.yaml` and write a minimal `.tracker/config.yaml`.
/// Returns true if config was written.
fn extract_config(
    beads_dir: &Path,
    tracker_dir: &Path,
    warnings: &mut Vec<String>,
) -> bool {
    let beads_config = beads_dir.join("config.yaml");
    if !beads_config.exists() {
        return false;
    }

    let content = match fs::read_to_string(&beads_config) {
        Ok(c) => c,
        Err(e) => {
            warnings.push(format!("Failed to read .beads/config.yaml: {}", e));
            return false;
        }
    };

    // Simple line-based extraction (avoid adding a YAML dependency)
    let mut prefix = None;
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        if line.starts_with("issue-prefix:") {
            if let Some(val) = line.strip_prefix("issue-prefix:") {
                let val = val.trim().trim_matches('"').trim_matches('\'');
                if !val.is_empty() {
                    prefix = Some(val.to_string());
                }
            }
        }
    }

    let tracker_config = tracker_dir.join("config.yaml");
    if tracker_config.exists() {
        // Don't overwrite existing config
        return false;
    }

    // Write minimal config with extracted prefix
    let config_content = if let Some(ref p) = prefix {
        format!(
            "# Tracker configuration (migrated from .beads/config.yaml)\nissue-prefix: \"{}\"\n",
            p
        )
    } else {
        "# Tracker configuration (migrated from .beads/config.yaml)\n# issue-prefix not set in source\n".to_string()
    };

    match fs::write(&tracker_config, &config_content) {
        Ok(_) => {
            log::info!("[tracker/migrate] Config migrated (prefix: {:?})", prefix);
            true
        }
        Err(e) => {
            warnings.push(format!("Failed to write .tracker/config.yaml: {}", e));
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tracker::db;
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

    fn make_jsonl_line(id: &str, title: &str, status: &str) -> String {
        serde_json::json!({
            "id": id,
            "title": title,
            "description": "",
            "status": status,
            "priority": 2,
            "issue_type": "task",
            "created_at": "2026-01-01T00:00:00Z",
            "created_by": "tester",
            "updated_at": "2026-01-01T00:00:00Z",
            "source_repo": ".",
            "compaction_level": 0,
            "original_size": 0,
        })
        .to_string()
    }

    // 1. Full migration with JSONL import
    #[test]
    fn test_migrate_imports_jsonl() {
        let (tmp, conn) = setup();
        let config = default_config();

        // Create .beads/issues.jsonl
        let beads_dir = tmp.path().join(".beads");
        fs::create_dir_all(&beads_dir).unwrap();
        let jsonl = format!(
            "{}\n{}\n",
            make_jsonl_line("test-001", "First issue", "open"),
            make_jsonl_line("test-002", "Second issue", "closed"),
        );
        fs::write(beads_dir.join("issues.jsonl"), &jsonl).unwrap();

        let result = migrate_from_beads(&conn, &config, tmp.path()).unwrap();
        assert_eq!(result.issues.inserted, 2);
        assert_eq!(result.issues.errors, 0);

        // Verify DB
        let count: i32 = conn
            .query_row("SELECT COUNT(*) FROM issues", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 2);
    }

    // 2. Attachments are copied
    #[test]
    fn test_migrate_copies_attachments() {
        let (tmp, conn) = setup();
        let config = default_config();

        let beads_dir = tmp.path().join(".beads");
        fs::create_dir_all(beads_dir.join("attachments/issue-1")).unwrap();
        fs::write(
            beads_dir.join("attachments/issue-1/screenshot.png"),
            b"fake-png",
        )
        .unwrap();
        fs::write(beads_dir.join("issues.jsonl"), "").unwrap();

        let result = migrate_from_beads(&conn, &config, tmp.path()).unwrap();
        assert_eq!(result.attachments_copied, 1);
        assert_eq!(result.attachments_skipped, 0);

        // Verify file exists at destination
        let dst = tmp.path().join(".tracker/attachments/issue-1/screenshot.png");
        assert!(dst.exists());
        assert_eq!(fs::read(&dst).unwrap(), b"fake-png");
    }

    // 3. Existing attachments are skipped
    #[test]
    fn test_migrate_skips_existing_attachments() {
        let (tmp, conn) = setup();
        let config = default_config();

        let beads_dir = tmp.path().join(".beads");
        fs::create_dir_all(beads_dir.join("attachments/issue-1")).unwrap();
        fs::write(
            beads_dir.join("attachments/issue-1/file.txt"),
            b"source",
        )
        .unwrap();
        fs::write(beads_dir.join("issues.jsonl"), "").unwrap();

        // Pre-create destination
        let tracker_attach = tmp.path().join(".tracker/attachments/issue-1");
        fs::create_dir_all(&tracker_attach).unwrap();
        fs::write(tracker_attach.join("file.txt"), b"already-here").unwrap();

        let result = migrate_from_beads(&conn, &config, tmp.path()).unwrap();
        assert_eq!(result.attachments_copied, 0);
        assert_eq!(result.attachments_skipped, 1);

        // Verify existing file was NOT overwritten
        assert_eq!(
            fs::read_to_string(tracker_attach.join("file.txt")).unwrap(),
            "already-here"
        );
    }

    // 4. No .beads/ directory returns error
    #[test]
    fn test_migrate_no_beads_dir() {
        let (tmp, conn) = setup();
        let config = default_config();

        let result = migrate_from_beads(&conn, &config, tmp.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No .beads/issues.jsonl found"));
    }

    // 5. .beads/ is untouched after migration
    #[test]
    fn test_migrate_preserves_beads() {
        let (tmp, conn) = setup();
        let config = default_config();

        let beads_dir = tmp.path().join(".beads");
        fs::create_dir_all(beads_dir.join("attachments/issue-1")).unwrap();
        let jsonl_content = make_jsonl_line("test-001", "Issue", "open");
        fs::write(beads_dir.join("issues.jsonl"), &jsonl_content).unwrap();
        fs::write(
            beads_dir.join("attachments/issue-1/img.png"),
            b"data",
        )
        .unwrap();

        migrate_from_beads(&conn, &config, tmp.path()).unwrap();

        // .beads/ must be completely untouched
        assert_eq!(
            fs::read_to_string(beads_dir.join("issues.jsonl")).unwrap(),
            jsonl_content
        );
        assert_eq!(
            fs::read(beads_dir.join("attachments/issue-1/img.png")).unwrap(),
            b"data"
        );
    }

    // 6. check_beads_source with valid data
    #[test]
    fn test_check_beads_source_with_data() {
        let tmp = tempfile::tempdir().unwrap();
        let beads_dir = tmp.path().join(".beads");
        fs::create_dir_all(&beads_dir).unwrap();

        let content = format!(
            "{}\n{}\n{}\n",
            make_jsonl_line("t-1", "Open", "open"),
            make_jsonl_line("t-2", "Closed", "closed"),
            r#"{"id":"t-3","title":"Deleted","status":"tombstone","priority":2,"issue_type":"task","created_at":"2026-01-01T00:00:00Z","created_by":"x","updated_at":"2026-01-01T00:00:00Z","source_repo":".","compaction_level":0,"original_size":0}"#,
        );
        fs::write(beads_dir.join("issues.jsonl"), content).unwrap();

        let info = check_beads_source(tmp.path());
        assert!(info.has_jsonl);
        assert_eq!(info.issue_count, 2); // tombstone excluded
    }

    // 7. check_beads_source with no .beads/
    #[test]
    fn test_check_beads_source_no_beads() {
        let tmp = tempfile::tempdir().unwrap();
        let info = check_beads_source(tmp.path());
        assert!(!info.has_jsonl);
        assert_eq!(info.issue_count, 0);
    }

    // 8. Config extraction
    #[test]
    fn test_migrate_extracts_config() {
        let (tmp, conn) = setup();
        let config = default_config();

        let beads_dir = tmp.path().join(".beads");
        fs::create_dir_all(&beads_dir).unwrap();
        fs::write(beads_dir.join("issues.jsonl"), "").unwrap();
        fs::write(
            beads_dir.join("config.yaml"),
            "# comment\nissue-prefix: \"myproject\"\n",
        )
        .unwrap();

        let result = migrate_from_beads(&conn, &config, tmp.path()).unwrap();
        assert!(result.config_migrated);

        let tracker_config =
            fs::read_to_string(tmp.path().join(".tracker/config.yaml")).unwrap();
        assert!(tracker_config.contains("myproject"));
    }
}

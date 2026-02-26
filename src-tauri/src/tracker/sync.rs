use rusqlite::Connection;
use serde::Serialize;
use std::path::Path;
use std::process::Command;

use super::config::ProjectConfig;
use super::export;
use super::import::{self, ImportResult};

/// Result of a tracker git sync cycle.
#[derive(Debug, Serialize)]
pub struct SyncResult {
    pub exported: bool,
    pub committed: bool,
    pub pushed: bool,
    pub pulled: bool,
    #[serde(rename = "importResult")]
    pub import_result: Option<SerializableImportResult>,
    pub conflict: bool,
}

/// Serializable version of ImportResult for the frontend.
#[derive(Debug, Serialize)]
pub struct SerializableImportResult {
    pub inserted: usize,
    pub updated: usize,
    pub skipped: usize,
    pub errors: usize,
}

impl From<ImportResult> for SerializableImportResult {
    fn from(r: ImportResult) -> Self {
        Self {
            inserted: r.inserted,
            updated: r.updated,
            skipped: r.skipped,
            errors: r.errors,
        }
    }
}

/// Run a git command in the given directory, returning stdout on success.
fn git(args: &[&str], cwd: &Path) -> Result<String, String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .map_err(|e| format!("Failed to run git {}: {}", args.first().unwrap_or(&""), e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(format!(
            "git {} failed: {}",
            args.first().unwrap_or(&""),
            stderr
        ))
    }
}

/// Check if the given path is inside a git repository.
fn is_git_repo(cwd: &Path) -> bool {
    git(&["rev-parse", "--is-inside-work-tree"], cwd)
        .map(|s| s == "true")
        .unwrap_or(false)
}

/// Check if the git repo has at least one remote configured.
fn has_remote(cwd: &Path) -> bool {
    git(&["remote"], cwd)
        .map(|s| !s.is_empty())
        .unwrap_or(false)
}

/// Full sync cycle: export → git add/commit → pull → import → push.
pub fn sync(
    conn: &Connection,
    config: &ProjectConfig,
    project_path: &Path,
) -> Result<SyncResult, String> {
    let mut result = SyncResult {
        exported: false,
        committed: false,
        pushed: false,
        pulled: false,
        import_result: None,
        conflict: false,
    };

    // 1. Must be a git repo
    if !is_git_repo(project_path) {
        log::info!("[tracker/sync] Not a git repo, skipping sync");
        return Ok(result);
    }

    // 2. Export DB → JSONL
    export::export_all(conn, config, project_path)
        .map_err(|e| format!("Export failed: {}", e))?;
    result.exported = true;

    let jsonl_rel = format!("{}/issues.jsonl", config.folder_name);

    // 3. Stage the JSONL file
    git(&["add", &jsonl_rel], project_path)?;

    // 4. Check if there are staged changes to commit
    let has_changes = git(
        &["diff", "--cached", "--quiet", &jsonl_rel],
        project_path,
    )
    .is_err(); // exit code 1 = there are differences

    if has_changes {
        git(&["commit", "-m", "sync: update issues.jsonl"], project_path)?;
        result.committed = true;
    }

    // 5. Remote operations (pull + push)
    if has_remote(project_path) {
        // Pull with rebase
        match git(&["pull", "--rebase"], project_path) {
            Ok(_) => {
                result.pulled = true;
            }
            Err(e) => {
                // Check for merge conflict on our file
                if e.contains("CONFLICT") || e.contains("conflict") {
                    log::warn!("[tracker/sync] Merge conflict detected: {}", e);
                    result.conflict = true;
                    return Ok(result);
                }
                // Other pull errors (e.g., network) — log but don't fail the whole sync
                log::warn!("[tracker/sync] Pull failed: {}", e);
                // Still try to push if we committed
            }
        }

        // After pull: import remote changes into local DB
        if result.pulled {
            let jsonl_path = project_path.join(&config.folder_name).join("issues.jsonl");
            if jsonl_path.exists() {
                match import::import_all(conn, &jsonl_path) {
                    Ok(import_res) => {
                        result.import_result = Some(import_res.into());
                        // Re-export to normalize after merge
                        let _ = export::export_all(conn, config, project_path);
                    }
                    Err(e) => {
                        log::warn!("[tracker/sync] Import after pull failed: {}", e);
                    }
                }
            }
        }

        // Push
        if result.committed || result.pulled {
            match git(&["push"], project_path) {
                Ok(_) => {
                    result.pushed = true;
                }
                Err(e) => {
                    log::warn!("[tracker/sync] Push failed: {}", e);
                }
            }
        }
    }

    Ok(result)
}

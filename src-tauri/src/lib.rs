use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::Instant;

// Global flags for logging
static LOGGING_ENABLED: AtomicBool = AtomicBool::new(false);
static VERBOSE_LOGGING: AtomicBool = AtomicBool::new(false);

// Sync cooldown: skip redundant syncs within 10 seconds
static LAST_SYNC_TIME: Mutex<Option<Instant>> = Mutex::new(None);
const SYNC_COOLDOWN_SECS: u64 = 10;

// Filesystem mtime tracking for change detection
static LAST_KNOWN_MTIME: Mutex<Option<std::time::SystemTime>> = Mutex::new(None);

// Conditional logging macros
macro_rules! log_info {
    ($($arg:tt)*) => {
        if LOGGING_ENABLED.load(std::sync::atomic::Ordering::Relaxed) {
            log::info!($($arg)*);
        }
    };
}

macro_rules! log_warn {
    ($($arg:tt)*) => {
        if LOGGING_ENABLED.load(std::sync::atomic::Ordering::Relaxed) {
            log::warn!($($arg)*);
        }
    };
}

macro_rules! log_error {
    ($($arg:tt)*) => {
        if LOGGING_ENABLED.load(std::sync::atomic::Ordering::Relaxed) {
            log::error!($($arg)*);
        }
    };
}

macro_rules! log_debug {
    ($($arg:tt)*) => {
        if LOGGING_ENABLED.load(std::sync::atomic::Ordering::Relaxed) && VERBOSE_LOGGING.load(std::sync::atomic::Ordering::Relaxed) {
            log::debug!($($arg)*);
        }
    };
}

// ============================================================================
// Update Checker Types
// ============================================================================

#[derive(Debug, Serialize)]
pub struct UpdateInfo {
    #[serde(rename = "currentVersion")]
    pub current_version: String,
    #[serde(rename = "latestVersion")]
    pub latest_version: String,
    #[serde(rename = "hasUpdate")]
    pub has_update: bool,
    #[serde(rename = "releaseUrl")]
    pub release_url: String,
}

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    html_url: String,
}

// File watcher removed - replaced by frontend polling for lower CPU usage

// ============================================================================
// Types
// ============================================================================

/// Dependency relationship as returned by bd CLI
/// Format: {"issue_id": "...", "depends_on_id": "...", "type": "blocks", "created_at": "...", "created_by": "..."}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BdRawDependency {
    pub issue_id: Option<String>,
    pub depends_on_id: Option<String>,
    #[serde(rename = "type")]
    pub dependency_type: Option<String>,
    pub created_at: Option<String>,
    pub created_by: Option<String>,
}

/// Dependent info (for parent-child relationships with full issue info)
/// Some bd versions may return this format instead
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BdRawDependent {
    pub id: Option<String>,
    pub title: Option<String>,
    pub status: Option<String>,
    pub priority: Option<i32>,
    pub issue_type: Option<String>,
    pub dependency_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BdRawIssue {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub priority: i32,
    pub issue_type: String,
    pub owner: Option<String>,
    pub assignee: Option<String>,
    pub labels: Option<Vec<String>>,
    pub created_at: String,
    pub created_by: Option<String>,
    pub updated_at: String,
    pub closed_at: Option<String>,
    pub close_reason: Option<String>,
    pub blocked_by: Option<Vec<String>>,
    pub blocks: Option<Vec<String>>,
    pub comments: Option<Vec<BdRawComment>>,
    pub external_ref: Option<String>,
    pub estimate: Option<i32>,
    pub design: Option<String>,
    pub acceptance_criteria: Option<String>,
    pub notes: Option<String>,
    pub parent: Option<String>,
    pub dependents: Option<Vec<BdRawDependent>>,
    pub dependencies: Option<Vec<BdRawDependency>>,
    pub dependency_count: Option<i32>,
    pub dependent_count: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BdRawComment {
    pub id: serde_json::Value,
    pub issue_id: Option<String>,
    pub author: String,
    pub text: Option<String>,
    pub content: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Issue {
    pub id: String,
    pub title: String,
    pub description: String,
    #[serde(rename = "type")]
    pub issue_type: String,
    pub status: String,
    pub priority: String,
    pub assignee: Option<String>,
    pub labels: Vec<String>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    #[serde(rename = "closedAt")]
    pub closed_at: Option<String>,
    pub comments: Vec<Comment>,
    #[serde(rename = "blockedBy")]
    pub blocked_by: Option<Vec<String>>,
    pub blocks: Option<Vec<String>>,
    #[serde(rename = "externalRef")]
    pub external_ref: Option<String>,
    #[serde(rename = "estimateMinutes")]
    pub estimate_minutes: Option<i32>,
    #[serde(rename = "designNotes")]
    pub design_notes: Option<String>,
    #[serde(rename = "acceptanceCriteria")]
    pub acceptance_criteria: Option<String>,
    #[serde(rename = "workingNotes")]
    pub working_notes: Option<String>,
    pub parent: Option<ParentIssue>,
    pub children: Option<Vec<ChildIssue>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Comment {
    pub id: String,
    pub author: String,
    pub content: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChildIssue {
    pub id: String,
    pub title: String,
    pub status: String,
    pub priority: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ParentIssue {
    pub id: String,
    pub title: String,
    pub status: String,
    pub priority: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CountResult {
    pub count: usize,
    #[serde(rename = "byType")]
    pub by_type: HashMap<String, usize>,
    #[serde(rename = "byPriority")]
    pub by_priority: HashMap<String, usize>,
    #[serde(rename = "lastUpdated")]
    pub last_updated: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DirectoryEntry {
    pub name: String,
    pub path: String,
    #[serde(rename = "isDirectory")]
    pub is_directory: bool,
    #[serde(rename = "hasBeads")]
    pub has_beads: bool,
}

#[derive(Debug, Serialize)]
pub struct PurgeResult {
    #[serde(rename = "deletedCount")]
    pub deleted_count: usize,
    #[serde(rename = "deletedFolders")]
    pub deleted_folders: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FsListResult {
    #[serde(rename = "currentPath")]
    pub current_path: String,
    #[serde(rename = "hasBeads")]
    pub has_beads: bool,
    pub entries: Vec<DirectoryEntry>,
}

// ============================================================================
// Options structs for commands
// ============================================================================

#[derive(Debug, Deserialize, Default)]
pub struct ListOptions {
    pub status: Option<Vec<String>>,
    #[serde(rename = "type")]
    pub issue_type: Option<Vec<String>>,
    pub priority: Option<Vec<String>>,
    pub assignee: Option<String>,
    #[serde(rename = "includeAll")]
    pub include_all: Option<bool>,
    pub cwd: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct CwdOptions {
    pub cwd: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePayload {
    pub title: String,
    pub description: Option<String>,
    #[serde(rename = "type")]
    pub issue_type: Option<String>,
    pub priority: Option<String>,
    pub assignee: Option<String>,
    pub labels: Option<Vec<String>>,
    #[serde(rename = "externalRef")]
    pub external_ref: Option<String>,
    #[serde(rename = "estimateMinutes")]
    pub estimate_minutes: Option<i32>,
    #[serde(rename = "designNotes")]
    pub design_notes: Option<String>,
    #[serde(rename = "acceptanceCriteria")]
    pub acceptance_criteria: Option<String>,
    #[serde(rename = "workingNotes")]
    pub working_notes: Option<String>,
    pub parent: Option<String>, // Parent epic ID for hierarchical child
    pub cwd: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePayload {
    pub title: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "type")]
    pub issue_type: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub assignee: Option<String>,
    pub labels: Option<Vec<String>>,
    #[serde(rename = "externalRef")]
    pub external_ref: Option<String>,
    #[serde(rename = "estimateMinutes")]
    pub estimate_minutes: Option<i32>,
    #[serde(rename = "designNotes")]
    pub design_notes: Option<String>,
    #[serde(rename = "acceptanceCriteria")]
    pub acceptance_criteria: Option<String>,
    #[serde(rename = "workingNotes")]
    pub working_notes: Option<String>,
    pub parent: Option<String>, // Some("") to detach, Some("id") to attach
    pub cwd: Option<String>,
}

// ============================================================================
// Helpers
// ============================================================================

fn priority_to_string(priority: i32) -> String {
    let p = if (0..=4).contains(&priority) { priority } else { 3 };
    format!("p{}", p)
}

fn priority_to_number(priority: &str) -> String {
    if let Some(caps) = priority.strip_prefix('p') {
        if caps.len() == 1 && caps.chars().next().unwrap_or('x').is_ascii_digit() {
            return caps.to_string();
        }
    }
    "3".to_string()
}

fn normalize_issue_type(issue_type: &str) -> String {
    let valid_types = ["bug", "task", "feature", "epic", "chore"];
    if valid_types.contains(&issue_type) {
        issue_type.to_string()
    } else {
        "task".to_string()
    }
}

fn normalize_issue_status(status: &str) -> String {
    let valid_statuses = ["open", "in_progress", "blocked", "closed"];
    if valid_statuses.contains(&status) {
        status.to_string()
    } else {
        "open".to_string()
    }
}

fn transform_issue(raw: BdRawIssue) -> Issue {
    // Parent info - dependencies array now contains relationship info, not full issue details
    // For now, we just use the parent ID if available
    let parent = raw.parent.as_ref().map(|parent_id| {
        ParentIssue {
            id: parent_id.clone(),
            title: String::new(), // Not available in dependency format
            status: "open".to_string(),
            priority: "p3".to_string(),
        }
    });

    // Extract children from dependents array (with dependency_type: "parent-child")
    let children: Option<Vec<ChildIssue>> = raw.dependents.as_ref().map(|deps| {
        deps.iter()
            .filter(|d| d.dependency_type.as_deref() == Some("parent-child") && d.id.is_some())
            .map(|c| ChildIssue {
                id: c.id.clone().unwrap_or_default(),
                title: c.title.clone().unwrap_or_default(),
                status: normalize_issue_status(&c.status.clone().unwrap_or_else(|| "open".to_string())),
                priority: priority_to_string(c.priority.unwrap_or(3)),
            })
            .collect()
    }).filter(|v: &Vec<ChildIssue>| !v.is_empty());

    Issue {
        id: raw.id,
        title: raw.title,
        description: raw.description.unwrap_or_default(),
        issue_type: normalize_issue_type(&raw.issue_type),
        status: normalize_issue_status(&raw.status),
        priority: priority_to_string(raw.priority),
        assignee: raw.assignee,
        labels: raw.labels.unwrap_or_default(),
        created_at: raw.created_at,
        updated_at: raw.updated_at,
        closed_at: raw.closed_at,
        comments: raw.comments.unwrap_or_default().into_iter().map(|c| {
            Comment {
                id: match c.id {
                    serde_json::Value::Number(n) => n.to_string(),
                    serde_json::Value::String(s) => s,
                    _ => "0".to_string(),
                },
                author: c.author,
                content: c.text.or(c.content).unwrap_or_default(),
                created_at: c.created_at,
            }
        }).collect(),
        blocked_by: raw.blocked_by,
        blocks: raw.blocks,
        external_ref: raw.external_ref,
        estimate_minutes: raw.estimate,
        design_notes: raw.design,
        acceptance_criteria: raw.acceptance_criteria,
        working_notes: raw.notes,
        parent,
        children,
    }
}

/// Parse issues with tolerance for malformed entries
/// Returns all successfully parsed issues and logs failures
fn parse_issues_tolerant(output: &str, context: &str) -> Result<Vec<BdRawIssue>, String> {
    // First try strict parsing
    if let Ok(issues) = serde_json::from_str::<Vec<BdRawIssue>>(output) {
        return Ok(issues);
    }

    // If strict parsing fails, try tolerant parsing
    log_warn!("[{}] Strict parsing failed, attempting tolerant parsing", context);

    let value: serde_json::Value = serde_json::from_str(output)
        .map_err(|e| {
            log_error!("[{}] JSON is completely invalid: {}", context, e);
            format!("Invalid JSON: {}", e)
        })?;

    let arr = value.as_array().ok_or_else(|| {
        log_error!("[{}] Expected array, got: {:?}", context, value);
        "Expected JSON array".to_string()
    })?;

    let mut issues = Vec::new();
    let mut failed_count = 0;

    for (i, obj) in arr.iter().enumerate() {
        let obj_str = serde_json::to_string(obj).unwrap_or_default();
        match serde_json::from_str::<BdRawIssue>(&obj_str) {
            Ok(issue) => issues.push(issue),
            Err(e) => {
                failed_count += 1;
                let id = obj.get("id").and_then(|v| v.as_str()).unwrap_or("unknown");
                log_error!("[{}] Skipping issue {} (id={}): {}", context, i, id, e);

                // Log which fields are present/missing
                if let Some(obj_map) = obj.as_object() {
                    let keys: Vec<&str> = obj_map.keys().map(|s| s.as_str()).collect();
                    log_error!("[{}] Issue {} has keys: {:?}", context, i, keys);

                    // Check for common missing required fields
                    let required = ["id", "title", "status", "priority", "issue_type", "created_at", "updated_at"];
                    let missing: Vec<&&str> = required.iter().filter(|k| !keys.contains(*k)).collect();
                    if !missing.is_empty() {
                        log_error!("[{}] Issue {} missing required fields: {:?}", context, i, missing);
                    }
                }
            }
        }
    }

    if failed_count > 0 {
        log_warn!("[{}] Parsed {} issues, skipped {} malformed entries", context, issues.len(), failed_count);
    }

    Ok(issues)
}

fn get_extended_path() -> String {
    let home = env::var("HOME").unwrap_or_default();
    let extra_paths = vec![
        "/opt/homebrew/bin".to_string(),
        "/usr/local/bin".to_string(),
        "/usr/bin".to_string(),
        "/bin".to_string(),
        format!("{}/.local/bin", home),
        format!("{}/bin", home),
    ];
    let current_path = env::var("PATH").unwrap_or_default();
    let mut all_paths = extra_paths;
    all_paths.extend(current_path.split(':').map(String::from));
    all_paths.join(":")
}

/// Creates a Command with platform-specific flags.
/// On Windows, sets CREATE_NO_WINDOW to prevent console popups.
fn new_command(program: &str) -> Command {
    #[allow(unused_mut)]
    let mut cmd = Command::new(program);
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }
    cmd
}

fn execute_bd(command: &str, args: &[String], cwd: Option<&str>) -> Result<String, String> {
    let working_dir = cwd
        .map(String::from)
        .or_else(|| env::var("BEADS_PATH").ok())
        .unwrap_or_else(|| {
            env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| ".".to_string())
        });

    // Split command by spaces to handle subcommands like "comments add"
    let mut full_args: Vec<&str> = command.split_whitespace().collect();
    for arg in args {
        full_args.push(arg);
    }
    full_args.push("--no-daemon");
    full_args.push("--json");

    log_info!("[bd] bd {} | cwd: {}", full_args.join(" "), working_dir);

    let output = new_command("bd")
        .args(&full_args)
        .current_dir(&working_dir)
        .env("PATH", get_extended_path())
        .env("BEADS_PATH", &working_dir)
        .output()
        .map_err(|e| {
            log_error!("[bd] Failed to execute bd: {}", e);
            format!("Failed to execute bd: {}", e)
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        log_error!("[bd] Command failed | status: {} | stderr: {}", output.status, stderr);

        // Detect schema migration failure (bd 0.49.4 migration bug)
        if stderr.contains("no such column: spec_id") {
            log_error!("[bd] Schema migration failure detected - database needs repair");
            return Err("SCHEMA_MIGRATION_ERROR: Database schema is incompatible. Please use the repair function to fix this issue.".to_string());
        }

        if !stderr.is_empty() {
            return Err(stderr.to_string());
        }
        return Err(format!("bd command failed with status: {}", output.status));
    }

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    log_info!("[bd] OK | {} bytes", stdout.len());

    // Log output preview only if verbose mode is enabled
    if VERBOSE_LOGGING.load(Ordering::Relaxed) {
        let preview: String = stdout.chars().take(500).collect();
        log_debug!("[bd] Output: {}", preview);
    }

    Ok(stdout)
}

/// Sync the beads database before read operations to ensure data is up-to-date
/// Uses bidirectional sync to preserve local changes while getting remote updates
/// Has a cooldown to avoid redundant syncs within the same poll cycle
fn sync_bd_database(cwd: Option<&str>) {
    // Check cooldown — skip if synced recently
    {
        let last = LAST_SYNC_TIME.lock().unwrap();
        if let Some(t) = *last {
            if t.elapsed().as_secs() < SYNC_COOLDOWN_SECS {
                log_info!("[sync] Skipping — cooldown active ({:.1}s ago)", t.elapsed().as_secs_f32());
                return;
            }
        }
    }

    let working_dir = cwd
        .map(String::from)
        .or_else(|| env::var("BEADS_PATH").ok())
        .unwrap_or_else(|| {
            env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| ".".to_string())
        });

    log_info!("[sync] Starting bidirectional sync for: {}", working_dir);

    // Run bd sync (bidirectional - exports local changes AND imports remote changes)
    match new_command("bd")
        .args(["sync", "--no-daemon"])
        .current_dir(&working_dir)
        .env("PATH", get_extended_path())
        .env("BEADS_PATH", &working_dir)
        .output()
    {
        Ok(output) if output.status.success() => {
            log_info!("[sync] Sync completed successfully");
            // Update cooldown timestamp
            let mut last = LAST_SYNC_TIME.lock().unwrap();
            *last = Some(Instant::now());
        }
        Ok(output) => {
            log_warn!(
                "[sync] bd sync failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        Err(e) => {
            log_error!("[sync] Failed to run bd sync: {}", e);
        }
    }
}

// ============================================================================
// Tauri Commands
// ============================================================================

#[tauri::command]
async fn bd_sync(cwd: Option<String>) -> Result<(), String> {
    let working_dir = cwd
        .or_else(|| env::var("BEADS_PATH").ok())
        .unwrap_or_else(|| {
            env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| ".".to_string())
        });

    log_info!("[bd_sync] Manual sync requested for: {}", working_dir);

    let output = new_command("bd")
        .args(["sync", "--no-daemon"])
        .current_dir(&working_dir)
        .env("PATH", get_extended_path())
        .env("BEADS_PATH", &working_dir)
        .output()
        .map_err(|e| format!("Failed to run bd sync: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        log_error!("[bd_sync] Sync failed: {}", stderr.trim());
        return Err(format!("Sync failed: {}", stderr.trim()));
    }

    log_info!("[bd_sync] Sync completed successfully");
    // Reset cooldown so subsequent reads pick up the fresh sync
    let mut last = LAST_SYNC_TIME.lock().unwrap();
    *last = Some(Instant::now());
    Ok(())
}

#[derive(Debug, serde::Serialize)]
struct RepairResult {
    success: bool,
    message: String,
    backup_path: Option<String>,
}

#[tauri::command]
async fn bd_repair_database(cwd: Option<String>) -> Result<RepairResult, String> {
    let working_dir = cwd
        .or_else(|| env::var("BEADS_PATH").ok())
        .unwrap_or_else(|| {
            env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| ".".to_string())
        });

    log_info!("[bd_repair] Starting database repair for: {}", working_dir);

    let beads_dir = std::path::Path::new(&working_dir).join(".beads");
    let db_path = beads_dir.join("beads.db");
    let jsonl_path = beads_dir.join("issues.jsonl");
    let backup_path = beads_dir.join("beads.db.backup");

    // Check if .beads directory exists
    if !beads_dir.exists() {
        return Err("No .beads directory found in this project".to_string());
    }

    // Check if database exists
    if !db_path.exists() {
        return Ok(RepairResult {
            success: true,
            message: "No database to repair - it will be created on next operation".to_string(),
            backup_path: None,
        });
    }

    // Check if issues.jsonl exists and has content
    let jsonl_size = std::fs::metadata(&jsonl_path)
        .map(|m| m.len())
        .unwrap_or(0);

    if !jsonl_path.exists() || jsonl_size == 0 {
        return Err("Cannot repair: issues.jsonl is missing or empty. Your data would be lost.".to_string());
    }

    // Create backup of current database
    if let Err(e) = std::fs::copy(&db_path, &backup_path) {
        log_error!("[bd_repair] Failed to create backup: {}", e);
        return Err(format!("Failed to create backup: {}", e));
    }
    log_info!("[bd_repair] Backup created at: {:?}", backup_path);

    // Remove database files
    std::fs::remove_file(&db_path).ok();
    std::fs::remove_file(beads_dir.join("beads.db-shm")).ok();
    std::fs::remove_file(beads_dir.join("beads.db-wal")).ok();
    log_info!("[bd_repair] Removed old database files");

    // Test that bd can now work (it will recreate the database)
    let test_output = new_command("bd")
        .args(["list", "--limit=1", "--no-daemon", "--json"])
        .current_dir(&working_dir)
        .env("PATH", get_extended_path())
        .env("BEADS_PATH", &working_dir)
        .output();

    match test_output {
        Ok(output) if output.status.success() => {
            log_info!("[bd_repair] Repair successful - database recreated");
            Ok(RepairResult {
                success: true,
                message: "Database repaired successfully. Your issues have been restored from the backup file.".to_string(),
                backup_path: Some(backup_path.to_string_lossy().to_string()),
            })
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            log_error!("[bd_repair] Repair verification failed: {}", stderr);
            Err(format!("Repair failed during verification: {}", stderr))
        }
        Err(e) => {
            log_error!("[bd_repair] Failed to verify repair: {}", e);
            Err(format!("Failed to verify repair: {}", e))
        }
    }
}

// ============================================================================
// Batched Poll Data
// ============================================================================

/// All data needed for a single poll cycle, fetched in one IPC call.
#[derive(Debug, Serialize)]
pub struct PollData {
    #[serde(rename = "openIssues")]
    pub open_issues: Vec<Issue>,
    #[serde(rename = "closedIssues")]
    pub closed_issues: Vec<Issue>,
    #[serde(rename = "readyIssues")]
    pub ready_issues: Vec<Issue>,
}

/// Batched poll: sync once, then fetch open + closed + ready issues in sequence.
/// Replaces 3 separate IPC calls (bd_list + bd_list(closed) + bd_ready) with one.
#[tauri::command]
async fn bd_poll_data(cwd: Option<String>) -> Result<PollData, String> {
    log_info!("[bd_poll_data] Batched poll starting");

    let cwd_ref = cwd.as_deref();

    // Single sync for the entire poll cycle
    sync_bd_database(cwd_ref);

    // Fetch open issues (no --status flag = non-closed)
    let open_output = execute_bd("list", &["--limit=0".to_string()], cwd_ref)?;
    let raw_open = parse_issues_tolerant(&open_output, "bd_poll_data_open")?;

    // Fetch closed issues
    let closed_output = execute_bd("list", &["--status=closed".to_string(), "--limit=0".to_string()], cwd_ref)?;
    let raw_closed = parse_issues_tolerant(&closed_output, "bd_poll_data_closed")?;

    // Fetch ready issues
    let ready_output = execute_bd("ready", &[], cwd_ref)?;
    let raw_ready = parse_issues_tolerant(&ready_output, "bd_poll_data_ready")?;

    log_info!("[bd_poll_data] Batched poll done: {} open, {} closed, {} ready",
        raw_open.len(), raw_closed.len(), raw_ready.len());

    // Update mtime AFTER our commands ran, so the next bd_check_changed
    // only detects EXTERNAL changes (not our own poll's side effects)
    {
        let working_dir = cwd_ref
            .map(String::from)
            .or_else(|| env::var("BEADS_PATH").ok())
            .unwrap_or_else(|| {
            env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| ".".to_string())
        });
        let beads_dir = std::path::Path::new(&working_dir).join(".beads");

        if let Some(mtime) = get_beads_mtime(&beads_dir) {
            let mut last = LAST_KNOWN_MTIME.lock().unwrap();
            *last = Some(mtime);
        }
    }

    Ok(PollData {
        open_issues: raw_open.into_iter().map(transform_issue).collect(),
        closed_issues: raw_closed.into_iter().map(transform_issue).collect(),
        ready_issues: raw_ready.into_iter().map(transform_issue).collect(),
    })
}

/// Get the latest mtime across all beads database files (db, WAL, jsonl).
/// SQLite WAL mode writes to beads.db-wal, so the main .db file mtime may not change.
fn get_beads_mtime(beads_dir: &std::path::Path) -> Option<std::time::SystemTime> {
    let paths = [
        beads_dir.join("beads.db"),
        beads_dir.join("beads.db-wal"),
        beads_dir.join("issues.jsonl"),
    ];
    paths.iter()
        .filter_map(|p| fs::metadata(p).and_then(|m| m.modified()).ok())
        .max()
}

/// Check if the beads database has changed since last check (via filesystem mtime).
/// Returns true if changes detected or if this is the first check.
/// This is extremely cheap — just a few stat() calls, no bd process spawns.
#[tauri::command]
async fn bd_check_changed(cwd: Option<String>) -> Result<bool, String> {
    let working_dir = cwd
        .or_else(|| env::var("BEADS_PATH").ok())
        .unwrap_or_else(|| {
            env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| ".".to_string())
        });

    let beads_dir = std::path::Path::new(&working_dir).join(".beads");
    let current_mtime = get_beads_mtime(&beads_dir);

    let mut last = LAST_KNOWN_MTIME.lock().unwrap();

    match (current_mtime, *last) {
        (Some(current), Some(previous)) => {
            if current != previous {
                log_info!("[bd_check_changed] mtime changed — data may have been modified");
                *last = Some(current);
                Ok(true)
            } else {
                log_debug!("[bd_check_changed] mtime unchanged — no changes");
                Ok(false)
            }
        }
        (Some(current), None) => {
            // First check — store mtime, report changed so initial load happens
            *last = Some(current);
            Ok(true)
        }
        (None, _) => {
            // No database file found
            log_warn!("[bd_check_changed] No beads database found in {}", working_dir);
            Ok(true) // Report changed to let caller handle missing db
        }
    }
}

#[tauri::command]
async fn bd_list(options: ListOptions) -> Result<Vec<Issue>, String> {
    log_info!("[bd_list] cwd: {:?}", options.cwd);

    // Sync database before reading to ensure data is up-to-date
    sync_bd_database(options.cwd.as_deref());

    let mut args: Vec<String> = Vec::new();

    if options.include_all.unwrap_or(false) {
        args.push("--all".to_string());
    }
    if let Some(ref statuses) = options.status {
        if !statuses.is_empty() {
            args.push(format!("--status={}", statuses.join(",")));
        }
    }
    if let Some(ref types) = options.issue_type {
        if !types.is_empty() {
            args.push(format!("--type={}", types.join(",")));
        }
    }
    if let Some(ref priorities) = options.priority {
        if !priorities.is_empty() {
            let nums: Vec<String> = priorities.iter().map(|p| priority_to_number(p)).collect();
            args.push(format!("--priority={}", nums.join(",")));
        }
    }
    if let Some(ref assignee) = options.assignee {
        args.push(format!("--assignee={}", assignee));
    }

    // Always disable limit to get all issues (bd defaults to 50)
    args.push("--limit=0".to_string());

    let output = execute_bd("list", &args, options.cwd.as_deref())?;

    let raw_issues = parse_issues_tolerant(&output, "bd_list")?;

    log_info!("[bd_list] Found {} issues", raw_issues.len());
    Ok(raw_issues.into_iter().map(transform_issue).collect())
}

#[tauri::command]
async fn bd_count(options: CwdOptions) -> Result<CountResult, String> {
    // Sync database before reading to ensure data is up-to-date
    sync_bd_database(options.cwd.as_deref());

    // Fetch both open and closed issues to match fetchIssues behavior
    // Use --limit=0 to get all issues (bd defaults to 50)
    let open_output = execute_bd("list", &["--limit=0".to_string()], options.cwd.as_deref())?;
    let closed_output = execute_bd("list", &["--status=closed".to_string(), "--limit=0".to_string()], options.cwd.as_deref())?;

    let open_issues = parse_issues_tolerant(&open_output, "bd_count_open")?;
    let closed_issues = parse_issues_tolerant(&closed_output, "bd_count_closed")?;

    // Combine all issues
    let mut raw_issues = open_issues;
    raw_issues.extend(closed_issues);

    let mut by_type: HashMap<String, usize> = HashMap::new();
    by_type.insert("bug".to_string(), 0);
    by_type.insert("task".to_string(), 0);
    by_type.insert("feature".to_string(), 0);
    by_type.insert("epic".to_string(), 0);
    by_type.insert("chore".to_string(), 0);

    let mut by_priority: HashMap<String, usize> = HashMap::new();
    by_priority.insert("p0".to_string(), 0);
    by_priority.insert("p1".to_string(), 0);
    by_priority.insert("p2".to_string(), 0);
    by_priority.insert("p3".to_string(), 0);
    by_priority.insert("p4".to_string(), 0);

    let mut last_updated: Option<String> = None;

    for issue in &raw_issues {
        let issue_type = issue.issue_type.to_lowercase();
        if by_type.contains_key(&issue_type) {
            *by_type.get_mut(&issue_type).unwrap() += 1;
        }

        let priority_key = format!("p{}", issue.priority);
        if by_priority.contains_key(&priority_key) {
            *by_priority.get_mut(&priority_key).unwrap() += 1;
        }

        if last_updated.is_none() || issue.updated_at > *last_updated.as_ref().unwrap() {
            last_updated = Some(issue.updated_at.clone());
        }
    }

    Ok(CountResult {
        count: raw_issues.len(),
        by_type,
        by_priority,
        last_updated,
    })
}

#[tauri::command]
async fn bd_ready(options: CwdOptions) -> Result<Vec<Issue>, String> {
    log_info!("[bd_ready] Called with cwd: {:?}", options.cwd);

    // Sync database before reading to ensure data is up-to-date
    sync_bd_database(options.cwd.as_deref());

    let output = execute_bd("ready", &[], options.cwd.as_deref())?;

    let raw_issues = parse_issues_tolerant(&output, "bd_ready")?;

    log_info!("[bd_ready] Found {} ready issues", raw_issues.len());
    Ok(raw_issues.into_iter().map(transform_issue).collect())
}

#[tauri::command]
async fn bd_status(options: CwdOptions) -> Result<serde_json::Value, String> {
    let output = execute_bd("status", &[], options.cwd.as_deref())?;

    serde_json::from_str(&output)
        .map_err(|e| format!("Failed to parse status: {}", e))
}

#[tauri::command]
async fn bd_show(id: String, options: CwdOptions) -> Result<Option<Issue>, String> {
    log_info!("[bd_show] Called for issue: {} with cwd: {:?}", id, options.cwd);

    // Sync database before reading to ensure data is up-to-date
    sync_bd_database(options.cwd.as_deref());

    let output = execute_bd("show", std::slice::from_ref(&id), options.cwd.as_deref())?;

    // bd show can return either a single object or an array
    let result: serde_json::Value = serde_json::from_str(&output)
        .map_err(|e| {
            log_error!("[bd_show] Failed to parse JSON for {}: {}", id, e);
            format!("Failed to parse issue: {}", e)
        })?;

    let raw_issue: Option<BdRawIssue> = if result.is_array() {
        result.as_array()
            .and_then(|arr| arr.first())
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    } else {
        serde_json::from_value(result).ok()
    };

    log_info!("[bd_show] Issue {} found: {}", id, raw_issue.is_some());
    Ok(raw_issue.map(transform_issue))
}

#[tauri::command]
async fn bd_create(payload: CreatePayload) -> Result<Option<Issue>, String> {
    log_info!("[bd_create] Creating issue: {:?}", payload.title);
    let mut args: Vec<String> = vec![payload.title.clone()];

    if let Some(ref desc) = payload.description {
        args.push("--description".to_string());
        args.push(desc.clone());
    }
    if let Some(ref t) = payload.issue_type {
        args.push("--type".to_string());
        args.push(t.clone());
    }
    if let Some(ref p) = payload.priority {
        args.push("--priority".to_string());
        args.push(priority_to_number(p));
    }
    if let Some(ref a) = payload.assignee {
        args.push("--assignee".to_string());
        args.push(a.clone());
    }
    if let Some(ref labels) = payload.labels {
        if !labels.is_empty() {
            args.push("--labels".to_string());
            args.push(labels.join(","));
        }
    }
    if let Some(ref ext) = payload.external_ref {
        args.push("--external-ref".to_string());
        args.push(ext.clone());
    }
    if let Some(est) = payload.estimate_minutes {
        args.push("--estimate".to_string());
        args.push(est.to_string());
    }
    if let Some(ref design) = payload.design_notes {
        args.push("--design".to_string());
        args.push(design.clone());
    }
    if let Some(ref acc) = payload.acceptance_criteria {
        args.push("--acceptance".to_string());
        args.push(acc.clone());
    }
    if let Some(ref notes) = payload.working_notes {
        args.push("--notes".to_string());
        args.push(notes.clone());
    }
    if let Some(ref parent) = payload.parent {
        if !parent.is_empty() {
            args.push("--parent".to_string());
            args.push(parent.clone());
        }
    }

    let output = execute_bd("create", &args, payload.cwd.as_deref())?;

    let raw_issue: BdRawIssue = serde_json::from_str(&output)
        .map_err(|e| format!("Failed to parse created issue: {}", e))?;

    Ok(Some(transform_issue(raw_issue)))
}

#[tauri::command]
async fn bd_update(id: String, updates: UpdatePayload) -> Result<Option<Issue>, String> {
    // Always log update calls for debugging (regardless of LOGGING_ENABLED)
    log::info!("[bd_update] Updating issue: {} with cwd: {:?}", id, updates.cwd);
    log::info!("[bd_update] Updates: status={:?}, title={:?}, type={:?}", updates.status, updates.title, updates.issue_type);

    let mut args: Vec<String> = vec![id.clone()];

    if let Some(ref title) = updates.title {
        args.push("--title".to_string());
        args.push(title.clone());
    }
    if let Some(ref desc) = updates.description {
        args.push("--description".to_string());
        args.push(desc.clone());
    }
    if let Some(ref t) = updates.issue_type {
        args.push("--type".to_string());
        args.push(t.clone());
    }
    if let Some(ref s) = updates.status {
        args.push("--status".to_string());
        args.push(s.clone());
    }
    if let Some(ref p) = updates.priority {
        args.push("--priority".to_string());
        args.push(priority_to_number(p));
    }
    if let Some(ref a) = updates.assignee {
        args.push("--assignee".to_string());
        args.push(a.clone());
    }
    if let Some(ref labels) = updates.labels {
        args.push("--set-labels".to_string());
        args.push(labels.join(","));
    }
    if let Some(ref ext) = updates.external_ref {
        args.push("--external-ref".to_string());
        if ext.is_empty() {
            // Use issue ID as unique sentinel to satisfy UNIQUE constraint
            // Frontend filters out "cleared:" prefixes for display
            args.push(format!("cleared:{}", id));
        } else {
            args.push(ext.clone());
        }
    }
    if let Some(est) = updates.estimate_minutes {
        args.push("--estimate".to_string());
        args.push(est.to_string());
    }
    if let Some(ref design) = updates.design_notes {
        args.push("--design".to_string());
        args.push(design.clone());
    }
    if let Some(ref acc) = updates.acceptance_criteria {
        args.push("--acceptance".to_string());
        args.push(acc.clone());
    }
    if let Some(ref notes) = updates.working_notes {
        args.push("--notes".to_string());
        args.push(notes.clone());
    }
    if let Some(ref parent) = updates.parent {
        args.push("--parent".to_string());
        args.push(parent.clone());
    }

    log::info!("[bd_update] Executing: bd update {}", args.join(" "));
    let output = execute_bd("update", &args, updates.cwd.as_deref())?;

    log::info!("[bd_update] Raw output: {}", output.chars().take(500).collect::<String>());

    // Handle empty output from bd CLI (some updates return empty response)
    let trimmed_output = output.trim();
    if trimmed_output.is_empty() {
        log::info!("[bd_update] Empty response from bd, fetching issue {} to get updated data", id);
        // Fetch the updated issue directly
        let show_output = execute_bd("show", std::slice::from_ref(&id), updates.cwd.as_deref())?;
        let show_result: serde_json::Value = serde_json::from_str(&show_output)
            .map_err(|e| {
                log::error!("[bd_update] Failed to parse show JSON: {}", e);
                format!("Failed to fetch updated issue: {}", e)
            })?;

        let raw_issue: Option<BdRawIssue> = if show_result.is_array() {
            show_result.as_array()
                .and_then(|arr| arr.first())
                .and_then(|v| serde_json::from_value(v.clone()).ok())
        } else {
            serde_json::from_value(show_result).ok()
        };

        return Ok(raw_issue.map(transform_issue));
    }

    // bd update can return either a single object or an array
    let result: serde_json::Value = serde_json::from_str(trimmed_output)
        .map_err(|e| {
            log::error!("[bd_update] Failed to parse JSON: {}", e);
            format!("Failed to parse updated issue: {}", e)
        })?;

    let raw_issue: Option<BdRawIssue> = if result.is_array() {
        log::info!("[bd_update] Result is array");
        result.as_array()
            .and_then(|arr| arr.first())
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    } else {
        log::info!("[bd_update] Result is object");
        serde_json::from_value(result.clone()).map_err(|e| {
            log::error!("[bd_update] Failed to parse issue from result: {}", e);
            e
        }).ok()
    };

    if let Some(ref issue) = raw_issue {
        log::info!("[bd_update] Updated issue {} - new status: {}", id, issue.status);
    } else {
        log::warn!("[bd_update] Could not parse updated issue from response");
    }

    Ok(raw_issue.map(transform_issue))
}

#[tauri::command]
async fn bd_close(id: String, options: CwdOptions) -> Result<serde_json::Value, String> {
    log_info!("[bd_close] Closing issue: {} with cwd: {:?}", id, options.cwd);

    let output = execute_bd("close", std::slice::from_ref(&id), options.cwd.as_deref())?;

    log_info!("[bd_close] Raw output: {}", output.chars().take(500).collect::<String>());

    let result: serde_json::Value = serde_json::from_str(&output)
        .map_err(|e| {
            log_error!("[bd_close] Failed to parse JSON: {}", e);
            format!("Failed to parse close result: {}", e)
        })?;

    log_info!("[bd_close] Issue {} closed successfully", id);
    Ok(result)
}

#[tauri::command]
async fn bd_delete(id: String, options: CwdOptions) -> Result<serde_json::Value, String> {
    log::info!("[bd_delete] Deleting issue: {} with --force --hard", id);
    execute_bd("delete", &[id.clone(), "--force".to_string(), "--hard".to_string()], options.cwd.as_deref())?;

    // Sync after delete to push deletion to remote and prevent resurrection
    sync_bd_database(options.cwd.as_deref());

    // Clean up attachments folder for this issue
    let project_path = options.cwd.as_deref().unwrap_or(".");
    let abs_project_path = if project_path == "." || project_path.is_empty() {
        env::current_dir().ok()
    } else {
        let p = PathBuf::from(project_path);
        if p.is_relative() {
            env::current_dir().ok().map(|cwd| cwd.join(&p))
        } else {
            Some(p)
        }
    };

    if let Some(path) = abs_project_path {
        if let Ok(abs_path) = path.canonicalize() {
            let attachments_dir = abs_path.join(".beads").join("attachments").join(&id);
            if attachments_dir.exists() && attachments_dir.is_dir() {
                if let Err(e) = fs::remove_dir_all(&attachments_dir) {
                    log::warn!("[bd_delete] Failed to remove attachments folder: {}", e);
                } else {
                    log::info!("[bd_delete] Removed attachments folder: {:?}", attachments_dir);
                }
            }
        }
    }

    Ok(serde_json::json!({ "success": true, "id": id }))
}

#[tauri::command]
async fn bd_comments_add(id: String, content: String, options: CwdOptions) -> Result<serde_json::Value, String> {
    let args = vec![id, content];

    execute_bd("comments add", &args, options.cwd.as_deref())?;

    Ok(serde_json::json!({ "success": true }))
}

#[tauri::command]
async fn fs_exists(path: String) -> Result<bool, String> {
    Ok(std::path::Path::new(&path).exists())
}

#[tauri::command]
async fn fs_list(path: Option<String>) -> Result<FsListResult, String> {
    use std::fs;

    let target_path = match path {
        Some(p) if p == "~" => dirs::home_dir().unwrap_or_else(|| PathBuf::from("/")),
        Some(p) => PathBuf::from(p),
        None => dirs::home_dir().unwrap_or_else(|| PathBuf::from("/")),
    };

    let target_path = target_path.canonicalize()
        .map_err(|e| format!("Cannot resolve path: {}", e))?;

    let entries = fs::read_dir(&target_path)
        .map_err(|e| format!("Cannot read directory: {}", e))?;

    let mut directories: Vec<DirectoryEntry> = Vec::new();

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        let name = entry.file_name().to_string_lossy().to_string();

        // Skip hidden files
        if name.starts_with('.') {
            continue;
        }

        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };

        if metadata.is_dir() {
            let full_path = entry.path();
            let beads_path = full_path.join(".beads");
            let has_beads = beads_path.is_dir();

            directories.push(DirectoryEntry {
                name,
                path: full_path.to_string_lossy().to_string(),
                is_directory: true,
                has_beads,
            });
        }
    }

    // Sort: beads projects first, then alphabetically
    directories.sort_by(|a, b| {
        match (a.has_beads, b.has_beads) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        }
    });

    let current_has_beads = target_path.join(".beads").is_dir();

    Ok(FsListResult {
        current_path: target_path.to_string_lossy().to_string(),
        has_beads: current_has_beads,
        entries: directories,
    })
}

// File watcher commands removed - replaced by frontend polling for lower CPU usage

// ============================================================================
// Update Checker
// ============================================================================

const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const GITHUB_RELEASES_URL: &str = "https://api.github.com/repos/w3dev33/beads-task-issue-tracker/releases/latest";

fn compare_versions(current: &str, latest: &str) -> bool {
    // Remove 'v' prefix if present
    let current = current.trim_start_matches('v');
    let latest = latest.trim_start_matches('v');

    let parse_version = |v: &str| -> Vec<u32> {
        v.split('.')
            .filter_map(|s| s.parse::<u32>().ok())
            .collect()
    };

    let current_parts = parse_version(current);
    let latest_parts = parse_version(latest);

    for i in 0..3 {
        let c = current_parts.get(i).copied().unwrap_or(0);
        let l = latest_parts.get(i).copied().unwrap_or(0);
        if l > c {
            return true;
        }
        if c > l {
            return false;
        }
    }
    false
}

#[tauri::command]
async fn check_for_updates() -> Result<UpdateInfo, String> {
    let client = reqwest::Client::builder()
        .user_agent("beads-task-issue-tracker")
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let response = client
        .get(GITHUB_RELEASES_URL)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch releases: {}", e))?;

    // Handle 404 (no published releases yet)
    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(UpdateInfo {
            current_version: CURRENT_VERSION.to_string(),
            latest_version: CURRENT_VERSION.to_string(),
            has_update: false,
            release_url: "https://github.com/w3dev33/beads-task-issue-tracker/releases".to_string(),
        });
    }

    if !response.status().is_success() {
        return Err(format!("GitHub API returned status: {}", response.status()));
    }

    let release: GitHubRelease = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse release info: {}", e))?;

    let latest_version = release.tag_name.trim_start_matches('v').to_string();
    let has_update = compare_versions(CURRENT_VERSION, &latest_version);

    Ok(UpdateInfo {
        current_version: CURRENT_VERSION.to_string(),
        latest_version,
        has_update,
        release_url: release.html_url,
    })
}

// ============================================================================
// Debug / Logging Commands
// ============================================================================

fn get_log_path() -> PathBuf {
    let home = env::var("HOME").unwrap_or_default();
    PathBuf::from(home)
        .join("Library/Logs/com.beads.manager/beads.log")
}

#[tauri::command]
async fn get_logging_enabled() -> bool {
    LOGGING_ENABLED.load(Ordering::Relaxed)
}

#[tauri::command]
async fn set_logging_enabled(enabled: bool) {
    LOGGING_ENABLED.store(enabled, Ordering::Relaxed);
    if enabled {
        log_info!("[debug] Logging enabled");
    }
}

#[tauri::command]
async fn get_verbose_logging() -> bool {
    VERBOSE_LOGGING.load(Ordering::Relaxed)
}

#[tauri::command]
async fn set_verbose_logging(enabled: bool) {
    VERBOSE_LOGGING.store(enabled, Ordering::Relaxed);
    log_info!("[debug] Verbose logging: {}", if enabled { "ON" } else { "OFF" });
}

#[tauri::command]
async fn clear_logs() -> Result<(), String> {
    let log_path = get_log_path();
    if log_path.exists() {
        fs::write(&log_path, "").map_err(|e| format!("Failed to clear logs: {}", e))?;
        log_info!("[debug] Logs cleared");
    }
    Ok(())
}

#[tauri::command]
async fn export_logs() -> Result<String, String> {
    let log_path = get_log_path();
    if !log_path.exists() {
        return Err("No logs to export".to_string());
    }

    // Get export folder: Downloads > Documents > Home
    let export_dir = dirs::download_dir()
        .or_else(dirs::document_dir)
        .or_else(dirs::home_dir)
        .ok_or_else(|| "Could not find a folder to export logs".to_string())?;

    // Generate filename with timestamp
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let export_filename = format!("beads-logs-{}.log", now);
    let export_path = export_dir.join(&export_filename);

    // Copy log file
    fs::copy(&log_path, &export_path)
        .map_err(|e| format!("Failed to export logs: {}", e))?;

    Ok(export_path.to_string_lossy().to_string())
}

#[tauri::command]
async fn read_logs(tail_lines: Option<usize>) -> Result<String, String> {
    let log_path = get_log_path();
    if !log_path.exists() {
        return Ok(String::new());
    }

    let content = fs::read_to_string(&log_path)
        .map_err(|e| format!("Failed to read logs: {}", e))?;

    // If tail_lines is specified, return only the last N lines
    if let Some(n) = tail_lines {
        let lines: Vec<&str> = content.lines().collect();
        let start = if lines.len() > n { lines.len() - n } else { 0 };
        Ok(lines[start..].join("\n"))
    } else {
        Ok(content)
    }
}

#[tauri::command]
async fn get_log_path_string() -> String {
    get_log_path().to_string_lossy().to_string()
}

#[tauri::command]
async fn log_frontend(level: String, message: String) {
    match level.as_str() {
        "error" => log::error!("[frontend] {}", message),
        "warn" => log::warn!("[frontend] {}", message),
        _ => log::info!("[frontend] {}", message),
    }
}

#[tauri::command]
async fn get_bd_version() -> String {
    match new_command("bd")
        .arg("--version")
        .env("PATH", get_extended_path())
        .output()
    {
        Ok(output) if output.status.success() => {
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        }
        _ => "bd not found".to_string(),
    }
}

#[tauri::command]
async fn open_image_file(path: String) -> Result<(), String> {
    log_info!("[open_image_file] Opening: {}", path);

    // Security: Only allow image file extensions
    let allowed_extensions = ["png", "jpg", "jpeg", "gif", "webp", "bmp", "svg", "ico", "tiff", "tif"];
    let path_lower = path.to_lowercase();
    let is_image = allowed_extensions.iter().any(|ext| path_lower.ends_with(&format!(".{}", ext)));

    if !is_image {
        return Err("Only image files are allowed".to_string());
    }

    // Verify file exists
    if !std::path::Path::new(&path).exists() {
        return Err(format!("File not found: {}", path));
    }

    // Security: Canonicalize to resolve symlinks/.. and verify inside .beads/attachments/
    let canonical = std::path::Path::new(&path).canonicalize()
        .map_err(|e| format!("Failed to resolve path: {}", e))?;
    let canonical_str = canonical.to_string_lossy();
    if !canonical_str.contains("/.beads/attachments/") {
        log_warn!("[open_image_file] Refusing to open file outside attachments: {} (resolved: {})", path, canonical_str);
        return Err("Can only open files inside .beads/attachments/".to_string());
    }

    // Use platform-specific command to open file with default application
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("Failed to open file: {}", e))?;
    }

    #[cfg(target_os = "windows")]
    {
        new_command("cmd")
            .args(["/C", "start", "", &path])
            .spawn()
            .map_err(|e| format!("Failed to open file: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("Failed to open file: {}", e))?;
    }

    Ok(())
}

#[derive(Debug, Serialize)]
pub struct ImageData {
    pub base64: String,
    pub mime_type: String,
}

#[tauri::command]
async fn read_image_file(path: String) -> Result<ImageData, String> {
    log_info!("[read_image_file] Reading: {}", path);

    // Security: Only allow image file extensions
    let allowed_extensions: &[(&str, &str)] = &[
        ("png", "image/png"),
        ("jpg", "image/jpeg"),
        ("jpeg", "image/jpeg"),
        ("gif", "image/gif"),
        ("webp", "image/webp"),
        ("bmp", "image/bmp"),
        ("svg", "image/svg+xml"),
        ("ico", "image/x-icon"),
        ("tiff", "image/tiff"),
        ("tif", "image/tiff"),
    ];

    let path_lower = path.to_lowercase();
    let mime_type = allowed_extensions
        .iter()
        .find(|(ext, _)| path_lower.ends_with(&format!(".{}", ext)))
        .map(|(_, mime)| *mime);

    let mime_type = match mime_type {
        Some(m) => m.to_string(),
        None => return Err("Only image files are allowed".to_string()),
    };

    // Verify file exists
    if !std::path::Path::new(&path).exists() {
        return Err(format!("File not found: {}", path));
    }

    // Security: Canonicalize to resolve symlinks/.. and verify inside .beads/attachments/
    let canonical = std::path::Path::new(&path).canonicalize()
        .map_err(|e| format!("Failed to resolve path: {}", e))?;
    let canonical_str = canonical.to_string_lossy();
    if !canonical_str.contains("/.beads/attachments/") {
        log_warn!("[read_image_file] Refusing to read file outside attachments: {} (resolved: {})", path, canonical_str);
        return Err("Can only read files inside .beads/attachments/".to_string());
    }

    // Read file and encode as base64
    let data = fs::read(&path).map_err(|e| format!("Failed to read file: {}", e))?;
    let base64 = base64_encode(&data);

    Ok(ImageData { base64, mime_type })
}

fn base64_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let mut result = String::with_capacity(data.len().div_ceil(3) * 4);

    for chunk in data.chunks(3) {
        let mut buf = [0u8; 3];
        buf[..chunk.len()].copy_from_slice(chunk);

        let n = ((buf[0] as u32) << 16) | ((buf[1] as u32) << 8) | (buf[2] as u32);

        result.push(ALPHABET[(n >> 18) as usize & 0x3F] as char);
        result.push(ALPHABET[(n >> 12) as usize & 0x3F] as char);

        if chunk.len() > 1 {
            result.push(ALPHABET[(n >> 6) as usize & 0x3F] as char);
        } else {
            result.push('=');
        }

        if chunk.len() > 2 {
            result.push(ALPHABET[n as usize & 0x3F] as char);
        } else {
            result.push('=');
        }
    }

    result
}

#[tauri::command]
async fn delete_attachment_file(file_path: String) -> Result<bool, String> {
    log::info!("[delete_attachment_file] path: {}", file_path);

    let path = PathBuf::from(&file_path);

    // Check if file exists before canonicalize (canonicalize requires the path to exist)
    if !path.exists() {
        log::info!("[delete_attachment_file] File does not exist: {}", file_path);
        return Ok(false);
    }

    // Security: Canonicalize to resolve symlinks/.. and verify inside .beads/attachments/
    let canonical = path.canonicalize()
        .map_err(|e| format!("Failed to resolve path: {}", e))?;
    let canonical_str = canonical.to_string_lossy();
    if !canonical_str.contains("/.beads/attachments/") {
        log::warn!("[delete_attachment_file] Refusing to delete file outside attachments: {} (resolved: {})", file_path, canonical_str);
        return Err("Can only delete files inside .beads/attachments/".to_string());
    }

    // Delete the file
    fs::remove_file(&path)
        .map_err(|e| format!("Failed to delete file: {}", e))?;

    log::info!("[delete_attachment_file] Deleted: {}", file_path);
    Ok(true)
}

#[tauri::command]
async fn cleanup_empty_attachment_folder(project_path: String, issue_id: String) -> Result<bool, String> {
    log::info!("[cleanup_empty_attachment_folder] project: {}, issue: {}", project_path, issue_id);

    // Calculate absolute project path
    let abs_project_path = if project_path == "." || project_path.is_empty() {
        env::current_dir().map_err(|e| format!("Failed to get current directory: {}", e))?
    } else {
        let p = PathBuf::from(&project_path);
        if p.is_relative() {
            let cwd = env::current_dir()
                .map_err(|e| format!("Failed to get current directory: {}", e))?;
            cwd.join(&p)
        } else {
            p
        }
    };

    let abs_project_path = abs_project_path
        .canonicalize()
        .map_err(|e| format!("Failed to resolve project path: {}", e))?;

    // Build attachment folder path for this issue
    let attachment_dir = abs_project_path
        .join(".beads")
        .join("attachments")
        .join(&issue_id);

    // If folder doesn't exist, nothing to do
    if !attachment_dir.exists() || !attachment_dir.is_dir() {
        log::info!("[cleanup_empty_attachment_folder] Folder does not exist: {:?}", attachment_dir);
        return Ok(false);
    }

    // Check if folder is empty
    let entries = fs::read_dir(&attachment_dir)
        .map_err(|e| format!("Failed to read attachment directory: {}", e))?;

    let is_empty = entries.count() == 0;

    if is_empty {
        log::info!("[cleanup_empty_attachment_folder] Deleting empty folder: {:?}", attachment_dir);
        fs::remove_dir(&attachment_dir)
            .map_err(|e| format!("Failed to remove empty folder: {}", e))?;
        Ok(true)
    } else {
        log::info!("[cleanup_empty_attachment_folder] Folder not empty, keeping: {:?}", attachment_dir);
        Ok(false)
    }
}

#[tauri::command]
async fn purge_orphan_attachments(project_path: String) -> Result<PurgeResult, String> {
    log::info!("[purge_orphan_attachments] project: {}", project_path);

    // Calculate absolute project path (reusing pattern from bd_delete)
    let abs_project_path = if project_path == "." || project_path.is_empty() {
        env::current_dir().map_err(|e| format!("Failed to get current directory: {}", e))?
    } else {
        let p = PathBuf::from(&project_path);
        if p.is_relative() {
            let cwd = env::current_dir()
                .map_err(|e| format!("Failed to get current directory: {}", e))?;
            cwd.join(&p)
        } else {
            p
        }
    };

    let abs_project_path = abs_project_path
        .canonicalize()
        .map_err(|e| format!("Failed to resolve project path: {}", e))?;

    // Build attachments directory path
    let attachments_dir = abs_project_path.join(".beads").join("attachments");

    // If attachments directory doesn't exist, nothing to purge
    if !attachments_dir.exists() || !attachments_dir.is_dir() {
        log::info!("[purge_orphan_attachments] No attachments directory found");
        return Ok(PurgeResult {
            deleted_count: 0,
            deleted_folders: vec![],
        });
    }

    // Get list of all existing issue IDs via bd list --all
    let existing_ids: std::collections::HashSet<String> = {
        let output = execute_bd("list", &["--all".to_string(), "--limit=0".to_string()], Some(&abs_project_path.to_string_lossy()))?;
        let issues = parse_issues_tolerant(&output, "purge_orphan_attachments")?;
        issues.into_iter().map(|i| i.id).collect()
    };

    log::info!("[purge_orphan_attachments] Found {} existing issues", existing_ids.len());

    // List all subdirectories in attachments folder
    let entries = fs::read_dir(&attachments_dir)
        .map_err(|e| format!("Failed to read attachments directory: {}", e))?;

    let mut deleted_folders: Vec<String> = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let folder_name = match path.file_name() {
            Some(name) => name.to_string_lossy().to_string(),
            None => continue,
        };

        // Check if this folder corresponds to an existing issue
        if !existing_ids.contains(&folder_name) {
            log::info!("[purge_orphan_attachments] Deleting orphan folder: {}", folder_name);
            if let Err(e) = fs::remove_dir_all(&path) {
                log::warn!("[purge_orphan_attachments] Failed to delete {}: {}", folder_name, e);
            } else {
                deleted_folders.push(folder_name);
            }
        }
    }

    let deleted_count = deleted_folders.len();
    log::info!("[purge_orphan_attachments] Purged {} orphan folders", deleted_count);

    Ok(PurgeResult {
        deleted_count,
        deleted_folders,
    })
}

#[tauri::command]
async fn copy_file_to_attachments(
    project_path: String,
    source_path: String,
    issue_id: String,
) -> Result<String, String> {
    log::info!(
        "[copy_file_to_attachments] project: {}, source: {}, issue: {}",
        project_path,
        source_path,
        issue_id
    );

    // Validate file extension (images + markdown)
    let allowed_extensions = ["png", "jpg", "jpeg", "gif", "webp", "bmp", "svg", "ico", "tiff", "tif", "md", "markdown"];
    let source_lower = source_path.to_lowercase();
    let is_allowed = allowed_extensions
        .iter()
        .any(|ext| source_lower.ends_with(&format!(".{}", ext)));

    if !is_allowed {
        return Err("Only image and markdown files are allowed".to_string());
    }

    // Verify source file exists
    let source = PathBuf::from(&source_path);
    if !source.exists() {
        return Err(format!("Source file not found: {}", source_path));
    }

    // Calculate absolute project path
    let abs_project_path = if project_path == "." || project_path.is_empty() {
        env::current_dir().map_err(|e| format!("Failed to get current directory: {}", e))?
    } else {
        let p = PathBuf::from(&project_path);
        if p.is_relative() {
            let cwd = env::current_dir()
                .map_err(|e| format!("Failed to get current directory: {}", e))?;
            cwd.join(&p)
        } else {
            p
        }
    };

    // Canonicalize to resolve symlinks and get absolute path
    let abs_project_path = abs_project_path
        .canonicalize()
        .map_err(|e| format!("Failed to resolve project path: {}", e))?;

    // Build destination directory: {project}/.beads/attachments/{issue_id}/
    let dest_dir = abs_project_path
        .join(".beads")
        .join("attachments")
        .join(&issue_id);

    // Create directory if needed
    fs::create_dir_all(&dest_dir)
        .map_err(|e| format!("Failed to create attachments directory: {}", e))?;

    // Extract source filename
    let source_filename = source
        .file_name()
        .ok_or_else(|| "Invalid source filename".to_string())?
        .to_string_lossy()
        .to_string();

    // Handle duplicates: if image.png exists, try image-1.png, image-2.png, etc.
    let (stem, ext) = match source_filename.rfind('.') {
        Some(pos) => (&source_filename[..pos], &source_filename[pos..]),
        None => (source_filename.as_str(), ""),
    };

    let mut dest_filename = source_filename.clone();
    let mut dest_path = dest_dir.join(&dest_filename);
    let mut counter = 1;

    while dest_path.exists() {
        dest_filename = format!("{}-{}{}", stem, counter, ext);
        dest_path = dest_dir.join(&dest_filename);
        counter += 1;

        // Safety limit
        if counter > 1000 {
            return Err("Too many duplicate files".to_string());
        }
    }

    // Copy the file
    fs::copy(&source, &dest_path).map_err(|e| format!("Failed to copy file: {}", e))?;

    let result_path = dest_path.to_string_lossy().to_string();
    log::info!("[copy_file_to_attachments] Copied to: {}", result_path);

    Ok(result_path)
}

#[derive(Debug, Serialize)]
pub struct TextData {
    pub content: String,
}

#[tauri::command]
async fn read_text_file(path: String) -> Result<TextData, String> {
    log_info!("[read_text_file] Reading: {}", path);

    // Security: Only allow markdown file extensions
    let path_lower = path.to_lowercase();
    let is_markdown = path_lower.ends_with(".md") || path_lower.ends_with(".markdown");

    if !is_markdown {
        return Err("Only markdown files are allowed".to_string());
    }

    // Verify file exists
    if !std::path::Path::new(&path).exists() {
        return Err(format!("File not found: {}", path));
    }

    // Security: Canonicalize to resolve symlinks/.. and verify inside .beads/attachments/
    let canonical = std::path::Path::new(&path).canonicalize()
        .map_err(|e| format!("Failed to resolve path: {}", e))?;
    let canonical_str = canonical.to_string_lossy();
    if !canonical_str.contains("/.beads/attachments/") {
        log_warn!("[read_text_file] Refusing to read file outside attachments: {} (resolved: {})", path, canonical_str);
        return Err("Can only read files inside .beads/attachments/".to_string());
    }

    // Read file as UTF-8
    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    Ok(TextData { content })
}

#[tauri::command]
async fn write_text_file(path: String, content: String) -> Result<(), String> {
    log_info!("[write_text_file] Writing: {}", path);

    // Security: Only allow markdown file extensions
    let path_lower = path.to_lowercase();
    let is_markdown = path_lower.ends_with(".md") || path_lower.ends_with(".markdown");

    if !is_markdown {
        return Err("Only markdown files are allowed".to_string());
    }

    // Verify file exists (no creation of new files)
    if !std::path::Path::new(&path).exists() {
        return Err(format!("File not found: {}", path));
    }

    // Security: Canonicalize to resolve symlinks/.. and verify inside .beads/attachments/
    let canonical = std::path::Path::new(&path).canonicalize()
        .map_err(|e| format!("Failed to resolve path: {}", e))?;
    let canonical_str = canonical.to_string_lossy();
    if !canonical_str.contains("/.beads/attachments/") {
        log_warn!("[write_text_file] Refusing to write file outside attachments: {} (resolved: {})", path, canonical_str);
        return Err("Can only write files inside .beads/attachments/".to_string());
    }

    // Write content to file
    fs::write(&path, &content)
        .map_err(|e| format!("Failed to write file: {}", e))?;

    log_info!("[write_text_file] Written {} bytes to {}", content.len(), path);
    Ok(())
}

// ============================================================================
// App Entry Point
// ============================================================================

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // Enable logging in both debug and release builds
            let log_level = if cfg!(debug_assertions) {
                log::LevelFilter::Debug
            } else {
                log::LevelFilter::Info
            };
            app.handle().plugin(
                tauri_plugin_log::Builder::default()
                    .level(log_level)
                    .max_file_size(5_000_000) // 5 MB max per log file
                    .rotation_strategy(tauri_plugin_log::RotationStrategy::KeepOne) // Keep only one backup
                    .target(tauri_plugin_log::Target::new(
                        tauri_plugin_log::TargetKind::LogDir { file_name: Some("beads.log".into()) },
                    ))
                    .target(tauri_plugin_log::Target::new(
                        tauri_plugin_log::TargetKind::Stdout,
                    ))
                    .build(),
            )?;

            // Log startup info
            log::info!("=== Beads Task-Issue Tracker starting ===");
            log::info!("[startup] Extended PATH: {}", get_extended_path());

            // Check if bd is accessible
            match new_command("bd")
                .arg("--version")
                .env("PATH", get_extended_path())
                .output()
            {
                Ok(output) if output.status.success() => {
                    let version = String::from_utf8_lossy(&output.stdout);
                    log::info!("[startup] bd found: {}", version.trim());
                }
                Ok(output) => {
                    log::warn!("[startup] bd command failed: {}", String::from_utf8_lossy(&output.stderr));
                }
                Err(e) => {
                    log::error!("[startup] bd not found or not executable: {}", e);
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            bd_sync,
            bd_repair_database,
            bd_check_changed,
            bd_poll_data,
            bd_list,
            bd_count,
            bd_ready,
            bd_status,
            bd_show,
            bd_create,
            get_logging_enabled,
            set_logging_enabled,
            get_verbose_logging,
            set_verbose_logging,
            clear_logs,
            export_logs,
            read_logs,
            get_log_path_string,
            log_frontend,
            get_bd_version,
            bd_update,
            bd_close,
            bd_delete,
            bd_comments_add,
            fs_exists,
            fs_list,
            check_for_updates,
            open_image_file,
            read_image_file,
            copy_file_to_attachments,
            read_text_file,
            write_text_file,
            delete_attachment_file,
            cleanup_empty_attachment_folder,
            purge_orphan_attachments,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

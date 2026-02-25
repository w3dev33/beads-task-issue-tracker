pub mod tracker;

use notify_debouncer_mini::{new_debouncer, DebouncedEventKind};
use serde::{Deserialize, Serialize};
use tauri::Emitter;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};

// Global flags for logging
static LOGGING_ENABLED: AtomicBool = AtomicBool::new(false);
static VERBOSE_LOGGING: AtomicBool = AtomicBool::new(false);

// Sync cooldown: skip redundant syncs within 10 seconds
static LAST_SYNC_TIME: Mutex<Option<Instant>> = Mutex::new(None);
const SYNC_COOLDOWN_SECS: u64 = 10;

// Filesystem mtime tracking for change detection (per-project)
static LAST_KNOWN_MTIME: LazyLock<Mutex<HashMap<String, std::time::SystemTime>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

// Configurable CLI binary name (default: "bd")
static CLI_BINARY: LazyLock<Mutex<String>> = LazyLock::new(|| Mutex::new("bd".to_string()));

// Backend mode: "bd", "br", or "built-in" (tracker engine)
static BACKEND_MODE: LazyLock<Mutex<String>> = LazyLock::new(|| Mutex::new("bd".to_string()));

// Per-project tracker engine pool (one open SQLite connection per project)
static TRACKER_ENGINES: LazyLock<Mutex<HashMap<String, tracker::Engine>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

// Global child process handle for beads-probe
static PROBE_CHILD: LazyLock<Mutex<Option<std::process::Child>>> =
    LazyLock::new(|| Mutex::new(None));

// Per-project mutex to prevent concurrent bd/Dolt access.
// bd 0.55 uses embedded Dolt which crashes (SIGSEGV) when two bd processes
// access the same database simultaneously. This serializes all bd calls per project.
static BD_PROJECT_LOCKS: LazyLock<Mutex<HashMap<String, std::sync::Arc<Mutex<()>>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

// Cached CLI client info — detected once on first use
// Stores: (client_type, major, minor, patch)
#[derive(Debug, Clone, Copy, PartialEq)]
enum CliClient {
    Bd,  // Original Go-based beads CLI
    Br,  // beads_rust — frozen at classic SQLite+JSONL architecture, no daemon
    Unknown,
}

static CLI_CLIENT_INFO: LazyLock<Mutex<Option<(CliClient, u32, u32, u32)>>> =
    LazyLock::new(|| Mutex::new(None));

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
    #[serde(rename = "downloadUrl")]
    pub download_url: Option<String>,
    pub platform: String,
    #[serde(rename = "releaseNotes")]
    pub release_notes: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
}

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    html_url: String,
    #[serde(default)]
    assets: Vec<GitHubAsset>,
    #[serde(default)]
    body: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BdCliUpdateInfo {
    #[serde(rename = "currentVersion")]
    pub current_version: String,
    #[serde(rename = "latestVersion")]
    pub latest_version: String,
    #[serde(rename = "hasUpdate")]
    pub has_update: bool,
    #[serde(rename = "releaseUrl")]
    pub release_url: String,
}

// ============================================================================
// File Watcher (debounced native fs watcher via notify crate)
// ============================================================================

struct WatcherState {
    debouncer: Option<notify_debouncer_mini::Debouncer<notify::RecommendedWatcher>>,
    watched_path: Option<String>,
}

impl Default for WatcherState {
    fn default() -> Self {
        Self {
            debouncer: None,
            watched_path: None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct BeadsChangedPayload {
    path: String,
}

// ============================================================================
// Types
// ============================================================================

/// Dependency relationship as returned by bd CLI
/// Format: {"issue_id": "...", "depends_on_id": "...", "type": "blocks", "created_at": "...", "created_by": "..."}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BdRawDependency {
    pub id: Option<String>,
    pub issue_id: Option<String>,
    pub depends_on_id: Option<String>,
    #[serde(rename = "type", alias = "dependency_type")]
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
    pub metadata: Option<String>,
    pub spec_id: Option<String>,
    pub comment_count: Option<i32>,
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
    pub relations: Option<Vec<Relation>>,
    pub metadata: Option<String>,
    #[serde(rename = "specId")]
    pub spec_id: Option<String>,
    #[serde(rename = "commentCount")]
    pub comment_count: Option<i32>,
    #[serde(rename = "dependencyCount")]
    pub dependency_count: Option<i32>,
    #[serde(rename = "dependentCount")]
    pub dependent_count: Option<i32>,
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
pub struct Relation {
    pub id: String,
    pub title: String,
    pub status: String,
    pub priority: String,
    #[serde(rename = "relationType")]
    pub relation_type: String,
    pub direction: String, // "dependency" or "dependent"
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
    #[serde(rename = "usesDolt")]
    pub uses_dolt: bool,
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
    #[serde(rename = "usesDolt")]
    pub uses_dolt: bool,
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
    #[serde(rename = "specId")]
    pub spec_id: Option<String>,
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
    pub metadata: Option<String>,
    #[serde(rename = "specId")]
    pub spec_id: Option<String>,
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

pub fn normalize_issue_type(issue_type: &str) -> String {
    let valid_types = ["bug", "task", "feature", "epic", "chore"];
    if valid_types.contains(&issue_type) {
        issue_type.to_string()
    } else {
        "task".to_string()
    }
}

pub fn normalize_issue_status(status: &str) -> String {
    let valid_statuses = ["open", "in_progress", "blocked", "closed", "deferred", "tombstone", "pinned", "hooked"];
    if valid_statuses.contains(&status) {
        status.to_string()
    } else {
        "open".to_string()
    }
}

/// Check if the backend mode is set to "built-in" (tracker engine).
fn is_builtin_backend() -> bool {
    let mode = BACKEND_MODE.lock().unwrap();
    *mode == "built-in"
}

/// Get or create a tracker::Engine for a given project path.
/// Resolves the project path from cwd (or env/cwd fallback), then
/// returns a result from the provided closure called with &Engine.
fn with_engine<F, T>(cwd: Option<&str>, f: F) -> Result<T, String>
where
    F: FnOnce(&tracker::Engine) -> Result<T, String>,
{
    let project_path = cwd
        .map(String::from)
        .or_else(|| env::var("BEADS_PATH").ok())
        .unwrap_or_else(|| {
            env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| ".".to_string())
        });

    let mut engines = TRACKER_ENGINES.lock().unwrap();
    if !engines.contains_key(&project_path) {
        let path = std::path::Path::new(&project_path);
        let tracker_db = path.join(".tracker").join("tracker.db");
        let engine = if tracker_db.exists() {
            tracker::Engine::open(path)
        } else {
            tracker::Engine::init(path, tracker::ProjectConfig::load(path))
        };
        let engine = engine.map_err(|e| format!("Failed to open tracker engine: {}", e))?;
        engines.insert(project_path.clone(), engine);
    }
    let engine = engines.get(&project_path).unwrap();
    f(engine)
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

    // Extract non-blocking relations (everything except "blocks" and "parent-child")
    let structural_types = ["blocks", "parent-child"];
    let mut relations: Vec<Relation> = Vec::new();
    let mut seen_relations: std::collections::HashSet<(String, String)> = std::collections::HashSet::new();

    // From dependencies array (these are issues the current issue depends on)
    if let Some(ref deps) = raw.dependencies {
        for dep in deps {
            if let Some(ref dep_type) = dep.dependency_type {
                if structural_types.contains(&dep_type.as_str()) {
                    continue;
                }
                let id = dep.id.clone().or_else(|| dep.depends_on_id.clone()).unwrap_or_default();
                if id.is_empty() {
                    continue;
                }
                let key = (id.clone(), dep_type.clone());
                if !seen_relations.contains(&key) {
                    seen_relations.insert(key);
                    relations.push(Relation {
                        id,
                        title: String::new(),
                        status: String::new(),
                        priority: String::new(),
                        relation_type: dep_type.clone(),
                        direction: "dependency".to_string(),
                    });
                }
            }
        }
    }

    // From dependents array (these are issues that depend on the current issue — has full metadata)
    if let Some(ref dependents) = raw.dependents {
        for dep in dependents {
            if let Some(ref dep_type) = dep.dependency_type {
                if structural_types.contains(&dep_type.as_str()) {
                    continue;
                }
                let id = dep.id.clone().unwrap_or_default();
                if id.is_empty() {
                    continue;
                }
                let key = (id.clone(), dep_type.clone());
                if seen_relations.contains(&key) {
                    // Replace existing entry from dependencies if this one has more metadata
                    if dep.title.is_some() {
                        if let Some(existing) = relations.iter_mut().find(|r| r.id == id && r.relation_type == *dep_type) {
                            existing.title = dep.title.clone().unwrap_or_default();
                            existing.status = normalize_issue_status(&dep.status.clone().unwrap_or_else(|| "open".to_string()));
                            existing.priority = priority_to_string(dep.priority.unwrap_or(3));
                            existing.direction = "dependent".to_string();
                        }
                    }
                } else {
                    seen_relations.insert(key);
                    relations.push(Relation {
                        id,
                        title: dep.title.clone().unwrap_or_default(),
                        status: normalize_issue_status(&dep.status.clone().unwrap_or_else(|| "open".to_string())),
                        priority: priority_to_string(dep.priority.unwrap_or(3)),
                        relation_type: dep_type.clone(),
                        direction: "dependent".to_string(),
                    });
                }
            }
        }
    }

    // Compute comment_count before consuming raw.comments
    let comment_count = raw.comment_count.or_else(|| {
        raw.comments.as_ref().map(|c| c.len() as i32)
    });

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
        blocked_by: {
            // Try raw.blocked_by first (if bd ever populates it directly)
            let mut bb = raw.blocked_by.unwrap_or_default();
            // Extract from dependencies array (bd show: objects with dependency_type "blocks" = blockers)
            if let Some(ref deps) = raw.dependencies {
                // bd show format: [{id, dependency_type: "blocks"}] — these block the current issue
                for dep in deps {
                    if let (Some(ref dep_type), Some(ref id)) = (&dep.dependency_type, &dep.id) {
                        if dep_type == "blocks" && !bb.contains(id) {
                            bb.push(id.clone());
                        }
                    }
                    // bd list format: [{issue_id, depends_on_id, type: "blocks"}]
                    if let (Some(ref dep_type), Some(ref depends_on_id), Some(ref _issue_id)) = (&dep.dependency_type, &dep.depends_on_id, &dep.issue_id) {
                        if dep_type == "blocks" && !bb.contains(depends_on_id) {
                            bb.push(depends_on_id.clone());
                        }
                    }
                }
            }
            if bb.is_empty() { None } else { Some(bb) }
        },
        blocks: {
            let mut bl = raw.blocks.unwrap_or_default();
            // Extract from dependents array (bd show: objects with dependency_type "blocks" = issues blocked by current)
            // Filter to only "blocks" type — exclude "parent-child" which are children, not dependencies
            if let Some(ref dependents) = raw.dependents {
                for dep in dependents {
                    if let (Some(ref dep_type), Some(ref id)) = (&dep.dependency_type, &dep.id) {
                        if dep_type == "blocks" && !bl.contains(id) {
                            bl.push(id.clone());
                        }
                    }
                }
            }
            if bl.is_empty() { None } else { Some(bl) }
        },
        external_ref: raw.external_ref,
        estimate_minutes: raw.estimate,
        design_notes: raw.design,
        acceptance_criteria: raw.acceptance_criteria,
        working_notes: raw.notes,
        parent,
        children,
        relations: if relations.is_empty() { None } else { Some(relations) },
        metadata: raw.metadata,
        spec_id: raw.spec_id,
        comment_count,
        dependency_count: raw.dependency_count.or_else(|| {
            raw.dependencies.as_ref().map(|d| d.len() as i32)
        }),
        dependent_count: raw.dependent_count.or_else(|| {
            raw.dependents.as_ref().map(|d| d.len() as i32)
        }),
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

// ============================================================================
// CLI Binary Configuration
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
struct AppConfig {
    #[serde(default = "default_cli_binary")]
    cli_binary: String,
}

fn default_cli_binary() -> String {
    // Auto-detect: prefer br (Rust), fallback to bd (Go)
    for bin in &["br", "bd"] {
        if let Ok(output) = std::process::Command::new(bin)
            .arg("--version")
            .current_dir(std::env::temp_dir())
            .output()
        {
            if output.status.success() {
                return bin.to_string();
            }
        }
    }
    // Neither found — default to br (will fail later with clear error)
    "br".to_string()
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            cli_binary: default_cli_binary(),
        }
    }
}

fn get_config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("com.beads.manager")
        .join("settings.json")
}

fn load_config() -> AppConfig {
    let path = get_config_path();
    if path.exists() {
        match fs::read_to_string(&path) {
            Ok(content) => match serde_json::from_str(&content) {
                Ok(config) => return config,
                Err(e) => log::warn!("[config] Failed to parse settings.json: {}", e),
            },
            Err(e) => log::warn!("[config] Failed to read settings.json: {}", e),
        }
    }
    AppConfig::default()
}

fn save_config(config: &AppConfig) -> Result<(), String> {
    let path = get_config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }
    let json = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    fs::write(&path, json)
        .map_err(|e| format!("Failed to write config: {}", e))?;
    Ok(())
}

fn get_cli_binary() -> String {
    CLI_BINARY.lock().unwrap().clone()
}

// ============================================================================
// CLI Client Detection (bd vs br)
// ============================================================================

/// Detect the client type from the version string.
/// - "bd version 0.49.6 (Homebrew)" → Bd
/// - "br 0.1.13 (rustc 1.85.0-nightly)" → Br
fn detect_cli_client(version_str: &str) -> CliClient {
    let lower = version_str.to_lowercase();
    if lower.starts_with("br ") || lower.contains("beads_rust") || lower.contains("beads-rust") {
        CliClient::Br
    } else if lower.starts_with("bd ") || lower.contains("bd version") {
        CliClient::Bd
    } else {
        CliClient::Unknown
    }
}

/// Parse a version string into (major, minor, patch).
/// Works for both "bd version 0.49.6 (Homebrew)" and "br 0.1.13 (rustc ...)".
fn parse_bd_version(version_str: &str) -> Option<(u32, u32, u32)> {
    // Look for a semver-like pattern: digits.digits.digits
    let re_like = version_str
        .split_whitespace()
        .find(|word| word.contains('.') && word.chars().next().map_or(false, |c| c.is_ascii_digit()));

    let version_part = re_like?;
    let parts: Vec<&str> = version_part.split('.').collect();
    if parts.len() >= 3 {
        let major = parts[0].parse::<u32>().ok()?;
        let minor = parts[1].parse::<u32>().ok()?;
        // Patch may have trailing non-numeric chars (e.g. "6-beta")
        let patch_str: String = parts[2].chars().take_while(|c| c.is_ascii_digit()).collect();
        let patch = patch_str.parse::<u32>().ok()?;
        Some((major, minor, patch))
    } else {
        None
    }
}

/// Detect and cache the CLI client type and version. Runs `binary --version` once.
fn get_cli_client_info() -> Option<(CliClient, u32, u32, u32)> {
    let mut cached = CLI_CLIENT_INFO.lock().unwrap();
    if let Some(info) = *cached {
        return Some(info);
    }

    let binary = get_cli_binary();
    // Run from temp dir to avoid bd auto-migrating projects in cwd
    let output = new_command(&binary)
        .arg("--version")
        .current_dir(std::env::temp_dir())
        .env("PATH", get_extended_path())
        .output()
        .ok()?;

    if !output.status.success() {
        log_warn!("[cli_detect] Failed to get version from {}", binary);
        return None;
    }

    let version_str = String::from_utf8_lossy(&output.stdout);
    let trimmed = version_str.trim();
    let client = detect_cli_client(trimmed);
    let tuple = parse_bd_version(trimmed);

    if let Some((major, minor, patch)) = tuple {
        let info = (client, major, minor, patch);
        let client_name = match client {
            CliClient::Bd => "bd",
            CliClient::Br => "br",
            CliClient::Unknown => "unknown",
        };
        log_info!("[cli_detect] Detected {} client v{}.{}.{}", client_name, major, minor, patch);
        *cached = Some(info);
        Some(info)
    } else {
        log_warn!("[cli_detect] Could not parse version from: {}", trimmed);
        None
    }
}

/// Returns true if the CLI supports the --no-daemon flag.
/// - br: NEVER (no daemon concept)
/// - bd < 0.50.0: YES
/// - bd >= 0.50.0: NO (daemon removed)
/// - unknown: NO (safe default)
fn supports_daemon_flag() -> bool {
    match get_cli_client_info() {
        Some((CliClient::Br, _, _, _)) => false, // br has no daemon
        Some((CliClient::Bd, major, minor, _)) => major == 0 && minor < 50,
        Some((CliClient::Unknown, _, _, _)) => false,
        None => false,
    }
}

/// Returns true if the CLI uses issues.jsonl files.
/// - br: ALWAYS (frozen on SQLite+JSONL architecture)
/// - bd < 0.50.0: YES
/// - bd >= 0.50.0: NO (Dolt only)
/// - unknown: NO (safe default)
fn uses_jsonl_files() -> bool {
    match get_cli_client_info() {
        Some((CliClient::Br, _, _, _)) => true, // br always uses JSONL
        Some((CliClient::Bd, major, minor, _)) => major == 0 && minor < 50,
        Some((CliClient::Unknown, _, _, _)) => false,
        None => false,
    }
}

/// Returns true if `bd list --all` works correctly.
/// The --all flag was buggy before bd 0.55.0 (returned incorrect results).
/// - br: NO
/// - bd >= 0.55.0: YES
/// - bd < 0.55.0: NO (use 2 separate calls instead)
/// - unknown: NO (safe default)
fn supports_list_all_flag() -> bool {
    match get_cli_client_info() {
        Some((CliClient::Bd, major, minor, _)) => major > 0 || minor >= 55,
        Some((CliClient::Br, _, _, _)) => true, // br always supports --all
        _ => false,
    }
}

/// Returns true if `bd delete --hard` is supported.
/// The --hard flag was removed in bd 0.50.0.
/// - br: NO
/// - bd < 0.50.0: YES
/// - bd >= 0.50.0: NO (only --force needed)
/// - unknown: NO (safe default)
fn supports_delete_hard_flag() -> bool {
    match get_cli_client_info() {
        Some((CliClient::Bd, major, minor, _)) => major == 0 && minor < 50,
        _ => false,
    }
}

/// Returns true if the CLI uses the Dolt backend (inverse of uses_jsonl_files).
/// - br: NEVER (frozen on SQLite+JSONL architecture)
/// - bd >= 0.50.0: YES (Dolt only)
/// - bd < 0.50.0: NO (SQLite+JSONL)
/// - unknown: NO (safe default)
fn uses_dolt_backend() -> bool {
    match get_cli_client_info() {
        Some((CliClient::Br, _, _, _)) => false, // br never uses Dolt
        Some((CliClient::Bd, major, minor, _)) => major > 0 || minor >= 50,
        Some((CliClient::Unknown, _, _, _)) => false,
        None => false,
    }
}

/// Returns true if a specific project uses the Dolt backend.
/// Checks for the presence of `.beads/.dolt/` directory in the project.
/// - br: NEVER (frozen on SQLite+JSONL architecture)
/// - bd < 0.50.0: NEVER (CLI doesn't support Dolt)
/// - bd >= 0.50.0: checks if `.dolt/` directory exists inside the beads dir
fn project_uses_dolt(beads_dir: &std::path::Path) -> bool {
    match get_cli_client_info() {
        Some((CliClient::Br, _, _, _)) => false,
        Some((CliClient::Bd, major, minor, _)) if major == 0 && minor < 50 => false,
        _ => {
            // Check .beads/.dolt (legacy) or .beads/dolt/<name>/.dolt (bd 0.52+)
            if beads_dir.join(".dolt").is_dir() {
                return true;
            }
            // Check metadata.json for backend: "dolt"
            let metadata_path = beads_dir.join("metadata.json");
            if let Ok(content) = std::fs::read_to_string(&metadata_path) {
                if content.contains("\"backend\":\"dolt\"") || content.contains("\"backend\": \"dolt\"") {
                    // Verify dolt database actually exists
                    let dolt_dir = beads_dir.join("dolt");
                    if dolt_dir.is_dir() {
                        // Check if any subdirectory has .dolt
                        if let Ok(entries) = std::fs::read_dir(&dolt_dir) {
                            for entry in entries.flatten() {
                                if entry.path().join(".dolt").is_dir() {
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
            false
        }
    }
}

/// Reset the cached client info (called when CLI binary path changes).
fn reset_bd_version_cache() {
    let mut cached = CLI_CLIENT_INFO.lock().unwrap();
    *cached = None;
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
    if supports_daemon_flag() {
        full_args.push("--no-daemon");
    }
    full_args.push("--json");

    let binary = get_cli_binary();
    log_info!("[bd] {} {} | cwd: {}", binary, full_args.join(" "), working_dir);

    // Acquire per-project lock to prevent concurrent Dolt access (causes SIGSEGV).
    let project_lock = {
        let mut locks = BD_PROJECT_LOCKS.lock().unwrap();
        locks.entry(working_dir.clone())
            .or_insert_with(|| std::sync::Arc::new(Mutex::new(())))
            .clone()
    };
    let _guard = project_lock.lock().unwrap();

    let output = new_command(&binary)
        .args(&full_args)
        .current_dir(&working_dir)
        .env("PATH", get_extended_path())
        .env("BEADS_PATH", &working_dir)
        .output()
        .map_err(|e| {
            log_error!("[bd] Failed to execute {}: {}", binary, e);
            format!("Failed to execute {}: {}", binary, e)
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

/// Auto-run refs migration v3 (filesystem-only attachments) if needed.
/// Called synchronously before br sync to prevent UNIQUE constraint errors.
fn ensure_refs_migrated_v3(beads_dir: &std::path::Path, working_dir: &str) {
    if beads_dir.join(".migrated-attachments").exists() {
        return;
    }
    let jsonl_path = beads_dir.join("issues.jsonl");
    if !jsonl_path.exists() {
        let _ = std::fs::write(beads_dir.join(".migrated-attachments"), "");
        return;
    }

    let content = match std::fs::read_to_string(&jsonl_path) {
        Ok(c) => c,
        Err(_) => return,
    };

    // Quick scan: does any line have non-real external refs?
    let mut needs_migration = false;
    for line in content.lines() {
        if line.trim().is_empty() { continue; }
        let v: serde_json::Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let ext_ref = v.get("external_ref").and_then(|r| r.as_str()).unwrap_or("");
        // Non-real ref (att:, paths, cleared: sentinels, etc.)
        if ext_ref.is_empty() { continue; }
        for r in ext_ref.split(|c: char| c == '\n' || c == '|') {
            let trimmed = r.trim();
            if !trimmed.is_empty() && !is_real_external_ref(trimmed) {
                needs_migration = true;
                break;
            }
        }
        if needs_migration { break; }
    }

    // Also check if attachment folders need renaming
    let attachments_dir_check = beads_dir.join("attachments");
    let mut needs_folder_work = false;
    if attachments_dir_check.exists() {
        if let Ok(entries) = std::fs::read_dir(&attachments_dir_check) {
            for entry in entries.flatten() {
                if !entry.path().is_dir() { continue; }
                let name = entry.file_name().to_string_lossy().to_string();
                if issue_short_id(&name) != name {
                    needs_folder_work = true;
                    break;
                }
            }
        }
    }

    if !needs_migration && !needs_folder_work {
        let _ = std::fs::write(beads_dir.join(".migrated-attachments"), "");
        return;
    }

    log_info!("[sync] Auto-migrating v3 (refs={}, folders={}) for: {}", needs_migration, needs_folder_work, working_dir);

    // Backup
    let backup_path = beads_dir.join("issues.jsonl.bak-refs-v3-migration");
    if std::fs::copy(&jsonl_path, &backup_path).is_err() {
        log_error!("[sync] Failed to backup JSONL for v3 migration, skipping");
        return;
    }

    // Migrate: strip non-real refs, deduplicate
    let mut refs_updated: u32 = 0;
    let mut output_lines: Vec<String> = Vec::new();
    let mut seen_refs: std::collections::HashSet<String> = std::collections::HashSet::new();

    for line in content.lines() {
        if line.trim().is_empty() {
            output_lines.push(line.to_string());
            continue;
        }
        let mut v: serde_json::Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => { output_lines.push(line.to_string()); continue; }
        };

        let issue_id = v.get("id").and_then(|i| i.as_str()).unwrap_or("").to_string();

        let ext_ref = v.get("external_ref").and_then(|r| r.as_str()).unwrap_or("").to_string();

        // Parse existing refs, keep only real external ones
        let real_refs: Vec<String> = if ext_ref.is_empty() {
            vec![]
        } else {
            ext_ref.split(|c: char| c == '\n' || c == '|')
                .map(|r| r.trim())
                .filter(|r| is_real_external_ref(r))
                .map(String::from)
                .collect()
        };

        let mut new_ref = if real_refs.is_empty() {
            String::new()
        } else {
            real_refs.join("|")
        };

        // Deduplicate: if another issue already has this exact ref, clear it
        if !new_ref.is_empty() && seen_refs.contains(&new_ref) {
            log_info!("[sync] Duplicate external_ref '{}' for issue {}, clearing", new_ref, issue_id);
            new_ref = String::new();
        }
        if !new_ref.is_empty() {
            seen_refs.insert(new_ref.clone());
        }

        if new_ref != ext_ref {
            v["external_ref"] = serde_json::Value::String(new_ref);
            refs_updated += 1;
            output_lines.push(serde_json::to_string(&v).unwrap_or_else(|_| line.to_string()));
            continue;
        }

        // Track existing refs that weren't modified too
        if let Some(ext_ref) = v.get("external_ref").and_then(|r| r.as_str()) {
            seen_refs.insert(ext_ref.to_string());
        }

        output_lines.push(line.to_string());
    }

    if refs_updated > 0 {
        let new_content = output_lines.join("\n");
        if std::fs::write(&jsonl_path, &new_content).is_err() {
            log_error!("[sync] Failed to write migrated JSONL");
            return;
        }
        log_info!("[sync] Refs v3 migration: {} ref(s) cleaned", refs_updated);
    }

    // Rename attachment folders: {full-id}/ → {short-id}/
    let attachments_dir = beads_dir.join("attachments");
    if attachments_dir.exists() {
        let mut renamed = 0u32;
        if let Ok(entries) = std::fs::read_dir(&attachments_dir) {
            let dirs: Vec<_> = entries.flatten()
                .filter(|e| e.path().is_dir())
                .collect();
            for entry in dirs {
                let folder_name = entry.file_name().to_string_lossy().to_string();
                let short = issue_short_id(&folder_name);
                if short != folder_name {
                    let target = attachments_dir.join(short);
                    if target.exists() {
                        log_warn!("[sync] Cannot rename '{}' → '{}': target already exists", folder_name, short);
                        continue;
                    }
                    if std::fs::rename(entry.path(), &target).is_ok() {
                        renamed += 1;
                    } else {
                        log_warn!("[sync] Failed to rename '{}' → '{}'", folder_name, short);
                    }
                }
            }
        }
        if renamed > 0 {
            log_info!("[sync] Renamed {} attachment folder(s) to short IDs", renamed);
        }
    }

    let _ = std::fs::write(beads_dir.join(".migrated-attachments"), "");
    // Signal for the frontend to show a notification
    let _ = std::fs::write(beads_dir.join(".migrated-attachments-notify"), "");
    log_info!("[sync] Migration v3 complete (refs cleaned + folders renamed)");
}

/// Sync the beads database before read operations to ensure data is up-to-date
/// Uses bidirectional sync to preserve local changes while getting remote updates
/// Has a cooldown to avoid redundant syncs within the same poll cycle
fn sync_bd_database(cwd: Option<&str>) {
    let working_dir = cwd
        .map(String::from)
        .or_else(|| env::var("BEADS_PATH").ok())
        .unwrap_or_else(|| {
            env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| ".".to_string())
        });

    // Dolt backend handles its own sync via git — skip bd sync
    let beads_dir = std::path::Path::new(&working_dir).join(".beads");
    if project_uses_dolt(&beads_dir) {
        log_info!("[sync] Skipping — Dolt backend handles sync via git");
        return;
    }

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

    log_info!("[sync] Starting bidirectional sync for: {}", working_dir);

    // Auto-migrate refs v3 before sync if needed (prevents UNIQUE constraint errors)
    ensure_refs_migrated_v3(&beads_dir, &working_dir);

    // Run bd sync (bidirectional - exports local changes AND imports remote changes)
    let binary = get_cli_binary();
    let mut sync_args = vec!["sync"];
    if supports_daemon_flag() {
        sync_args.push("--no-daemon");
    }
    match new_command(&binary)
        .args(&sync_args)
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
                "[sync] {} sync failed: {}",
                binary,
                String::from_utf8_lossy(&output.stderr)
            );
        }
        Err(e) => {
            log_error!("[sync] Failed to run {} sync: {}", binary, e);
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

    // Dolt backend handles its own sync via git — skip bd sync
    let beads_dir = std::path::Path::new(&working_dir).join(".beads");
    if project_uses_dolt(&beads_dir) {
        log_info!("[bd_sync] Skipping — Dolt backend handles sync via git");
        return Ok(());
    }

    let binary = get_cli_binary();
    log_info!("[bd_sync] Manual sync requested for: {}", working_dir);

    let mut sync_args = vec!["sync"];
    if supports_daemon_flag() {
        sync_args.push("--no-daemon");
    }
    let output = new_command(&binary)
        .args(&sync_args)
        .current_dir(&working_dir)
        .env("PATH", get_extended_path())
        .env("BEADS_PATH", &working_dir)
        .output()
        .map_err(|e| format!("Failed to run {} sync: {}", binary, e))?;

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

    // Check if .beads directory exists
    if !beads_dir.exists() {
        return Err("No .beads directory found in this project".to_string());
    }

    // Dolt backend: use `bd doctor --fix --yes`
    if project_uses_dolt(&beads_dir) {
        log_info!("[bd_repair] Using Dolt-based repair strategy (bd >= 0.50.0): bd doctor --fix --yes");
        let binary = get_cli_binary();
        let output = new_command(&binary)
            .args(&["doctor", "--fix", "--yes"])
            .current_dir(&working_dir)
            .env("PATH", get_extended_path())
            .env("BEADS_PATH", &working_dir)
            .output()
            .map_err(|e| format!("Failed to run bd doctor: {}", e))?;

        return if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            log_info!("[bd_repair] Dolt repair successful: {}", stdout.trim());
            Ok(RepairResult {
                success: true,
                message: format!("Database repaired via bd doctor. {}", stdout.trim()),
                backup_path: None,
            })
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            log_error!("[bd_repair] Dolt repair failed: {}", stderr.trim());
            Err(format!("Repair failed: {}", stderr.trim()))
        };
    }

    // SQLite backend: original repair logic
    let db_path = beads_dir.join("beads.db");
    let jsonl_path = beads_dir.join("issues.jsonl");
    let backup_path = beads_dir.join("beads.db.backup");

    // Check if database exists
    if !db_path.exists() {
        return Ok(RepairResult {
            success: true,
            message: "No database to repair - it will be created on next operation".to_string(),
            backup_path: None,
        });
    }

    // For bd < 0.50.0: require issues.jsonl for repair (db is rebuilt from JSONL)
    if uses_jsonl_files() {
        let jsonl_size = std::fs::metadata(&jsonl_path)
            .map(|m| m.len())
            .unwrap_or(0);

        if !jsonl_path.exists() || jsonl_size == 0 {
            return Err("Cannot repair: issues.jsonl is missing or empty. Your data would be lost.".to_string());
        }
        log_info!("[bd_repair] Using JSONL-based repair strategy (bd < 0.50.0)");
    } else {
        log_info!("[bd_repair] Using repair strategy for unknown version");
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
    let mut test_args = vec!["list", "--limit=1"];
    if supports_daemon_flag() {
        test_args.push("--no-daemon");
    }
    test_args.push("--json");
    let test_output = new_command(&get_cli_binary())
        .args(&test_args)
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
// Dolt Migration
// ============================================================================

#[derive(Debug, serde::Serialize)]
struct MigrateResult {
    success: bool,
    message: String,
}

/// Remove orphaned Dolt lock files that block database access.
///
/// Uses `lsof` to check if any process actually holds the lock file open.
/// - If no process has it open → orphaned lock from a crashed/finished bd → safe to remove.
/// - If a process has it open → active agent (Claude Code, Gastown, etc.) → leave it alone.
///
/// This is the only reliable way to distinguish a stale lock from an active one,
/// regardless of timing. bd 0.55+ in embedded Dolt mode leaves noms/LOCK behind
/// after every command, so these accumulate and block subsequent operations.

#[derive(Debug, serde::Serialize)]
struct CleanupResult {
    removed: Vec<String>,
}

/// Stale lock cleanup — currently a no-op.
///
/// bd 0.55 in embedded Dolt mode leaves lock files (dolt-access.lock, noms/LOCK)
/// after every command. These locks are NOT safe to remove externally:
/// - Removing noms/LOCK causes Dolt SIGSEGV (nil pointer dereference) on next bd call
/// - Removing dolt-access.lock also triggers the same Dolt crash
///
/// This is a bd/Dolt bug that needs to be fixed upstream. The command is kept as a
/// no-op so the frontend call doesn't need to change when a fix becomes available.
#[tauri::command]
async fn bd_cleanup_stale_locks(cwd: Option<String>) -> Result<CleanupResult, String> {
    let _ = cwd; // suppress unused warning
    Ok(CleanupResult { removed: vec![] })
}

/// Check if a project needs Dolt migration.
/// Returns true when bd >= 0.50, project has .beads/, but is not fully migrated to Dolt.
/// Detects both "never migrated" and "partially migrated" (dolt/ dir exists but .dolt marker missing).
#[derive(Debug, serde::Serialize)]
struct MigrationStatus {
    needs_migration: bool,
    reason: String,
}

#[tauri::command]
async fn bd_check_needs_migration(cwd: Option<String>) -> Result<MigrationStatus, String> {
    let working_dir = cwd
        .or_else(|| env::var("BEADS_PATH").ok())
        .unwrap_or_else(|| {
            env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| ".".to_string())
        });

    let beads_dir = std::path::Path::new(&working_dir).join(".beads");

    if !beads_dir.exists() {
        return Ok(MigrationStatus {
            needs_migration: false,
            reason: "No .beads directory".to_string(),
        });
    }

    // Check bd version — only bd >= 0.50 requires Dolt
    match get_cli_client_info() {
        Some((CliClient::Bd, major, minor, _)) if major > 0 || minor >= 50 => {
            // bd >= 0.50: check if project is fully migrated
        }
        _ => {
            return Ok(MigrationStatus {
                needs_migration: false,
                reason: "bd version does not require Dolt".to_string(),
            });
        }
    }

    // Already fully using Dolt? (.beads/.dolt exists)
    if project_uses_dolt(&beads_dir) {
        return Ok(MigrationStatus {
            needs_migration: false,
            reason: "Already using Dolt backend".to_string(),
        });
    }

    // Check for partial migration (dolt/ dir exists but not complete)
    let dolt_dir = beads_dir.join("dolt");
    if dolt_dir.exists() {
        return Ok(MigrationStatus {
            needs_migration: true,
            reason: "Partial migration detected (dolt/ exists but migration incomplete)".to_string(),
        });
    }

    // Has JSONL data but no Dolt — needs migration
    let jsonl_path = beads_dir.join("issues.jsonl");
    if jsonl_path.exists() {
        let jsonl_size = std::fs::metadata(&jsonl_path).map(|m| m.len()).unwrap_or(0);
        if jsonl_size > 0 {
            return Ok(MigrationStatus {
                needs_migration: true,
                reason: "SQLite/JSONL project needs Dolt migration".to_string(),
            });
        }
    }

    // Has SQLite db but no Dolt
    let db_path = beads_dir.join("beads.db");
    if db_path.exists() {
        return Ok(MigrationStatus {
            needs_migration: true,
            reason: "SQLite project needs Dolt migration".to_string(),
        });
    }

    // Empty project — no migration needed (bd init will create Dolt directly)
    Ok(MigrationStatus {
        needs_migration: false,
        reason: "Empty project".to_string(),
    })
}

/// Re-prefix an issue ID if it uses a non-target prefix
fn reprefix_id(id: &str, target_prefix: &str, prefix_counts: &std::collections::HashMap<String, usize>) -> String {
    if let Some(last_dash) = id.rfind('-') {
        let current_prefix = &id[..last_dash];
        if current_prefix != target_prefix && prefix_counts.contains_key(current_prefix) {
            return format!("{}{}", target_prefix, &id[last_dash..]);
        }
    }
    id.to_string()
}

#[tauri::command]
async fn bd_migrate_to_dolt(cwd: Option<String>) -> Result<MigrateResult, String> {
    let working_dir = cwd
        .or_else(|| env::var("BEADS_PATH").ok())
        .unwrap_or_else(|| {
            env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| ".".to_string())
        });

    log_info!("[bd_migrate] Starting Dolt migration for: {}", working_dir);

    let beads_dir = std::path::Path::new(&working_dir).join(".beads");

    // Check if .beads directory exists
    if !beads_dir.exists() {
        return Err("No .beads directory found in this project".to_string());
    }

    // Already using Dolt?
    if project_uses_dolt(&beads_dir) {
        return Ok(MigrateResult {
            success: true,
            message: "Project already uses the Dolt backend.".to_string(),
        });
    }

    // Verify bd >= 0.50
    if let Some((_, major, minor, _)) = get_cli_client_info() {
        if major == 0 && minor < 50 {
            return Err(format!(
                "bd version 0.50+ is required for Dolt migration (current: {}.{})",
                major, minor
            ));
        }
    } else {
        return Err("Could not determine bd version".to_string());
    }

    // Clean up partial migration if dolt/ directory exists
    let dolt_dir = beads_dir.join("dolt");
    if dolt_dir.exists() {
        log_info!("[bd_migrate] Removing partial dolt/ directory for re-migration");
        std::fs::remove_dir_all(&dolt_dir)
            .map_err(|e| format!("Failed to remove partial dolt/ directory: {}", e))?;
    }

    // Remove dolt-access.lock if present
    let dolt_lock = beads_dir.join("dolt-access.lock");
    if dolt_lock.exists() {
        std::fs::remove_file(&dolt_lock).ok();
    }

    // Try `bd migrate --to-dolt --yes` first
    let binary = get_cli_binary();
    let output = new_command(&binary)
        .args(&["migrate", "--to-dolt", "--yes"])
        .current_dir(&working_dir)
        .env("PATH", get_extended_path())
        .env("BEADS_PATH", &working_dir)
        .output()
        .map_err(|e| format!("Failed to run bd migrate: {}", e))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        log_info!("[bd_migrate] Migration via bd migrate successful: {}", stdout.trim());
        return Ok(MigrateResult {
            success: true,
            message: format!("Migration to Dolt completed successfully. {}", stdout.trim()),
        });
    }

    // bd migrate failed (typically: corrupt SQLite, missing table, etc.)
    // Fallback: bd init + bd import from JSONL
    let stderr_migrate = String::from_utf8_lossy(&output.stderr);
    log_info!("[bd_migrate] bd migrate failed ({}), trying init+import fallback", stderr_migrate.trim());

    let jsonl_path = beads_dir.join("issues.jsonl");
    if !jsonl_path.exists() || std::fs::metadata(&jsonl_path).map(|m| m.len()).unwrap_or(0) == 0 {
        // Empty project — no JSONL data to import, just run bd init
        log_info!("[bd_migrate] No issues.jsonl data — empty project, attempting init-only migration");
        // Rename existing .db files to .db.backup so bd init doesn't refuse
        if let Ok(entries) = std::fs::read_dir(&beads_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.ends_with(".db") && !name.ends_with(".db.backup") {
                    let src = entry.path();
                    let dst = beads_dir.join(format!("{}.backup", name));
                    log_info!("[bd_migrate] Renaming {} -> {}", src.display(), dst.display());
                    std::fs::rename(&src, &dst).ok();
                }
                // Also remove .db-shm and .db-wal
                if name.ends_with(".db-shm") || name.ends_with(".db-wal") {
                    std::fs::remove_file(entry.path()).ok();
                }
            }
        }
        let init_output = new_command(&binary)
            .args(&["init", "--prefix", "project"])
            .current_dir(&working_dir)
            .env("PATH", get_extended_path())
            .env("BEADS_PATH", &working_dir)
            .output()
            .map_err(|e| format!("Failed to run bd init: {}", e))?;
        if init_output.status.success() {
            log_info!("[bd_migrate] Empty project initialized with Dolt backend");
            return Ok(MigrateResult {
                success: true,
                message: "Migration complete (empty project — initialized with Dolt backend)".to_string(),
            });
        }
        let init_stderr = String::from_utf8_lossy(&init_output.stderr);
        return Err(format!(
            "Migration failed (empty project, bd init also failed): {}. Original error: {}",
            init_stderr.trim(), stderr_migrate.trim()
        ));
    }

    // Detect prefix from JSONL — use the most common prefix
    let jsonl_content = std::fs::read_to_string(&jsonl_path)
        .map_err(|e| format!("Failed to read issues.jsonl: {}", e))?;
    let mut prefix_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for line in jsonl_content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() { continue; }
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(trimmed) {
            if let Some(id) = v.get("id").and_then(|i| i.as_str()) {
                if let Some(last_dash) = id.rfind('-') {
                    let suffix = &id[last_dash + 1..];
                    if suffix.chars().all(|c| c.is_alphanumeric()) && !suffix.is_empty() {
                        *prefix_counts.entry(id[..last_dash].to_string()).or_insert(0) += 1;
                    }
                }
            }
        }
    }
    let prefix = prefix_counts
        .iter()
        .max_by_key(|(_, count)| *count)
        .map(|(p, _)| p.clone())
        .ok_or_else(|| "Could not detect issue prefix from issues.jsonl".to_string())?;

    if prefix_counts.len() > 1 {
        log_info!(
            "[bd_migrate] Multiple prefixes found: {:?}. Using most common: {}",
            prefix_counts, prefix
        );
    }
    log_info!("[bd_migrate] Detected prefix: {}", prefix);

    // Clean dolt dir again (bd migrate may have created a partial one)
    if dolt_dir.exists() {
        log_info!("[bd_migrate] Removing dolt/ directory before init");
        if let Err(e) = std::fs::remove_dir_all(&dolt_dir) {
            log_error!("[bd_migrate] Failed to remove dolt/: {}", e);
            return Err(format!("Failed to clean up dolt/ directory: {}", e));
        }
    }
    // Remove dolt-access.lock
    let dolt_lock2 = beads_dir.join("dolt-access.lock");
    if dolt_lock2.exists() {
        std::fs::remove_file(&dolt_lock2).ok();
    }
    // Backup main SQLite .db file (for comment restoration), then remove all SQLite files
    if let Ok(entries) = std::fs::read_dir(&beads_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".db") && !name.ends_with(".db.backup") {
                // Rename to .backup before deleting (preserves comments for Step 6)
                let backup_name = format!("{}.backup", name);
                let backup_path = beads_dir.join(&backup_name);
                if !backup_path.exists() {
                    log_info!("[bd_migrate] Backing up SQLite: {} -> {}", name, backup_name);
                    std::fs::rename(entry.path(), &backup_path).ok();
                } else {
                    log_info!("[bd_migrate] Removing SQLite file: {} (backup already exists)", name);
                    std::fs::remove_file(entry.path()).ok();
                }
            } else if name.ends_with(".db-shm") || name.ends_with(".db-wal") || name.ends_with(".db?mode=ro") {
                log_info!("[bd_migrate] Removing SQLite file: {}", name);
                std::fs::remove_file(entry.path()).ok();
            }
        }
    }

    // Reset metadata.json if it was set to dolt by a previous failed attempt
    let metadata_path = beads_dir.join("metadata.json");
    if metadata_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&metadata_path) {
            if content.contains("\"backend\":\"dolt\"") || content.contains("\"backend\": \"dolt\"") {
                log_info!("[bd_migrate] Resetting metadata.json backend from dolt to sqlite");
                std::fs::remove_file(&metadata_path).ok();
            }
        }
    }

    // Remove .local_version (stale after cleanup)
    let local_version = beads_dir.join(".local_version");
    if local_version.exists() {
        std::fs::remove_file(&local_version).ok();
    }

    // Step 1: bd init --prefix <prefix>
    let init_output = new_command(&binary)
        .args(&["init", "--prefix", &prefix])
        .current_dir(&working_dir)
        .env("PATH", get_extended_path())
        .env("BEADS_PATH", &working_dir)
        .output()
        .map_err(|e| format!("Failed to run bd init: {}", e))?;

    if !init_output.status.success() {
        let stderr = String::from_utf8_lossy(&init_output.stderr);
        return Err(format!("bd init failed: {}", stderr.trim()));
    }
    log_info!("[bd_migrate] bd init successful");

    // Step 2: Filter tombstone issues and sanitize fields for Dolt compatibility
    let temp_jsonl = beads_dir.join("_migrate_clean.jsonl");
    {
        let mut clean_lines = Vec::new();
        let mut skipped = 0u32;
        for line in jsonl_content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            match serde_json::from_str::<serde_json::Value>(trimmed) {
                Ok(mut v) => {
                    if v.get("status").and_then(|s| s.as_str()) == Some("tombstone") {
                        skipped += 1;
                        continue;
                    }
                    // Re-prefix issues with a different prefix to match the target
                    if let Some(id) = v.get("id").and_then(|i| i.as_str()).map(String::from) {
                        if let Some(last_dash) = id.rfind('-') {
                            let issue_prefix = &id[..last_dash];
                            if issue_prefix != prefix {
                                let suffix = &id[last_dash..]; // includes the '-'
                                let new_id = format!("{}{}", prefix, suffix);
                                let old_prefix = issue_prefix.to_string();
                                log_info!("[bd_migrate] Re-prefixing {} -> {}", id, new_id);
                                let obj = v.as_object_mut().unwrap();
                                obj.insert("id".to_string(), serde_json::Value::String(new_id));
                                // Re-prefix dependency references
                                if let Some(deps) = obj.get_mut("dependencies").and_then(|d| d.as_array_mut()) {
                                    for dep in deps.iter_mut() {
                                        if let Some(dep_obj) = dep.as_object_mut() {
                                            for key in &["issue_id", "depends_on_id"] {
                                                if let Some(val) = dep_obj.get(*key).and_then(|v| v.as_str()).map(String::from) {
                                                    if val.starts_with(&old_prefix) {
                                                        let new_val = format!("{}{}", prefix, &val[old_prefix.len()..]);
                                                        dep_obj.insert(key.to_string(), serde_json::Value::String(new_val));
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    // Truncate external_ref if it contains multiple lines (attachment paths)
                    // Dolt's external_ref column can't hold multi-line values with long paths
                    // Keep only the first line (the meaningful ref: redmine ID, URL, etc.)
                    let needs_truncate = v.get("external_ref")
                        .and_then(|e| e.as_str())
                        .map(|s| s.contains('\n') || s.len() > 100)
                        .unwrap_or(false);
                    if needs_truncate {
                        let ext_ref = v["external_ref"].as_str().unwrap();
                        let first_line = ext_ref.lines().next().unwrap_or("").to_string();
                        let issue_id = v.get("id").and_then(|i| i.as_str()).unwrap_or("?").to_string();
                        let orig_len = ext_ref.len();
                        v.as_object_mut().unwrap().insert(
                            "external_ref".to_string(),
                            serde_json::Value::String(first_line),
                        );
                        log_info!(
                            "[bd_migrate] Truncated external_ref for issue {} (was {} chars)",
                            issue_id, orig_len
                        );
                    }
                    clean_lines.push(serde_json::to_string(&v).unwrap_or_else(|_| trimmed.to_string()));
                }
                Err(_) => {
                    skipped += 1;
                    continue;
                }
            }
        }
        log_info!(
            "[bd_migrate] Filtered JSONL: {} valid, {} skipped (tombstone/malformed)",
            clean_lines.len(),
            skipped
        );

        // Empty project — no issues to import, just init is enough
        if clean_lines.is_empty() {
            log_info!("[bd_migrate] No issues to import — empty project, init-only migration");
            return Ok(MigrateResult {
                success: true,
                message: "Migration complete (empty project — initialized with Dolt backend)".to_string(),
            });
        }

        std::fs::write(&temp_jsonl, clean_lines.join("\n") + "\n")
            .map_err(|e| format!("Failed to write cleaned JSONL: {}", e))?;
    }

    // Step 3: bd import -i <cleaned_jsonl>
    let import_output = new_command(&binary)
        .args(&["import", "-i", &temp_jsonl.to_string_lossy()])
        .current_dir(&working_dir)
        .env("PATH", get_extended_path())
        .env("BEADS_PATH", &working_dir)
        .output()
        .map_err(|e| format!("Failed to run bd import: {}", e))?;

    // Clean up temp file
    std::fs::remove_file(&temp_jsonl).ok();

    if !import_output.status.success() {
        let stderr = String::from_utf8_lossy(&import_output.stderr);
        log_error!("[bd_migrate] Import failed: {}", stderr.trim());
        // Clean up failed migration so the modal will reappear
        if dolt_dir.exists() {
            std::fs::remove_dir_all(&dolt_dir).ok();
        }
        if beads_dir.join("dolt-access.lock").exists() {
            std::fs::remove_file(beads_dir.join("dolt-access.lock")).ok();
        }
        return Err(format!("Import failed: {}", stderr.trim()));
    }

    let stdout = String::from_utf8_lossy(&import_output.stdout);
    log_info!("[bd_migrate] Import successful: {}", stdout.trim());

    // Step 4: Restore labels (bd import doesn't preserve them)
    // Re-read JSONL to find issues with labels and apply them via bd update
    let mut labels_restored = 0u32;
    for line in jsonl_content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(trimmed) {
            if v.get("status").and_then(|s| s.as_str()) == Some("tombstone") {
                continue;
            }
            let labels: Vec<String> = v
                .get("labels")
                .and_then(|l| l.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();

            if labels.is_empty() {
                continue;
            }

            let issue_id = match v.get("id").and_then(|i| i.as_str()) {
                Some(id) => id,
                None => continue,
            };

            // bd update <id> --set-labels label1 --set-labels label2
            let mut args = vec!["update".to_string(), issue_id.to_string()];
            for label in &labels {
                args.push("--set-labels".to_string());
                args.push(label.clone());
            }

            let label_output = new_command(&binary)
                .args(&args.iter().map(|s| s.as_str()).collect::<Vec<_>>())
                .current_dir(&working_dir)
                .env("PATH", get_extended_path())
                .env("BEADS_PATH", &working_dir)
                .output();

            match label_output {
                Ok(o) if o.status.success() => {
                    labels_restored += 1;
                }
                Ok(o) => {
                    let stderr = String::from_utf8_lossy(&o.stderr);
                    log_info!("[bd_migrate] Failed to restore labels for {}: {}", issue_id, stderr.trim());
                }
                Err(e) => {
                    log_info!("[bd_migrate] Failed to run bd update for {}: {}", issue_id, e);
                }
            }
        }
    }

    if labels_restored > 0 {
        log_info!("[bd_migrate] Restored labels for {} issues", labels_restored);
    }

    // Step 5: Restore dependencies/relations (bd import doesn't preserve them)
    let mut deps_restored = 0u32;
    for line in jsonl_content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() { continue; }
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(trimmed) {
            if v.get("status").and_then(|s| s.as_str()) == Some("tombstone") { continue; }
            let dependencies = match v.get("dependencies").and_then(|d| d.as_array()) {
                Some(deps) if !deps.is_empty() => deps,
                _ => continue,
            };

            for dep in dependencies {
                let dep_obj = match dep.as_object() {
                    Some(o) => o,
                    None => continue,
                };

                let issue_id = match dep_obj.get("issue_id").and_then(|v| v.as_str()) {
                    Some(id) => id.to_string(),
                    None => continue,
                };
                let depends_on_id = match dep_obj.get("depends_on_id").and_then(|v| v.as_str()) {
                    Some(id) => id.to_string(),
                    None => continue,
                };
                let dep_type = dep_obj.get("type").and_then(|v| v.as_str()).unwrap_or("blocks").to_string();

                // Re-prefix if needed
                let issue_id = reprefix_id(&issue_id, &prefix, &prefix_counts);
                let depends_on_id = reprefix_id(&depends_on_id, &prefix, &prefix_counts);

                // bd dep add <issue_id> <depends_on_id> --type <type>
                let dep_output = new_command(&binary)
                    .args(&["dep", "add", &issue_id, &depends_on_id, "--type", &dep_type])
                    .current_dir(&working_dir)
                    .env("PATH", get_extended_path())
                    .env("BEADS_PATH", &working_dir)
                    .output();

                match dep_output {
                    Ok(o) if o.status.success() => { deps_restored += 1; }
                    Ok(o) => {
                        let stderr = String::from_utf8_lossy(&o.stderr);
                        log_info!("[bd_migrate] Failed to restore dep {} -> {}: {}", issue_id, depends_on_id, stderr.trim());
                    }
                    Err(e) => {
                        log_info!("[bd_migrate] Failed to run bd dep add: {}", e);
                    }
                }
            }
        }
    }

    if deps_restored > 0 {
        log_info!("[bd_migrate] Restored {} dependencies/relations", deps_restored);
    }

    // Step 6: Restore comments from SQLite backup (if available)
    // bd import doesn't preserve comments, and JSONL only has empty bodies.
    // Look for a .db.backup file with a comments table.
    let mut comments_restored = 0u32;
    let sqlite_backup = {
        let mut found: Option<std::path::PathBuf> = None;
        if let Ok(entries) = std::fs::read_dir(&beads_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.ends_with(".db.backup") {
                    found = Some(entry.path());
                    break;
                }
            }
        }
        found
    };

    if let Some(backup_path) = sqlite_backup {
        log_info!("[bd_migrate] Found SQLite backup: {:?}, restoring comments", backup_path);
        // Use sqlite3 CLI to extract comments as JSON
        let sqlite_output = std::process::Command::new("sqlite3")
            .args(&[
                backup_path.to_string_lossy().as_ref(),
                "-json",
                "SELECT issue_id, author, text FROM comments WHERE text IS NOT NULL AND text != '' ORDER BY created_at ASC",
            ])
            .output();

        if let Ok(output) = sqlite_output {
            if output.status.success() {
                let json_str = String::from_utf8_lossy(&output.stdout);
                if let Ok(rows) = serde_json::from_str::<Vec<serde_json::Value>>(&json_str) {
                    for row in &rows {
                        let issue_id = match row.get("issue_id").and_then(|v| v.as_str()) {
                            Some(id) => id.to_string(),
                            None => continue,
                        };
                        let author = row.get("author").and_then(|v| v.as_str()).unwrap_or("unknown");
                        let text = match row.get("text").and_then(|v| v.as_str()) {
                            Some(t) if !t.is_empty() => t,
                            _ => continue,
                        };

                        // Re-prefix if needed
                        let issue_id = reprefix_id(&issue_id, &prefix, &prefix_counts);

                        // Write comment to temp file to handle multiline text
                        let comment_file = beads_dir.join("_migrate_comment.txt");
                        if std::fs::write(&comment_file, text).is_err() {
                            continue;
                        }

                        let comment_output = new_command(&binary)
                            .args(&["comments", "add", &issue_id, "-f", &comment_file.to_string_lossy(), "--author", author])
                            .current_dir(&working_dir)
                            .env("PATH", get_extended_path())
                            .env("BEADS_PATH", &working_dir)
                            .output();

                        match comment_output {
                            Ok(o) if o.status.success() => { comments_restored += 1; }
                            Ok(o) => {
                                let stderr = String::from_utf8_lossy(&o.stderr);
                                log_info!("[bd_migrate] Failed to restore comment for {}: {}", issue_id, stderr.trim());
                            }
                            Err(e) => {
                                log_info!("[bd_migrate] Failed to run bd comments add: {}", e);
                            }
                        }
                        std::fs::remove_file(&comment_file).ok();
                    }
                }
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                log_info!("[bd_migrate] sqlite3 query failed: {}", stderr.trim());
            }
        }

        if comments_restored > 0 {
            log_info!("[bd_migrate] Restored {} comments from SQLite backup", comments_restored);
        }
    }

    Ok(MigrateResult {
        success: true,
        message: format!(
            "Migration to Dolt completed (via init+import). {} Labels: {}. Deps: {}. Comments: {}.",
            stdout.trim(),
            labels_restored,
            deps_restored,
            comments_restored,
        ),
    })
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

/// Batched poll: sync once, then fetch all issues + ready in 2 commands (was 3).
/// Replaces 3 separate IPC calls (bd_list + bd_list(closed) + bd_ready) with one.
#[tauri::command]
async fn bd_poll_data(cwd: Option<String>) -> Result<PollData, String> {
    if is_builtin_backend() {
        return with_engine(cwd.as_deref(), |engine| {
            let all_issues = engine.list_issues(Some("all"))
                .map_err(|e| format!("list_issues: {}", e))?;
            let (open_raw, closed_raw): (Vec<_>, Vec<_>) = all_issues
                .into_iter()
                .partition(|i| i.status != "closed" && i.status != "tombstone");
            let ready_raw = engine.list_ready_issues()
                .map_err(|e| format!("list_ready_issues: {}", e))?;

            Ok(PollData {
                open_issues: open_raw.into_iter().map(tracker::convert::tracker_issue_to_issue).collect(),
                closed_issues: closed_raw.into_iter().map(tracker::convert::tracker_issue_to_issue).collect(),
                ready_issues: ready_raw.into_iter().map(tracker::convert::tracker_issue_to_issue).collect(),
            })
        });
    }

    log_info!("[bd_poll_data] Batched poll starting");

    let cwd_ref = cwd.as_deref();

    // Single sync for the entire poll cycle
    sync_bd_database(cwd_ref);

    // Fetch issues: single --all call for bd >= 0.55, fallback to 2 calls for older versions
    let (raw_open, raw_closed) = if supports_list_all_flag() {
        let all_output = execute_bd("list", &["--all".to_string(), "--limit=0".to_string()], cwd_ref)?;
        let raw_all = parse_issues_tolerant(&all_output, "bd_poll_data_all")?;
        let (open, closed): (Vec<_>, Vec<_>) = raw_all.into_iter()
            .partition(|issue: &BdRawIssue| issue.status != "closed");
        (open, closed)
    } else {
        let open_output = execute_bd("list", &["--limit=0".to_string()], cwd_ref)?;
        let closed_output = execute_bd("list", &["--status=closed".to_string(), "--limit=0".to_string()], cwd_ref)?;
        (
            parse_issues_tolerant(&open_output, "bd_poll_data_open")?,
            parse_issues_tolerant(&closed_output, "bd_poll_data_closed")?,
        )
    };

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
            let mut map = LAST_KNOWN_MTIME.lock().unwrap();
            map.insert(working_dir, mtime);
        }
    }

    Ok(PollData {
        open_issues: raw_open.into_iter().map(transform_issue).collect(),
        closed_issues: raw_closed.into_iter().map(transform_issue).collect(),
        ready_issues: raw_ready.into_iter().map(transform_issue).collect(),
    })
}

/// Get the latest mtime across all beads database files.
/// - Dolt backend (bd >= 0.50.0): checks .beads/ dir, .beads/.dolt/ (legacy) or
///   .beads/dolt/<name>/.dolt/ (bd 0.52+ nested layout), and manifest files
/// - SQLite backend: checks beads.db, beads.db-wal, and optionally issues.jsonl
fn get_beads_mtime(beads_dir: &std::path::Path) -> Option<std::time::SystemTime> {
    if project_uses_dolt(beads_dir) {
        // Dolt backend: check directory mtimes and manifest files
        let mut times: Vec<std::time::SystemTime> = Vec::new();

        // .beads/ dir mtime
        if let Ok(m) = fs::metadata(beads_dir) {
            if let Ok(t) = m.modified() { times.push(t); }
        }

        // Collect all .dolt/ directories to check:
        // - Legacy layout: .beads/.dolt/
        // - Nested layout (bd 0.52+): .beads/dolt/<name>/.dolt/
        let mut dolt_dirs: Vec<std::path::PathBuf> = Vec::new();

        let legacy_dolt = beads_dir.join(".dolt");
        if legacy_dolt.is_dir() {
            dolt_dirs.push(legacy_dolt);
        }

        let nested_dolt = beads_dir.join("dolt");
        if nested_dolt.is_dir() {
            if let Ok(entries) = std::fs::read_dir(&nested_dolt) {
                for entry in entries.flatten() {
                    let sub_dolt = entry.path().join(".dolt");
                    if sub_dolt.is_dir() {
                        dolt_dirs.push(sub_dolt);
                    }
                }
            }
        }

        // Check mtime of each .dolt/ dir and its manifest files
        for dolt_dir in &dolt_dirs {
            if let Ok(m) = fs::metadata(dolt_dir) {
                if let Ok(t) = m.modified() { times.push(t); }
            }
            for name in &["manifest", "noms/manifest"] {
                let p = dolt_dir.join(name);
                if let Ok(m) = fs::metadata(&p) {
                    if let Ok(t) = m.modified() { times.push(t); }
                }
            }
        }

        // Also check issues.jsonl (Dolt exports to it for git sync)
        let jsonl_path = beads_dir.join("issues.jsonl");
        if let Ok(m) = fs::metadata(&jsonl_path) {
            if let Ok(t) = m.modified() { times.push(t); }
        }

        times.into_iter().max()
    } else {
        // SQLite backend: check db, WAL, and optionally JSONL
        let mut paths = vec![
            beads_dir.join("beads.db"),
            beads_dir.join("beads.db-wal"),
        ];
        if uses_jsonl_files() {
            paths.push(beads_dir.join("issues.jsonl"));
        }
        paths.iter()
            .filter_map(|p| fs::metadata(p).and_then(|m| m.modified()).ok())
            .max()
    }
}

// ============================================================================
// Built-in tracker backend commands
// ============================================================================

#[tauri::command]
async fn get_backend_mode() -> String {
    BACKEND_MODE.lock().unwrap().clone()
}

#[tauri::command]
async fn set_backend_mode(mode: String) -> Result<(), String> {
    match mode.as_str() {
        "bd" | "br" | "built-in" => {
            let mut m = BACKEND_MODE.lock().unwrap();
            *m = mode;
            Ok(())
        }
        _ => Err(format!("Invalid backend mode: {}. Must be 'bd', 'br', or 'built-in'.", mode)),
    }
}

#[tauri::command]
async fn tracker_init(cwd: Option<String>) -> Result<(), String> {
    let project_path = cwd
        .or_else(|| env::var("BEADS_PATH").ok())
        .unwrap_or_else(|| {
            env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| ".".to_string())
        });
    let path = std::path::Path::new(&project_path);
    let config = tracker::ProjectConfig::load(path);
    let engine = tracker::Engine::init(path, config)
        .map_err(|e| format!("Failed to initialize tracker: {}", e))?;

    let mut engines = TRACKER_ENGINES.lock().unwrap();
    engines.insert(project_path, engine);
    Ok(())
}

#[tauri::command]
async fn tracker_detect(cwd: Option<String>) -> Result<bool, String> {
    let project_path = cwd
        .or_else(|| env::var("BEADS_PATH").ok())
        .unwrap_or_else(|| {
            env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| ".".to_string())
        });
    let tracker_db = std::path::Path::new(&project_path)
        .join(".tracker")
        .join("tracker.db");
    Ok(tracker_db.exists())
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

    // Built-in backend: check .tracker/tracker.db mtime instead of .beads/
    if is_builtin_backend() {
        let tracker_db = std::path::Path::new(&working_dir)
            .join(".tracker")
            .join("tracker.db");
        let current_mtime = fs::metadata(&tracker_db)
            .and_then(|m| m.modified())
            .ok();

        let mut map = LAST_KNOWN_MTIME.lock().unwrap();
        let previous = map.get(&working_dir).copied();

        return match (current_mtime, previous) {
            (Some(current), Some(prev)) => {
                if current != prev {
                    map.insert(working_dir, current);
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            (Some(current), None) => {
                map.insert(working_dir, current);
                Ok(true)
            }
            (None, _) => Ok(true),
        };
    }

    let beads_dir = std::path::Path::new(&working_dir).join(".beads");
    let current_mtime = get_beads_mtime(&beads_dir);

    let mut map = LAST_KNOWN_MTIME.lock().unwrap();
    let previous = map.get(&working_dir).copied();

    match (current_mtime, previous) {
        (Some(current), Some(prev)) => {
            if current != prev {
                log_info!("[bd_check_changed] mtime changed — data may have been modified");
                map.insert(working_dir, current);
                Ok(true)
            } else {
                log_debug!("[bd_check_changed] mtime unchanged — no changes");
                Ok(false)
            }
        }
        (Some(current), None) => {
            // First check — store mtime, report changed so initial load happens
            map.insert(working_dir, current);
            Ok(true)
        }
        (None, _) => {
            // No database file found
            log_warn!("[bd_check_changed] No beads database found in {}", working_dir);
            Ok(true) // Report changed to let caller handle missing db
        }
    }
}

/// Reset the cached mtime for a specific project (or all projects).
/// Called from the frontend when switching projects to force a fresh poll.
#[tauri::command]
async fn bd_reset_mtime(cwd: Option<String>) -> Result<(), String> {
    let mut map = LAST_KNOWN_MTIME.lock().unwrap();
    if let Some(path) = cwd {
        log_info!("[bd_reset_mtime] Resetting mtime for: {}", path);
        map.remove(&path);
    } else {
        log_info!("[bd_reset_mtime] Resetting all cached mtimes");
        map.clear();
    }
    Ok(())
}

#[tauri::command]
async fn bd_list(options: ListOptions) -> Result<Vec<Issue>, String> {
    if is_builtin_backend() {
        return with_engine(options.cwd.as_deref(), |engine| {
            // Determine status filter
            let status_filter = if options.include_all.unwrap_or(false) {
                Some("all")
            } else if let Some(ref statuses) = options.status {
                if statuses.len() == 1 {
                    // Single status: pass directly
                    Some(statuses[0].as_str())
                } else {
                    Some("all") // Multi-status: fetch all, filter in-memory
                }
            } else {
                None // Default: open only
            };

            let mut issues = engine.list_issues(status_filter)
                .map_err(|e| format!("list_issues: {}", e))?;

            // In-memory filtering for type, priority, assignee, multi-status
            if let Some(ref statuses) = options.status {
                if statuses.len() > 1 {
                    issues.retain(|i| statuses.contains(&i.status));
                }
            }
            if let Some(ref types) = options.issue_type {
                if !types.is_empty() {
                    issues.retain(|i| types.contains(&i.issue_type));
                }
            }
            if let Some(ref priorities) = options.priority {
                if !priorities.is_empty() {
                    issues.retain(|i| priorities.contains(&i.priority));
                }
            }
            if let Some(ref assignee) = options.assignee {
                issues.retain(|i| i.assignee.as_deref() == Some(assignee.as_str()));
            }

            Ok(issues.into_iter().map(tracker::convert::tracker_issue_to_issue).collect())
        });
    }

    log_info!("[bd_list] cwd: {:?}", options.cwd);

    // Sync database before reading to ensure data is up-to-date
    sync_bd_database(options.cwd.as_deref());

    let mut args: Vec<String> = Vec::new();

    // --all flag only works correctly on bd >= 0.55; for older versions, fallback to 2 calls
    let use_all = options.include_all.unwrap_or(false);
    if use_all && !supports_list_all_flag() {
        // Fallback: fetch open + closed separately and merge
        log_info!("[bd_list] --all requested but bd < 0.55 — falling back to 2 calls");
        let mut fallback_args = args.clone();
        fallback_args.push("--limit=0".to_string());

        let open_output = execute_bd("list", &fallback_args, options.cwd.as_deref())?;
        let open_issues = parse_issues_tolerant(&open_output, "bd_list_open")?;

        fallback_args.push("--status=closed".to_string());
        let closed_output = execute_bd("list", &fallback_args, options.cwd.as_deref())?;
        let closed_issues = parse_issues_tolerant(&closed_output, "bd_list_closed")?;

        let mut all_issues = open_issues;
        all_issues.extend(closed_issues);
        log_info!("[bd_list] Found {} issues (fallback)", all_issues.len());
        return Ok(all_issues.into_iter().map(transform_issue).collect());
    }

    if use_all {
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
    if is_builtin_backend() {
        return with_engine(options.cwd.as_deref(), |engine| {
            let issues = engine.list_issues(Some("all"))
                .map_err(|e| format!("list_issues: {}", e))?;

            let mut by_type: HashMap<String, usize> = HashMap::new();
            for t in &["bug", "task", "feature", "epic", "chore"] {
                by_type.insert(t.to_string(), 0);
            }
            let mut by_priority: HashMap<String, usize> = HashMap::new();
            for p in &["p0", "p1", "p2", "p3", "p4"] {
                by_priority.insert(p.to_string(), 0);
            }
            let mut last_updated: Option<String> = None;

            for issue in &issues {
                if let Some(count) = by_type.get_mut(&issue.issue_type) {
                    *count += 1;
                }
                if let Some(count) = by_priority.get_mut(&issue.priority) {
                    *count += 1;
                }
                if last_updated.is_none() || issue.updated_at > *last_updated.as_ref().unwrap() {
                    last_updated = Some(issue.updated_at.clone());
                }
            }

            Ok(CountResult {
                count: issues.len(),
                by_type,
                by_priority,
                last_updated,
            })
        });
    }

    // Sync database before reading to ensure data is up-to-date
    sync_bd_database(options.cwd.as_deref());

    // Fetch all issues: single --all call for bd >= 0.55, fallback to 2 calls for older versions
    let raw_issues = if supports_list_all_flag() {
        let all_output = execute_bd("list", &["--all".to_string(), "--limit=0".to_string()], options.cwd.as_deref())?;
        parse_issues_tolerant(&all_output, "bd_count_all")?
    } else {
        let open_output = execute_bd("list", &["--limit=0".to_string()], options.cwd.as_deref())?;
        let closed_output = execute_bd("list", &["--status=closed".to_string(), "--limit=0".to_string()], options.cwd.as_deref())?;
        let mut issues = parse_issues_tolerant(&open_output, "bd_count_open")?;
        issues.extend(parse_issues_tolerant(&closed_output, "bd_count_closed")?);
        issues
    };

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
    if is_builtin_backend() {
        return with_engine(options.cwd.as_deref(), |engine| {
            let issues = engine.list_ready_issues()
                .map_err(|e| format!("list_ready_issues: {}", e))?;
            Ok(issues.into_iter().map(tracker::convert::tracker_issue_to_issue).collect())
        });
    }

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
    if is_builtin_backend() {
        return with_engine(options.cwd.as_deref(), |engine| {
            match engine.get_issue(&id) {
                Ok(ti) => {
                    let mut issue = tracker::convert::tracker_issue_to_issue(ti);
                    // Enrich with children
                    if let Ok(children) = engine.list_children(&id) {
                        if !children.is_empty() {
                            issue.children = Some(
                                children.iter().map(tracker::convert::tracker_issue_to_child).collect()
                            );
                        }
                    }
                    Ok(Some(issue))
                }
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(format!("get_issue: {}", e)),
            }
        });
    }

    log_info!("[bd_show] Called for issue: {} with cwd: {:?}", id, options.cwd);

    // Sync database before reading to ensure data is up-to-date
    sync_bd_database(options.cwd.as_deref());

    let output = match execute_bd("show", std::slice::from_ref(&id), options.cwd.as_deref()) {
        Ok(output) => output,
        Err(e) => {
            // Handle "not found" errors gracefully (future bd versions may use non-zero exit)
            let err_lower = e.to_lowercase();
            if err_lower.contains("no issue found") || err_lower.contains("not found") {
                log_info!("[bd_show] Issue {} not found (error from bd): {}", id, e);
                return Ok(None);
            }
            return Err(e);
        }
    };

    // Handle empty output (current bd behavior for missing issues: exit 0, empty stdout)
    let trimmed = output.trim();
    if trimmed.is_empty() {
        log_info!("[bd_show] Issue {} not found (empty output from bd)", id);
        return Ok(None);
    }

    // bd show can return either a single object or an array
    let result: serde_json::Value = serde_json::from_str(trimmed)
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
    if is_builtin_backend() {
        return with_engine(payload.cwd.as_deref(), |engine| {
            let params = tracker::convert::create_payload_to_params(&payload);
            let ti = engine.create_issue(params)
                .map_err(|e| format!("create_issue: {}", e))?;
            Ok(Some(tracker::convert::tracker_issue_to_issue(ti)))
        });
    }

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
    if let Some(ref spec_id) = payload.spec_id {
        if !spec_id.is_empty() {
            args.push("--spec-id".to_string());
            args.push(spec_id.clone());
        }
    }

    let output = execute_bd("create", &args, payload.cwd.as_deref())?;

    let raw_issue: BdRawIssue = serde_json::from_str(&output)
        .map_err(|e| format!("Failed to parse created issue: {}", e))?;

    Ok(Some(transform_issue(raw_issue)))
}

#[tauri::command]
async fn bd_update(id: String, updates: UpdatePayload) -> Result<Option<Issue>, String> {
    if is_builtin_backend() {
        return with_engine(updates.cwd.as_deref(), |engine| {
            let params = tracker::convert::update_payload_to_params(&updates);
            let ti = engine.update_issue(&id, params)
                .map_err(|e| format!("update_issue: {}", e))?;
            Ok(Some(tracker::convert::tracker_issue_to_issue(ti)))
        });
    }

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
        args.push(ext.clone());
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
    if let Some(ref metadata) = updates.metadata {
        args.push("--metadata".to_string());
        args.push(metadata.clone());
    }
    if let Some(ref spec_id) = updates.spec_id {
        args.push("--spec-id".to_string());
        args.push(spec_id.clone());
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
    if is_builtin_backend() {
        return with_engine(options.cwd.as_deref(), |engine| {
            let ti = engine.close_issue(&id)
                .map_err(|e| format!("close_issue: {}", e))?;
            serde_json::to_value(tracker::convert::tracker_issue_to_issue(ti))
                .map_err(|e| format!("serialize: {}", e))
        });
    }

    log_info!("[bd_close] Closing issue: {} with cwd: {:?}", id, options.cwd);

    let mut args = vec![id.clone()];
    // br supports --suggest-next for showing newly unblocked issues
    if matches!(get_cli_client_info(), Some((CliClient::Br, _, _, _))) {
        args.push("--suggest-next".to_string());
    }

    let output = execute_bd("close", &args, options.cwd.as_deref())?;

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
async fn bd_search(query: String, options: CwdOptions) -> Result<Vec<Issue>, String> {
    if is_builtin_backend() {
        return with_engine(options.cwd.as_deref(), |engine| {
            let results = engine.search(&query, Some(50))
                .map_err(|e| format!("search: {}", e))?;
            // Fetch full issues for each search result
            let mut issues = Vec::with_capacity(results.len());
            for result in results {
                if let Ok(ti) = engine.get_issue(&result.issue_id) {
                    issues.push(tracker::convert::tracker_issue_to_issue(ti));
                }
            }
            Ok(issues)
        });
    }

    log_info!("[bd_search] Searching for: {} with cwd: {:?}", query, options.cwd);

    let args = vec![query];
    let output = execute_bd("search", &args, options.cwd.as_deref())?;

    log_info!("[bd_search] Raw output: {}", output.chars().take(500).collect::<String>());

    let trimmed = output.trim();
    if trimmed.is_empty() || trimmed == "[]" {
        return Ok(vec![]);
    }

    let raw: Vec<BdRawIssue> = serde_json::from_str(trimmed)
        .map_err(|e| {
            log_error!("[bd_search] Failed to parse JSON: {}", e);
            format!("Failed to parse search results: {}", e)
        })?;

    Ok(raw.into_iter().map(transform_issue).collect())
}

#[tauri::command]
async fn bd_label_add(id: String, label: String, options: CwdOptions) -> Result<(), String> {
    if is_builtin_backend() {
        return with_engine(options.cwd.as_deref(), |engine| {
            engine.add_label(&id, &label)
                .map_err(|e| format!("add_label: {}", e))
        });
    }
    log_info!("[bd_label_add] Adding label '{}' to issue {}", label, id);
    let args = vec![id, label];
    execute_bd("label add", &args, options.cwd.as_deref())?;
    Ok(())
}

#[tauri::command]
async fn bd_label_remove(id: String, label: String, options: CwdOptions) -> Result<(), String> {
    if is_builtin_backend() {
        return with_engine(options.cwd.as_deref(), |engine| {
            engine.remove_label(&id, &label)
                .map_err(|e| format!("remove_label: {}", e))
        });
    }
    log_info!("[bd_label_remove] Removing label '{}' from issue {}", label, id);
    let args = vec![id, label];
    execute_bd("label remove", &args, options.cwd.as_deref())?;
    Ok(())
}

#[tauri::command]
async fn bd_delete(id: String, options: CwdOptions) -> Result<serde_json::Value, String> {
    if is_builtin_backend() {
        with_engine(options.cwd.as_deref(), |engine| {
            engine.delete_issue(&id, true)
                .map_err(|e| format!("delete_issue: {}", e))?;
            Ok(serde_json::json!({ "success": true, "id": id }))
        })?;
        // Fall through to attachment cleanup below
    } else {
        let mut args = vec![id.clone(), "--force".to_string()];
        if supports_delete_hard_flag() {
            args.push("--hard".to_string());
        }
        log::info!("[bd_delete] Deleting issue: {} with args: {:?}", id, args);
        execute_bd("delete", &args, options.cwd.as_deref())?;

        // Sync after delete to push deletion to remote and prevent resurrection
        sync_bd_database(options.cwd.as_deref());
    }

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
            let att_dir = abs_path.join(".beads").join("attachments").join(issue_short_id(&id));
            if att_dir.exists() && att_dir.is_dir() {
                if let Err(e) = fs::remove_dir_all(&att_dir) {
                    log::warn!("[bd_delete] Failed to remove attachments folder: {}", e);
                } else {
                    log::info!("[bd_delete] Removed attachments folder: {:?}", att_dir);
                }
            }
        }
    }

    Ok(serde_json::json!({ "success": true, "id": id }))
}

#[tauri::command]
async fn bd_comments_add(id: String, content: String, options: CwdOptions) -> Result<serde_json::Value, String> {
    if is_builtin_backend() {
        return with_engine(options.cwd.as_deref(), |engine| {
            // Use a default actor for now
            engine.add_comment(&id, "user", &content)
                .map_err(|e| format!("add_comment: {}", e))?;
            Ok(serde_json::json!({ "success": true }))
        });
    }

    let args = vec![id, content];

    execute_bd("comments add", &args, options.cwd.as_deref())?;

    Ok(serde_json::json!({ "success": true }))
}

#[tauri::command]
async fn bd_dep_add(issue_id: String, blocker_id: String, options: CwdOptions) -> Result<serde_json::Value, String> {
    if is_builtin_backend() {
        return with_engine(options.cwd.as_deref(), |engine| {
            engine.add_dependency(&issue_id, &blocker_id, "blocks")
                .map_err(|e| format!("add_dependency: {}", e))?;
            Ok(serde_json::json!({ "success": true }))
        });
    }

    let args = vec![issue_id, blocker_id];

    execute_bd("dep add", &args, options.cwd.as_deref())?;

    Ok(serde_json::json!({ "success": true }))
}

#[tauri::command]
async fn bd_dep_remove(issue_id: String, blocker_id: String, options: CwdOptions) -> Result<serde_json::Value, String> {
    if is_builtin_backend() {
        return with_engine(options.cwd.as_deref(), |engine| {
            engine.remove_dependency(&issue_id, &blocker_id)
                .map_err(|e| format!("remove_dependency: {}", e))?;
            Ok(serde_json::json!({ "success": true }))
        });
    }

    let args = vec![issue_id, blocker_id];

    execute_bd("dep remove", &args, options.cwd.as_deref())?;

    Ok(serde_json::json!({ "success": true }))
}

#[tauri::command]
async fn bd_dep_add_relation(id1: String, id2: String, relation_type: String, options: CwdOptions) -> Result<serde_json::Value, String> {
    if is_builtin_backend() {
        return with_engine(options.cwd.as_deref(), |engine| {
            engine.add_dependency(&id1, &id2, &relation_type)
                .map_err(|e| format!("add_dependency: {}", e))?;
            Ok(serde_json::json!({ "success": true }))
        });
    }

    let args = vec![id1, id2, "--type".to_string(), relation_type];

    execute_bd("dep add", &args, options.cwd.as_deref())?;

    Ok(serde_json::json!({ "success": true }))
}

#[tauri::command]
async fn bd_dep_remove_relation(id1: String, id2: String, options: CwdOptions) -> Result<serde_json::Value, String> {
    if is_builtin_backend() {
        return with_engine(options.cwd.as_deref(), |engine| {
            engine.remove_dependency(&id1, &id2)
                .map_err(|e| format!("remove_dependency: {}", e))?;
            Ok(serde_json::json!({ "success": true }))
        });
    }

    let args = vec![id1, id2];

    execute_bd("dep remove", &args, options.cwd.as_deref())?;

    Ok(serde_json::json!({ "success": true }))
}

#[tauri::command]
async fn bd_available_relation_types() -> Vec<serde_json::Value> {
    let common: Vec<(&str, &str)> = vec![
        ("relates-to", "Relates To"),
        ("related", "Related"),
        ("discovered-from", "Discovered From"),
        ("duplicates", "Duplicates"),
        ("supersedes", "Supersedes"),
        ("caused-by", "Caused By"),
        ("replies-to", "Replies To"),
    ];
    let bd_only: Vec<(&str, &str)> = vec![
        ("tracks", "Tracks"),
        ("until", "Until"),
        ("validates", "Validates"),
    ];

    let types = if is_builtin_backend() {
        // Built-in backend supports all common types (same as br)
        common
    } else {
        match get_cli_client_info() {
            Some((CliClient::Br, _, _, _)) => common,
            _ => {
                let mut all = common;
                all.extend(bd_only);
                all
            }
        }
    };

    types.into_iter().map(|(v, l)| serde_json::json!({ "value": v, "label": l })).collect()
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
            let uses_dolt = has_beads && project_uses_dolt(&beads_path);

            directories.push(DirectoryEntry {
                name,
                path: full_path.to_string_lossy().to_string(),
                is_directory: true,
                has_beads,
                uses_dolt,
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

    let current_beads_path = target_path.join(".beads");
    let current_has_beads = current_beads_path.is_dir();
    let current_uses_dolt = current_has_beads && project_uses_dolt(&current_beads_path);

    Ok(FsListResult {
        current_path: target_path.to_string_lossy().to_string(),
        has_beads: current_has_beads,
        uses_dolt: current_uses_dolt,
        entries: directories,
    })
}

// File watcher commands removed - replaced by frontend polling for lower CPU usage

// ============================================================================
// Update Checker
// ============================================================================

const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const GITHUB_RELEASES_URL: &str = "https://api.github.com/repos/w3dev33/beads-task-issue-tracker/releases/latest";

/// Get a GitHub token from `gh auth token` (if gh CLI is installed and authenticated).
/// Raises the API rate limit from 60/hour (anonymous) to 5,000/hour (authenticated).
fn get_github_token() -> Option<String> {
    // Check GITHUB_TOKEN env var first
    if let Ok(token) = env::var("GITHUB_TOKEN") {
        if !token.is_empty() {
            return Some(token);
        }
    }
    // Fall back to gh CLI
    let output = new_command("gh")
        .args(&["auth", "token"])
        .output()
        .ok()?;
    if output.status.success() {
        let token = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !token.is_empty() {
            return Some(token);
        }
    }
    None
}

/// Build a reqwest client with GitHub auth if available.
fn github_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .user_agent("beads-task-issue-tracker")
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))
}

/// Add GitHub auth header to a request if a token is available.
fn with_github_auth(req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
    match get_github_token() {
        Some(token) => req.bearer_auth(token),
        None => req,
    }
}

fn get_platform_string() -> &'static str {
    if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else {
        "linux"
    }
}

fn find_platform_asset(assets: &[GitHubAsset]) -> Option<&GitHubAsset> {
    let suffix = if cfg!(target_os = "macos") {
        if cfg!(target_arch = "aarch64") {
            "_macOS-ARM64.dmg"
        } else {
            "_macOS-Intel.dmg"
        }
    } else if cfg!(target_os = "windows") {
        "_Windows.msi"
    } else {
        "_Linux-amd64.AppImage"
    };

    assets.iter().find(|a| a.name.ends_with(suffix))
}

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
    let client = github_client()?;

    let response = with_github_auth(client.get(GITHUB_RELEASES_URL))
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
            download_url: None,
            platform: get_platform_string().to_string(),
            release_notes: None,
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

    let download_url = find_platform_asset(&release.assets)
        .map(|a| a.browser_download_url.clone());

    // Fetch CHANGELOG.md via GitHub API (raw.githubusercontent CDN ignores query params for caching)
    let changelog = with_github_auth(
        client
            .get("https://api.github.com/repos/w3dev33/beads-task-issue-tracker/contents/CHANGELOG.md")
            .header("Accept", "application/vnd.github.raw+json")
    )
        .send()
        .await
        .ok()
        .and_then(|r| if r.status().is_success() { Some(r) } else { None });
    let changelog_text = match changelog {
        Some(r) => r.text().await.ok(),
        None => None,
    };

    Ok(UpdateInfo {
        current_version: CURRENT_VERSION.to_string(),
        latest_version,
        has_update,
        release_url: release.html_url,
        download_url,
        platform: get_platform_string().to_string(),
        release_notes: changelog_text.or(release.body),
    })
}

#[tauri::command]
async fn check_for_updates_demo() -> Result<UpdateInfo, String> {
    let client = github_client()?;

    let response = with_github_auth(client.get(GITHUB_RELEASES_URL))
        .send()
        .await
        .map_err(|e| format!("Failed to fetch releases: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("GitHub API returned status: {}", response.status()));
    }

    let release: GitHubRelease = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse release info: {}", e))?;

    let latest_version = release.tag_name.trim_start_matches('v').to_string();

    let download_url = find_platform_asset(&release.assets)
        .map(|a| a.browser_download_url.clone());

    // Fetch CHANGELOG.md via GitHub API (raw.githubusercontent CDN ignores query params for caching)
    let changelog = with_github_auth(
        client
            .get("https://api.github.com/repos/w3dev33/beads-task-issue-tracker/contents/CHANGELOG.md")
            .header("Accept", "application/vnd.github.raw+json")
    )
        .send()
        .await
        .ok()
        .and_then(|r| if r.status().is_success() { Some(r) } else { None });
    let changelog_text = match changelog {
        Some(r) => r.text().await.ok(),
        None => None,
    };

    // Demo mode: force has_update = true, fake current version as 0.0.0
    Ok(UpdateInfo {
        current_version: "0.0.0".to_string(),
        latest_version,
        has_update: true,
        release_url: release.html_url,
        download_url,
        platform: get_platform_string().to_string(),
        release_notes: changelog_text.or(release.body),
    })
}

#[tauri::command]
async fn check_bd_cli_update() -> Result<BdCliUpdateInfo, String> {
    // Get current bd version
    let version_str = get_bd_version().await;
    if version_str.contains("not found") {
        return Err("bd CLI not found".to_string());
    }

    // Parse semver from version string
    let current_tuple = parse_bd_version(&version_str)
        .ok_or_else(|| format!("Could not parse version from: {}", version_str))?;
    let current_version = format!("{}.{}.{}", current_tuple.0, current_tuple.1, current_tuple.2);

    // Determine the correct GitHub repo based on client type (bd vs br)
    let client_type = detect_cli_client(&version_str);
    let api_url = match client_type {
        CliClient::Br => "https://api.github.com/repos/Dicklesworthstone/beads_rust/releases/latest",
        _ => "https://api.github.com/repos/steveyegge/beads/releases/latest",
    };
    let releases_url = match client_type {
        CliClient::Br => "https://github.com/Dicklesworthstone/beads_rust/releases",
        _ => "https://github.com/steveyegge/beads/releases",
    };

    let client = github_client()?;

    let response = with_github_auth(client.get(api_url))
        .send()
        .await
        .map_err(|e| format!("Failed to fetch releases: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("GitHub API returned status: {}", response.status()));
    }

    let release: GitHubRelease = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse release info: {}", e))?;

    let latest_version = release.tag_name.trim_start_matches('v').to_string();
    let has_update = compare_versions(&current_version, &latest_version);

    Ok(BdCliUpdateInfo {
        current_version,
        latest_version,
        has_update,
        release_url: releases_url.to_string(),
    })
}

#[tauri::command]
async fn download_and_install_update(download_url: String) -> Result<String, String> {
    log::info!("[download_update] Starting download from: {}", download_url);

    // Extract filename from URL
    let filename = download_url
        .rsplit('/')
        .next()
        .unwrap_or("update-download")
        .to_string();
    log::info!("[download_update] Target filename: {}", filename);

    // Download the file
    let client = reqwest::Client::builder()
        .user_agent("beads-task-issue-tracker")
        .build()
        .map_err(|e| {
            log::error!("[download_update] Failed to create HTTP client: {}", e);
            format!("Failed to create HTTP client: {}", e)
        })?;

    log::info!("[download_update] Sending GET request...");
    let response = client
        .get(&download_url)
        .send()
        .await
        .map_err(|e| {
            log::error!("[download_update] HTTP request failed: {} (url: {})", e, download_url);
            format!("Failed to download update: {}", e)
        })?;

    let status = response.status();
    let final_url = response.url().to_string();
    log::info!("[download_update] Response status: {} (final URL: {})", status, final_url);

    if !status.is_success() {
        log::error!("[download_update] Download failed with status: {} (url: {})", status, final_url);
        return Err(format!("Download failed with status: {}", status));
    }

    log::info!("[download_update] Reading response bytes...");
    let bytes = response
        .bytes()
        .await
        .map_err(|e| {
            log::error!("[download_update] Failed to read response bytes: {}", e);
            format!("Failed to read download bytes: {}", e)
        })?;
    log::info!("[download_update] Downloaded {} bytes", bytes.len());

    // Save to ~/Downloads
    let download_dir = dirs::download_dir()
        .ok_or_else(|| {
            log::error!("[download_update] Could not find Downloads directory");
            "Could not find Downloads directory".to_string()
        })?;

    let dest_path = download_dir.join(&filename);
    log::info!("[download_update] Saving to: {}", dest_path.display());
    fs::write(&dest_path, &bytes)
        .map_err(|e| {
            log::error!("[download_update] Failed to save file to {}: {}", dest_path.display(), e);
            format!("Failed to save file: {}", e)
        })?;

    let dest_str = dest_path.to_string_lossy().to_string();
    log::info!("[download_update] Saved successfully: {} ({} bytes)", dest_str, bytes.len());

    // On macOS, mount the DMG
    #[cfg(target_os = "macos")]
    {
        if filename.ends_with(".dmg") {
            log::info!("[download_update] Mounting DMG: {}", dest_str);
            Command::new("open")
                .arg(&dest_path)
                .spawn()
                .map_err(|e| {
                    log::error!("[download_update] Failed to open DMG: {}", e);
                    format!("Failed to open DMG: {}", e)
                })?;
        }
    }

    Ok(dest_str)
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
    let binary = get_cli_binary();
    match new_command(&binary)
        .arg("--version")
        .env("PATH", get_extended_path())
        .output()
    {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if binary != "bd" {
                format!("{} ({})", version, binary)
            } else {
                version
            }
        }
        _ => format!("{} not found", binary),
    }
}

#[derive(Debug, Serialize)]
struct CompatibilityInfo {
    version: String,
    /// "bd", "br", or "unknown"
    #[serde(rename = "clientType")]
    client_type: String,
    #[serde(rename = "versionTuple")]
    version_tuple: Option<Vec<u32>>,
    #[serde(rename = "supportsDaemonFlag")]
    supports_daemon_flag: bool,
    #[serde(rename = "usesJsonlFiles")]
    uses_jsonl_files: bool,
    #[serde(rename = "usesDoltBackend")]
    uses_dolt_backend: bool,
    #[serde(rename = "supportsListAllFlag")]
    supports_list_all_flag: bool,
    warnings: Vec<String>,
}

#[tauri::command]
async fn check_bd_compatibility() -> CompatibilityInfo {
    let version_string = get_bd_version().await;
    let info = get_cli_client_info();

    let mut warnings = Vec::new();

    let (client, tuple) = match info {
        Some((client, major, minor, patch)) => (client, Some((major, minor, patch))),
        None => {
            warnings.push(format!("Could not detect CLI client from: {}", version_string));
            (CliClient::Unknown, None)
        }
    };

    let client_type_str = match client {
        CliClient::Bd => "bd",
        CliClient::Br => "br",
        CliClient::Unknown => "unknown",
    };

    if client == CliClient::Br {
        warnings.push("br (beads_rust) detected: frozen on classic SQLite+JSONL architecture, no daemon support".to_string());
    }

    if let Some((major, minor, _)) = tuple {
        if client == CliClient::Bd && major == 0 && minor >= 50 {
            warnings.push("bd >= 0.50.0 detected: daemon and JSONL systems have been removed".to_string());
        }
    }

    CompatibilityInfo {
        version: version_string,
        client_type: client_type_str.to_string(),
        version_tuple: tuple.map(|(a, b, c)| vec![a, b, c]),
        supports_daemon_flag: supports_daemon_flag(),
        uses_jsonl_files: uses_jsonl_files(),
        uses_dolt_backend: uses_dolt_backend(),
        supports_list_all_flag: supports_list_all_flag(),
        warnings,
    }
}

// ============================================================================
// CLI Binary Configuration Commands
// ============================================================================

#[tauri::command]
async fn get_cli_binary_path() -> String {
    get_cli_binary()
}

#[tauri::command]
async fn set_cli_binary_path(path: String) -> Result<String, String> {
    let binary = if path.trim().is_empty() { "bd".to_string() } else { path.trim().to_string() };

    // Validate the binary first
    let version = validate_cli_binary_internal(&binary)?;

    // Update global state and reset version cache (new binary may be different version)
    *CLI_BINARY.lock().unwrap() = binary.clone();
    reset_bd_version_cache();

    // Persist to config file
    let mut config = load_config();
    config.cli_binary = binary.clone();
    save_config(&config)?;

    log_info!("[config] CLI binary set to: {} ({})", binary, version);
    Ok(version)
}

#[tauri::command]
async fn validate_cli_binary(path: String) -> Result<String, String> {
    let binary = if path.trim().is_empty() { "bd".to_string() } else { path.trim().to_string() };
    validate_cli_binary_internal(&binary)
}

fn validate_cli_binary_internal(binary: &str) -> Result<String, String> {
    // Security: reject shell metacharacters — Command::new() doesn't use a shell,
    // but defense-in-depth prevents any future misuse
    let forbidden = [';', '|', '&', '$', '`', '>', '<', '(', ')', '{', '}', '!', '\n', '\r'];
    if binary.chars().any(|c| forbidden.contains(&c)) {
        return Err("Invalid binary path: contains shell metacharacters".to_string());
    }
    if binary.contains("..") {
        return Err("Invalid binary path: directory traversal not allowed".to_string());
    }

    match new_command(binary)
        .arg("--version")
        .env("PATH", get_extended_path())
        .output()
    {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if version.is_empty() {
                Err(format!("'{}' returned empty version output", binary))
            } else {
                Ok(version)
            }
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            Err(format!("'{}' failed: {}", binary, if stderr.is_empty() { "unknown error".to_string() } else { stderr }))
        }
        Err(e) => {
            Err(format!("'{}' not found or not executable: {}", binary, e))
        }
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

        // Check if this folder corresponds to an existing issue (folders use short IDs)
        let is_owned = existing_ids.iter().any(|id| issue_short_id(id) == folder_name);
        if !is_owned {
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

/// Sanitize a filename for safe storage and br JSONL compatibility.
/// Converts to kebab-case, strips diacritics, removes unsafe chars.
/// Example: "Screenshot 2026-02-24 à 10.30.png" → "screenshot-2026-02-24-a-10-30.png"
fn sanitize_filename(filename: &str) -> String {
    // Split into stem and extension
    let (stem, ext) = match filename.rfind('.') {
        Some(pos) => (&filename[..pos], &filename[pos..]),
        None => (filename, ""),
    };

    // Strip diacritics by replacing common accented chars, then lowercase + kebab-case
    let mut sanitized = String::with_capacity(stem.len());
    for c in stem.chars() {
        let replacement = match c {
            'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' | 'À' | 'Á' | 'Â' | 'Ã' | 'Ä' | 'Å' => "a",
            'è' | 'é' | 'ê' | 'ë' | 'È' | 'É' | 'Ê' | 'Ë' => "e",
            'ì' | 'í' | 'î' | 'ï' | 'Ì' | 'Í' | 'Î' | 'Ï' => "i",
            'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'Ò' | 'Ó' | 'Ô' | 'Õ' | 'Ö' => "o",
            'ù' | 'ú' | 'û' | 'ü' | 'Ù' | 'Ú' | 'Û' | 'Ü' => "u",
            'ñ' | 'Ñ' => "n",
            'ç' | 'Ç' => "c",
            'ß' => "ss",
            'æ' | 'Æ' => "ae",
            'œ' | 'Œ' => "oe",
            'ý' | 'ÿ' | 'Ý' => "y",
            'A'..='Z' => { sanitized.push((c as u8 + 32) as char); continue; },
            'a'..='z' | '0'..='9' => { sanitized.push(c); continue; },
            '-' => { sanitized.push('-'); continue; },
            ' ' | '_' | '.' => { sanitized.push('-'); continue; },
            _ => "-",
        };
        sanitized.push_str(replacement);
    }

    // Collapse multiple consecutive dashes and trim
    let mut result = String::with_capacity(sanitized.len());
    let mut prev_dash = false;
    for c in sanitized.chars() {
        if c == '-' {
            if !prev_dash {
                result.push('-');
            }
            prev_dash = true;
        } else {
            result.push(c);
            prev_dash = false;
        }
    }
    let result = result.trim_matches('-');

    let ext_lower = ext.to_lowercase();
    if result.is_empty() {
        format!("file{}", ext_lower)
    } else {
        format!("{}{}", result, ext_lower)
    }
}

// ============================================================================
// Attachment helpers
// ============================================================================

/// Image file extensions supported for attachment preview
const IMAGE_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "gif", "webp", "bmp", "svg", "ico", "tiff", "tif"];

/// Markdown file extensions supported for attachment preview
const MARKDOWN_EXTENSIONS: &[&str] = &["md", "markdown"];

/// Extract the short ID from a full issue ID by stripping the project prefix.
/// e.g. "beads-manager-2qk" → "2qk", "kybio-1pxe" → "1pxe",
///      "kybio-front-nuxt-4-466d" → "466d", "beads-manager-02e.1" → "02e.1"
/// Falls back to the full ID if no prefix separator is found.
fn issue_short_id(full_id: &str) -> &str {
    // The short ID is after the last '-' that isn't followed by another segment
    // containing only digits (which would be part of the prefix like "nuxt-4").
    // Strategy: find the last '-' where everything after it matches [a-z0-9.]+ (the short ID).
    // But "kybio-front-nuxt-4-466d": after last '-' is "466d" ✓
    // "kybio-front-nuxt-4": after last '-' is "4" which could be a short ID or prefix part.
    // Since we only call this with real issue IDs (not project names), the last segment is always the short ID.
    match full_id.rfind('-') {
        Some(pos) => &full_id[pos + 1..],
        None => full_id,
    }
}

/// Resolve the attachment directory for an issue.
/// Always uses short ID: .beads/attachments/{short_id}/
fn resolve_attachment_dir(attachments_dir: &std::path::Path, issue_id: &str) -> PathBuf {
    attachments_dir.join(issue_short_id(issue_id))
}

/// Classify a filename as "image", "markdown", or "other"
fn classify_attachment(filename: &str) -> &'static str {
    let lower = filename.to_lowercase();
    if IMAGE_EXTENSIONS.iter().any(|ext| lower.ends_with(&format!(".{}", ext))) {
        "image"
    } else if MARKDOWN_EXTENSIONS.iter().any(|ext| lower.ends_with(&format!(".{}", ext))) {
        "markdown"
    } else {
        "other"
    }
}

/// Resolve a duplicate filename: image.png → image-1.png → image-2.png
fn resolve_duplicate_filename(dir: &std::path::Path, name: &str) -> String {
    if !dir.join(name).exists() {
        return name.to_string();
    }
    let (stem, ext) = match name.rfind('.') {
        Some(pos) => (&name[..pos], &name[pos..]),
        None => (name, ""),
    };
    for i in 1..1000 {
        let candidate = format!("{}-{}{}", stem, i, ext);
        if !dir.join(&candidate).exists() {
            return candidate;
        }
    }
    // Fallback with timestamp
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!("{}-{}{}", stem, ts, ext)
}

// ============================================================================
// Attachment Refs Migration v3 — filesystem-only
// ============================================================================

/// Check if a ref is a "real" external reference (Redmine, GitHub, or other URL/ID).
/// Returns false for att: refs, local file paths, cleared: sentinels.
fn is_real_external_ref(r: &str) -> bool {
    let trimmed = r.trim();
    if trimmed.is_empty() { return false; }
    if trimmed.starts_with("cleared:") { return false; }
    if trimmed.starts_with("att:") { return false; }
    // Local file paths (absolute or relative .beads/)
    if trimmed.starts_with('/') { return false; }
    if trimmed.starts_with(".beads/") { return false; }
    // Anything with path separators inside .beads or attachments is local
    if trimmed.contains("/attachments/") || trimmed.contains("/.beads/") { return false; }
    true
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct RefsMigrationStatus {
    needs_migration: bool,
    ref_count: u32,
    just_migrated: bool,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct MigrateRefsResult {
    success: bool,
    refs_updated: u32,
}

/// Check if a project needs attachment refs migration v3.
/// v3 strips all attachment refs (att:, local paths) from external_ref,
/// keeping only real external refs (Redmine, GitHub, URLs).
/// Returns quickly if the .migrated-attachments marker file exists.
#[tauri::command]
async fn check_refs_migration(cwd: Option<String>) -> Result<RefsMigrationStatus, String> {
    let working_dir = cwd
        .or_else(|| env::var("BEADS_PATH").ok())
        .unwrap_or_else(|| {
            env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| ".".to_string())
        });

    let beads_dir = PathBuf::from(&working_dir).join(".beads");
    if !beads_dir.exists() {
        return Ok(RefsMigrationStatus { needs_migration: false, ref_count: 0, just_migrated: false });
    }

    // Already migrated to v3?
    if beads_dir.join(".migrated-attachments").exists() {
        // Check if auto-migration just ran (notify signal)
        let notify_path = beads_dir.join(".migrated-attachments-notify");
        let just_migrated = notify_path.exists();
        if just_migrated {
            let _ = std::fs::remove_file(&notify_path);
        }
        return Ok(RefsMigrationStatus { needs_migration: false, ref_count: 0, just_migrated });
    }

    let jsonl_path = beads_dir.join("issues.jsonl");
    if !jsonl_path.exists() {
        let _ = std::fs::write(beads_dir.join(".migrated-attachments"), "");
        return Ok(RefsMigrationStatus { needs_migration: false, ref_count: 0, just_migrated: false });
    }

    // Scan JSONL for refs that need cleanup (non-real external refs)
    let content = std::fs::read_to_string(&jsonl_path)
        .map_err(|e| format!("Failed to read issues.jsonl: {}", e))?;

    let mut ref_count: u32 = 0;

    for line in content.lines() {
        if line.trim().is_empty() { continue; }
        let v: serde_json::Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if let Some(ext_ref) = v.get("external_ref").and_then(|r| r.as_str()) {
            if ext_ref.is_empty() { continue; }
            let refs: Vec<&str> = ext_ref.split(|c: char| c == '\n' || c == '|').collect();
            for r in &refs {
                let trimmed = r.trim();
                if !trimmed.is_empty() && !is_real_external_ref(trimmed) {
                    ref_count += 1;
                    break; // One bad ref per issue is enough to flag it
                }
            }
        }
    }

    // Also check if attachment folders need renaming (full-id → short-id)
    let mut folder_work_count: u32 = 0;
    let attachments_dir = beads_dir.join("attachments");
    if attachments_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&attachments_dir) {
            for entry in entries.flatten() {
                if !entry.path().is_dir() { continue; }
                let name = entry.file_name().to_string_lossy().to_string();
                if issue_short_id(&name) != name {
                    folder_work_count += 1;
                }
            }
        }
    }

    let total = ref_count + folder_work_count;
    if total == 0 {
        let _ = std::fs::write(beads_dir.join(".migrated-attachments"), "");
        return Ok(RefsMigrationStatus { needs_migration: false, ref_count: 0, just_migrated: false });
    }

    log_info!("[refs_migration_v3] Project needs migration: {} ref(s) to clean, {} folder(s) to update", ref_count, folder_work_count);
    Ok(RefsMigrationStatus { needs_migration: true, ref_count: total, just_migrated: false })
}

/// Perform the attachment refs migration v3 (filesystem-only).
/// Delegates to ensure_refs_migrated_v3 which handles backup, cleanup, dedup, and marker.
/// The br sync is NOT called here — it will happen naturally after via sync_bd_database.
#[tauri::command]
async fn migrate_attachment_refs(cwd: Option<String>) -> Result<MigrateRefsResult, String> {
    let working_dir = cwd
        .or_else(|| env::var("BEADS_PATH").ok())
        .unwrap_or_else(|| {
            env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| ".".to_string())
        });

    let beads_dir = PathBuf::from(&working_dir).join(".beads");
    ensure_refs_migrated_v3(&beads_dir, &working_dir);
    Ok(MigrateRefsResult { success: true, refs_updated: 0 })
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
    let source_lower = source_path.to_lowercase();
    let is_allowed = IMAGE_EXTENSIONS.iter().chain(MARKDOWN_EXTENSIONS.iter())
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

    let abs_project_path = abs_project_path
        .canonicalize()
        .map_err(|e| format!("Failed to resolve project path: {}", e))?;

    // Build destination directory: {project}/.beads/attachments/{short_id}/
    let attachments_dir = abs_project_path.join(".beads").join("attachments");
    let dest_dir = resolve_attachment_dir(&attachments_dir, &issue_id);

    // Create directory if needed
    fs::create_dir_all(&dest_dir)
        .map_err(|e| format!("Failed to create attachments directory: {}", e))?;

    // Sanitize the original filename and handle duplicates
    let raw_filename = source
        .file_name()
        .ok_or_else(|| "Invalid source filename".to_string())?
        .to_string_lossy()
        .to_string();
    let sanitized = sanitize_filename(&raw_filename);
    let dest_filename = resolve_duplicate_filename(&dest_dir, &sanitized);
    let dest_path = dest_dir.join(&dest_filename);

    // Copy the file
    fs::copy(&source, &dest_path).map_err(|e| format!("Failed to copy file: {}", e))?;

    log::info!("[copy_file_to_attachments] Copied to: {}", dest_path.display());

    // Return just the filename (frontend doesn't need to store it in external_ref)
    Ok(dest_filename)
}

// ============================================================================
// Filesystem-based Attachment Commands
// ============================================================================

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentFile {
    pub filename: String,
    pub file_type: String,   // "image" or "markdown"
    pub path: String,        // absolute path
    pub modified: u64,       // mtime in epoch seconds (for sorting)
}

/// List all attachments for an issue by reading the filesystem directly.
/// Returns images and markdown files sorted by modification time (newest first).
#[tauri::command]
async fn list_attachments(project_path: String, issue_id: String) -> Result<Vec<AttachmentFile>, String> {
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

    let attachments_dir = abs_project_path.join(".beads").join("attachments");
    let issue_dir = resolve_attachment_dir(&attachments_dir, &issue_id);

    if !issue_dir.exists() || !issue_dir.is_dir() {
        return Ok(vec![]);
    }

    let mut files: Vec<AttachmentFile> = Vec::new();

    let entries = fs::read_dir(&issue_dir)
        .map_err(|e| format!("Failed to read attachment directory: {}", e))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() { continue; }

        let name = entry.file_name().to_string_lossy().to_string();
        // Skip legacy index.json files
        if name == "index.json" { continue; }

        let file_type = classify_attachment(&name);
        // Only return images and markdown
        if file_type == "other" { continue; }

        let modified = entry.metadata()
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);

        files.push(AttachmentFile {
            filename: name,
            file_type: file_type.to_string(),
            path: path.to_string_lossy().to_string(),
            modified,
        });
    }

    // Sort by mtime descending (newest first)
    files.sort_by(|a, b| b.modified.cmp(&a.modified));

    Ok(files)
}

/// Delete an attachment file by filename within an issue's attachment directory.
#[tauri::command]
async fn delete_attachment(project_path: String, issue_id: String, filename: String) -> Result<(), String> {
    log::info!("[delete_attachment] project: {}, issue: {}, file: {}", project_path, issue_id, filename);

    // Security: reject path traversal
    if filename.contains('/') || filename.contains('\\') || filename.contains("..") {
        return Err("Invalid filename".to_string());
    }

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

    let attachments_dir = abs_project_path.join(".beads").join("attachments");
    let issue_dir = resolve_attachment_dir(&attachments_dir, &issue_id);
    let file_path = issue_dir.join(&filename);

    if !file_path.exists() {
        log::info!("[delete_attachment] File does not exist: {:?}", file_path);
        return Ok(());
    }

    // Security: verify file is inside .beads/attachments/
    let canonical = file_path.canonicalize()
        .map_err(|e| format!("Failed to resolve path: {}", e))?;
    let canonical_str = canonical.to_string_lossy();
    if !canonical_str.contains("/.beads/attachments/") {
        return Err("Can only delete files inside .beads/attachments/".to_string());
    }

    fs::remove_file(&file_path)
        .map_err(|e| format!("Failed to delete file: {}", e))?;

    log::info!("[delete_attachment] Deleted: {:?}", file_path);

    // Cleanup empty folder (issue_dir already resolved above via resolve_attachment_dir)
    if issue_dir.exists() {
        if let Ok(entries) = fs::read_dir(&issue_dir) {
            // Count non-index.json entries
            let count = entries.flatten()
                .filter(|e| e.file_name().to_string_lossy() != "index.json")
                .count();
            if count == 0 {
                // Remove index.json if present, then the directory
                let _ = fs::remove_file(issue_dir.join("index.json"));
                let _ = fs::remove_dir(&issue_dir);
                log::info!("[delete_attachment] Cleaned up empty folder: {:?}", issue_dir);
            }
        }
    }

    Ok(())
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
// File Watcher Commands
// ============================================================================

#[tauri::command]
fn start_watching(
    path: String,
    app: tauri::AppHandle,
    state: tauri::State<'_, Mutex<WatcherState>>,
) -> Result<(), String> {
    let mut watcher_state = state.lock().map_err(|e| format!("Lock error: {}", e))?;

    // Stop existing watcher if any
    if watcher_state.debouncer.is_some() {
        log::info!("[watcher] Stopping previous watcher for: {:?}", watcher_state.watched_path);
        watcher_state.debouncer = None;
        watcher_state.watched_path = None;
    }

    let beads_dir = PathBuf::from(&path).join(".beads");
    if !beads_dir.exists() {
        return Err(format!(".beads directory not found at: {}", beads_dir.display()));
    }

    let project_path = path.clone();
    let app_handle = app.clone();

    let mut debouncer = new_debouncer(
        Duration::from_millis(1000),
        move |res: Result<Vec<notify_debouncer_mini::DebouncedEvent>, notify::Error>| {
            match res {
                Ok(events) => {
                    // Filter: only emit if we have actual data-change events
                    let has_data_events = events.iter().any(|e| {
                        matches!(e.kind, DebouncedEventKind::Any | DebouncedEventKind::AnyContinuous)
                    });
                    if has_data_events {
                        log::info!("[watcher] Change detected in .beads/ ({} events)", events.len());
                        let _ = app_handle.emit(
                            "beads-changed",
                            BeadsChangedPayload { path: project_path.clone() },
                        );
                    }
                }
                Err(e) => {
                    log::error!("[watcher] Error: {:?}", e);
                }
            }
        },
    ).map_err(|e| format!("Failed to create watcher: {}", e))?;

    // Watch .beads/ directory
    // Dolt backend: recursive (changes happen in .dolt/ subdirectories)
    // SQLite backend: non-recursive (all target files are at root level)
    let watch_mode = if project_uses_dolt(&beads_dir) {
        notify::RecursiveMode::Recursive
    } else {
        notify::RecursiveMode::NonRecursive
    };
    debouncer.watcher().watch(
        beads_dir.as_path(),
        watch_mode,
    ).map_err(|e| format!("Failed to watch .beads/: {}", e))?;

    log::info!("[watcher] Started watching: {}", beads_dir.display());
    watcher_state.debouncer = Some(debouncer);
    watcher_state.watched_path = Some(path);

    Ok(())
}

#[tauri::command]
fn stop_watching(
    state: tauri::State<'_, Mutex<WatcherState>>,
) -> Result<(), String> {
    let mut watcher_state = state.lock().map_err(|e| format!("Lock error: {}", e))?;

    if watcher_state.debouncer.is_some() {
        log::info!("[watcher] Stopped watching: {:?}", watcher_state.watched_path);
        watcher_state.debouncer = None;
        watcher_state.watched_path = None;
    }

    Ok(())
}

#[derive(Debug, Serialize)]
struct WatcherStatusInfo {
    active: bool,
    #[serde(rename = "watchedPath")]
    watched_path: Option<String>,
}

#[tauri::command]
fn get_watcher_status(
    state: tauri::State<'_, Mutex<WatcherState>>,
) -> Result<WatcherStatusInfo, String> {
    let watcher_state = state.lock().map_err(|e| format!("Lock error: {}", e))?;

    Ok(WatcherStatusInfo {
        active: watcher_state.debouncer.is_some(),
        watched_path: watcher_state.watched_path.clone(),
    })
}

// ============================================================================
// External Data Source Commands
// ============================================================================

#[tauri::command]
async fn fetch_external_data(url: String) -> Result<String, String> {
    log_info!("[probe] GET {}", url);
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("HTTP client error: {}", e))?;

    let response = client
        .get(&url)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if !response.status().is_success() {
        let err = format!("HTTP {}: {}", response.status().as_u16(), response.status().canonical_reason().unwrap_or("Unknown"));
        log_error!("[probe] GET failed: {}", err);
        return Err(err);
    }

    response.text().await.map_err(|e| format!("Failed to read response: {}", e))
}

#[tauri::command]
async fn check_external_health(url: String) -> Result<bool, String> {
    let health_url = format!("{}/health", url.trim_end_matches('/'));
    log_info!("[probe] Health check: {}", health_url);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| format!("HTTP client error: {}", e))?;

    match client.get(&health_url).send().await {
        Ok(response) => Ok(response.status().is_success()),
        Err(_) => Ok(false),
    }
}

#[tauri::command]
async fn post_external_data(url: String, body: String) -> Result<String, String> {
    log_info!("[probe] POST {}", url);
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("HTTP client error: {}", e))?;

    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .body(body)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(format!("HTTP {}: {}", status.as_u16(), text));
    }

    response.text().await.map_err(|e| format!("Failed to read response: {}", e))
}

#[tauri::command]
async fn delete_external_data(url: String) -> Result<String, String> {
    log_info!("[probe] DELETE {}", url);
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("HTTP client error: {}", e))?;

    let response = client
        .delete(&url)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(format!("HTTP {}: {}", status.as_u16(), text));
    }

    response.text().await.map_err(|e| format!("Failed to read response: {}", e))
}

#[tauri::command]
async fn patch_external_data(url: String, body: String) -> Result<String, String> {
    log_info!("[probe] PATCH {}", url);
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("HTTP client error: {}", e))?;

    let response = client
        .patch(&url)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .body(body)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(format!("HTTP {}: {}", status.as_u16(), text));
    }

    response.text().await.map_err(|e| format!("Failed to read response: {}", e))
}

// ============================================================================
// Probe Launcher
// ============================================================================

#[tauri::command]
async fn launch_probe(port: u16) -> Result<String, String> {
    use std::process::Stdio;

    let health_url = format!("http://127.0.0.1:{}/health", port);

    // Check if probe is already reachable via HTTP health endpoint
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .map_err(|e| format!("HTTP client error: {}", e))?;

    if let Ok(resp) = client.get(&health_url).send().await {
        if resp.status().is_success() {
            log_info!("[probe] Already running on port {}", port);
            return Ok("already running".to_string());
        }
    }

    // Determine binary: BEADS_PROBE_BIN env var, fallback to "beads-probe"
    let bin = env::var("BEADS_PROBE_BIN").unwrap_or_else(|_| "beads-probe".to_string());
    log_info!("[probe] Launching: {} --port {}", bin, port);

    let child = Command::new(&bin)
        .arg("--port")
        .arg(port.to_string())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to spawn {}: {}", bin, e))?;

    // Store child handle so it lives as long as the app
    if let Ok(mut guard) = PROBE_CHILD.lock() {
        *guard = Some(child);
    }

    log_info!("[probe] Launched on port {}", port);
    Ok("launched".to_string())
}

// ============================================================================
// App Entry Point
// ============================================================================

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Load .env file (dev only — in prod there's no .env, env vars come from the system)
    let _ = dotenvy::dotenv();

    tauri::Builder::default()
        .manage(Mutex::new(WatcherState::default()))
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

            // Load config and set CLI binary (auto-detects br→bd if no config exists)
            let config = load_config();
            log::info!("[startup] CLI binary: {}", config.cli_binary);
            *CLI_BINARY.lock().unwrap() = config.cli_binary.clone();

            // Check if CLI binary is accessible
            // IMPORTANT: Run from /tmp to avoid bd auto-migrating projects in cwd
            let binary = get_cli_binary();
            match new_command(&binary)
                .arg("--version")
                .current_dir(std::env::temp_dir())
                .env("PATH", get_extended_path())
                .output()
            {
                Ok(output) if output.status.success() => {
                    let version = String::from_utf8_lossy(&output.stdout);
                    log::info!("[startup] {} found: {}", binary, version.trim());
                }
                Ok(output) => {
                    log::warn!("[startup] {} command failed: {}", binary, String::from_utf8_lossy(&output.stderr));
                }
                Err(e) => {
                    log::error!("[startup] {} not found or not executable: {}", binary, e);
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_backend_mode,
            set_backend_mode,
            tracker_init,
            tracker_detect,
            bd_sync,
            bd_repair_database,
            bd_migrate_to_dolt,
            bd_check_needs_migration,
            bd_cleanup_stale_locks,
            bd_check_changed,
            bd_reset_mtime,
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
            check_bd_compatibility,
            get_cli_binary_path,
            set_cli_binary_path,
            validate_cli_binary,
            bd_update,
            bd_close,
            bd_search,
            bd_label_add,
            bd_label_remove,
            bd_delete,
            bd_comments_add,
            bd_dep_add,
            bd_dep_remove,
            bd_dep_add_relation,
            bd_dep_remove_relation,
            bd_available_relation_types,
            fs_exists,
            fs_list,
            check_for_updates,
            check_for_updates_demo,
            check_bd_cli_update,
            download_and_install_update,
            open_image_file,
            read_image_file,
            copy_file_to_attachments,
            list_attachments,
            delete_attachment,
            read_text_file,
            write_text_file,
            purge_orphan_attachments,
            check_refs_migration,
            migrate_attachment_refs,
            start_watching,
            stop_watching,
            get_watcher_status,
            fetch_external_data,
            check_external_health,
            post_external_data,
            delete_external_data,
            patch_external_data,
            launch_probe,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

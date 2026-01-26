use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::process::Command;

// File watcher removed - replaced by frontend polling for lower CPU usage

// ============================================================================
// Types
// ============================================================================

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
    pub blocked_by: Option<Vec<String>>,
    pub blocks: Option<Vec<String>>,
    pub comments: Option<Vec<BdRawComment>>,
    pub external_ref: Option<String>,
    pub estimate: Option<i32>,
    pub design: Option<String>,
    pub acceptance_criteria: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BdRawComment {
    pub id: serde_json::Value,
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
        if caps.len() == 1 && caps.chars().next().unwrap().is_ascii_digit() {
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
    }
}

fn escape_quotes(s: &str) -> String {
    s.replace('"', "\\\"")
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

fn execute_bd(command: &str, args: &[String], cwd: Option<&str>) -> Result<String, String> {
    let working_dir = cwd
        .map(String::from)
        .or_else(|| env::var("BEADS_PATH").ok())
        .unwrap_or_else(|| env::current_dir().unwrap().to_string_lossy().to_string());

    let mut full_args: Vec<&str> = vec![command];
    for arg in args {
        full_args.push(arg);
    }
    full_args.push("--no-daemon");
    full_args.push("--json");

    let output = Command::new("bd")
        .args(&full_args)
        .current_dir(&working_dir)
        .env("PATH", get_extended_path())
        .env("BEADS_PATH", &working_dir)
        .output()
        .map_err(|e| format!("Failed to execute bd: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stderr.is_empty() {
            return Err(stderr.to_string());
        }
        return Err(format!("bd command failed with status: {}", output.status));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Sync the beads database before read operations to ensure data is up-to-date
fn sync_bd_database(cwd: Option<&str>) {
    let working_dir = cwd
        .map(String::from)
        .or_else(|| env::var("BEADS_PATH").ok())
        .unwrap_or_else(|| env::current_dir().unwrap().to_string_lossy().to_string());

    // Run bd sync --import-only silently (ignore errors, best effort)
    let _ = Command::new("bd")
        .args(["sync", "--import-only", "--no-daemon"])
        .current_dir(&working_dir)
        .env("PATH", get_extended_path())
        .env("BEADS_PATH", &working_dir)
        .output();
}

// ============================================================================
// Tauri Commands
// ============================================================================

#[tauri::command]
async fn bd_list(options: ListOptions) -> Result<Vec<Issue>, String> {
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

    let output = execute_bd("list", &args, options.cwd.as_deref())?;

    let raw_issues: Vec<BdRawIssue> = serde_json::from_str(&output)
        .map_err(|e| format!("Failed to parse issues: {}", e))?;

    Ok(raw_issues.into_iter().map(transform_issue).collect())
}

#[tauri::command]
async fn bd_count(options: CwdOptions) -> Result<CountResult, String> {
    // Sync database before reading to ensure data is up-to-date
    sync_bd_database(options.cwd.as_deref());

    // Fetch both open and closed issues to match fetchIssues behavior
    let open_output = execute_bd("list", &[], options.cwd.as_deref())?;
    let closed_output = execute_bd("list", &["--status=closed".to_string()], options.cwd.as_deref())?;

    let open_issues: Vec<BdRawIssue> = serde_json::from_str(&open_output)
        .map_err(|e| format!("Failed to parse open issues: {}", e))?;
    let closed_issues: Vec<BdRawIssue> = serde_json::from_str(&closed_output)
        .map_err(|e| format!("Failed to parse closed issues: {}", e))?;

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
    // Sync database before reading to ensure data is up-to-date
    sync_bd_database(options.cwd.as_deref());

    let output = execute_bd("ready", &[], options.cwd.as_deref())?;

    let raw_issues: Vec<BdRawIssue> = serde_json::from_str(&output)
        .map_err(|e| format!("Failed to parse issues: {}", e))?;

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
    // Sync database before reading to ensure data is up-to-date
    sync_bd_database(options.cwd.as_deref());

    let output = execute_bd("show", &[id], options.cwd.as_deref())?;

    // bd show can return either a single object or an array
    let result: serde_json::Value = serde_json::from_str(&output)
        .map_err(|e| format!("Failed to parse issue: {}", e))?;

    let raw_issue: Option<BdRawIssue> = if result.is_array() {
        result.as_array()
            .and_then(|arr| arr.first())
            .map(|v| serde_json::from_value(v.clone()).ok())
            .flatten()
    } else {
        serde_json::from_value(result).ok()
    };

    Ok(raw_issue.map(transform_issue))
}

#[tauri::command]
async fn bd_create(payload: CreatePayload) -> Result<Option<Issue>, String> {
    let mut args: Vec<String> = vec![format!("\"{}\"", payload.title)];

    if let Some(ref desc) = payload.description {
        args.push(format!("--description=\"{}\"", escape_quotes(desc)));
    }
    if let Some(ref t) = payload.issue_type {
        args.push(format!("--type={}", t));
    }
    if let Some(ref p) = payload.priority {
        args.push(format!("--priority={}", priority_to_number(p)));
    }
    if let Some(ref a) = payload.assignee {
        args.push(format!("--assignee={}", a));
    }
    if let Some(ref labels) = payload.labels {
        if !labels.is_empty() {
            args.push(format!("--labels={}", labels.join(",")));
        }
    }
    if let Some(ref ext) = payload.external_ref {
        args.push(format!("--external-ref=\"{}\"", escape_quotes(ext)));
    }
    if let Some(est) = payload.estimate_minutes {
        args.push(format!("--estimate={}", est));
    }
    if let Some(ref design) = payload.design_notes {
        args.push(format!("--design=\"{}\"", escape_quotes(design)));
    }
    if let Some(ref acc) = payload.acceptance_criteria {
        args.push(format!("--acceptance=\"{}\"", escape_quotes(acc)));
    }
    if let Some(ref notes) = payload.working_notes {
        args.push(format!("--notes=\"{}\"", escape_quotes(notes)));
    }

    let output = execute_bd("create", &args, payload.cwd.as_deref())?;

    let raw_issue: BdRawIssue = serde_json::from_str(&output)
        .map_err(|e| format!("Failed to parse created issue: {}", e))?;

    Ok(Some(transform_issue(raw_issue)))
}

#[tauri::command]
async fn bd_update(id: String, updates: UpdatePayload) -> Result<Option<Issue>, String> {
    let mut args: Vec<String> = vec![id];

    if let Some(ref title) = updates.title {
        args.push(format!("--title=\"{}\"", escape_quotes(title)));
    }
    if let Some(ref desc) = updates.description {
        args.push(format!("--description=\"{}\"", escape_quotes(desc)));
    }
    if let Some(ref t) = updates.issue_type {
        args.push(format!("--type={}", t));
    }
    if let Some(ref s) = updates.status {
        args.push(format!("--status={}", s));
    }
    if let Some(ref p) = updates.priority {
        args.push(format!("--priority={}", priority_to_number(p)));
    }
    if let Some(ref a) = updates.assignee {
        args.push(format!("--assignee={}", a));
    }
    if let Some(ref labels) = updates.labels {
        if !labels.is_empty() {
            args.push(format!("--labels={}", labels.join(",")));
        }
    }
    if let Some(ref ext) = updates.external_ref {
        args.push(format!("--external-ref=\"{}\"", escape_quotes(ext)));
    }
    if let Some(est) = updates.estimate_minutes {
        args.push(format!("--estimate={}", est));
    }
    if let Some(ref design) = updates.design_notes {
        args.push(format!("--design=\"{}\"", escape_quotes(design)));
    }
    if let Some(ref acc) = updates.acceptance_criteria {
        args.push(format!("--acceptance=\"{}\"", escape_quotes(acc)));
    }
    if let Some(ref notes) = updates.working_notes {
        args.push(format!("--notes=\"{}\"", escape_quotes(notes)));
    }

    let output = execute_bd("update", &args, updates.cwd.as_deref())?;

    // bd update can return either a single object or an array
    let result: serde_json::Value = serde_json::from_str(&output)
        .map_err(|e| format!("Failed to parse updated issue: {}", e))?;

    let raw_issue: Option<BdRawIssue> = if result.is_array() {
        result.as_array()
            .and_then(|arr| arr.first())
            .map(|v| serde_json::from_value(v.clone()).ok())
            .flatten()
    } else {
        serde_json::from_value(result).ok()
    };

    Ok(raw_issue.map(transform_issue))
}

#[tauri::command]
async fn bd_close(id: String, options: CwdOptions) -> Result<serde_json::Value, String> {
    let output = execute_bd("close", &[id], options.cwd.as_deref())?;

    serde_json::from_str(&output)
        .map_err(|e| format!("Failed to parse close result: {}", e))
}

#[tauri::command]
async fn bd_delete(id: String, options: CwdOptions) -> Result<serde_json::Value, String> {
    execute_bd("delete", &[id.clone(), "--force".to_string()], options.cwd.as_deref())?;

    Ok(serde_json::json!({ "success": true, "id": id }))
}

#[tauri::command]
async fn bd_comments_add(id: String, content: String, options: CwdOptions) -> Result<serde_json::Value, String> {
    let escaped_content = escape_quotes(&content);
    let args = vec![id, format!("\"{}\"", escaped_content)];

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
// App Entry Point
// ============================================================================

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            bd_list,
            bd_count,
            bd_ready,
            bd_status,
            bd_show,
            bd_create,
            bd_update,
            bd_close,
            bd_delete,
            bd_comments_add,
            fs_exists,
            fs_list,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

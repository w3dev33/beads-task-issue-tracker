use std::path::Path;

/// Configuration for a tracker project.
#[derive(Debug, Clone)]
pub struct ProjectConfig {
    /// Directory name for the tracker data (default: `.tracker`)
    pub folder_name: String,
    /// Prefix for issue IDs (default: `tracker`)
    pub issue_prefix: String,
    /// Default actor name for authoring issues/comments
    pub actor: String,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            folder_name: ".tracker".to_string(),
            issue_prefix: "tracker".to_string(),
            actor: git_user_name().unwrap_or_default(),
        }
    }
}

impl ProjectConfig {
    /// Load config from `.tracker/config.yaml` if it exists, otherwise use defaults.
    /// Full YAML parsing is deferred to a later task — for now, always returns defaults.
    pub fn load(_project_path: &Path) -> Self {
        // TODO: parse .tracker/config.yaml when present
        Self::default()
    }
}

/// Attempt to read the git user name from global config.
fn git_user_name() -> Option<String> {
    std::process::Command::new("git")
        .args(["config", "user.name"])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                String::from_utf8(o.stdout).ok().map(|s| s.trim().to_string())
            } else {
                None
            }
        })
}

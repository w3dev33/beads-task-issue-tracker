use std::path::PathBuf;
use std::process;

use clap::{Parser, Subcommand};

use app_lib::tracker::{
    CreateIssueParams, Engine, ProjectConfig, SearchResult, TrackerIssue, UpdateIssueParams,
};

#[derive(Parser)]
#[command(name = "beads-tracker", about = "Built-in issue tracker for Beads projects")]
struct Cli {
    /// Project directory (default: current directory)
    #[arg(short = 'C', long = "project", global = true)]
    project: Option<PathBuf>,

    /// Output as JSON
    #[arg(long, global = true)]
    json: bool,

    /// Actor name for authoring (default: git user.name)
    #[arg(long, global = true)]
    actor: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new tracker in the project
    Init,

    /// List issues
    List {
        /// Filter by status (open, closed, in_progress, all)
        #[arg(short, long)]
        status: Option<String>,

        /// Filter by issue type
        #[arg(short = 't', long = "type")]
        issue_type: Option<String>,

        /// Filter by priority
        #[arg(short, long)]
        priority: Option<String>,

        /// Filter by assignee
        #[arg(long)]
        assignee: Option<String>,

        /// Show all issues (shorthand for --status all)
        #[arg(short, long)]
        all: bool,

        /// Limit number of results
        #[arg(long)]
        limit: Option<usize>,
    },

    /// Show a single issue with full details
    Show {
        /// Issue ID
        id: String,
    },

    /// Create a new issue
    Create {
        /// Issue title
        title: String,

        /// Issue description/body
        #[arg(short, long)]
        description: Option<String>,

        /// Issue type (task, bug, feature, epic)
        #[arg(short = 't', long = "type")]
        issue_type: Option<String>,

        /// Priority (p0, p1, p2, p3)
        #[arg(short, long)]
        priority: Option<String>,

        /// Assignee
        #[arg(long)]
        assignee: Option<String>,

        /// Labels (comma-separated)
        #[arg(short, long)]
        labels: Option<String>,

        /// Parent issue ID
        #[arg(long)]
        parent: Option<String>,

        /// Estimate in minutes
        #[arg(long)]
        estimate: Option<i32>,

        /// Design notes
        #[arg(long)]
        design: Option<String>,

        /// Acceptance criteria
        #[arg(long)]
        acceptance: Option<String>,

        /// Notes
        #[arg(long)]
        notes: Option<String>,

        /// External reference
        #[arg(long)]
        external_ref: Option<String>,

        /// Spec ID
        #[arg(long)]
        spec_id: Option<String>,
    },

    /// Update an existing issue
    Update {
        /// Issue ID
        id: String,

        /// New title
        #[arg(long)]
        title: Option<String>,

        /// New description/body
        #[arg(short, long)]
        description: Option<String>,

        /// New status
        #[arg(short, long)]
        status: Option<String>,

        /// New issue type
        #[arg(short = 't', long = "type")]
        issue_type: Option<String>,

        /// New priority
        #[arg(short, long)]
        priority: Option<String>,

        /// New assignee (use "" to clear)
        #[arg(long)]
        assignee: Option<String>,

        /// Replace all labels (comma-separated)
        #[arg(short = 'l', long = "set-labels")]
        set_labels: Option<String>,

        /// New parent issue ID (use "" to clear)
        #[arg(long)]
        parent: Option<String>,

        /// New estimate in minutes (use 0 to clear)
        #[arg(long)]
        estimate: Option<i32>,

        /// New design notes (use "" to clear)
        #[arg(long)]
        design: Option<String>,

        /// New acceptance criteria (use "" to clear)
        #[arg(long)]
        acceptance: Option<String>,

        /// New notes (use "" to clear)
        #[arg(long)]
        notes: Option<String>,

        /// New metadata JSON (use "" to clear)
        #[arg(long)]
        metadata: Option<String>,

        /// New spec ID (use "" to clear)
        #[arg(long)]
        spec_id: Option<String>,
    },

    /// Close an issue
    Close {
        /// Issue ID
        id: String,
    },

    /// Delete an issue
    Delete {
        /// Issue ID
        id: String,

        /// Hard delete (permanent removal)
        #[arg(long)]
        hard: bool,
    },

    /// Search issues by text
    Search {
        /// Search query
        query: String,

        /// Limit number of results
        #[arg(long, default_value = "50")]
        limit: usize,
    },

    /// List ready issues (open, not blocked)
    Ready,

    /// Manage comments
    #[command(subcommand)]
    Comments(CommentsCmd),

    /// Manage labels
    #[command(subcommand)]
    Label(LabelCmd),

    /// Manage dependencies
    #[command(subcommand)]
    Dep(DepCmd),
}

#[derive(Subcommand)]
enum CommentsCmd {
    /// Add a comment to an issue
    Add {
        /// Issue ID
        id: String,

        /// Comment body
        body: String,
    },

    /// Delete a comment
    Delete {
        /// Comment ID
        id: String,
    },
}

#[derive(Subcommand)]
enum LabelCmd {
    /// Add a label to an issue
    Add {
        /// Issue ID
        id: String,

        /// Label name
        label: String,
    },

    /// Remove a label from an issue
    Remove {
        /// Issue ID
        id: String,

        /// Label name
        label: String,
    },
}

#[derive(Subcommand)]
enum DepCmd {
    /// Add a dependency (from blocks to)
    Add {
        /// Issue ID
        id: String,

        /// Blocker issue ID
        blocker: String,

        /// Dependency type (default: blocks)
        #[arg(long = "type", default_value = "blocks")]
        dep_type: String,
    },

    /// Remove a dependency
    Remove {
        /// Issue ID
        id: String,

        /// Other issue ID
        other: String,
    },
}

fn main() {
    let cli = Cli::parse();

    let project_path = cli
        .project
        .unwrap_or_else(|| std::env::current_dir().expect("Failed to get current directory"));

    let result = run(cli.command, &project_path, cli.json, cli.actor.as_deref());
    if let Err(e) = result {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn run(
    command: Commands,
    project_path: &std::path::Path,
    json: bool,
    actor: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        Commands::Init => {
            let mut config = ProjectConfig::default();
            if let Some(a) = actor {
                config.actor = a.to_string();
            }
            let _engine = Engine::init(project_path, config)?;
            if json {
                println!(r#"{{"ok":true}}"#);
            } else {
                println!(
                    "Initialized tracker in {}/.tracker/",
                    project_path.display()
                );
            }
        }

        Commands::List {
            status,
            issue_type,
            priority,
            assignee,
            all,
            limit,
        } => {
            let engine = open_engine(project_path)?;
            let filter = if all {
                Some("all")
            } else {
                status.as_deref()
            };
            let mut issues = engine.list_issues(filter)?;

            // In-memory filters
            if let Some(ref t) = issue_type {
                issues.retain(|i| i.issue_type == *t);
            }
            if let Some(ref p) = priority {
                issues.retain(|i| i.priority == *p);
            }
            if let Some(ref a) = assignee {
                issues.retain(|i| i.assignee.as_deref() == Some(a.as_str()));
            }
            if let Some(n) = limit {
                issues.truncate(n);
            }

            if json {
                println!("{}", serde_json::to_string(&issues)?);
            } else {
                print_issue_list(&issues);
            }
        }

        Commands::Show { id } => {
            let engine = open_engine(project_path)?;
            let issue = engine.get_issue(&id)?;
            let children = engine.list_children(&id)?;

            if json {
                #[derive(serde::Serialize)]
                struct ShowResult {
                    #[serde(flatten)]
                    issue: TrackerIssue,
                    #[serde(skip_serializing_if = "Vec::is_empty")]
                    children: Vec<TrackerIssue>,
                }
                let result = ShowResult { issue, children };
                println!("{}", serde_json::to_string(&result)?);
            } else {
                print_issue_detail(&issue);
                if !children.is_empty() {
                    println!("\nChildren:");
                    print_issue_list(&children);
                }
            }
        }

        Commands::Create {
            title,
            description,
            issue_type,
            priority,
            assignee,
            labels,
            parent,
            estimate,
            design,
            acceptance,
            notes,
            external_ref,
            spec_id,
        } => {
            let engine = open_engine(project_path)?;
            let params = CreateIssueParams {
                title,
                body: description,
                issue_type,
                status: None,
                priority,
                assignee,
                author: actor.map(|s| s.to_string()),
                labels: labels.map(|l| l.split(',').map(|s| s.trim().to_string()).collect()),
                external_ref,
                estimate_minutes: estimate,
                design,
                acceptance_criteria: acceptance,
                notes,
                parent,
                metadata: None,
                spec_id,
            };
            let issue = engine.create_issue(params)?;

            if json {
                println!("{}", serde_json::to_string(&issue)?);
            } else {
                println!("Created {} — {}", issue.id, issue.title);
            }
        }

        Commands::Update {
            id,
            title,
            description,
            status,
            issue_type,
            priority,
            assignee,
            set_labels,
            parent,
            estimate,
            design,
            acceptance,
            notes,
            metadata,
            spec_id,
        } => {
            let engine = open_engine(project_path)?;
            let params = UpdateIssueParams {
                title,
                body: description,
                issue_type,
                status,
                priority,
                assignee: assignee.map(|a| if a.is_empty() { None } else { Some(a) }),
                labels: set_labels
                    .map(|l| l.split(',').map(|s| s.trim().to_string()).collect()),
                external_ref: None,
                estimate_minutes: estimate
                    .map(|e| if e == 0 { None } else { Some(e) }),
                design: design.map(|d| if d.is_empty() { None } else { Some(d) }),
                acceptance_criteria: acceptance
                    .map(|a| if a.is_empty() { None } else { Some(a) }),
                notes: notes.map(|n| if n.is_empty() { None } else { Some(n) }),
                parent: parent.map(|p| if p.is_empty() { None } else { Some(p) }),
                metadata: metadata.map(|m| if m.is_empty() { None } else { Some(m) }),
                spec_id: spec_id.map(|s| if s.is_empty() { None } else { Some(s) }),
            };
            let issue = engine.update_issue(&id, params)?;

            if json {
                println!("{}", serde_json::to_string(&issue)?);
            } else {
                println!("Updated {} — {}", issue.id, issue.title);
            }
        }

        Commands::Close { id } => {
            let engine = open_engine(project_path)?;
            let issue = engine.close_issue(&id)?;

            if json {
                println!("{}", serde_json::to_string(&issue)?);
            } else {
                println!("Closed {} — {}", issue.id, issue.title);
            }
        }

        Commands::Delete { id, hard } => {
            let engine = open_engine(project_path)?;
            engine.delete_issue(&id, hard)?;

            if json {
                println!(r#"{{"ok":true,"id":"{}"}}"#, id);
            } else {
                let mode = if hard { "Hard deleted" } else { "Deleted" };
                println!("{} {}", mode, id);
            }
        }

        Commands::Search { query, limit } => {
            let engine = open_engine(project_path)?;
            let results = engine.search(&query, Some(limit))?;

            if json {
                println!("{}", serde_json::to_string(&results)?);
            } else {
                print_search_results(&results);
            }
        }

        Commands::Ready => {
            let engine = open_engine(project_path)?;
            let issues = engine.list_ready_issues()?;

            if json {
                println!("{}", serde_json::to_string(&issues)?);
            } else {
                print_issue_list(&issues);
            }
        }

        Commands::Comments(cmd) => match cmd {
            CommentsCmd::Add { id, body } => {
                let engine = open_engine(project_path)?;
                let author = actor.unwrap_or("unknown");
                let comment = engine.add_comment(&id, author, &body)?;

                if json {
                    println!("{}", serde_json::to_string(&comment)?);
                } else {
                    println!("Added comment {} to {}", comment.id, id);
                }
            }
            CommentsCmd::Delete { id } => {
                let engine = open_engine(project_path)?;
                engine.delete_comment(&id)?;

                if json {
                    println!(r#"{{"ok":true,"id":"{}"}}"#, id);
                } else {
                    println!("Deleted comment {}", id);
                }
            }
        },

        Commands::Label(cmd) => match cmd {
            LabelCmd::Add { id, label } => {
                let engine = open_engine(project_path)?;
                engine.add_label(&id, &label)?;

                if json {
                    println!(r#"{{"ok":true}}"#);
                } else {
                    println!("Added label '{}' to {}", label, id);
                }
            }
            LabelCmd::Remove { id, label } => {
                let engine = open_engine(project_path)?;
                engine.remove_label(&id, &label)?;

                if json {
                    println!(r#"{{"ok":true}}"#);
                } else {
                    println!("Removed label '{}' from {}", label, id);
                }
            }
        },

        Commands::Dep(cmd) => match cmd {
            DepCmd::Add {
                id,
                blocker,
                dep_type,
            } => {
                let engine = open_engine(project_path)?;
                engine.add_dependency(&id, &blocker, &dep_type)?;

                if json {
                    println!(r#"{{"ok":true}}"#);
                } else {
                    println!("{} {} → {}", dep_type, blocker, id);
                }
            }
            DepCmd::Remove { id, other } => {
                let engine = open_engine(project_path)?;
                engine.remove_dependency(&id, &other)?;

                if json {
                    println!(r#"{{"ok":true}}"#);
                } else {
                    println!("Removed dependency {} ↔ {}", id, other);
                }
            }
        },
    }

    Ok(())
}

fn open_engine(
    project_path: &std::path::Path,
) -> Result<Engine, Box<dyn std::error::Error>> {
    Engine::open(project_path).map_err(|e| {
        format!(
            "Failed to open tracker at {}: {}. Run 'beads-tracker init' first.",
            project_path.display(),
            e
        )
        .into()
    })
}

// ── Human-readable output ──────────────────────────────────────────

fn status_icon(status: &str) -> &str {
    match status {
        "open" => "○",
        "in_progress" => "◐",
        "closed" => "●",
        _ => "?",
    }
}

fn priority_display(priority: &str) -> &str {
    match priority {
        "p0" => "P0",
        "p1" => "P1",
        "p2" => "P2",
        "p3" => "P3",
        _ => priority.split_at(0).1, // passthrough
    }
}

fn print_issue_list(issues: &[TrackerIssue]) {
    if issues.is_empty() {
        println!("No issues found.");
        return;
    }
    for issue in issues {
        let labels = if issue.labels.is_empty() {
            String::new()
        } else {
            format!(" [{}]", issue.labels.join(", "))
        };
        println!(
            "{} {} [{} {}] [{}]{} — {}",
            status_icon(&issue.status),
            issue.id,
            status_icon(&issue.status),
            priority_display(&issue.priority),
            issue.issue_type,
            labels,
            issue.title,
        );
    }
}

fn print_issue_detail(issue: &TrackerIssue) {
    println!(
        "{} {} — {}",
        status_icon(&issue.status),
        issue.id,
        issue.title
    );
    println!("  Type:     {}", issue.issue_type);
    println!("  Status:   {}", issue.status);
    println!("  Priority: {}", priority_display(&issue.priority));
    println!("  Author:   {}", issue.author);
    println!("  Created:  {}", issue.created_at);
    println!("  Updated:  {}", issue.updated_at);

    if let Some(ref a) = issue.assignee {
        println!("  Assignee: {}", a);
    }
    if let Some(ref c) = issue.closed_at {
        println!("  Closed:   {}", c);
    }
    if let Some(ref p) = issue.parent {
        println!("  Parent:   {}", p);
    }
    if let Some(ref e) = issue.external_ref {
        println!("  ExtRef:   {}", e);
    }
    if let Some(ref s) = issue.spec_id {
        println!("  Spec:     {}", s);
    }
    if let Some(m) = issue.estimate_minutes {
        println!("  Estimate: {}m", m);
    }

    if !issue.labels.is_empty() {
        println!("  Labels:   {}", issue.labels.join(", "));
    }
    if !issue.blocked_by.is_empty() {
        println!("  Blocked by: {}", issue.blocked_by.join(", "));
    }
    if !issue.blocks.is_empty() {
        println!("  Blocks:   {}", issue.blocks.join(", "));
    }

    if !issue.body.is_empty() {
        println!("\n{}", issue.body);
    }

    if let Some(ref d) = issue.design {
        println!("\n── Design ──\n{}", d);
    }
    if let Some(ref a) = issue.acceptance_criteria {
        println!("\n── Acceptance Criteria ──\n{}", a);
    }
    if let Some(ref n) = issue.notes {
        println!("\n── Notes ──\n{}", n);
    }

    if !issue.comments.is_empty() {
        println!("\n── Comments ({}) ──", issue.comments.len());
        for c in &issue.comments {
            println!("  [{}] {} — {}", c.created_at, c.author, c.id);
            for line in c.body.lines() {
                println!("    {}", line);
            }
        }
    }
}

fn print_search_results(results: &[SearchResult]) {
    if results.is_empty() {
        println!("No results found.");
        return;
    }
    for r in results {
        println!(
            "{} {} — {} (score: {:.2})",
            status_icon(&r.status),
            r.issue_id,
            r.title,
            -r.rank // BM25 returns negative scores; negate for display
        );
        if !r.snippet.is_empty() {
            println!("  {}", r.snippet);
        }
    }
}

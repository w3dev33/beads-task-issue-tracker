# Tracker CLI Reference

Reference skill for the built-in `beads-tracker` CLI binary. Use this when the project uses the built-in tracker backend instead of `br`/`bd`.

## Step 0 — Backend Detection

1. Check if `.tracker/tracker.db` exists in the project root
2. If yes → use `beads-tracker` binary (built-in tracker)
3. If no → fall back to `br`/`bd` (read `~/.claude/config/beads-cli.md` for the binary name)

```bash
# Quick detection
if [ -f ".tracker/tracker.db" ]; then
  echo "Backend: built-in (beads-tracker)"
else
  echo "Backend: beads (br/bd)"
fi
```

The `beads-tracker` binary is at `src-tauri/target/debug/beads-tracker` (dev) or `src-tauri/target/release/beads-tracker` (release). If not in PATH, build it:

```bash
cd src-tauri && cargo build --bin beads-tracker
```

## Global Flags

| Flag | Description |
|------|-------------|
| `-C <path>` / `--project <path>` | Project directory (default: current directory) |
| `--json` | Output as JSON |
| `--actor <name>` | Actor name for authoring (default: git user.name) |

## Commands

### `init` — Initialize tracker

Creates `.tracker/` directory with database and `.gitignore`.

```bash
beads-tracker init
beads-tracker --actor "Laurent" init
```

### `list` — List issues

```bash
beads-tracker list                        # Open issues (default)
beads-tracker list -s open                # Filter by status
beads-tracker list -s in_progress
beads-tracker list -s closed
beads-tracker list -a                     # All issues (shorthand for -s all)
beads-tracker list -t bug                 # Filter by type (task, bug, feature, epic)
beads-tracker list -p p0                  # Filter by priority (p0, p1, p2, p3)
beads-tracker list --assignee "Laurent"   # Filter by assignee
beads-tracker list --limit 10             # Limit results
beads-tracker list -s open -t bug -p p1   # Combine filters
beads-tracker list --json                 # JSON output
```

### `show <id>` — Show issue details

Displays full issue detail including children, comments, labels, dependencies.

```bash
beads-tracker show tracker-a1b2
beads-tracker show tracker-a1b2 --json
```

### `create <title>` — Create a new issue

```bash
# Minimal
beads-tracker create "Fix login bug"

# Full options
beads-tracker create "Add dark mode" \
  -d "Implement dark mode toggle with system preference detection" \
  -t feature \
  -p p1 \
  --assignee "Laurent" \
  -l "ui,theme" \
  --parent tracker-x1y2 \
  --estimate 120 \
  --design "Use CSS variables for theme switching" \
  --acceptance "Toggle works, persists across sessions" \
  --notes "Check existing color tokens" \
  --external-ref "https://github.com/org/repo/issues/42" \
  --spec-id "SPEC-001"
```

| Flag | Short | Description |
|------|-------|-------------|
| `--description` | `-d` | Issue body/description |
| `--type` | `-t` | Issue type: `task`, `bug`, `feature`, `epic` |
| `--priority` | `-p` | Priority: `p0`, `p1`, `p2`, `p3` |
| `--assignee` | | Assignee name |
| `--labels` | `-l` | Comma-separated labels |
| `--parent` | | Parent issue ID (for sub-tasks) |
| `--estimate` | | Estimate in minutes |
| `--design` | | Design notes |
| `--acceptance` | | Acceptance criteria |
| `--notes` | | Additional notes |
| `--external-ref` | | External reference (URL, Redmine ID) |
| `--spec-id` | | Spec ID |

### `update <id>` — Update an issue

```bash
beads-tracker update tracker-a1b2 -s in_progress
beads-tracker update tracker-a1b2 --title "New title"
beads-tracker update tracker-a1b2 -d "Updated description"
beads-tracker update tracker-a1b2 -t bug -p p0
beads-tracker update tracker-a1b2 --assignee "Laurent"
beads-tracker update tracker-a1b2 --assignee ""          # Clear assignee
beads-tracker update tracker-a1b2 -l "ui,urgent"         # Replace all labels
beads-tracker update tracker-a1b2 --parent tracker-x1y2
beads-tracker update tracker-a1b2 --parent ""             # Clear parent
beads-tracker update tracker-a1b2 --estimate 60
beads-tracker update tracker-a1b2 --estimate 0            # Clear estimate
beads-tracker update tracker-a1b2 --design ""             # Clear design
beads-tracker update tracker-a1b2 --acceptance ""         # Clear acceptance
beads-tracker update tracker-a1b2 --notes ""              # Clear notes
beads-tracker update tracker-a1b2 --spec-id ""            # Clear spec ID
```

**Note:** Use empty string `""` to clear optional fields, `0` to clear estimate.

### `close <id>` — Close an issue

```bash
beads-tracker close tracker-a1b2
```

### `delete <id>` — Delete an issue

```bash
beads-tracker delete tracker-a1b2          # Soft delete
beads-tracker delete tracker-a1b2 --hard   # Permanent removal
```

### `search <query>` — Full-text search (FTS5)

```bash
beads-tracker search "login bug"
beads-tracker search "authentication" --limit 10
beads-tracker search "dark mode" --json
```

### `ready` — List unblocked open issues

Shows open issues that are not blocked by any other issue.

```bash
beads-tracker ready
beads-tracker ready --json
```

### `comments` — Manage comments

```bash
beads-tracker comments add tracker-a1b2 "Progress: implemented the toggle"
beads-tracker comments delete comment-id-here
```

### `label` — Manage labels

```bash
beads-tracker label add tracker-a1b2 "urgent"
beads-tracker label remove tracker-a1b2 "urgent"
```

### `dep` — Manage dependencies

```bash
beads-tracker dep add tracker-a1b2 tracker-c3d4              # c3d4 blocks a1b2
beads-tracker dep add tracker-a1b2 tracker-c3d4 --type blocks
beads-tracker dep remove tracker-a1b2 tracker-c3d4
```

## Command Mapping: br/bd → beads-tracker

| br/bd | beads-tracker | Notes |
|-------|---------------|-------|
| `br list -s open` | `beads-tracker list -s open` | Same flags |
| `br show <id>` | `beads-tracker show <id>` | Same |
| `br create -t "<title>"` | `beads-tracker create "<title>"` | Title is positional, not `-t` |
| `br update <id> -s in_progress` | `beads-tracker update <id> -s in_progress` | Same |
| `br close <id>` | `beads-tracker close <id>` | Same |
| `br comments add <id> "<body>"` | `beads-tracker comments add <id> "<body>"` | Same |
| `br dep tree <id>` | `beads-tracker show <id>` | Shows children inline |
| `br list --parent <id>` | `beads-tracker show <id>` | Children shown in `show` output |
| `br search "<query>"` | `beads-tracker search "<query>"` | Same |
| `br ready` | `beads-tracker ready` | Same |

**Key differences:**
- `create`: title is a positional argument, not `-t` flag. `-t` is for issue type.
- `show`: includes children directly (no separate `dep tree` command needed)
- No `sync` command — the built-in tracker is local-only (SQLite), no Dolt sync needed
- IDs use the project prefix (default `tracker-xxxx` vs `br-xxxx`)

## Workflow Examples

### Create an epic with sub-tasks

```bash
# Create epic
beads-tracker create "User authentication" -t epic -p p1

# Note the returned ID, e.g. tracker-a1b2
beads-tracker create "Login form" -t task --parent tracker-a1b2
beads-tracker create "OAuth integration" -t task --parent tracker-a1b2
beads-tracker create "Session management" -t task --parent tracker-a1b2
```

### Triage workflow

```bash
beads-tracker list -s open                  # See all open
beads-tracker ready                         # See what's unblocked
beads-tracker update tracker-a1b2 -p p0     # Bump priority
beads-tracker update tracker-a1b2 --assignee "Laurent"
beads-tracker update tracker-a1b2 -s in_progress
```

### Work on an issue

```bash
beads-tracker update tracker-a1b2 -s in_progress
# ... do the work ...
beads-tracker comments add tracker-a1b2 "Implemented the feature"
beads-tracker close tracker-a1b2
```

### Search and filter

```bash
beads-tracker search "performance"          # FTS5 search
beads-tracker list -t bug -p p0             # Critical bugs
beads-tracker list --assignee "Laurent"     # My issues
```

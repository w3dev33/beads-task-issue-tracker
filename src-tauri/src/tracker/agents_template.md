# AGENTS.md — Built-in Tracker

This project uses `beads-tracker` for issue tracking. Issues are stored locally in SQLite (`.tracker/tracker.db`).

## CLI Binary

`beads-tracker` — if not in PATH, check the project's build output.

## Global Flags

| Flag | Description |
|------|-------------|
| `-C <path>` / `--project <path>` | Project directory (default: current directory) |
| `--json` | Output as JSON |
| `--actor <name>` | Actor name for authoring (default: git user.name) |

## Commands

### `init` — Initialize tracker

```bash
beads-tracker init
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
beads-tracker list --assignee "Name"      # Filter by assignee
beads-tracker list --limit 10             # Limit results
beads-tracker list --json                 # JSON output
```

### `show <id>` — Show issue details

Displays full issue detail including children, comments, labels, dependencies.

```bash
beads-tracker show <id>
beads-tracker show <id> --json
```

### `create <title>` — Create a new issue

```bash
beads-tracker create "Fix login bug"

beads-tracker create "Add dark mode" \
  -d "Description here" \
  -t feature \
  -p p1 \
  --assignee "Name" \
  -l "ui,theme" \
  --parent <parent-id> \
  --estimate 120 \
  --design "Design notes" \
  --acceptance "Acceptance criteria" \
  --notes "Additional notes" \
  --external-ref "https://example.com/issue/42" \
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
beads-tracker update <id> -s in_progress
beads-tracker update <id> --title "New title"
beads-tracker update <id> -d "Updated description"
beads-tracker update <id> -t bug -p p0
beads-tracker update <id> --assignee "Name"
beads-tracker update <id> --assignee ""          # Clear assignee
beads-tracker update <id> -l "ui,urgent"         # Replace all labels
beads-tracker update <id> --parent <parent-id>
beads-tracker update <id> --parent ""             # Clear parent
beads-tracker update <id> --estimate 60
beads-tracker update <id> --estimate 0            # Clear estimate
```

Use empty string `""` to clear optional fields, `0` to clear estimate.

### `close <id>` — Close an issue

```bash
beads-tracker close <id>
```

### `delete <id>` — Delete an issue

```bash
beads-tracker delete <id>              # Soft delete
beads-tracker delete <id> --hard       # Permanent removal
```

### `search <query>` — Full-text search

```bash
beads-tracker search "login bug"
beads-tracker search "query" --limit 10
```

### `ready` — List unblocked open issues

```bash
beads-tracker ready
```

### `comments` — Manage comments

```bash
beads-tracker comments add <id> "Comment body"
beads-tracker comments delete <comment-id>
```

### `label` — Manage labels

```bash
beads-tracker label add <id> "label-name"
beads-tracker label remove <id> "label-name"
```

### `dep` — Manage dependencies

```bash
beads-tracker dep add <id> <blocker-id>              # blocker blocks id
beads-tracker dep add <id> <blocker-id> --type blocks
beads-tracker dep remove <id> <other-id>
```

## Workflow: Work on an issue

```bash
beads-tracker list -s open              # Pick an issue
beads-tracker update <id> -s in_progress
# ... do the work ...
beads-tracker comments add <id> "Done: implemented the feature"
beads-tracker close <id>
```

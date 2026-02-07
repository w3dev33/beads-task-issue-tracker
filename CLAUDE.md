# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## About Beads Task-Issue Tracker

**Beads Task-Issue Tracker** is a desktop application for managing issues tracked with [Beads](https://beads.dev) (`bd`), the AI-native issue tracker that lives directly in your codebase.

Unlike web-based issue trackers (Jira, GitHub Issues), Beads stores issues as files in a `.beads` folder within your repository. This application provides a visual interface to browse, create, edit, and manage these issues without using the command line.

## Prerequisites

- **bd CLI**: Install the Beads CLI tool (`bd`) globally
  ```bash
  # Installation varies - check beads.dev for instructions
  ```
- **Node.js 18+**: Required for frontend development
- **pnpm**: Package manager
- **Rust**: Required for Tauri backend (install via [rustup.rs](https://rustup.rs))
- **A Beads-enabled project**: A repository with a `.beads` folder initialized via `bd init`

## Features

- **Dashboard**: Overview of issue counts by status, type, and priority
- **Issue List**: Filterable table with sorting by any column
- **Issue Details**: Full view with description, comments, metadata, and edit capabilities
- **Quick Actions**: Change status, priority, assignee directly from the list
- **Create/Edit Issues**: Full form with all Beads fields (type, priority, labels, etc.)
- **Comments**: Add comments to issues
- **Project Switching**: Browse and select different Beads projects on your machine
- **Favorites**: Save frequently used project paths
- **Search**: Filter issues by title, ID, or description
- **Dark/Light Theme**: Toggle via settings

## Tech Stack

- **Nuxt 4** with Vue 3 (SPA mode)
- **Tauri 2** for desktop packaging (Rust backend)
- **Shadcn-vue** for UI components
- **TailwindCSS 4** for styling
- **bd CLI** executed via Tauri commands from the Rust backend

## Repository Overview

This is a bd Beads issue tracking management repository. It uses **bd** (beads) for AI-native issue tracking that lives directly in the codebase.

## Issue Workflow

**IMPORTANT**: When starting work on any issue (bug, task, or feature), always use the `/run-issue` skill with the issue ID:

```
/run-issue <issue-id>
```

This ensures:
- Issue status is set to `in_progress`
- Session stats are tracked for the issue
- Proper plan mode is entered for exploration
- Issue is closed properly at the end via `/close-issue`

**Never start working on an issue without running `/run-issue` first.**

## Commit Workflow

Always use the `/review-to-commit` skill when the user asks to commit changes. This skill reviews changes and creates proper commits.

## GitHub Workflow

### Authentication
- Use `gh auth login` if GitHub CLI is not authenticated
- Current account: **w3dev33**

### Pull Requests
- Create with `gh pr create --title "..." --body "..." --base master`
- PR body format:
  ```markdown
  ## Summary
  - Key changes (bullet points)

  ## Test plan
  - [ ] Test steps as checklist
  ```
- Merge with `gh pr merge <number> --merge`

### Releases
1. Set version: `npm version X.Y.Z --no-git-tag-version && node scripts/sync-version.js`
2. Commit: `git commit -m "Release vX.Y.Z"`
3. Tag: `git tag -a vX.Y.Z -m "Release vX.Y.Z"`
4. Push: `git push origin master --tags`
5. Create release: `gh release create vX.Y.Z --title "..." --notes "..."`

**IMPORTANT - Release Notes**:
- **Never upload DMG files manually** - GitHub Actions automatically builds and attaches release artifacts
- Always include the macOS unsigned certificate notice in release notes:

```markdown
## macOS: First Launch

macOS may block the app because it's not signed with an Apple Developer certificate.
You'll see a message saying the app "is damaged and can't be opened."

**To fix this**, run the following command after installing:

\`\`\`bash
xattr -cr /Applications/Beads\ Task-Issue\ Tracker.app
\`\`\`

Then open the app normally. This only needs to be done once.
```

### Co-Authorship
Keep `Co-Authored-By: Claude Code <noreply@anthropic.com>` in commits for transparency. This will show Claude Code as a contributor on GitHub - this is intentional and honest.

## Essential Commands

```bash
# Issue Management
bd ready              # Find available work
bd list               # View all issues
bd show <id>          # View issue details
bd create "title"     # Create new issue
bd update <id> --status in_progress  # Claim work
bd close <id>         # Complete work
bd sync               # Sync with git remote
bd onboard            # Get started with beads
```

## Session Completion Workflow

When ending a work session, ALL steps must be completed. Work is NOT complete until `git push` succeeds.

1. File issues for remaining work
2. Run quality gates (if code changed)
3. Update issue status - close finished work
4. Push to remote (MANDATORY):
   ```bash
   git pull --rebase
   bd sync
   git push
   git status  # Must show "up to date with origin"
   ```
5. Verify all changes committed AND pushed

**Critical**: Never stop before pushing - that leaves work stranded locally.

## Tauri Integration

This application is packaged as a Tauri 2 desktop app with a Rust backend.

### Architecture

- **Frontend**: Nuxt 4 SPA served by Tauri's webview
- **Backend**: Rust code in `src-tauri/src/lib.rs` handles all `bd` CLI calls
- **Communication**: Frontend calls Tauri commands via `@tauri-apps/api/core`

### Key Files

- `src-tauri/tauri.conf.json` - Tauri configuration (window size, bundle settings)
- `src-tauri/src/lib.rs` - Rust backend with all Tauri commands
- `src-tauri/Cargo.toml` - Rust dependencies
- `app/utils/bd-api.ts` - Frontend wrapper for Tauri commands

### Development Commands

```bash
pnpm tauri:dev    # Development with hot reload (runs Nuxt + Tauri)
pnpm tauri:build  # Build production app (DMG on macOS, MSI on Windows)
```

**CRITICAL - Before Starting Dev Server**:
Always kill any existing dev server instances of **this application** before starting a new one. Zombie dev processes can cause duplicate issues, phantom data, and unpredictable behavior.

```bash
# Kill existing beads-issue-tracker dev instances first
pkill -f "beads-issue-tracker" 2>/dev/null
# Then start fresh
pnpm tauri:dev
```

This prevents issues caused by multiple dev instances sharing the same `.beads` database.

### Tauri Commands Available

The Rust backend exposes these commands to the frontend:
- `bd_list`, `bd_count`, `bd_ready`, `bd_status` - Query issues
- `bd_show`, `bd_create`, `bd_update`, `bd_close`, `bd_delete` - CRUD operations
- `bd_comments_add` - Add comments to issues
- `fs_exists`, `fs_list` - File system operations for project picker

**Important**: All `bd` CLI calls go through the Rust backend (no Nitro server in production). The wrapper `app/utils/bd-api.ts` handles the Tauri invoke calls.

## Known Issues & Gotchas

### bd CLI: UNIQUE constraint on external_ref

The `bd` CLI has a **UNIQUE constraint** on the `external_ref` field. This causes issues when:
- Sending `--external-ref ""` (empty string) in update commands
- Multiple issues would have the same empty external_ref value

**What external_ref is used for:**
- **Attached images/files**: Paths to images attached to issues (stored in `.beads/attachments/{issue-id}/`). Multiple paths are separated by newlines.
- **Redmine IDs**: When importing issues from Redmine, the original Redmine bug/issue ID is stored here for traceability.
- **External references**: Any external system reference (URLs, ticket IDs from other trackers, etc.)

**Solution in code**: In `src-tauri/src/lib.rs`, the `bd_update` function uses a unique sentinel value when clearing the external_ref:
```rust
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
```

**How clearing works**: When a user clears the external_ref field:
1. Backend sends `cleared:{issue_id}` instead of empty string to satisfy UNIQUE constraint
2. Frontend's `cleanExternalRef()` filters out lines starting with `cleared:` for display
3. Markdown utilities (`extractImagesFromExternalRef`, `extractNonImageRefs`) also filter `cleared:` prefixes

This allows users to clear the field while avoiding the SQLite UNIQUE constraint error.

**Caution**: When modifying code that handles `external_ref`:
- Never clear it blindly - it may contain important attachment paths or external IDs
- When detaching images, update the field by removing only the specific path, not the entire content
- The field uses newline-separated values for multiple references

## Allowed Paths & Commands (NO PERMISSION REQUIRED)

**CRITICAL**: The following paths and commands have PERMANENT permission granted. Claude Code must NEVER ask for permission to execute them.

### Directories - Always Accessible
- `.claude/` - Session stats, plans, issue tracking stats, and other Claude-related files
- `.beads/` - Issue tracker files managed by bd CLI
- `/Users/laurentchapin/.claude/` - Root Claude Code configuration (commands, plans, scripts, tasks, settings)

### All `bd` CLI Commands - Always Allowed
All `bd` commands can be run without asking:
- `bd list`, `bd show`, `bd ready`, `bd count`, `bd status`
- `bd create`, `bd update`, `bd close`, `bd delete`
- `bd comments add`
- `bd sync`, `bd onboard`, `bd init`
- Any other `bd` subcommand

### File Operations on These Paths - Always Allowed
- Reading files (`cp`, `cat`, file reads)
- Writing files (creating, editing, copying)
- Any bash command targeting `.claude/` or `.beads/`

**Do not prompt the user for permission for any of the above - permission is permanently granted.**

### Actions That ALWAYS Require User Confirmation
- `git commit` - Always ask before committing
- `git push` - Always ask before pushing
- `/close-issue` - Always ask before closing an issue

## Plan Mode

When creating plans for this project, always save them in the local project folder:
- **Correct**: `.claude/plans/` (local to project)
- **Wrong**: `~/.claude/plans/` (global user folder)

This ensures plans are versioned with the project and accessible to all contributors.

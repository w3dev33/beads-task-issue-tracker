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
- **Node.js 18+**: Required for development
- **pnpm**: Package manager
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
- **Electron** for desktop packaging
- **Shadcn-vue** for UI components
- **TailwindCSS 4** for styling
- **bd CLI** executed via IPC from the Electron main process

## Repository Overview

This is a bd Beads issue tracking management repository. It uses **bd** (beads) for AI-native issue tracking that lives directly in the codebase.

## Commit Workflow

Always use the `/review-to-commit` skill when the user asks to commit changes. This skill reviews changes and creates proper commits.

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

## Electron Integration

This application is packaged as an Electron desktop app. See **[electron/CLAUDE.md](electron/CLAUDE.md)** for the complete guide including:

- Architecture and configuration
- IPC communication between main process and renderer
- Critical pitfalls to avoid (CommonJS preload, asar unpacking, etc.)
- Deployment checklist

**Key commands:**
```bash
pnpm dev      # Development with hot reload
pnpm build    # Build production app (DMG on macOS)
```

**Important**: API calls use IPC in Electron (no Nitro server in production). The wrapper `app/utils/bd-api.ts` handles this automatically.

## Allowed Paths

The following paths should always be accessible without asking for permission:
- `.claude/` - Session stats, plans, and other Claude-related files
- `.beads/` - Issue tracker files

## Plan Mode

When creating plans for this project, always save them in the local project folder:
- **Correct**: `.claude/plans/` (local to project)
- **Wrong**: `~/.claude/plans/` (global user folder)

This ensures plans are versioned with the project and accessible to all contributors.

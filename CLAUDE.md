# CLAUDE.md

## Context Documents

- **[.claude/codebase-map.md](.claude/codebase-map.md)** — Architecture, all pages, components, composables, utils, Tauri commands, types, data flow
- **[docs/attachments.md](docs/attachments.md)** — Attachment system (filesystem-only, `external_ref` reserved for real external refs)

Consult these before starting any task.

## Workflows

### Issues
- `/run-issue <id>` — Always run before starting work on any issue
- `/close-issue` — Always ask confirmation before closing
- `/review-to-commit` — Always use when user asks to commit

### Session Completion
All steps mandatory. Work is NOT complete until `git push` succeeds.
1. File issues for remaining work
2. Run quality gates (if code changed): `pnpm test && npx vue-tsc --noEmit`
3. Close finished issues
4. `git pull --rebase && bd sync && git push && git status`

### Testing
- **Run before committing**: `pnpm test` — runs all Vitest unit tests
- **Watch mode**: `pnpm test:watch` — for development
- Tests live in `tests/` mirroring `app/` structure (e.g., `tests/utils/markdown.test.ts` → `app/utils/markdown.ts`)
- Pure logic must be extracted into `app/utils/` for testability (not buried in composables)
- When adding or modifying pure logic (filtering, sorting, parsing, transformations), add or update corresponding tests

### Code Organization
- **Never overload `app/pages/index.vue`** — extract logic into composables (`app/composables/`) and UI sections into dedicated components (`app/components/`)
- Keep `index.vue` as an orchestrator: layout structure, composable wiring, and minimal glue code
- Prefer reusable composables over inline logic for state, dialogs, resize, filtering, etc.
- **Prefer shared components** over duplication — if a UI element is used in multiple places, extract it into a shared component

### Context Management
- **Always prefer `/continue-task` over `/compact`** — it preserves issue context, progress, and next steps far better
- When the session is long and context is getting large, proactively run `/continue-task` before auto-compact triggers
- If a `PreCompact` hook fires with "auto" trigger, immediately run `/continue-task` instead of letting compact proceed blindly

### bd Version Policy
- **Stay on bd 0.49.x** — this is the last stable version with embedded Dolt (CGO) and SQLite backend. It was installed via Homebrew (`brew install bd`) and compiled locally with CGO support.
- **Do NOT upgrade to bd 0.50–0.56+** — versions 0.50+ progressively removed embedded Dolt in favor of server mode (`dolt sql-server`). Version 0.56 removed CGO entirely. Server mode is a regression for standalone desktop apps: no file watcher support, requires polling, server lifecycle management, single-project-per-port limitation.
- **Pre-compiled binaries from GitHub releases (0.50+) lack CGO** — even versions that still have embedded Dolt in source code (0.50–0.55) ship without CGO in their release binaries, making embedded mode non-functional.
- **The branch `feat/bd-056-server-mode`** contains all the work to support bd 0.56 (server mode detection, adaptive polling fix, migration logic, DoltServerBanner). It can be merged if/when bd provides a viable path for standalone apps (e.g., change notification mechanism, multi-database server support).
- **GitHub issue [#2050](https://github.com/steveyegge/beads/issues/2050)** tracks our feedback to the bd team about server mode regressions.
- Always preserve backward compatibility with bd 0.49 — use version-gated helpers (`supports_bd_sync()`, `supports_daemon_flag()`, etc.) in the Rust backend to branch behavior by CLI version.

### bd Backward Compatibility
- Never assume all projects use Dolt — check `project_uses_dolt()` before skipping legacy paths
- Use version-gated helpers in `src-tauri/src/lib.rs` for any feature that depends on a specific bd version

### Logging
- **Never use `console.log`** — always use the native logger so logs end up in the app log file.
- **Frontend (TypeScript)**: `logFrontend('info', '[context] message')` — import from `~/utils/bd-api`. Calls the Rust `log_frontend` Tauri command which writes via `log::info!("[frontend] ...")`.
- **Backend (Rust)**: `log_info!("[context] message")`, `log_error!(...)` macros — write directly to the native log.
- Levels: `'info'`, `'warn'`, `'error'`
- **Log file**: `~/Library/Logs/com.beads.manager/beads.log` — readable via `tail -f` or in the app.

### Dev Server
Always kill zombies before starting: `pkill -f "beads-issue-tracker" 2>/dev/null && pnpm tauri:dev`

## GitHub — Account: w3dev33

### Releases
1. **Update `CHANGELOG.md`** with the target version heading and all changes
2. `npm version X.Y.Z --no-git-tag-version && python3 ~/.claude/scripts/sync-version.py` (same version as CHANGELOG)
3. Commit, tag (`git tag -a vX.Y.Z`), push with tags
4. `gh release create vX.Y.Z --title "..." --notes "..."`
5. **Update `.claude/codebase-map.md`** to reflect any structural changes (new files, composables, commands, etc.)

**Release notes must include:**
- bd compatibility version (e.g., `> Requires **bd 0.49.x** — do not use bd 0.50–0.56+`)
- **Never upload DMG manually** — GitHub Actions handles artifacts
- macOS unsigned certificate notice:
  ```
  xattr -cr /Applications/Beads\ Task-Issue\ Tracker.app
  ```

### Commits
Keep `Co-Authored-By: Claude Code <noreply@anthropic.com>` for transparency.

## Permissions

### Always Allowed (no confirmation needed)
- All `bd` CLI commands
- File operations on `.claude/` and `.beads/`
- `~/.claude/` (global config)

### Always Require Confirmation
- `git commit`, `git push`, `/close-issue`

## Plan Mode

Save plans in `.claude/plans/` (local to project), never `~/.claude/plans/`.

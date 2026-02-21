# CLAUDE.md

## Context Documents

- **[.claude/codebase-map.md](.claude/codebase-map.md)** — Architecture, all pages, components, composables, utils, Tauri commands, types, data flow
- **[docs/attachments.md](docs/attachments.md)** — Attachment system via `external_ref` field, UNIQUE constraint workaround (`cleared:{id}` sentinel)

Consult these before starting any task.

## Workflows

### Issues
- `/run-issue <id>` — Always run before starting work on any issue
- `/close-issue` — Always ask confirmation before closing
- `/review-to-commit` — Always use when user asks to commit

### Session Completion
All steps mandatory. Work is NOT complete until `git push` succeeds.
1. File issues for remaining work
2. Run quality gates (if code changed)
3. Close finished issues
4. `git pull --rebase && bd sync && git push && git status`

### Dev Server
Always kill zombies before starting: `pkill -f "beads-issue-tracker" 2>/dev/null && pnpm tauri:dev`

## GitHub — Account: w3dev33

### Releases
1. **Update `CHANGELOG.md`** with the target version heading and all changes
2. `npm version X.Y.Z --no-git-tag-version && node scripts/sync-version.js` (same version as CHANGELOG)
3. Commit, tag (`git tag -a vX.Y.Z`), push with tags
4. `gh release create vX.Y.Z --title "..." --notes "..."`
5. **Update `.claude/codebase-map.md`** to reflect any structural changes (new files, composables, commands, etc.)

**Release notes must include:**
- bd compatibility version (e.g., `> Requires **bd 0.49.3+**`)
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

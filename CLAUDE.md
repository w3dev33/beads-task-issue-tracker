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

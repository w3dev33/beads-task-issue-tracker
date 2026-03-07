# Codebase Map - Beads Task-Issue Tracker

> Auto-generated comprehensive map of the codebase for faster AI reasoning.
> Last updated: 2026-02-26 | App version: 1.24.0

## Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│  Nuxt 4 SPA (Vue 3) — Single page: pages/index.vue     │
│  ├── Left sidebar: project picker + dashboard           │
│  ├── Center: issue table with filters/sort/pagination   │
│  └── Right sidebar: issue detail/preview/edit           │
├─────────────────────────────────────────────────────────┤
│  Tauri 2 Desktop Shell                                  │
│  ├── Rust backend (src-tauri/src/lib.rs) — 75 commands  │
│  ├── Built-in tracker engine (src-tauri/src/tracker/)   │
│  ├── bd/br CLI bridge (legacy backend)                  │
│  └── File watcher, logging, update checker              │
├─────────────────────────────────────────────────────────┤
│  Three selectable backends per project (Settings UI):   │
│  ├── built-in: SQLite-native via tracker::Engine        │
│  ├── br: beads_rust CLI → .beads/                       │
│  └── bd: beads Go CLI 0.49.x → .beads/                 │
└─────────────────────────────────────────────────────────┘
```

---

## Frontend Structure

### Pages

| File | Purpose |
|------|---------|
| `app/pages/index.vue` (~1250 lines) | SPA root — three-panel layout orchestrator. Wires composables, manages polling/watchers, page-level dialogs (repair, migration, sync error) |
| `app/app.vue` | Entry point — theme init, menu init, global dialogs (Update, About, Settings), notifications |

### Composables (`app/composables/`)

#### Core Data
| File | Exports | Key State | Purpose |
|------|---------|-----------|---------|
| `useIssues.ts` | `useIssues()`, `useEpicExpand()` | `issues`, `selectedIssue`, pagination, sort | Main CRUD + filtering/sorting/grouping. Deduplicates by ID, builds parent-child hierarchy. Derives parent/children from dot notation IDs for bd >= 0.50 |
| `useFilters.ts` | `useFilters()` | `filters` (per-project) | Inclusion filters: status, type, priority, assignee, search, labels |
| `useExclusionFilters.ts` | `useExclusionFilters()` | `exclusions` (per-project) | Exclusion filters: status, priority, type, labels, assignee |
| `useDashboard.ts` | `useDashboard()` | `stats`, `readyIssues` | Computes dashboard KPIs from issues array (no extra API calls) |

#### Storage & Projects
| File | Exports | Purpose |
|------|---------|---------|
| `useProjectStorage.ts` | `useProjectStorage()`, `saveProjectValue()` | Per-project localStorage via path hash (`beads:proj:{hash}:{key}`) |
| `useLocalStorage.ts` | `useLocalStorage()` | Global localStorage with singleton cache |
| `useBeadsPath.ts` | `useBeadsPath()` | Project path management — validates, triggers storage reload on change |
| `useFavorites.ts` | `useProjects()` | Projects — add/remove/rename/reorder, sort modes (renamed from favorites) |

#### Backend & Sync
| File | Exports | Purpose |
|------|---------|---------|
| `useBackendMode.ts` | `useBackendMode()` | Backend mode switching (`br`/`bd`/`built-in`). `syncFromStorage()` syncs to Rust on mount. `ensureTrackerInit()` auto-inits `.tracker/` if needed |
| `useCliClient.ts` | `useCliClient()` | Detects `br` vs `bd` CLI binary in use via `getCliBinaryPath` |
| `useSyncStatus.ts` | `useSyncStatus()` | Git sync status, force sync, error dialog |
| `useConflicts.ts` | `useConflicts()` | Module-singleton for sync conflicts; wraps `trackerGetConflicts/resolveConflict/dismissConflict`; computes `diffFields`, `parsedLocal`, `parsedRemote` for diff UI |

#### UI State
| File | Exports | Purpose |
|------|---------|---------|
| `useColumnConfig.ts` | `useColumnConfig()` | Issue table column visibility (per-project) |
| `useNotification.ts` | `useNotification()` | Toast notifications — auto-dismiss after 3s |
| `useTheme.ts` | `useTheme()` | Dark/light mode toggle |
| `useCollapsible.ts` | `useCollapsible()` | Panel collapse state (dashboard, project sections) |
| `useZoom.ts` | `useZoom()` | Content zoom 75-150% |
| `useAppMenu.ts` | `useAppMenu()` | Tauri native menu bar setup |
| `useTauriWindow.ts` | `useTauriWindow()` | Window drag for custom title bar |
| `usePinnedIssues.ts` | `usePinnedIssues()` | Pinned issue list with sort modes (`added`, `updated`, `manual`) |
| `useKeyboardNavigation.ts` | `useKeyboardNavigation()` | Arrow key navigation for issue list with scroll-to-focused |

#### Polling & Change Detection
| File | Exports | Purpose |
|------|---------|---------|
| `useAdaptivePolling.ts` | `useAdaptivePolling()` | Smart polling: 5s active, 30s blurred, 60s idle, paused when hidden. Cheap mtime check (1s) + expensive data fetch |
| `useChangeDetection.ts` | `useChangeDetection()` | Change detection via native file watcher (Tauri events). Watches `.beads/` or `.tracker/` based on backend mode. SSE backend kept as dead code. 300ms debounce + 3s cooldown |

#### Page Orchestration
| File | Exports | Purpose |
|------|---------|---------|
| `useSidebarResize.ts` | `useSidebarResize()` | Sidebar open/close state (persisted) + drag resize handlers |
| `useIssueDialogs.ts` | `useIssueDialogs()` | All dialog state + ~20 handlers (delete, close, detach, deps, relations). Singleton pattern. Delete notifications |

#### Dialogs & Previews
| File | Exports | Purpose |
|------|---------|---------|
| `useImagePreview.ts` | `useImagePreview()` | Image gallery viewer — loads base64 from filesystem |
| `useMarkdownPreview.ts` | `useMarkdownPreview()` | Markdown file viewer with edit mode + multi-file gallery |
| `useAttachments.ts` | `useAttachments()` | Attachment management for issue detail |

#### System
| File | Exports | Purpose |
|------|---------|---------|
| `useUpdateChecker.ts` | `useUpdateChecker()` | App update checker via GitHub API, supports demo mode |
| `useRepairDatabase.ts` | `useRepairDatabase()` | Detects SCHEMA_MIGRATION_ERROR, repairs single or all projects |
| `useMigrateToDolt.ts` | `useMigrateToDolt()` | Detects Dolt migration needed (bd >= 0.50 with SQLite project), runs 7-step migration preserving labels/deps/comments/attachments |
| `useMigrateRefs.ts` | `useMigrateRefs()` | Wraps `bdCheckRefsMigration`/`bdMigrateRefs` for attachment ref migration checks |

### Components (`app/components/`)

#### Layout (`layout/`)
| Component | Purpose |
|-----------|---------|
| `AppHeader.vue` | Top bar: title, zoom controls, theme toggle, Tauri drag region |
| `UpdateIndicator.vue` | Sync/watcher status badges |
| `UpdateDialog.vue` | Available updates UI |
| `SettingsDialog.vue` | Theme, CLI client, backend selector, probe toggle (dev-only) |
| `AboutDialog.vue` | App info, credits |
| `DebugPanel.vue` | Live log viewer with filters |
| `DebugDialog.vue` | BD CLI version, compatibility info |
| `DialogsLayer.vue` | All issue-management dialogs (delete, epic delete, close, detach, deps, relations) + image/markdown preview |
| `CollapsibleSection.vue` | Generic collapsible header+content wrapper |

#### Dashboard (`dashboard/`)
| Component | Purpose |
|-----------|---------|
| `PathSelector.vue` | Project picker — filesystem tree navigation, probe expose toggle (dev-only) |
| `FolderPicker.vue` | Breadcrumb folder navigation with Beads/Dolt badges (sub-component of PathSelector) |
| `KpiCard.vue` | Stats card (total, open, in-progress, blocked, closed, ready) |
| `StatusChart.vue` | Status pie chart |
| `PriorityChart.vue` | Priority pie chart |
| `QuickList.vue` | Ready/review issues quick access |
| `DashboardContent.vue` | KPIs + collapsible charts + quick lists. Deduplicates desktop/mobile via `hideKpis` and `kpiGridCols` props |
| `OnboardingCard.vue` | Welcome card when no project selected |
| `PrerequisitesCard.vue` | BD CLI + Beads project validation |

#### Issues (`issues/`)
| Component | Purpose |
|-----------|---------|
| `IssueTable.vue` (35KB) | Main table — sortable columns, epic grouping, multi-select, load-more pagination |
| `IssueListPanel.vue` | Wrapper: toolbar + filter chips + issue table. Deduplicates desktop/mobile |
| `IssuesToolbar.vue` | Search, create button, column config, filter toggles |
| `FilterChips.vue` | Active filter chips with clear-all |
| `StatusFilterDropdown.vue` | Status checkbox filter |
| `TypeFilterDropdown.vue` | Type checkbox filter |
| `PriorityFilterDropdown.vue` | Priority checkbox filter |
| `AssigneeFilterDropdown.vue` | Assignee filter (dynamic from data) |
| `LabelFilterDropdown.vue` | Label filter (dynamic from data) |
| `ExclusionFilterDropdown.vue` | Exclusion filters panel |
| `ColumnConfig.vue` | Column visibility dialog |
| `StatusBadge.vue` | Color-coded status display |
| `TypeBadge.vue` | Color-coded type display |
| `PriorityBadge.vue` | Color-coded priority display |
| `LabelBadge.vue` | Multi-label tags |

#### Details (`details/`)
| Component | Purpose |
|-----------|---------|
| `IssueDetailHeader.vue` | Issue badges (id, type, status, priority) + title + action buttons (edit, close, reopen, delete). Deduplicates desktop/mobile |
| `IssuePreview.vue` (32KB) | Full detail view — collapsible sections, image/markdown gallery, quick-edit, comments, relations |
| `IssueForm.vue` (15KB) | Create/edit form — all fields, epic picker, label multi-select. Hides parent selector for bd >= 0.50 (dot notation parent-child) |
| `CommentSection.vue` | Comments display + add |

#### UI Library (`ui/`) — shadcn-vue
- **Form:** Button, Input, Textarea, Label, Checkbox, Select, LabelMultiSelect
- **Layout:** Card, Separator, ScrollArea, Avatar, Collapsible
- **Menus:** DropdownMenu, Select
- **Dialogs:** Dialog, ConfirmDialog, Sheet
- **Data:** Table, Badge, Tooltip
- **Custom:** LinkifiedText, CopyableId, NotificationToast, ImagePreviewDialog, ImageThumbnail, MarkdownPreviewDialog, Sonner

### Utils (`app/utils/`)

| File | Key Exports | Purpose |
|------|-------------|---------|
| `bd-api.ts` (~995 lines) | `bdList()`, `bdCreate()`, `bdUpdate()`, `bdShow()`, `bdClose()`, `bdDelete()`, `bdPollData()`, `bdCheckChanged()`, `bdSync()`, `bdMigrateToDolt()`, `bdCheckNeedsMigration()`, `trackerSync()`, `trackerDetect()`, `trackerInit()`, `trackerGetConflicts()`, `trackerResolveConflict()`, `trackerDismissConflict()`, `trackerCheckBeadsSource()`, `trackerMigrateFromBeads()`, `getBackendMode()`, `setBackendMode()`, etc. | Tauri invoke bridge — all 75 commands. Tracker types: `TrackerSyncResult`, `ConflictRecord`, `BeadsSourceInfo`, `TrackerMigrationResult`. Falls back to web API in browser mode |
| `probe-adapter.ts` | `probeMetricsToIssues()`, `probeMetricsToPollData()`, `matchProbeProject()` | Probe response → app types adapter. `matchProbeProject()`: pure path matching with `.beads` suffix normalization |
| `issue-helpers.ts` | `deduplicateIssues()`, `naturalCompare()`, `sortIssues()`, `filterIssues()`, `groupIssues()`, `computeStatsFromIssues()` | Pure functions extracted from useIssues + useDashboard for testability. Sorting, filtering, epic grouping, dashboard KPIs |
| `favorites-helpers.ts` | `normalizePath()`, `deduplicateFavorites()`, `sortFavorites()`, `isFavorite()`, `createFavoriteEntry()` | Pure functions extracted from useFavorites for testability |
| `markdown.ts` | `renderMarkdown()`, `extractImagesFromMarkdown()`, `extractImagesFromExternalRef()`, `extractMarkdownFromExternalRef()`, `extractNonImageRefs()` | Markdown rendering + image/ref extraction. Filters `cleared:` prefixes |
| `open-url.ts` | `openUrl()`, `openImageFile()`, `readImageFile()`, `readTextFile()`, `writeTextFile()` | URL/file opening + image loading as base64 |
| `path.ts` | `splitPath()`, `getPathSeparator()`, `getFolderName()`, `getParentPath()` | Cross-platform path utilities |
| `hash.ts` | `hashPath()` | DJB2 hash for per-project storage namespacing |
| `lib/utils.ts` | `cn()` | TailwindCSS class merging (clsx + twMerge) |

### Types (`app/types/issue.ts`)

```typescript
type IssueType = 'bug' | 'task' | 'feature' | 'epic' | 'chore'
type IssueStatus = 'open' | 'in_progress' | 'blocked' | 'closed' | 'deferred' | 'tombstone' | 'pinned' | 'hooked'
type IssuePriority = 'p0' | 'p1' | 'p2' | 'p3' | 'p4'

interface Issue { id, title, description, type, status, priority, assignee?, labels[],
  createdAt, updatedAt, closedAt?, comments[], blockedBy?[], blocks?[],
  externalRef?, estimateMinutes?, designNotes?, acceptanceCriteria?, workingNotes?,
  parent?, children?[], relations?[], metadata?, specId?, commentCount?,
  dependencyCount?, dependentCount? }

interface Comment { id, author, content, createdAt }
interface ChildIssue { id, title, status, priority }
interface ParentIssue { id, title, status, priority }
interface Relation { id, title, status, priority, relationType, direction }
interface FilterState { status[], type[], priority[], assignee[], search, labels[] }
interface DashboardStats { total, open, inProgress, blocked, closed, ready, byType{}, byPriority{} }
```

### Plugin

| File | Purpose |
|------|---------|
| `app/plugins/console-to-log.client.ts` | Wraps console.error/warn to send to Rust backend logging. Captures unhandled errors |

---

## Backend Structure (`src-tauri/`)

### Files

| File | Purpose |
|------|---------|
| `src/lib.rs` (5274 lines) | All Tauri commands, data structures, helpers. Single-file backend |
| `src/main.rs` | Entry point — calls `lib::run()` |
| `src/tracker/` (5114 lines, 12 modules) | Built-in SQLite-native issue tracker engine |
| `tauri.conf.json` | Window config (1400x900, overlay title bar), bundle, CSP (connect-src includes `http://localhost:*` for probe SSE), dev port 3133 |
| `Cargo.toml` | Deps: tauri 2.9.5, serde, reqwest, notify 7, dirs 6, rusqlite (bundled) |
| `capabilities/default.json` | Tauri capability permissions |

### Built-in Tracker Engine (`src/tracker/`)

| Module | Lines | Purpose |
|--------|-------|---------|
| `mod.rs` | 313 | `Engine` struct — facade for all operations. `open`/`init`, delegates to sub-modules |
| `config.rs` | 46 | `ProjectConfig` loader (`.tracker/config.toml`) |
| `db.rs` | 229 | SQLite schema (v4), migrations v1→v4 (core tables, FTS5, synced_at, conflicts) |
| `ids.rs` | 103 | ID generation — base36, 4-char suffix, collision-retry |
| `issues.rs` | 1317 | CRUD for issues, comments, labels, deps. FTS5 insert/delete |
| `export.rs` | 628 | JSONL export (atomic write via .tmp+rename) |
| `import.rs` | 1033 | JSONL import with last-write-wins merge, conflict detection via synced_at |
| `sync.rs` | 183 | Git sync cycle: export → commit → pull --rebase → import → push |
| `conflicts.rs` | 171 | Sync conflict storage/resolution ("local" keep / "remote" apply) |
| `migrate.rs` | 511 | `.beads/` → `.tracker/` migration (non-destructive: import JSONL, copy attachments) |
| `search.rs` | 218 | FTS5 full-text search with prefix matching, BM25 rank, snippets |
| `convert.rs` | 362 | Type conversion: `TrackerIssue` ↔ frontend `Issue`, payload mapping |
| `agents_template.md` | — | AGENTS.md template embedded via `include_str!` |

**SQLite schema (v4):** `issues`, `comments`, `labels`, `dependencies`, `issues_fts` (FTS5), `conflicts`, `schema_version`

### Tauri Commands (75 total)

#### Issue Operations
| Command | bd CLI | Special Logic |
|---------|--------|--------------|
| `bd_list` | `bd list --limit=0 [filters]` or `bd list --all` (bd 0.55+) | Syncs before read; transforms raw→frontend format. Uses single `--all` call on bd 0.55+, falls back to 2 calls (open+closed) on older versions |
| `bd_count` | `bd list` (open + closed) | Builds aggregations by type/priority |
| `bd_ready` | `bd ready` | Fetches ready issues |
| `bd_status` | `bd status --json` | Raw JSON passthrough |
| `bd_show` | `bd show <id>` | Returns None if not found |
| `bd_create` | `bd create "title" [--flags]` | Maps all payload fields to CLI flags |
| `bd_update` | `bd update <id> [--flags]` | **External ref sentinel:** empty string → `cleared:{id}` for UNIQUE constraint |
| `bd_close` | `bd close <id>` | Raw JSON response |
| `bd_delete` | `bd delete <id> --force --hard` | Cleans up attachment folder after delete |

#### Comments & Dependencies
| Command | bd CLI |
|---------|--------|
| `bd_comments_add` | `bd comments add <id> <content>` |
| `bd_dep_add` | `bd dep add <issue_id> <blocker_id>` |
| `bd_dep_remove` | `bd dep remove <issue_id> <blocker_id>` |
| `bd_dep_add_relation` | `bd dep add <id1> <id2> --type <type>` |
| `bd_dep_remove_relation` | `bd dep remove <id1> <id2>` |
| `bd_available_relation_types` | Hardcoded list (differs for bd vs br client) |

#### Polling & Sync
| Command | Purpose |
|---------|---------|
| `bd_check_changed` | Cheap mtime check — `.beads/` files (bd/br) or `.tracker/tracker.db` (built-in). No CLI call |
| `bd_reset_mtime` | Clear mtime cache (on project switch) |
| `bd_poll_data` | Batched: 1 sync + 3 fetches (open + closed + ready) |
| `bd_sync` | Manual `bd sync` trigger; 10s cooldown |

#### Built-in Tracker Commands (8)
| Command | Purpose |
|---------|---------|
| `tracker_init` | Initialize `.tracker/` dir + DB + .gitignore + AGENTS.md |
| `tracker_detect` | Check if `.tracker/tracker.db` exists |
| `tracker_sync` | Full git sync cycle (export → commit → pull → import → push) with cooldown |
| `tracker_get_conflicts` | List unresolved sync conflicts |
| `tracker_resolve_conflict` | Apply "local" or "remote" resolution |
| `tracker_dismiss_conflict` | Dismiss conflict without changing issue |
| `tracker_check_beads_source` | Check `.beads/issues.jsonl` availability + count for migration |
| `tracker_migrate_from_beads` | Full `.beads/` → `.tracker/` migration (JSONL import + attachment copy) |

#### Backend Mode
| Command | Purpose |
|---------|---------|
| `get_backend_mode` | Get current backend mode (`bd`/`br`/`built-in`) |
| `set_backend_mode` | Set backend mode (updates `BACKEND_MODE` global) |

#### Filesystem
| Command | Purpose |
|---------|---------|
| `fs_exists` | File existence check |
| `fs_list` | Directory listing with `.beads` and Dolt backend detection (`usesDolt`) |

#### Attachments (all path-validated to `.beads/attachments/`)
| Command | Purpose |
|---------|---------|
| `read_image_file` | Load image as base64 (validates extension + path) |
| `open_image_file` | Open image with native app |
| `delete_attachment_file` | Delete attachment file |
| `copy_file_to_attachments` | Copy file to `.beads/attachments/{issue-id}/` (dedup names) |
| `cleanup_empty_attachment_folder` | Remove empty attachment folder |
| `purge_orphan_attachments` | Delete folders for non-existent issues |
| `read_text_file` | Read .md files from attachments |
| `write_text_file` | Write .md files in attachments |

#### Update Checking
| Command | Purpose |
|---------|---------|
| `check_for_updates` | GitHub API — app version check, platform detection |
| `check_for_updates_demo` | Same but forces `hasUpdate: true` |
| `check_bd_cli_update` | bd CLI version check (routes to bd or br repo) |
| `download_and_install_update` | Download to ~/Downloads; open DMG on macOS |

#### Logging & Debug
| Command | Purpose |
|---------|---------|
| `get/set_logging_enabled` | Toggle logging flag |
| `get/set_verbose_logging` | Toggle verbose (debug) logging |
| `clear_logs` | Clear log file |
| `export_logs` | Copy logs to ~/Downloads |
| `read_logs` | Read full or tail N lines |
| `get_log_path_string` | Return log file path |
| `log_frontend` | Log messages from frontend |
| `get_bd_version` | `bd --version` (cached) |

#### CLI Configuration
| Command | Purpose |
|---------|---------|
| `get/set_cli_binary_path` | Get/set CLI binary (default: "bd"). Validates + persists |
| `validate_cli_binary` | Security checks (no shell metacharacters) + version validation |
| `check_bd_compatibility` | Detect bd vs br client, semver parse, warnings |

#### File Watching
| Command | Purpose |
|---------|---------|
| `start_watching` | Watch `.beads/` (bd/br) or `.tracker/` (built-in) — emits `beads-changed` event (1s debounce) |
| `stop_watching` | Stop watcher |
| `get_watcher_status` | Return active + watched path |

#### Database & Migration
| Command | Purpose |
|---------|---------|
| `bd_repair_database` | Backup db → delete → rebuild. Handles bd < 0.50 (JSONL) vs >= 0.50 (Dolt) |
| `bd_migrate_to_dolt` | 7-step SQLite→Dolt migration: export JSONL, init Dolt, import, restore labels/deps/comments/attachments. Handles empty projects |
| `bd_check_needs_migration` | Detect if project needs Dolt migration (SQLite project + bd >= 0.50) |

### Key Backend Patterns

1. **Backend Mode** — `BACKEND_MODE` global (`bd`/`br`/`built-in`). `is_builtin_backend()` gates behavior in `bd_check_changed`, `bd_poll_data`, `start_watching`. Per-project setting synced from frontend on mount
2. **Tracker Engine Pool** — `TRACKER_ENGINES: HashMap<String, tracker::Engine>` keyed by project path. `with_engine()` helper opens or reuses engine per project
3. **CLI Client Detection** — Detects `bd` (Go) vs `br` (Rust) client from `--version` output. Cached globally. Affects: daemon flag, JSONL support, relation types. Runs `bd --version` from temp dir to avoid triggering auto-migration
4. **Per-Project Mutex** — `BD_PROJECT_LOCKS` serializes all `bd` CLI calls per project to prevent concurrent Dolt embedded access (SIGSEGV crash in dolthub/driver)
5. **Sync Cooldown** — 10s cooldown between `bd sync` calls via `LAST_SYNC_TIME` mutex. Dolt projects skip sync entirely (Dolt handles sync via git)
6. **Mtime Change Detection** — Tracks `.beads/beads.db` + WAL mtime (bd/br) or `.tracker/tracker.db` (built-in) per-project. Scans nested Dolt layout for bd 0.52+. Cheap polling without CLI calls
7. **External Ref Sentinel** — Empty string → `cleared:{id}` to satisfy SQLite UNIQUE constraint
8. **Issue Transform** — `BdRawIssue` → `Issue`: priority int→string, extracts parent/children/relations/blockers from dependency arrays. `TrackerIssue` → `Issue` via `convert.rs`
9. **Tolerant Parsing** — `parse_issues_tolerant()`: tries strict JSON first, falls back to line-by-line
10. **Path Security** — All attachment operations canonicalize paths + verify inside `.beads/attachments/`
11. **Dolt Migration** — 7-step process: export JSONL from SQLite, backup, init Dolt, import JSONL, restore labels/deps/comments, convert attachment paths to absolute. Handles empty projects via init-only path
12. **Dot Notation Parent-Child** — bd >= 0.50 uses structural parent-child via ID (e.g., `abc.1` is child of `abc`). Frontend derives relationships from loaded issues list instead of relying on JSON fields

### Global State (Rust)

```
LOGGING_ENABLED: AtomicBool (false)
VERBOSE_LOGGING: AtomicBool (false)
LAST_SYNC_TIME: Mutex<Option<Instant>>       — sync cooldown
LAST_KNOWN_MTIME: HashMap<String, SystemTime> — per-project mtime cache
BD_PROJECT_LOCKS: HashMap<String, Arc<Mutex<()>>> — per-project mutex (prevents concurrent Dolt SIGSEGV)
CLI_BINARY: Mutex<String> ("bd")             — configurable CLI binary
CLI_CLIENT_INFO: Mutex<Option<(CliClient, u32, u32, u32)>> — cached version
BACKEND_MODE: Mutex<String> ("bd")           — active backend (bd/br/built-in)
TRACKER_ENGINES: HashMap<String, tracker::Engine> — per-project engine pool
```

---

## Data Flow

```
User Action → Vue Component → Composable → bd-api.ts → Tauri invoke()
  → Rust Command → bd CLI or tracker::Engine → .beads/ or .tracker/ SQLite
  → JSON response → Rust transform → Frontend state → Reactive UI update

Backend routing (bd_poll_data, bd_check_changed, etc.):
  is_builtin_backend() → with_engine() → tracker::Engine (direct rusqlite)
  else → bd/br CLI subprocess → parse JSON output

Change Detection (useChangeDetection — native file watcher):
  .beads/ or .tracker/ change → notify crate → Tauri event → watcher backend
    → pollForChanges() → bdPollData() → refresh all data

Polling: useAdaptivePolling → bdCheckChanged() (mtime) → if changed → bdPollData()
  → useIssues + useDashboard update
  (30s safety-net when change detection active, 5s/1s fallback otherwise)

Git Sync (built-in backend):
  trackerSync() → export JSONL → git add/commit → pull --rebase
    → import JSONL (last-write-wins) → detect conflicts → push
```

---

## Configuration

| Setting | Location | Purpose |
|---------|----------|---------|
| CLI binary | `~/.config/com.beads.manager/settings.json` | bd or br path |
| Backend mode | `localStorage beads:proj:{hash}:backendMode` | Per-project: `br`/`bd`/`built-in` |
| Logs | `~/Library/Logs/com.beads.manager/beads.log` | Backend logs (5MB max) |
| Project settings | `localStorage beads:proj:{hash}:*` | Filters, columns, expanded epics, collapsible states |
| Global settings | `localStorage beads:*` | Theme, favorites, zoom, notifications |
| Window | `tauri.conf.json` | 1400x900, min 800x600, overlay title bar |
| Dev server | Port 3133 | `pnpm nuxt dev --port 3133` |

---

## Testing

**Framework**: Vitest with jsdom environment | **Config**: `vitest.config.ts` | **Run**: `pnpm test` / `pnpm test:watch`

### Test Files

| File | Tests | Covers |
|------|-------|--------|
| `tests/utils/issue-helpers.test.ts` | 56 | `deduplicateIssues`, `naturalCompare`, `getParentIdFromIssue`, `compareChildIssues`, sort orders, `sortIssues`, `filterIssues`, `groupIssues`, `computeReadyIssues` |
| `tests/utils/markdown.test.ts` | 42 | `isImagePath`, `isMarkdownPath`, `isUrl`, `extractImagesFromMarkdown`, `extractImagesFromExternalRef`, `extractMarkdownFromExternalRef`, `extractNonImageRefs`, `renderMarkdown` |
| `tests/utils/favorites-helpers.test.ts` | 22 | `normalizePath`, `deduplicateFavorites`, `sortFavorites`, `isFavorite`, `createFavoriteEntry` |
| `tests/utils/path.test.ts` | 19 | `splitPath`, `getPathSeparator`, `getFolderName`, `getParentPath` |
| `tests/utils/open-url.test.ts` | 19 | `isValidUrl`, `isLocalPath`, `normalizeUrl` |
| `tests/composables/useKeyboardNavigation.test.ts` | 17 | Arrow key navigation, scroll-to-focused |
| `tests/utils/attachment-encoding.test.ts` | 14 | Attachment path encoding/decoding |
| `tests/utils/dashboard-stats.test.ts` | 11 | `computeStatsFromIssues` |
| `tests/utils/probe-adapter.test.ts` | 8 | `matchProbeProject` — path matching with `.beads` suffix normalization |
| `tests/utils/hash.test.ts` | 6 | `hashPath` |

**Total: 214 tests** (10 files) | **Strategy**: Extract pure functions from composables into `app/utils/` for unit testing. Composables remain thin reactive wrappers.

**Rust tests**: Tracker modules contain `#[cfg(test)]` blocks — run via `cargo test` in `src-tauri/`.

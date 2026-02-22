# Codebase Map - Beads Task-Issue Tracker

> Auto-generated comprehensive map of the codebase for faster AI reasoning.
> Last updated: 2026-02-22 | App version: 1.22.0

## Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│  Nuxt 4 SPA (Vue 3) — Single page: pages/index.vue     │
│  ├── Left sidebar: project picker + dashboard           │
│  ├── Center: issue table with filters/sort/pagination   │
│  └── Right sidebar: issue detail/preview/edit           │
├─────────────────────────────────────────────────────────┤
│  Tauri 2 Desktop Shell                                  │
│  ├── Rust backend (src-tauri/src/lib.rs)                │
│  ├── 53 Tauri commands wrapping bd CLI + per-project mutex│
│  └── File watcher, logging, update checker              │
├─────────────────────────────────────────────────────────┤
│  bd CLI (beads) — AI-native issue tracker                │
│  └── .beads/ folder in each project                     │
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

#### Polling & Change Detection
| File | Exports | Purpose |
|------|---------|---------|
| `useAdaptivePolling.ts` | `useAdaptivePolling()` | Smart polling: 5s active, 30s blurred, 60s idle, paused when hidden. Cheap mtime check (1s) + expensive data fetch |
| `useChangeDetection.ts` | `useChangeDetection()` | Change detection via native file watcher (Tauri events). SSE backend kept as dead code for future dashboard use. 300ms debounce + 3s cooldown |

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

#### System
| File | Exports | Purpose |
|------|---------|---------|
| `useUpdateChecker.ts` | `useUpdateChecker()` | App update checker via GitHub API, supports demo mode |
| `useSyncStatus.ts` | `useSyncStatus()` | Git sync status, force sync, error dialog |
| `useRepairDatabase.ts` | `useRepairDatabase()` | Detects SCHEMA_MIGRATION_ERROR, repairs single or all projects |
| `useMigrateToDolt.ts` | `useMigrateToDolt()` | Detects Dolt migration needed (bd >= 0.50 with SQLite project), runs 7-step migration preserving labels/deps/comments/attachments |

### Components (`app/components/`)

#### Layout (`layout/`)
| Component | Purpose |
|-----------|---------|
| `AppHeader.vue` | Top bar: title, zoom controls, theme toggle, Tauri drag region |
| `UpdateIndicator.vue` | Sync/watcher status badges |
| `UpdateDialog.vue` | Available updates UI |
| `SettingsDialog.vue` | Theme, CLI client, probe toggle (dev-only) |
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
| `bd-api.ts` (~650 lines) | `bdList()`, `bdCreate()`, `bdUpdate()`, `bdShow()`, `bdClose()`, `bdDelete()`, `bdPollData()`, `bdCheckChanged()`, `bdSync()`, `bdMigrateToDolt()`, `bdCheckNeedsMigration()`, etc. | Tauri invoke bridge — all 53 commands. Falls back to web API in browser mode. Probe functions guarded by `isProbeEnabled()` (dev-only) |
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
| `src/lib.rs` (4140 lines) | All Tauri commands, data structures, helpers. Single-file backend |
| `src/main.rs` | Entry point — calls `lib::run()` |
| `tauri.conf.json` | Window config (1400x900, overlay title bar), bundle, CSP (connect-src includes `http://localhost:*` for probe SSE), dev port 3133 |
| `Cargo.toml` | Deps: tauri 2.9.5, serde, reqwest, notify 7, dirs 6 |
| `capabilities/default.json` | Tauri capability permissions |

### Tauri Commands (53 total)

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
| `bd_check_changed` | Cheap mtime check on `.beads/` files — no CLI call. Scans nested Dolt layout (`.beads/dolt/<name>/.dolt/`) for bd 0.52+ |
| `bd_reset_mtime` | Clear mtime cache (on project switch) |
| `bd_poll_data` | Batched: 1 sync + 3 fetches (open + closed + ready) |
| `bd_sync` | Manual `bd sync` trigger; 10s cooldown |

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
| `start_watching` | Watch `.beads/` — emits `beads-changed` event (1s debounce) |
| `stop_watching` | Stop watcher |
| `get_watcher_status` | Return active + watched path |

#### Database & Migration
| Command | Purpose |
|---------|---------|
| `bd_repair_database` | Backup db → delete → rebuild. Handles bd < 0.50 (JSONL) vs >= 0.50 (Dolt) |
| `bd_migrate_to_dolt` | 7-step SQLite→Dolt migration: export JSONL, init Dolt, import, restore labels/deps/comments/attachments. Handles empty projects |
| `bd_check_needs_migration` | Detect if project needs Dolt migration (SQLite project + bd >= 0.50) |

### Key Backend Patterns

1. **CLI Client Detection** — Detects `bd` (Go) vs `br` (Rust) client from `--version` output. Cached globally. Affects: daemon flag, JSONL support, relation types. Runs `bd --version` from temp dir to avoid triggering auto-migration
2. **Per-Project Mutex** — `BD_PROJECT_LOCKS` serializes all `bd` CLI calls per project to prevent concurrent Dolt embedded access (SIGSEGV crash in dolthub/driver)
3. **Sync Cooldown** — 10s cooldown between `bd sync` calls via `LAST_SYNC_TIME` mutex. Dolt projects skip sync entirely (Dolt handles sync via git)
4. **Mtime Change Detection** — Tracks `.beads/beads.db` + WAL mtime per-project. Scans nested Dolt layout (`.beads/dolt/<name>/.dolt/`) for bd 0.52+. Cheap polling without CLI calls
5. **External Ref Sentinel** — Empty string → `cleared:{id}` to satisfy SQLite UNIQUE constraint
6. **Issue Transform** — `BdRawIssue` → `Issue`: priority int→string, extracts parent/children/relations/blockers from dependency arrays
7. **Tolerant Parsing** — `parse_issues_tolerant()`: tries strict JSON first, falls back to line-by-line
8. **Path Security** — All attachment operations canonicalize paths + verify inside `.beads/attachments/`
9. **Dolt Migration** — 7-step process: export JSONL from SQLite, backup, init Dolt, import JSONL, restore labels/deps/comments, convert attachment paths to absolute. Handles empty projects via init-only path
10. **Dot Notation Parent-Child** — bd >= 0.50 uses structural parent-child via ID (e.g., `abc.1` is child of `abc`). Frontend derives relationships from loaded issues list instead of relying on JSON fields

### Global State (Rust)

```
LOGGING_ENABLED: AtomicBool (false)
VERBOSE_LOGGING: AtomicBool (false)
LAST_SYNC_TIME: Mutex<Option<Instant>>       — sync cooldown
LAST_KNOWN_MTIME: HashMap<String, SystemTime> — per-project mtime cache
BD_PROJECT_LOCKS: HashMap<String, Arc<Mutex<()>>> — per-project mutex (prevents concurrent Dolt SIGSEGV)
CLI_BINARY: Mutex<String> ("bd")             — configurable CLI binary
CLI_CLIENT_INFO: Mutex<Option<(CliClient, u32, u32, u32)>> — cached version
```

---

## Data Flow

```
User Action → Vue Component → Composable → bd-api.ts → Tauri invoke()
  → Rust Command → bd CLI → .beads/ SQLite
  → JSON response → Rust transform → Frontend state → Reactive UI update

Change Detection (useChangeDetection — native file watcher):
  .beads/ change → notify crate → Tauri event → watcher backend
    → pollForChanges() → bdPollData() → refresh all data

Polling: useAdaptivePolling → bdCheckChanged() (mtime) → if changed → bdPollData()
  → useIssues + useDashboard update
  (30s safety-net when change detection active, 5s/1s fallback otherwise)
```

---

## Configuration

| Setting | Location | Purpose |
|---------|----------|---------|
| CLI binary | `~/.config/com.beads.manager/settings.json` | bd or br path |
| Logs | `~/Library/Logs/com.beads.manager/beads.log` | Backend logs (5MB max) |
| Project settings | `localStorage beads:proj:{hash}:*` | Filters, columns, expanded epics, collapsible states |
| Global settings | `localStorage beads:*` | Theme, favorites, zoom, notifications |
| Window | `tauri.conf.json` | 1400x900, min 800x600, overlay title bar |
| Dev server | Port 3133 | `pnpm nuxt dev --port 3133` |

---

## Testing

**Framework**: Vitest with jsdom environment | **Config**: `vitest.config.ts` | **Run**: `pnpm test` / `pnpm test:watch`

### Test Files (`tests/utils/`)

| File | Tests | Covers |
|------|-------|--------|
| `markdown.test.ts` | 54 | `isImagePath`, `isMarkdownPath`, `isUrl`, `extractImagesFromMarkdown`, `extractImagesFromExternalRef`, `extractMarkdownFromExternalRef`, `extractNonImageRefs`, `renderMarkdown` |
| `issue-helpers.test.ts` | 53 | `deduplicateIssues`, `naturalCompare`, `getParentIdFromIssue`, `compareChildIssues`, sort orders, `sortIssues`, `filterIssues`, `groupIssues`, `computeReadyIssues` |
| `favorites-helpers.test.ts` | 22 | `normalizePath`, `deduplicateFavorites`, `sortFavorites`, `isFavorite`, `createFavoriteEntry` |
| `path.test.ts` | 19 | `splitPath`, `getPathSeparator`, `getFolderName`, `getParentPath` |
| `open-url.test.ts` | 19 | `isValidUrl`, `isLocalPath`, `normalizeUrl` |
| `dashboard-stats.test.ts` | 11 | `computeStatsFromIssues` |
| `hash.test.ts` | 6 | `hashPath` |
| `probe-adapter.test.ts` | 8 | `matchProbeProject` — path matching with `.beads` suffix normalization |

**Total: 192 tests** | **Strategy**: Extract pure functions from composables into `app/utils/` for unit testing. Composables remain thin reactive wrappers.

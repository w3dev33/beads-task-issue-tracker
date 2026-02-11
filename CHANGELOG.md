# Changelog

## [1.13.2] - 2026-02-11

> Requires **bd 0.49.3+** for full feature support.

### New Features
- **Fast mtime detection**: Decouple cheap mtime check (1s interval) from data fetch (5s poll). External changes are now detected in ~1s instead of ~5s, with zero CPU cost when nothing changed
- **bd CLI update detection**: Debug Panel now shows when a newer version of the bd CLI is available, with direct link to releases

### Bug Fixes
- **View on GitHub button**: Always show "View on GitHub" button in the update dialog, not just when an update is available

## [1.13.1] - 2026-02-11

> Requires **bd 0.49.3+** for full feature support.

### Bug Fixes
- **File picker on Linux**: Add "All supported files" filter as default in file dialogs so Markdown files are visible without manually switching filters (GTK defaults to first filter)

## [1.13.0] - 2026-02-10

> Requires **bd 0.49.3+** for full feature support. Core features work with bd 0.42+.

### New Features
- **Metadata display**: Read-only formatted JSON display of per-issue metadata in detail view (set via `bd update --metadata`)
- **Spec ID field**: Full create/edit support for the `spec_id` field linking issues to specification documents
- **Comment count column**: New "Comments" column in issue list (hidden by default, enable via column config), with fallback to comments array length
- **bd/br client detection**: Automatic detection of CLI client type (bd vs br) with version-aware feature profiles
- **bd 0.50.0 compatibility**: Version-aware compatibility layer that auto-disables `--no-daemon` flag and JSONL file watching for bd 0.50.0+
- **In Progress sidebar**: Dashboard sidebar now shows issues currently in progress

### Improvements
- **Column config auto-sync**: New default columns are automatically added to persisted column config for existing users
- **Philosophy documentation**: Project philosophy integrated into README

### Bug Fixes
- **Graceful missing issues**: Handle missing issues in `bd_show` without crashing

## [1.12.2] - 2026-02-10

### New Features
- **LATEST release mode**: New workflow mode to quickly publish development builds without version bump, overwriting the same `latest` GitHub release
- **Full changelog in update dialog**: Update dialog now fetches and displays the full CHANGELOG.md instead of just the release body
- **Auto-copy xattr command**: On macOS, clicking "Download & Quit" automatically copies the `xattr -cr` command to clipboard

### Improvements
- **App menu reorganization**: Moved Settings, Check for Update, and Show Logs into the main app menu; removed standalone Debug menu
- **Colored markdown headings**: Compact markdown variant now has colored headings (h1-h4) and styled strong text for dark/light modes
- **Wider update dialog**: Increased dialog width and changelog scroll height for better readability

## [1.12.1] - 2026-02-10

### Bug Fixes
- **Update download errors**: Add error logging and fix error display for update download failures

## [1.12.0] - 2026-02-10

### New Features
- **Configurable CLI binary**: Add configurable CLI binary path for bd-compatible forks, allowing users to specify a custom binary in settings

## [1.11.0] - 2026-02-10

### New Features
- **Sortable favorites**: Drag-and-drop reordering of sidebar favorites with grip handles
- **Sort mode toggle**: Cycle between A-Z, Z-A, and manual order via header button
- **Reset button**: Quick reset to alphabetical order after manual reordering, appears only when needed

### Bug Fixes
- **Project path field**: Fixed path field not updating when opening picker from favorites

## [1.10.4] - 2026-02-10

### New Features
- **Changelog in update dialog**: Release notes from GitHub are now displayed in a scrollable "What's new" section when an update is available
- **Download & Quit**: New button downloads the update (DMG on macOS), mounts it, and closes the app automatically
- **macOS xattr helper**: Shows the `xattr -cr` command with click-to-copy for unsigned app workaround

### Bug Fixes
- **Window close permission**: Added missing `core:window:allow-close` Tauri capability that prevented the app from closing after download

## [1.10.3] - 2026-02-09

### Bug Fixes
- **Stale issue list on project switch**: Fixed mtime tracking using a global singleton that caused stale data when switching between favorite projects. Now uses per-project HashMap to track mtimes independently
- **Slow refresh after favorite change**: Added `bd_reset_mtime` command to invalidate cached mtimes on project switch, ensuring immediate refresh with correct data

### Improvements
- **Markdown CSS consolidation**: Refactored markdown preview styles into a shared CSS base with table support
- **README documentation**: Expanded feature documentation with attachments, bulk operations, and keyboard shortcuts

## [1.10.1] - 2026-02-08

### Improvements
- **Debug panel toggle**: Replaced sync indicator with a debug panel toggle button in the footer for quicker access
- **Attachment documentation**: Added `docs/attachments.md` explaining how the app repurposes `bd`'s `--external-ref` field to implement file attachments, with scripting examples

## [1.10.0] - 2026-02-08

### Improvements
- **Reduced CPU/disk usage**: 4-layer polling optimization — sync cooldown, filesystem mtime check, batched poll command, and adaptive polling intervals. Most poll cycles now spawn zero bd processes
- **Debug Panel smart scroll**: Log view no longer jumps to bottom on auto-refresh when scrolled up, allowing inspection of older entries

### Bug Fixes
- **Debug Panel logging**: Backend logging is now automatically enabled when the Debug Panel is open (was silently disabled)
- **mtime guard accuracy**: Fixed mtime check always reporting "changed" by snapshotting after all poll-triggered db operations complete

## [1.9.0] - 2026-02-07

### New Features
- **Markdown file preview**: View attached `.md` files in a full-screen dialog with rich rendering (headers, tables, code blocks, blockquotes)
- **Inline markdown editing**: Edit markdown files directly in the preview dialog using contentEditable, with save confirmation
- **Markdown attachments**: Attach `.md`/`.markdown` files to issues alongside images, displayed as clickable links in the attachments section
- **Markdown gallery navigation**: Browse multiple markdown attachments with arrow navigation (same UX as image gallery)

### Improvements
- **Diagonal gradient badges**: All badge types now use diagonal gradient styling
- **GitHub footer link**: Added GitHub icon in footer and repository link in update dialog
- **Favorites auto-cleanup**: Users are notified when invalid favorite paths are automatically removed at startup

## [1.8.2] - 2026-02-06

### Bug Fixes
- **Epic children grouping**: Re-parented issues (moved under an epic via `bd update --parent`) now correctly appear grouped under their epic in the table view, not as standalone issues

## [1.8.0] - 2026-02-06

### New Features
- **Label multiselect component**: Replace comma-separated labels input with a multiselect featuring colored badges, search/filter, and create new labels on the fly
- **Periodic update check**: App now checks for updates hourly in the background

### Bug Fixes
- **Database migration repair**: Detect and repair bd 0.49.4 schema migration errors with user-controlled repair dialog for affected projects

## [1.7.0] - 2026-02-04

### New Features
- **Per-project settings isolation**: Filters, column configuration, expanded epics, and collapsible section states are now stored per project using localStorage namespacing with djb2 hash
- **Multi-image navigation**: Preview modal now supports navigating between multiple attached images

### Improvements
- **Image thumbnails**: Reduced thumbnail size to 180px for better layout
- **Preview sections**: 11 collapsible sections in issue preview now persist state per project

## [1.6.5] - 2026-02-03

### Bug Fixes
- **Permanent issue deletion**: Issues now use `--force --hard` flags for permanent deletion, preventing deleted issues from reappearing after sync
- **Delete error notification**: Show error notification when issue deletion fails
- **Filter dropdown behavior**: Exclusion dropdown now properly closes other filter dropdowns
- **Duplicate issues**: Fixed deduplication when merging open/closed issue lists

### Improvements
- **Documentation**: Updated CLAUDE.md with dev server instance management instructions

## [1.6.4] - 2026-01-29

### Bug Fixes
- **EPIC display issues**: Fixed missing EPIC ID in preview, improved children grouping display, and corrected border styling

## [1.6.3] - 2026-01-29

### Bug Fixes
- **Label filter OR logic**: When filtering by multiple labels, issues now show if they have at least one of the selected labels instead of requiring all labels

## [1.6.2] - 2026-01-29

### Bug Fixes
- **External ref persistence**: Clearing the external_ref field now persists correctly using a sentinel value to satisfy the SQLite UNIQUE constraint

## [1.6.1] - 2026-01-29

### New Features
- **Epic deletion confirmation**: Confirmation dialog when deleting an Epic with options for handling child issues

### Bug Fixes
- **Sticky table header**: Table header now stays fixed at top when scrolling through issues

## [1.6.0] - 2026-01-29

### New Features
- **Parent/child relationship management**: Attach or detach issues to/from Epic parents via dropdown selector in edit form
- **Create child from Epic**: New "Create child" button in Epic preview to quickly create child issues with parent pre-selected
- **Epic visual styling**: Colored left borders on Epic rows for better visual distinction

### Improvements
- **Smart form fields**: Parent selector hidden when editing Epic issues (Epics cannot have parents)
- **CLAUDE.md documentation**: Added gotchas about external_ref UNIQUE constraint and its various uses

### Bug Fixes
- **Fix update failures**: Skip empty --external-ref to avoid UNIQUE constraint errors that caused silent update failures

## [1.5.0] - 2026-01-29

### New Features
- **Exclusion filter panel**: Hide issues by type, labels, status, priority, or assignee via a new dropdown with collapsible sections
- **Assignee filter dropdown**: Multi-select filter by assignee with slate-colored badge
- **Two-row filter chips**: "Filters:" row for inclusions, "Hidden:" row for exclusions with independent Clear buttons

### Improvements
- **Red checkmark indicator**: Excluded items show bright red checkmark (#ff3333) with grayed text
- **Auto-open sections**: Exclusion sections auto-open when they contain active filters
- **Unified filter order**: Type, Labels, Status, Priority, Assignee across all filter components
- **Project-specific reset**: Labels and assignees exclusions cleared on project change

## [1.4.0] - 2026-01-29

### New Features
- **Hierarchical epic display**: Child issues are now grouped under their parent epic with collapsible sections
- **Epic progress badge**: Shows closed/total count on epic rows (e.g., "1/10")
- **Short ID display**: Table shows only the unique ID suffix without project prefix (full ID still copied)
- **Natural ID sorting**: IDs with numbers now sort correctly (40b.2 before 40b.10)

### Improvements
- **Visual hierarchy**: Child rows have darker background to distinguish from parent
- **Compact table rows**: Reduced vertical padding in table cells
- **Markdown spacing**: Fixed double-spacing issue in description panel
- **Quick list spacing**: Reduced spacing in "Ready to Work" list

## [1.3.0] - 2026-01-29

### New Features
- **Image attachment system**: Attach images from local files or URLs to issues, stored in `.beads/attachments/{issue-id}/`
- **Attachment cleanup**: Automatic purge of orphan attachment folders when deleting issues
- **File deletion on detach**: Detaching an image from an issue now deletes the file from attachments folder
- **Closed issue restrictions**: Closed issues are now read-only (no edit, attach, comment) until reopened
- **Reopen button**: New button to reopen closed issues directly from the preview panel
- **Action notifications**: Toast notifications for all issue actions (create, save, close, reopen, comment) with issue ID and title

### Improvements
- **Update dialog**: Replaced footer version tooltip with a proper update dialog
- **TypeScript fixes**: Fixed type errors in IssueTable, bd-api, markdown, and count.get

## [1.2.2] - 2026-01-28

### Bug Fixes
- **Filter dropdown behavior**: Fix Tooltip/DropdownMenu nesting order that was blocking click events
- **Exclusive filter state**: Clicking one filter now automatically closes the others
- **Click outside handling**: Clicking outside filter buttons now properly closes the open dropdown

## [1.2.1] - 2026-01-28

### New Features
- **Image preview system**: Issue attachments (screenshots) now display as thumbnails in an "Attachments" section
- **Full-screen image viewer**: Click on thumbnails to view images in a full-screen modal
- **Secure image handling**: Tauri commands restricted to image files only (png, jpg, gif, webp, svg, etc.)

## [1.2.0] - 2026-01-28

### New Features
- **Multi-select filter dropdowns**: Replaced the monolithic "Filter" dropdown with 4 individual filter buttons (Status, Type, Priority, Labels)
- **Label multi-select filter**: Labels now support multi-selection with AND logic (issues must have ALL selected labels)
- **Collapsible favorites section**: Favorites in the sidebar can now be collapsed/expanded

### Improvements
- **Colored filter chips**: Filter badges now use the same colors as the app badges (status, type, priority, labels)
- **Filter tooltips**: Added helpful tooltips to each filter button

## [1.1.5] - 2026-01-28

### Bug Fixes
- **Fix clearing issue fields**: Fields like design notes, acceptance criteria, working notes, assignee, and labels can now be properly cleared when editing an issue

## [1.1.4] - 2026-01-28

### Bug Fixes
- **Search filter now bypasses other filters**: When searching, all issues (including closed ones) are now searched, instead of only searching within already-filtered results

## [1.1.3] - 2026-01-27

### Bug Fixes
- **Bidirectional sync**: Local changes now persist correctly (was using `--import-only` which overwrote local changes)
- **Tolerant JSON parsing**: Handles malformed bd CLI output gracefully, displays valid issues even when some fail to parse
- **bd update fix**: Empty arguments no longer cause update failures

### Debug Panel Enhancements
- **Export logs**: New button to export logs to Downloads folder with path display
- **BD version display**: Shows bd CLI version in Debug Panel header
- **Conditional logging**: Logs disabled by default for better performance
- **Log rotation**: 5MB max file size with automatic rotation (keeps 1 backup)

### Data Structure Updates
- Added support for new bd CLI dependency format
- Added `close_reason`, `issue_id`, `dependency_count` fields
- Made dependency fields optional for compatibility

## [1.1.2] - 2026-01-27

### New Features
- **Debug Panel** accessible via menu `Debug > Show Logs...` or `Cmd+Shift+L`
- **Live/Paused mode** for real-time log monitoring
- **Verbose mode** to display detailed bd command output
- **Force Sync** moved to Debug Panel
- **Colorized logs** by command type for better readability
- **Clear logs** with one click
- **Resizable panel** (up to 50% of screen height)

### Technical Improvements
- Logging enabled in release builds for diagnostics
- Simplified logs by default (byte count only)
- Verbose option to see bd response content

## [1.1.1] - 2026-01-26

### New Features
- **Native macOS Menu**: Added "Check for Update..." menu item in the app menu
- Full native menu bar with Edit (Undo, Redo, Cut, Copy, Paste) and Window menus
- Update dialog shows loading state, version comparison, and download button
- About dialog with app icon, version and credits

### UI Improvements
- Added checkmark icon to "You're up to date" message in footer
- Unified update status text across menu dialog and footer

### Bug Fixes
- **Credits Tooltip**: Fixed tooltip position that was appearing below the viewport instead of above the footer

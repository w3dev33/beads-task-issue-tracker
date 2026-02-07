# Changelog

All notable changes to Beads Task-Issue Tracker will be documented in this file.

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

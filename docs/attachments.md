# Attachment Management

## Context

The `bd` CLI provides **no built-in attachment system**. There is no `bd attach` command, no file storage, and no mechanism to associate files with an issue.

To address this need, we **repurposed the `--external-ref` field** — a simple free-text field originally designed to store a single external reference (e.g., `gh-9`, `jira-ABC-123`) — and turned it into a complete attachment management system.

## Repurposing `external_ref`

### Intended usage by `bd`

```bash
bd update beads-abc --external-ref "gh-42"
```

A text field to store a reference to an external system (GitHub, Jira, etc.).

### Our usage

We store **absolute file paths**, one per line. The application then parses each line and categorizes it by extension:

```
<project>/.beads/attachments/beads-abc/screenshot.png   <- image
<project>/.beads/attachments/beads-abc/spec.md          <- markdown file
https://github.com/org/repo/issues/42                   <- regular external reference
```

The field remains **backward-compatible**: regular external references (URLs, IDs) coexist with file paths.

## Physical Storage

Since `bd` does not manage files, we created our own storage convention:

```
.beads/
  attachments/
    {issue-id}/
      screenshot.png
      spec.md
      screenshot-1.png    <- automatic duplicate handling
```

The `.beads/attachments/` directory is **our own convention** — `bd` is not aware of it and does not manage it. All copy, naming, and cleanup operations are handled by the Tauri backend (Rust).

## Full Workflow

### Adding an attachment

```
1. User clicks "Attach" in the issue form
2. File is selected (filtered by allowed extensions)
3. Tauri backend copies the file to .beads/attachments/{issue-id}/
   -> Duplicate handling: image.png -> image-1.png -> image-2.png
4. The absolute path of the copied file is appended to the external_ref field
5. The app calls: bd update {issue-id} --external-ref "existing lines\nnew path"
```

### Removing an attachment

```
1. User clicks the "x" button on the attachment
2. The corresponding line is removed from external_ref
3. The app calls: bd update {issue-id} --external-ref "remaining lines"
4. The physical file is deleted from .beads/attachments/{issue-id}/
5. If the folder is empty, it is removed
```

### Deleting an issue

```
1. The issue is deleted via bd delete {id} --force --hard
2. The entire .beads/attachments/{issue-id}/ folder is deleted
3. An orphan purge mechanism also cleans up leftover attachment folders
```

## Allowed File Types

| Category | Extensions | Display in app |
|----------|-----------|----------------|
| Images | `png`, `jpg`, `jpeg`, `gif`, `webp`, `bmp`, `svg`, `ico`, `tiff`, `tif` | Thumbnail gallery |
| Markdown | `md`, `markdown` | File list with preview/edit |

## Working Around the UNIQUE Constraint

### The problem

`bd` enforces a `UNIQUE` constraint on the `external_ref` field in its SQLite database. When the last attachment is removed, we cannot send an empty string — otherwise all issues with no reference would share the same `""` value, causing a constraint violation.

### Our solution

We send a **unique sentinel value**: `cleared:{issue-id}`

```rust
// src-tauri/src/lib.rs
if ext.is_empty() {
    args.push(format!("cleared:{}", id));  // e.g., "cleared:beads-abc"
}
```

On the display side, the application filters out all lines starting with `cleared:` so they are never shown to the user.

## Scripting Attachments

If you want to automate attachment creation (e.g., attach a screenshot when creating an issue via a script), you can replicate the workflow manually:

```bash
# 1. Create the issue
ISSUE_ID=$(bd create "Bug: login page broken" --silent)

# 2. Copy your file into the attachments folder
mkdir -p .beads/attachments/$ISSUE_ID
cp /path/to/screenshot.png .beads/attachments/$ISSUE_ID/

# 3. Store the absolute path in external_ref
ABS_PATH="$(pwd)/.beads/attachments/$ISSUE_ID/screenshot.png"
bd update $ISSUE_ID --external-ref "$ABS_PATH"
```

For multiple attachments, separate paths with newlines:

```bash
PATHS="$(pwd)/.beads/attachments/$ISSUE_ID/screenshot.png
$(pwd)/.beads/attachments/$ISSUE_ID/notes.md"
bd update $ISSUE_ID --external-ref "$PATHS"
```

## Related Files

| File | Role |
|------|------|
| `src-tauri/src/lib.rs` | Rust backend: file copy, deletion, reading, orphan purge |
| `app/utils/bd-api.ts` | Frontend wrappers for Tauri commands |
| `app/utils/markdown.ts` | Parsing `external_ref`: sorting images / markdown / references |
| `app/components/details/IssueForm.vue` | Attachment upload UI |
| `app/components/details/IssuePreview.vue` | Attachment display (thumbnails, file list) |
| `app/components/ui/image-preview/ImageThumbnail.vue` | Image thumbnail component |

## Important Notes

- **Never clear `external_ref` with an empty string** — use the `cleared:{id}` sentinel instead
- **Never wipe the entire field** when removing a single attachment — only remove the specific line
- **The field is multi-purpose**: it can contain both file paths and regular external references (URLs, Redmine IDs)
- **`bd` knows nothing about our files**: all management (copy, deletion, cleanup) is our responsibility

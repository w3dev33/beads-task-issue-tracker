# Attachment Management

## Architecture

The **filesystem** is the sole source of truth for attachments. The `external_ref` field contains only real external references (Redmine IDs, GitHub URLs).

```
.beads/
  attachments/
    {issue-id}/
      screenshot-2026-02-24.png
      spec-technique.md
      screenshot-2026-02-24-1.png    <- automatic duplicate handling
```

## How It Works

### Adding an attachment

1. User clicks "Attach" in the issue preview or form
2. One or more files are selected (multi-file supported, filtered by allowed extensions)
3. Tauri backend copies each file to `.beads/attachments/{issue-id}/`
   - Filename is sanitized (diacritics stripped, kebab-case)
   - Duplicates: `image.png` → `image-1.png` → `image-2.png`
4. Frontend refreshes the attachment list from the filesystem
5. **No modification to `external_ref`** — attachments are not stored there

### Viewing attachments

1. Frontend calls `list_attachments(projectPath, issueId)` Tauri command
2. Backend reads `.beads/attachments/{issue-id}/` directory
3. Files are classified as `image` or `markdown` by extension
4. Results are sorted by modification time (newest first)
5. Absolute paths are returned — no resolution needed on the frontend

### Removing an attachment

1. User clicks the "x" button on the attachment
2. Frontend calls `delete_attachment(projectPath, issueId, filename)` Tauri command
3. Backend deletes the file and cleans up empty directories
4. **No modification to `external_ref`**

### Deleting an issue

1. The issue is deleted via `bd delete {id} --force --hard`
2. The entire `.beads/attachments/{issue-id}/` folder is deleted
3. An orphan purge mechanism also cleans up leftover attachment folders

## External References

The `external_ref` field is reserved for **real external references only**:
- Redmine IDs: `redmine-26167`
- GitHub URLs: `https://github.com/org/repo/issues/42`
- Other URLs or IDs

### UNIQUE Constraint

`bd`/`br` enforce a `UNIQUE` constraint on `external_ref`. When an issue has no external reference, we pass an empty string `""` which is internally converted to `null` — no conflict, no sentinel needed.

## Allowed File Types

| Category | Extensions | Display in app |
|----------|-----------|----------------|
| Images | `png`, `jpg`, `jpeg`, `gif`, `webp`, `bmp`, `svg`, `ico`, `tiff`, `tif` | Thumbnail gallery |
| Markdown | `md`, `markdown` | File list with preview/edit |

## Migration History

- **v1**: Absolute file paths stored in `external_ref` (newline-separated)
- **v2**: Compact `att:xxx.ext` format with `index.json` for display names
- **v3** (current): Filesystem-only. `external_ref` cleaned of all attachment refs.
  Marker file: `.beads/.migrated-refs-v3`

## Related Files

| File | Role |
|------|------|
| `src-tauri/src/lib.rs` | Rust backend: `list_attachments`, `delete_attachment`, `copy_file_to_attachments`, migration v3 |
| `app/composables/useAttachments.ts` | Frontend: filesystem-based attachment listing with cache |
| `app/composables/useIssueDialogs.ts` | Attach/detach handlers |
| `app/components/details/IssuePreview.vue` | Attachment display (thumbnails, file list) |
| `app/components/ui/image-preview/ImageThumbnail.vue` | Image thumbnail component |
| `app/utils/attachment-encoding.ts` | `splitRefs`/`joinRefs` for external_ref parsing |
| `app/utils/markdown.ts` | `extractNonImageRefs` for real external refs |

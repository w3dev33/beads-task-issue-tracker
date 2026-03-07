import { invoke } from '@tauri-apps/api/core'

export interface AttachmentFile {
  filename: string
  fileType: string   // "image" or "markdown"
  path: string       // absolute path
  modified: number   // mtime epoch seconds
}

// In-memory cache per issue
const cache = new Map<string, AttachmentFile[]>()

export function useAttachments() {
  const { beadsPath } = useBeadsPath()

  async function listAttachments(issueId: string): Promise<{ images: AttachmentFile[], markdown: AttachmentFile[] }> {
    const cacheKey = `${beadsPath.value}:${issueId}`
    let files = cache.get(cacheKey)

    if (!files) {
      try {
        files = await invoke<AttachmentFile[]>('list_attachments', {
          projectPath: beadsPath.value || '.',
          issueId,
        })
        cache.set(cacheKey, files)
      } catch {
        files = []
      }
    }

    return {
      images: files.filter(f => f.fileType === 'image'),
      markdown: files.filter(f => f.fileType === 'markdown'),
    }
  }

  async function deleteAttachment(issueId: string, filename: string): Promise<void> {
    await invoke('delete_attachment', {
      projectPath: beadsPath.value || '.',
      issueId,
      filename,
    })
    clearCache(issueId)
  }

  function clearCache(issueId?: string) {
    if (issueId) {
      cache.delete(`${beadsPath.value}:${issueId}`)
    } else {
      cache.clear()
    }
  }

  return {
    listAttachments,
    deleteAttachment,
    clearCache,
  }
}

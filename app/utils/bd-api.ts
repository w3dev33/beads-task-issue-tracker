import type { Issue, CreateIssuePayload, UpdateIssuePayload, DashboardStats } from '~/types/issue'
import { invoke } from '@tauri-apps/api/core'

// Type declarations for Tauri detection
declare global {
  interface Window {
    __TAURI__?: unknown
    __TAURI_INTERNALS__?: unknown
  }
}

// Check if running in Tauri
function isTauri(): boolean {
  return typeof window !== 'undefined' && (!!window.__TAURI__ || !!window.__TAURI_INTERNALS__)
}

// ============================================================================
// BD API Functions - Use Tauri invoke in app, fetch in web
// ============================================================================

export interface BdListOptions {
  status?: string[]
  type?: string[]
  priority?: string[]
  assignee?: string
  includeAll?: boolean
  path?: string
}

export async function bdList(options: BdListOptions = {}): Promise<Issue[]> {
  if (isTauri()) {
    return invoke<Issue[]>('bd_list', {
      options: {
        status: options.status,
        type: options.type,
        priority: options.priority,
        assignee: options.assignee,
        includeAll: options.includeAll,
        cwd: options.path,
      },
    })
  }

  // Web: use fetch
  const params = new URLSearchParams()
  if (options.path && options.path !== '.') params.set('path', options.path)
  if (options.includeAll) params.set('all', 'true')
  if (options.status?.length === 1) params.set('status', options.status[0])
  if (options.type?.length) params.set('type', options.type.join(','))
  if (options.priority?.length) params.set('priority', options.priority.join(','))
  if (options.assignee) params.set('assignee', options.assignee)

  const queryString = params.toString()
  const url = queryString ? `/api/bd/list?${queryString}` : '/api/bd/list'
  return $fetch<Issue[]>(url)
}

export interface BdCountResult {
  count: number
  byType: Record<string, number>
  byPriority: Record<string, number>
  lastUpdated: string | null
}

export async function bdCount(path?: string): Promise<BdCountResult> {
  if (isTauri()) {
    return invoke<BdCountResult>('bd_count', { options: { cwd: path } })
  }

  const url = path && path !== '.' ? `/api/bd/count?path=${encodeURIComponent(path)}` : '/api/bd/count'
  return $fetch<BdCountResult>(url)
}

export async function bdReady(path?: string): Promise<Issue[]> {
  if (isTauri()) {
    return invoke<Issue[]>('bd_ready', { options: { cwd: path } })
  }

  const url = path && path !== '.' ? `/api/bd/ready?path=${encodeURIComponent(path)}` : '/api/bd/ready'
  return $fetch<Issue[]>(url)
}

export async function bdStatus(path?: string): Promise<unknown> {
  if (isTauri()) {
    return invoke('bd_status', { options: { cwd: path } })
  }

  const url = path && path !== '.' ? `/api/bd/status?path=${encodeURIComponent(path)}` : '/api/bd/status'
  return $fetch(url)
}

export async function bdShow(id: string, path?: string): Promise<Issue | null> {
  if (isTauri()) {
    return invoke<Issue | null>('bd_show', { id, options: { cwd: path } })
  }

  const url = path && path !== '.' ? `/api/bd/show/${id}?path=${encodeURIComponent(path)}` : `/api/bd/show/${id}`
  return $fetch<Issue>(url)
}

export async function bdCreate(payload: CreateIssuePayload, path?: string): Promise<Issue | null> {
  if (isTauri()) {
    return invoke<Issue | null>('bd_create', {
      payload: { ...payload, cwd: path },
    })
  }

  const url = path && path !== '.' ? `/api/bd/create?path=${encodeURIComponent(path)}` : '/api/bd/create'
  return $fetch<Issue>(url, {
    method: 'POST',
    body: payload,
  })
}

export async function bdUpdate(id: string, payload: UpdateIssuePayload, path?: string): Promise<Issue | null> {
  if (isTauri()) {
    return invoke<Issue | null>('bd_update', {
      id,
      updates: { ...payload, cwd: path },
    })
  }

  const url = path && path !== '.' ? `/api/bd/update/${id}?path=${encodeURIComponent(path)}` : `/api/bd/update/${id}`
  return $fetch<Issue>(url, {
    method: 'PATCH',
    body: payload,
  })
}

export async function bdClose(id: string, path?: string): Promise<unknown> {
  if (isTauri()) {
    return invoke('bd_close', { id, options: { cwd: path } })
  }

  const url = path && path !== '.' ? `/api/bd/close/${id}?path=${encodeURIComponent(path)}` : `/api/bd/close/${id}`
  return $fetch(url, { method: 'POST' })
}

export async function bdDelete(id: string, path?: string): Promise<{ success: boolean; id: string }> {
  if (isTauri()) {
    return invoke<{ success: boolean; id: string }>('bd_delete', { id, options: { cwd: path } })
  }

  const url = path && path !== '.' ? `/api/bd/delete/${id}?path=${encodeURIComponent(path)}` : `/api/bd/delete/${id}`
  return $fetch(url, { method: 'DELETE' })
}

export async function bdAddComment(id: string, content: string, path?: string): Promise<{ success: boolean }> {
  if (isTauri()) {
    return invoke<{ success: boolean }>('bd_comments_add', { id, content, options: { cwd: path } })
  }

  const url = path && path !== '.' ? `/api/bd/comments/${id}?path=${encodeURIComponent(path)}` : `/api/bd/comments/${id}`
  return $fetch(url, {
    method: 'POST',
    body: { content },
  })
}

export async function bdSync(path?: string): Promise<void> {
  if (isTauri()) {
    return invoke<void>('bd_sync', { cwd: path })
  }

  // Web fallback - no-op (sync is handled by bd daemon in web mode)
}

// ============================================================================
// Debug / Logging API
// ============================================================================

export async function getLoggingEnabled(): Promise<boolean> {
  if (isTauri()) {
    return invoke<boolean>('get_logging_enabled')
  }
  return false
}

export async function setLoggingEnabled(enabled: boolean): Promise<void> {
  if (isTauri()) {
    return invoke<void>('set_logging_enabled', { enabled })
  }
}

export async function getVerboseLogging(): Promise<boolean> {
  if (isTauri()) {
    return invoke<boolean>('get_verbose_logging')
  }
  return false
}

export async function setVerboseLogging(enabled: boolean): Promise<void> {
  if (isTauri()) {
    return invoke<void>('set_verbose_logging', { enabled })
  }
}

export async function clearLogs(): Promise<void> {
  if (isTauri()) {
    return invoke<void>('clear_logs')
  }
}

export async function readLogs(tailLines?: number): Promise<string> {
  if (isTauri()) {
    return invoke<string>('read_logs', { tailLines })
  }
  return ''
}

export async function getLogPath(): Promise<string> {
  if (isTauri()) {
    return invoke<string>('get_log_path_string')
  }
  return ''
}

// ============================================================================
// File System API - For folder picker
// ============================================================================

export interface DirectoryEntry {
  name: string
  path: string
  isDirectory: boolean
  hasBeads: boolean
}

export interface FsListResult {
  currentPath: string
  hasBeads: boolean
  entries: DirectoryEntry[]
}

export async function fsList(path?: string): Promise<FsListResult> {
  if (isTauri()) {
    return invoke<FsListResult>('fs_list', { path })
  }

  return $fetch<FsListResult>('/api/fs/list', {
    params: path ? { path } : undefined,
  })
}

export async function fsExists(path: string): Promise<boolean> {
  if (isTauri()) {
    return invoke<boolean>('fs_exists', { path })
  }

  // In web mode, assume path exists (can't check filesystem)
  return true
}

// File watcher removed - replaced by polling for lower CPU usage

// ============================================================================
// Update Checker API
// ============================================================================

export interface UpdateInfo {
  currentVersion: string
  latestVersion: string
  hasUpdate: boolean
  releaseUrl: string
}

export async function checkForUpdates(): Promise<UpdateInfo> {
  if (isTauri()) {
    return invoke<UpdateInfo>('check_for_updates')
  }

  // Web: call GitHub API directly
  const response = await fetch('https://api.github.com/repos/w3dev33/beads-task-issue-tracker/releases/latest')
  if (!response.ok) {
    throw new Error(`GitHub API returned status: ${response.status}`)
  }

  const release = await response.json()
  const currentVersion = useRuntimeConfig().public.appVersion as string
  const latestVersion = release.tag_name.replace(/^v/, '')

  const compareVersions = (current: string, latest: string): boolean => {
    const parse = (v: string) => v.split('.').map(n => parseInt(n, 10) || 0)
    const c = parse(current)
    const l = parse(latest)
    for (let i = 0; i < 3; i++) {
      if ((l[i] || 0) > (c[i] || 0)) return true
      if ((c[i] || 0) > (l[i] || 0)) return false
    }
    return false
  }

  return {
    currentVersion,
    latestVersion,
    hasUpdate: compareVersions(currentVersion, latestVersion),
    releaseUrl: release.html_url,
  }
}

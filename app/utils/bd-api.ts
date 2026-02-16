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

// Check if error is a schema migration error (bd 0.49.4 bug)
export function isSchemaMigrationError(error: unknown): boolean {
  if (error instanceof Error) {
    return error.message.includes('SCHEMA_MIGRATION_ERROR') || error.message.includes('no such column: spec_id')
  }
  if (typeof error === 'string') {
    return error.includes('SCHEMA_MIGRATION_ERROR') || error.includes('no such column: spec_id')
  }
  return false
}

// Repair database result
export interface RepairResult {
  success: boolean
  message: string
  backupPath?: string
}

// Repair database (for schema migration issues)
export async function bdRepairDatabase(path?: string): Promise<RepairResult> {
  if (!isTauri()) {
    throw new Error('Database repair is only available in the desktop app')
  }
  return invoke<RepairResult>('bd_repair_database', { cwd: path })
}

// ============================================================================
// Polling Optimization API
// ============================================================================

/**
 * Check if the beads database has changed since last check (filesystem mtime).
 * Extremely cheap — just 1-2 stat() calls, no bd process spawns.
 * Returns true if changes detected or first check.
 */
export async function bdCheckChanged(path?: string): Promise<boolean> {
  if (isTauri()) {
    return invoke<boolean>('bd_check_changed', { cwd: path })
  }
  // Web fallback: always report changed (can't check filesystem)
  return true
}

/**
 * Reset the cached mtime for a specific project (or all projects).
 * Called when switching projects to force a fresh poll on next cycle.
 */
export async function bdResetMtime(path?: string): Promise<void> {
  if (isTauri()) {
    return invoke<void>('bd_reset_mtime', { cwd: path })
  }
}

/**
 * Batched poll: fetches open + closed + ready issues in a single IPC call.
 * Syncs once, then runs 3 bd commands sequentially on the backend.
 * Replaces 3 separate IPC calls for lower overhead.
 */
export interface PollData {
  openIssues: Issue[]
  closedIssues: Issue[]
  readyIssues: Issue[]
}

export async function bdPollData(path?: string): Promise<PollData> {
  if (isTauri()) {
    return invoke<PollData>('bd_poll_data', { cwd: path })
  }

  // Web fallback: make separate calls
  const [openIssues, closedIssues, readyIssues] = await Promise.all([
    bdList({ path }),
    bdList({ path, status: ['closed'] }),
    bdReady(path),
  ])
  return { openIssues, closedIssues, readyIssues }
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
  if (options.status?.length === 1 && options.status[0]) params.set('status', options.status[0])
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
  return $fetch<{ success: boolean; id: string }>(url, { method: 'DELETE' })
}

export async function bdAddComment(id: string, content: string, path?: string): Promise<{ success: boolean }> {
  if (isTauri()) {
    return invoke<{ success: boolean }>('bd_comments_add', { id, content, options: { cwd: path } })
  }

  const url = path && path !== '.' ? `/api/bd/comments/${id}?path=${encodeURIComponent(path)}` : `/api/bd/comments/${id}`
  return $fetch<{ success: boolean }>(url, {
    method: 'POST',
    body: { content },
  })
}

export async function bdAddDependency(issueId: string, blockerId: string, path?: string): Promise<{ success: boolean }> {
  if (isTauri()) {
    return invoke<{ success: boolean }>('bd_dep_add', { issueId, blockerId, options: { cwd: path } })
  }

  throw new Error('Dependency management is only available in the desktop app')
}

export async function bdRemoveDependency(issueId: string, blockerId: string, path?: string): Promise<{ success: boolean }> {
  if (isTauri()) {
    return invoke<{ success: boolean }>('bd_dep_remove', { issueId, blockerId, options: { cwd: path } })
  }

  throw new Error('Dependency management is only available in the desktop app')
}

export async function bdAvailableRelationTypes(): Promise<Array<{ value: string; label: string }>> {
  if (isTauri()) {
    return invoke<Array<{ value: string; label: string }>>('bd_available_relation_types')
  }
  throw new Error('Only available in the desktop app')
}

export async function bdAddRelation(id1: string, id2: string, relationType: string, path?: string): Promise<{ success: boolean }> {
  if (isTauri()) {
    return invoke<{ success: boolean }>('bd_dep_add_relation', { id1, id2, relationType, options: { cwd: path } })
  }
  throw new Error('Relation management is only available in the desktop app')
}

export async function bdRemoveRelation(id1: string, id2: string, path?: string): Promise<{ success: boolean }> {
  if (isTauri()) {
    return invoke<{ success: boolean }>('bd_dep_remove_relation', { id1, id2, options: { cwd: path } })
  }
  throw new Error('Relation management is only available in the desktop app')
}

export async function bdSync(path?: string): Promise<void> {
  if (isTauri()) {
    return invoke<void>('bd_sync', { cwd: path })
  }

  // Web fallback - no-op (sync not available in web mode)
}

export interface PurgeResult {
  deletedCount: number
  deletedFolders: string[]
}

export async function bdPurgeOrphanAttachments(path?: string): Promise<PurgeResult> {
  if (isTauri()) {
    return invoke<PurgeResult>('purge_orphan_attachments', {
      projectPath: path || '.',
    })
  }
  // No server implementation needed
  return { deletedCount: 0, deletedFolders: [] }
}

export async function bdCleanupEmptyAttachmentFolder(issueId: string, path?: string): Promise<boolean> {
  if (isTauri()) {
    return invoke<boolean>('cleanup_empty_attachment_folder', {
      projectPath: path || '.',
      issueId,
    })
  }
  // No server implementation needed
  return false
}

export async function bdDeleteAttachmentFile(filePath: string): Promise<boolean> {
  if (isTauri()) {
    return invoke<boolean>('delete_attachment_file', { filePath })
  }
  // No server implementation needed
  return false
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

export async function exportLogs(): Promise<string> {
  if (isTauri()) {
    return invoke<string>('export_logs')
  }
  return ''
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

export async function logFrontend(level: 'error' | 'warn' | 'info', message: string): Promise<void> {
  if (isTauri()) {
    return invoke<void>('log_frontend', { level, message })
  }
}

export async function getBdVersion(): Promise<string> {
  if (isTauri()) {
    return invoke<string>('get_bd_version')
  }
  return 'web mode'
}

export interface BdCliUpdateInfo {
  currentVersion: string
  latestVersion: string
  hasUpdate: boolean
  releaseUrl: string
}

export async function checkBdCliUpdate(): Promise<BdCliUpdateInfo> {
  if (isTauri()) {
    return invoke<BdCliUpdateInfo>('check_bd_cli_update')
  }
  return { currentVersion: 'web', latestVersion: 'web', hasUpdate: false, releaseUrl: '' }
}

// ============================================================================
// bd Compatibility Check API
// ============================================================================

export interface BdCompatibilityInfo {
  version: string
  /** "bd", "br", or "unknown" */
  clientType: string
  versionTuple: number[] | null
  supportsDaemonFlag: boolean
  usesJsonlFiles: boolean
  warnings: string[]
}

export async function checkBdCompatibility(): Promise<BdCompatibilityInfo> {
  if (isTauri()) {
    return invoke<BdCompatibilityInfo>('check_bd_compatibility')
  }
  return {
    version: 'web mode',
    clientType: 'unknown',
    versionTuple: null,
    supportsDaemonFlag: false,
    usesJsonlFiles: false,
    warnings: [],
  }
}

// ============================================================================
// CLI Binary Configuration API
// ============================================================================

export async function getCliBinaryPath(): Promise<string> {
  if (isTauri()) {
    return invoke<string>('get_cli_binary_path')
  }
  return 'bd'
}

export async function setCliBinaryPath(path: string): Promise<string> {
  if (!isTauri()) {
    throw new Error('CLI binary configuration is only available in the desktop app')
  }
  return invoke<string>('set_cli_binary_path', { path })
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

// ============================================================================
// File Watcher API
// ============================================================================

export interface WatcherStatus {
  active: boolean
  watchedPath: string | null
}

export async function startWatching(path: string): Promise<void> {
  if (isTauri()) {
    return invoke<void>('start_watching', { path })
  }
}

export async function stopWatching(): Promise<void> {
  if (isTauri()) {
    return invoke<void>('stop_watching')
  }
}

export async function getWatcherStatus(): Promise<WatcherStatus> {
  if (isTauri()) {
    return invoke<WatcherStatus>('get_watcher_status')
  }
  return { active: false, watchedPath: null }
}

// ============================================================================
// Update Checker API
// ============================================================================

export interface UpdateInfo {
  currentVersion: string
  latestVersion: string
  hasUpdate: boolean
  releaseUrl: string
  downloadUrl: string | null
  platform: string
  releaseNotes: string | null
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

  // Fetch CHANGELOG.md via GitHub API (raw.githubusercontent CDN ignores query params for caching)
  let releaseNotes: string | null = release.body || null
  try {
    const changelogResponse = await fetch('https://api.github.com/repos/w3dev33/beads-task-issue-tracker/contents/CHANGELOG.md', {
      headers: { 'Accept': 'application/vnd.github.raw+json' },
    })
    if (changelogResponse.ok) {
      releaseNotes = await changelogResponse.text()
    }
  } catch {
    // Fallback to release body
  }

  return {
    currentVersion,
    latestVersion,
    hasUpdate: compareVersions(currentVersion, latestVersion),
    releaseUrl: release.html_url,
    downloadUrl: null,
    platform: 'unknown',
    releaseNotes,
  }
}

export async function checkForUpdatesDemo(): Promise<UpdateInfo> {
  if (isTauri()) {
    return invoke<UpdateInfo>('check_for_updates_demo')
  }

  // Web fallback: simulate demo with real GitHub data
  const info = await checkForUpdates()
  return {
    ...info,
    currentVersion: '0.0.0',
    hasUpdate: true,
  }
}

export async function downloadAndInstallUpdate(downloadUrl: string): Promise<string> {
  if (isTauri()) {
    return invoke<string>('download_and_install_update', { downloadUrl })
  }
  throw new Error('Download is only available in the desktop app')
}

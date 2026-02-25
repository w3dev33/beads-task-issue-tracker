import type { Issue, CreateIssuePayload, UpdateIssuePayload } from '~/types/issue'
import { invoke } from '@tauri-apps/api/core'
import { matchProbeProject } from '~/utils/probe-adapter'
import type { ProbeMetricsResponse, ProbeProject } from '~/utils/probe-adapter'

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
// External Data Source
// ============================================================================

function parseLocalStorageValue(raw: string | null, fallback: string): string {
  if (!raw) return fallback
  // VueUse's useLocalStorage may store strings JSON-wrapped (e.g. "true" → '"true"')
  // Handle both raw and JSON-wrapped values
  if (raw.startsWith('"') && raw.endsWith('"')) {
    try { return JSON.parse(raw) } catch { /* fall through */ }
  }
  return raw
}

function isProbeEnabled(): boolean {
  // Probe is dev-only until it becomes a public feature
  if (!import.meta.dev) return false
  if (typeof window === 'undefined') return false
  const raw = localStorage.getItem('beads:probeEnabled')
  if (!raw) return false
  // VueUse stores booleans as JSON: "true" or "false"
  try { return JSON.parse(raw) === true } catch { return raw === 'true' }
}

export function getExternalUrl(): string {
  if (typeof window === 'undefined') return 'http://localhost:9100'
  return parseLocalStorageValue(localStorage.getItem('beads:dataSourceUrl'), 'http://localhost:9100')
}

export async function fetchExternalData(url: string): Promise<string> {
  if (isTauri()) {
    return invoke<string>('fetch_external_data', { url })
  }
  throw new Error('External data source is only available in the desktop app')
}

export async function checkExternalHealth(url: string): Promise<boolean> {
  if (isTauri()) {
    logFrontend('info', `[probe] Health check: ${url}`)
    const ok = await invoke<boolean>('check_external_health', { url })
    logFrontend('info', `[probe] Health check result: ${ok ? 'connected' : 'disconnected'}`)
    return ok
  }
  return false
}

async function postExternalData(url: string, body: string): Promise<string> {
  if (isTauri()) {
    return invoke<string>('post_external_data', { url, body })
  }
  throw new Error('External data source is only available in the desktop app')
}

async function deleteExternalData(url: string): Promise<string> {
  if (isTauri()) {
    return invoke<string>('delete_external_data', { url })
  }
  throw new Error('External data source is only available in the desktop app')
}

export async function registerProbeProject(baseUrl: string, beadsPath: string): Promise<{ registered: boolean; project: string; path: string }> {
  const raw = await postExternalData(
    `${baseUrl}/projects`,
    JSON.stringify({ path: beadsPath }),
  )
  return JSON.parse(raw)
}

export async function unregisterProbeProject(baseUrl: string, projectName: string): Promise<void> {
  await deleteExternalData(`${baseUrl}/projects/${encodeURIComponent(projectName)}`)
}

async function patchExternalData(url: string, body: string): Promise<string> {
  if (isTauri()) {
    return invoke<string>('patch_external_data', { url, body })
  }
  throw new Error('External data source is only available in the desktop app')
}

export async function patchProbeProject(baseUrl: string, projectName: string, data: { expose?: boolean }): Promise<void> {
  await patchExternalData(
    `${baseUrl}/projects/${encodeURIComponent(projectName)}`,
    JSON.stringify(data),
  )
}

/**
 * Register a project with the probe and set expose flag.
 * If already registered (409), patch the expose flag instead.
 */
export async function registerOrExposeProject(baseUrl: string, beadsPath: string, expose: boolean): Promise<{ project: string }> {
  logFrontend('info', `[probe] Register/expose project: ${beadsPath} (expose: ${expose})`)
  try {
    const raw = await postExternalData(
      `${baseUrl}/projects`,
      JSON.stringify({ path: beadsPath, expose }),
    )
    const result = JSON.parse(raw)
    logFrontend('info', `[probe] Project registered: ${result.project}`)
    return result
  } catch (error) {
    // If already registered (409), try to find and patch
    const msg = error instanceof Error ? error.message : String(error)
    if (msg.includes('409')) {
      logFrontend('info', '[probe] Project already registered (409), patching expose flag')
      // Find the project name from the list
      const projects = await listProbeProjects(baseUrl)
      const match = matchProbeProject(projects, beadsPath)
      if (match) {
        await patchProbeProject(baseUrl, match.name, { expose })
        logFrontend('info', `[probe] Project patched: ${match.name} (expose: ${expose})`)
        return { project: match.name }
      }
    }
    logFrontend('error', `[probe] Failed to register/expose project: ${msg}`)
    throw error
  }
}

/**
 * Auto-register a project with the probe (fire-and-forget).
 * Silently handles 409 (already registered) and all errors.
 */
export async function ensureProbeRegistration(beadsPath: string): Promise<void> {
  if (!isProbeEnabled()) return
  const url = getExternalUrl()
  try {
    await registerProbeProject(url, beadsPath)
    await logFrontend('info', `[probe] Auto-registered project: ${beadsPath}`)
  } catch (error) {
    const msg = error instanceof Error ? error.message : String(error)
    if (msg.includes('409')) {
      // Already registered — success
      await logFrontend('info', `[probe] Project already registered: ${beadsPath}`)
    } else {
      await logFrontend('warn', `[probe] Auto-register failed (non-blocking): ${msg}`)
    }
  }
}

/**
 * Unregister a project from the probe (fire-and-forget).
 * Silently handles all errors (project not found = ok).
 */
export async function probeUnregisterProject(beadsPath: string): Promise<void> {
  if (!isProbeEnabled()) return
  const url = getExternalUrl()
  try {
    const projects = await listProbeProjects(url)
    const match = matchProbeProject(projects, beadsPath)
    if (match) {
      await unregisterProbeProject(url, match.name)
      await logFrontend('info', `[probe] Unregistered project: ${match.name}`)
    }
  } catch (error) {
    const msg = error instanceof Error ? error.message : String(error)
    await logFrontend('warn', `[probe] Unregister failed (non-blocking): ${msg}`)
  }
}

/**
 * Launch beads-probe if not already running.
 * Waits for the probe to be ready before returning (up to 5s).
 */
export async function launchProbeIfNeeded(): Promise<void> {
  if (!isProbeEnabled()) return
  if (!isTauri()) return
  const url = getExternalUrl()
  const port = parseInt(new URL(url).port || '9100')
  try {
    const result = await invoke<string>('launch_probe', { port })
    await logFrontend('info', `[probe] launch_probe: ${result}`)
    // If we just launched, wait for the probe to be ready
    if (result === 'launched') {
      const healthUrl = url.replace(/\/$/, '')
      for (let i = 0; i < 10; i++) {
        await new Promise(r => setTimeout(r, 500))
        const ok = await checkExternalHealth(healthUrl)
        if (ok) {
          await logFrontend('info', `[probe] Ready after ${(i + 1) * 500}ms`)
          return
        }
      }
      await logFrontend('warn', '[probe] Launched but not ready after 5s')
    }
  } catch (error) {
    const msg = error instanceof Error ? error.message : String(error)
    await logFrontend('warn', `[probe] Failed to launch probe: ${msg}`)
  }
}

/**
 * Find the probe project name for a given beads path.
 * Matches against registered projects (path or path/.beads).
 */
export async function getProbeProjectName(beadsPath: string): Promise<string | null> {
  const url = getExternalUrl()
  const projects = await listProbeProjects(url)
  return matchProbeProject(projects, beadsPath)?.name ?? null
}

export async function listProbeProjects(baseUrl: string): Promise<ProbeProject[]> {
  const raw = await fetchExternalData(`${baseUrl}/projects`)
  const list = JSON.parse(raw) as ProbeProject[]
  logFrontend('info', `[probe] Listed ${list.length} project(s)`)
  return list
}

async function fetchProbeMetrics(): Promise<ProbeMetricsResponse> {
  const baseUrl = getExternalUrl()
  await logFrontend('info', `[probe] Fetching metrics (url: ${baseUrl})`)

  // Find the project name by matching the current beads path against registered projects
  const currentPath = typeof window !== 'undefined' ? localStorage.getItem('beads:path') || '.' : '.'
  let projectPath: string
  try { projectPath = JSON.parse(currentPath) || '.' } catch { projectPath = '.' }

  const projects = await listProbeProjects(baseUrl)
  const match = matchProbeProject(projects, projectPath)
  if (!match) {
    await logFrontend('warn', `[probe] Current project not registered: ${projectPath}`)
    throw new Error('Current project is not registered with the probe')
  }

  await logFrontend('info', `[probe] Fetching metrics for project: ${match.name}`)
  const raw = await fetchExternalData(`${baseUrl}/metrics/${encodeURIComponent(match.name)}`)
  const metrics = JSON.parse(raw) as ProbeMetricsResponse
  await logFrontend('info', `[probe] Metrics received: ${metrics.counts.total} issues (open: ${metrics.counts.open}, in_progress: ${metrics.counts.in_progress}, closed: ${metrics.counts.closed})`)
  return metrics
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

// Dolt migration result
export interface MigrateResult {
  success: boolean
  message: string
}

// Migration status check result
export interface MigrationStatus {
  needsMigration: boolean
  reason: string
}

// Check if error indicates Dolt migration is needed (bd >= 0.50 with SQLite project)
export function isDoltMigrationError(error: unknown): boolean {
  const msg = error instanceof Error ? error.message : String(error)
  return msg.includes('Dolt backend configured but database not found')
}

// Proactively check if a project needs Dolt migration
// (detects partial migrations and SQLite projects with bd >= 0.50)
export async function bdCheckNeedsMigration(path?: string): Promise<MigrationStatus> {
  if (!isTauri()) {
    return { needsMigration: false, reason: 'web mode' }
  }
  return invoke<MigrationStatus>('bd_check_needs_migration', { cwd: path })
}

// Migrate a project from SQLite to Dolt backend
export async function bdMigrateToDolt(path?: string): Promise<MigrateResult> {
  if (!isTauri()) {
    throw new Error('Database migration is only available in the desktop app')
  }
  return invoke<MigrateResult>('bd_migrate_to_dolt', { cwd: path })
}

// Attachment refs migration v3 (filesystem-only)
export interface RefsMigrationStatus {
  needsMigration: boolean
  refCount: number
  justMigrated: boolean
}

export interface MigrateRefsResult {
  success: boolean
  refsUpdated: number
}

export async function bdCheckRefsMigration(path?: string): Promise<RefsMigrationStatus> {
  if (!isTauri()) {
    return { needsMigration: false, refCount: 0, justMigrated: false }
  }
  return invoke<RefsMigrationStatus>('check_refs_migration', { cwd: path })
}

export async function bdMigrateRefs(path?: string): Promise<MigrateRefsResult> {
  if (!isTauri()) {
    throw new Error('Attachment migration is only available in the desktop app')
  }
  return invoke<MigrateRefsResult>('migrate_attachment_refs', { cwd: path })
}

// Remove stale Dolt lock files that block database access (left by crashed processes)
export async function bdCleanupStaleLocks(path?: string): Promise<{ removed: string[] }> {
  if (!isTauri()) {
    return { removed: [] }
  }
  return invoke<{ removed: string[] }>('bd_cleanup_stale_locks', { cwd: path })
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

export async function bdSearch(query: string, path?: string): Promise<Issue[]> {
  if (isTauri()) {
    return invoke<Issue[]>('bd_search', { query, options: { cwd: path } })
  }
  // Web: no search support — client-side filtering handles it
  return []
}

export async function bdLabelAdd(id: string, label: string, path?: string): Promise<void> {
  if (isTauri()) {
    return invoke<void>('bd_label_add', { id, label, options: { cwd: path } })
  }
  throw new Error('Label operations are only available in the desktop app')
}

export async function bdLabelRemove(id: string, label: string, path?: string): Promise<void> {
  if (isTauri()) {
    return invoke<void>('bd_label_remove', { id, label, options: { cwd: path } })
  }
  throw new Error('Label operations are only available in the desktop app')
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
  usesDoltBackend: boolean
  /** bd >= 0.55: --all flag works correctly for bd list */
  supportsListAllFlag: boolean
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
    usesDoltBackend: false,
    supportsListAllFlag: false,
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
// Backend Mode API
// ============================================================================

export async function getBackendMode(): Promise<string> {
  if (isTauri()) return invoke<string>('get_backend_mode')
  return 'br'
}

export async function setBackendMode(mode: string): Promise<void> {
  if (!isTauri()) throw new Error('Backend mode is only available in the desktop app')
  return invoke<void>('set_backend_mode', { mode })
}

export async function trackerDetect(cwd?: string): Promise<boolean> {
  if (isTauri()) return invoke<boolean>('tracker_detect', { cwd: cwd || null })
  return false
}

export async function trackerInit(cwd?: string): Promise<void> {
  if (!isTauri()) throw new Error('Tracker init is only available in the desktop app')
  return invoke<void>('tracker_init', { cwd: cwd || null })
}


// ============================================================================
// File System API - For folder picker
// ============================================================================

export interface DirectoryEntry {
  name: string
  path: string
  isDirectory: boolean
  hasBeads: boolean
  usesDolt: boolean
}

export interface FsListResult {
  currentPath: string
  hasBeads: boolean
  usesDolt: boolean
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

import { ref, watch, triggerRef } from 'vue'
import { hashPath } from '~/utils/hash'

// Store settings metadata for reloading on project change
interface SettingMeta<T = unknown> {
  setting: string
  defaultValue: T
  valueRef: Ref<T>
}

// Global registry of all project-scoped settings
const settingsRegistry = new Map<string, SettingMeta>()

// Current project hash for change detection
let currentProjectHash: string | null = null

// Old global keys to clean up (v1 -> v2 migration)
const OLD_KEYS_TO_CLEANUP = [
  'beads:filters',
  'beads:exclusionFilters',
  'beads:columns',
  'beads:expandedEpics',
  'beads:chartsCollapsed',
  'beads:readyCollapsed',
  'beads:collapsed',
]

/**
 * Run one-time cleanup of old global localStorage keys.
 * This removes v1 keys on first use of v2 storage.
 */
function runCleanupIfNeeded() {
  if (!import.meta.client) return

  const cleanupFlag = localStorage.getItem('beads:v2-cleanup')
  if (cleanupFlag) return

  // Remove old global keys
  for (const key of OLD_KEYS_TO_CLEANUP) {
    localStorage.removeItem(key)
  }

  // Set flag so we don't run cleanup again
  localStorage.setItem('beads:v2-cleanup', 'true')
}

/**
 * Get the current beadsPath for namespacing.
 * Returns '.' as default if not available.
 */
function getBeadsPath(): string {
  if (!import.meta.client) return '.'

  const stored = localStorage.getItem('beads:path')
  if (stored) {
    try {
      return JSON.parse(stored) || '.'
    } catch {
      return '.'
    }
  }
  return '.'
}

/**
 * Get the full localStorage key for a setting.
 */
function getFullKey(setting: string, projectHash: string): string {
  return `beads:proj:${projectHash}:${setting}`
}

/**
 * Load a value from localStorage for the current project.
 */
function loadValue<T>(setting: string, defaultValue: T): T {
  if (!import.meta.client) return defaultValue

  const projectPath = getBeadsPath()
  const projectHash = hashPath(projectPath)
  const fullKey = getFullKey(setting, projectHash)
  const stored = localStorage.getItem(fullKey)

  if (stored) {
    try {
      return JSON.parse(stored)
    } catch {
      return defaultValue
    }
  }
  return defaultValue
}

/**
 * Save a value to localStorage for the current project.
 */
export function saveProjectValue<T>(setting: string, value: T): void {
  if (!import.meta.client) return

  const projectPath = getBeadsPath()
  const projectHash = hashPath(projectPath)
  const fullKey = getFullKey(setting, projectHash)
  localStorage.setItem(fullKey, JSON.stringify(value))
}

/**
 * Reload all registered settings from the new project's localStorage.
 * Call this when switching projects.
 */
export function reloadProjectStorage(): void {
  if (!import.meta.client) return

  const projectPath = getBeadsPath()
  const newProjectHash = hashPath(projectPath)

  // Skip if same project
  if (currentProjectHash === newProjectHash) {
    return
  }

  currentProjectHash = newProjectHash

  // Reload each registered setting from the new project's storage
  for (const [, meta] of settingsRegistry) {
    const newValue = loadValue(meta.setting, meta.defaultValue)
    meta.valueRef.value = newValue
    // Force reactivity trigger for computed refs that depend on this
    triggerRef(meta.valueRef as Ref<unknown>)
  }
}

/**
 * Clear the project storage cache (for backwards compatibility).
 * Now calls reloadProjectStorage internally.
 */
export function clearProjectStorageCache(): void {
  // Force hash reset so reloadProjectStorage will run
  currentProjectHash = null
  reloadProjectStorage()
}

/**
 * Project-scoped localStorage composable.
 * Keys are namespaced using a hash of the project path.
 *
 * Format: beads:proj:{hash}:{setting}
 *
 * @param setting - The setting name (e.g., 'filters', 'columns')
 * @param defaultValue - Default value if no stored value exists
 */
export function useProjectStorage<T>(setting: string, defaultValue: T): Ref<T> {
  // Run cleanup on first use
  runCleanupIfNeeded()

  // Initialize project hash if not set
  if (currentProjectHash === null) {
    currentProjectHash = hashPath(getBeadsPath())
  }

  // Return existing ref if already registered
  if (settingsRegistry.has(setting)) {
    return settingsRegistry.get(setting)!.valueRef as Ref<T>
  }

  // Create new ref with loaded value
  const initialValue = loadValue(setting, defaultValue)
  const valueRef = ref<T>(initialValue) as Ref<T>

  // Register for reloading on project change
  settingsRegistry.set(setting, {
    setting,
    defaultValue,
    valueRef: valueRef as Ref<unknown>,
  })

  // Watch for changes and persist to localStorage (using current project)
  watch(
    valueRef,
    (newValue) => {
      saveProjectValue(setting, newValue)
    },
    { deep: true }
  )

  return valueRef
}

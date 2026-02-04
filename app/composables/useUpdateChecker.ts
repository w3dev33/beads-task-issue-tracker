import type { UpdateInfo } from '~/utils/bd-api'
import { checkForUpdates } from '~/utils/bd-api'
import { openUrl } from '~/utils/open-url'

// Global state (shared across components)
const updateInfo = ref<UpdateInfo | null>(null)
const isChecking = ref(false)
const error = ref<string | null>(null)

// Internal state for periodic check interval
let checkInterval: ReturnType<typeof setInterval> | null = null

export function useUpdateChecker() {
  const check = async () => {
    if (isChecking.value) return

    isChecking.value = true
    error.value = null

    try {
      updateInfo.value = await checkForUpdates()
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to check for updates'
      console.error('Update check failed:', e)
    } finally {
      isChecking.value = false
    }
  }

  const startPeriodicCheck = (intervalMs: number = 3600000) => {
    // Don't start if already running
    if (checkInterval) return

    checkInterval = setInterval(() => {
      check()
    }, intervalMs)
  }

  const stopPeriodicCheck = () => {
    if (checkInterval) {
      clearInterval(checkInterval)
      checkInterval = null
    }
  }

  const openReleasesPage = async () => {
    const url = updateInfo.value?.releaseUrl || 'https://github.com/w3dev33/beads-task-issue-tracker/releases'
    await openUrl(url)
  }

  return {
    updateInfo,
    isChecking,
    error,
    check,
    startPeriodicCheck,
    stopPeriodicCheck,
    openReleasesPage,
  }
}

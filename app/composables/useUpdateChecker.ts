import type { UpdateInfo } from '~/utils/bd-api'
import { checkForUpdates, checkForUpdatesDemo, downloadAndInstallUpdate, logFrontend } from '~/utils/bd-api'
import { openUrl } from '~/utils/open-url'

// Global state (shared across components)
const updateInfo = ref<UpdateInfo | null>(null)
const isChecking = ref(false)
const isDownloading = ref(false)
const downloadError = ref<string | null>(null)
const error = ref<string | null>(null)
const demoMode = ref(false)

// Internal state for periodic check interval
let checkInterval: ReturnType<typeof setInterval> | null = null

export function useUpdateChecker() {
  const check = async () => {
    if (isChecking.value) return

    isChecking.value = true
    error.value = null

    try {
      updateInfo.value = demoMode.value
        ? await checkForUpdatesDemo()
        : await checkForUpdates()
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to check for updates'
      console.error('Update check failed:', e)
    } finally {
      isChecking.value = false
    }
  }

  const toggleDemo = () => {
    demoMode.value = !demoMode.value
    updateInfo.value = null
    error.value = null
    downloadError.value = null
    check()
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

  const downloadAndQuit = async () => {
    isDownloading.value = true
    downloadError.value = null

    try {
      const info = updateInfo.value
      if (info?.platform === 'macos' && info.downloadUrl) {
        logFrontend('info', `[update] Downloading update from: ${info.downloadUrl}`)
        const savedPath = await downloadAndInstallUpdate(info.downloadUrl)
        logFrontend('info', `[update] Download complete: ${savedPath}`)
      } else {
        logFrontend('info', '[update] No direct download available, opening releases page')
        await openReleasesPage()
      }
    } catch (e) {
      const msg = e instanceof Error ? e.message : (typeof e === 'string' ? e : 'Failed to download update')
      downloadError.value = msg
      logFrontend('error', `[update] Download failed: ${msg}`)
      console.error('Download failed:', e)
      isDownloading.value = false
      return
    }

    // Quit the app (separate try so download errors don't get masked)
    try {
      logFrontend('info', '[update] Closing app after successful download')
      const { getCurrentWindow } = await import('@tauri-apps/api/window')
      await getCurrentWindow().close()
    } catch (e) {
      // Close failed — download was still successful, don't show error
      logFrontend('warn', `[update] App close failed (download was successful): ${e}`)
      console.warn('App close failed after successful download:', e)
      isDownloading.value = false
    }
  }

  return {
    updateInfo,
    isChecking,
    isDownloading,
    downloadError,
    error,
    demoMode,
    check,
    toggleDemo,
    startPeriodicCheck,
    stopPeriodicCheck,
    openReleasesPage,
    downloadAndQuit,
  }
}

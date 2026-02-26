import { bdSync, logFrontend } from '~/utils/bd-api'

// Shared state across all components
const lastSyncError = ref<string | null>(null)
const lastSyncSuccess = ref(false)
const isSyncing = ref(false)
const syncMessage = ref<string | null>(null)
const showErrorDialog = ref(false)

// Auto-clear success message after delay
let messageTimeout: ReturnType<typeof setTimeout> | null = null

function showSuccessMessage() {
  syncMessage.value = 'Sync OK'
  if (messageTimeout) clearTimeout(messageTimeout)
  messageTimeout = setTimeout(() => {
    syncMessage.value = null
  }, 3000)
}

export function useSyncStatus() {
  const { beadsPath } = useBeadsPath()
  const { refreshConflicts } = useConflicts()

  const forceSync = async () => {
    if (isSyncing.value) return

    isSyncing.value = true
    lastSyncError.value = null
    lastSyncSuccess.value = false
    syncMessage.value = null

    try {
      await bdSync(beadsPath.value || undefined)
      lastSyncSuccess.value = true
      showSuccessMessage()
      // Check for conflicts after sync
      await refreshConflicts()
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e)
      lastSyncError.value = message
      showErrorDialog.value = true
    } finally {
      isSyncing.value = false
    }
  }

  /** Check for conflicts without triggering a full sync (e.g., after poll). */
  const checkConflicts = async () => {
    try {
      await refreshConflicts()
    } catch (e) {
      logFrontend('warn', `[sync] Failed to check conflicts: ${e}`)
    }
  }

  const closeErrorDialog = () => {
    showErrorDialog.value = false
  }

  return {
    lastSyncError: readonly(lastSyncError),
    lastSyncSuccess: readonly(lastSyncSuccess),
    isSyncing: readonly(isSyncing),
    syncMessage: readonly(syncMessage),
    showErrorDialog,
    forceSync,
    checkConflicts,
    closeErrorDialog,
  }
}

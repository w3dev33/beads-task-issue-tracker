import { bdSync } from '~/utils/bd-api'

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
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e)
      lastSyncError.value = message
      showErrorDialog.value = true
    } finally {
      isSyncing.value = false
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
    closeErrorDialog,
  }
}

import { trackerGetConflicts, trackerResolveConflict, trackerDismissConflict, logFrontend } from '~/utils/bd-api'
import type { ConflictRecord } from '~/utils/bd-api'

// Shared state across all components
const conflicts = ref<ConflictRecord[]>([])
const isDialogOpen = ref(false)
const isResolving = ref(false)
const activeConflictIndex = ref(0)

export function useConflicts() {
  const { beadsPath } = useBeadsPath()

  const conflictCount = computed(() => conflicts.value.length)
  const hasConflicts = computed(() => conflicts.value.length > 0)

  const activeConflict = computed(() => {
    if (activeConflictIndex.value >= conflicts.value.length) return null
    return conflicts.value[activeConflictIndex.value] ?? null
  })

  const parsedLocal = computed(() => {
    if (!activeConflict.value) return null
    try { return JSON.parse(activeConflict.value.local_json) } catch { return null }
  })

  const parsedRemote = computed(() => {
    if (!activeConflict.value) return null
    try { return JSON.parse(activeConflict.value.remote_json) } catch { return null }
  })

  /** Fields that differ between local and remote versions. */
  const diffFields = computed(() => {
    if (!parsedLocal.value || !parsedRemote.value) return []
    const fields = ['title', 'description', 'status', 'priority', 'issue_type', 'owner', 'assignee', 'closed_at', 'notes', 'design', 'acceptance_criteria', 'labels', 'parent', 'estimate_minutes', 'external_ref']
    return fields.filter((f) => {
      const l = JSON.stringify(parsedLocal.value[f] ?? null)
      const r = JSON.stringify(parsedRemote.value[f] ?? null)
      return l !== r
    })
  })

  async function refreshConflicts() {
    try {
      conflicts.value = await trackerGetConflicts(beadsPath.value || undefined)
    } catch (e) {
      logFrontend('warn', `[conflicts] Failed to load conflicts: ${e}`)
      conflicts.value = []
    }
  }

  function openDialog() {
    activeConflictIndex.value = 0
    isDialogOpen.value = true
  }

  async function resolve(resolution: 'local' | 'remote') {
    const conflict = activeConflict.value
    if (!conflict) return

    isResolving.value = true
    try {
      await trackerResolveConflict(conflict.id, resolution, beadsPath.value || undefined)
      logFrontend('info', `[conflicts] Resolved conflict ${conflict.id} with "${resolution}"`)
      await refreshConflicts()
      // Move to next conflict or close if none left
      if (activeConflictIndex.value >= conflicts.value.length) {
        activeConflictIndex.value = Math.max(0, conflicts.value.length - 1)
      }
      if (conflicts.value.length === 0) {
        isDialogOpen.value = false
      }
    } catch (e) {
      logFrontend('error', `[conflicts] Failed to resolve conflict: ${e}`)
    } finally {
      isResolving.value = false
    }
  }

  async function dismiss() {
    const conflict = activeConflict.value
    if (!conflict) return

    isResolving.value = true
    try {
      await trackerDismissConflict(conflict.id, beadsPath.value || undefined)
      logFrontend('info', `[conflicts] Dismissed conflict ${conflict.id}`)
      await refreshConflicts()
      if (activeConflictIndex.value >= conflicts.value.length) {
        activeConflictIndex.value = Math.max(0, conflicts.value.length - 1)
      }
      if (conflicts.value.length === 0) {
        isDialogOpen.value = false
      }
    } catch (e) {
      logFrontend('error', `[conflicts] Failed to dismiss conflict: ${e}`)
    } finally {
      isResolving.value = false
    }
  }

  return {
    conflicts: readonly(conflicts),
    conflictCount,
    hasConflicts,
    isDialogOpen,
    isResolving: readonly(isResolving),
    activeConflictIndex,
    activeConflict,
    parsedLocal,
    parsedRemote,
    diffFields,
    refreshConflicts,
    openDialog,
    resolve,
    dismiss,
  }
}

import { bdRepairDatabase, isSchemaMigrationError, type RepairResult } from '~/utils/bd-api'

// Shared state
const needsRepair = ref(false)
const affectedProject = ref<string | null>(null)
const isRepairing = ref(false)
const repairError = ref<string | null>(null)
const lastRepairResult = ref<RepairResult | null>(null)
const repairProgress = ref<{ current: number; total: number; currentPath: string } | null>(null)

export function useRepairDatabase() {
  const { beadsPath } = useBeadsPath()

  // Check if an error indicates a repair is needed
  const checkError = (error: unknown, projectPath?: string): boolean => {
    if (isSchemaMigrationError(error)) {
      needsRepair.value = true
      affectedProject.value = projectPath || beadsPath.value
      return true
    }
    return false
  }

  // Perform the repair on current project
  const repair = async (projectPath?: string): Promise<boolean> => {
    const path = projectPath || beadsPath.value
    isRepairing.value = true
    repairError.value = null
    lastRepairResult.value = null

    try {
      const result = await bdRepairDatabase(path)
      lastRepairResult.value = result

      if (result.success) {
        needsRepair.value = false
        affectedProject.value = null
        return true
      } else {
        repairError.value = result.message
        return false
      }
    } catch (e) {
      repairError.value = e instanceof Error ? e.message : 'Repair failed'
      return false
    } finally {
      isRepairing.value = false
    }
  }

  // Repair all favorite projects
  const repairAll = async (favoritePaths: string[]): Promise<{ success: number; failed: number; errors: string[] }> => {
    isRepairing.value = true
    repairError.value = null
    const results = { success: 0, failed: 0, errors: [] as string[] }

    repairProgress.value = { current: 0, total: favoritePaths.length, currentPath: '' }

    for (let i = 0; i < favoritePaths.length; i++) {
      const path = favoritePaths[i]
      repairProgress.value = { current: i + 1, total: favoritePaths.length, currentPath: path }

      try {
        const result = await bdRepairDatabase(path)
        if (result.success) {
          results.success++
        } else {
          results.failed++
          results.errors.push(`${path}: ${result.message}`)
        }
      } catch (e) {
        results.failed++
        const errorMsg = e instanceof Error ? e.message : 'Unknown error'
        // Skip "No database to repair" - not an error
        if (!errorMsg.includes('No database to repair')) {
          results.errors.push(`${path}: ${errorMsg}`)
        } else {
          results.success++ // Count as success if no repair needed
        }
      }
    }

    repairProgress.value = null
    isRepairing.value = false

    if (results.failed === 0) {
      needsRepair.value = false
      affectedProject.value = null
    }

    return results
  }

  // Dismiss the repair dialog without repairing
  const dismiss = () => {
    needsRepair.value = false
    affectedProject.value = null
  }

  return {
    needsRepair: readonly(needsRepair),
    affectedProject: readonly(affectedProject),
    isRepairing: readonly(isRepairing),
    repairError: readonly(repairError),
    lastRepairResult: readonly(lastRepairResult),
    repairProgress: readonly(repairProgress),
    checkError,
    repair,
    repairAll,
    dismiss,
  }
}

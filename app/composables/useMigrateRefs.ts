import { bdCheckRefsMigration, bdMigrateRefs } from '~/utils/bd-api'

// Shared state (module-level so it persists across component re-renders)
const needsMigration = ref(false)
const refCount = ref(0)
const isMigrating = ref(false)
const migrateError = ref<string | null>(null)

export function useMigrateRefs() {
  const { beadsPath } = useBeadsPath()

  // Check if the current project needs attachment refs migration
  // Also returns true if auto-migration just ran (to trigger notification)
  const checkProject = async (projectPath?: string): Promise<boolean | 'just_migrated'> => {
    const path = projectPath || beadsPath.value
    try {
      const status = await bdCheckRefsMigration(path)
      if (status.justMigrated) {
        return 'just_migrated'
      }
      if (status.needsMigration) {
        needsMigration.value = true
        refCount.value = status.refCount
        migrateError.value = null
        return true
      }
    } catch {
      // Silently fail — don't block the app
    }
    return false
  }

  // Perform the migration
  const migrate = async (projectPath?: string): Promise<boolean> => {
    const path = projectPath || beadsPath.value
    isMigrating.value = true
    migrateError.value = null

    try {
      const result = await bdMigrateRefs(path)

      if (result.success) {
        needsMigration.value = false
        refCount.value = 0
        return true
      } else {
        migrateError.value = 'Migration failed'
        return false
      }
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e)
      console.error('[useMigrateRefs] migrate error:', msg, e)
      migrateError.value = msg || 'Migration failed'
      return false
    } finally {
      isMigrating.value = false
    }
  }

  // Dismiss the migration dialog without migrating
  const dismiss = () => {
    needsMigration.value = false
    refCount.value = 0
    migrateError.value = null
  }

  return {
    needsMigration: readonly(needsMigration),
    refCount: readonly(refCount),
    isMigrating: readonly(isMigrating),
    migrateError: readonly(migrateError),
    checkProject,
    migrate,
    dismiss,
  }
}

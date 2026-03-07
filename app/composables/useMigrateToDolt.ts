import { bdMigrateToDolt, bdCheckNeedsMigration, isDoltMigrationError } from '~/utils/bd-api'

// Shared state
const needsMigration = ref(false)
const affectedProject = ref<string | null>(null)
const isMigrating = ref(false)
const migrateError = ref<string | null>(null)

export function useMigrateToDolt() {
  const { beadsPath } = useBeadsPath()

  // Check if an error indicates a Dolt migration is needed
  const checkError = (error: unknown, projectPath?: string): boolean => {
    if (isDoltMigrationError(error)) {
      needsMigration.value = true
      affectedProject.value = projectPath || beadsPath.value
      migrateError.value = null // Reset previous error when opening for a new project
      return true
    }
    return false
  }

  // Proactively check if the current project needs migration
  // (detects partial migrations and SQLite projects with bd >= 0.50)
  const checkProject = async (projectPath?: string): Promise<boolean> => {
    const path = projectPath || beadsPath.value
    try {
      const status = await bdCheckNeedsMigration(path)
      if (status.needsMigration) {
        needsMigration.value = true
        affectedProject.value = path
        migrateError.value = null // Reset previous error when opening for a new project
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
      const result = await bdMigrateToDolt(path)

      if (result.success) {
        needsMigration.value = false
        affectedProject.value = null
        return true
      } else {
        migrateError.value = result.message
        return false
      }
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e)
      console.error('[useMigrateToDolt] migrate error:', msg, e)
      migrateError.value = msg || 'Migration failed'
      return false
    } finally {
      isMigrating.value = false
    }
  }

  // Dismiss the migration dialog without migrating
  const dismiss = () => {
    needsMigration.value = false
    affectedProject.value = null
    migrateError.value = null
  }

  return {
    needsMigration: readonly(needsMigration),
    affectedProject: readonly(affectedProject),
    isMigrating: readonly(isMigrating),
    migrateError: readonly(migrateError),
    checkError,
    checkProject,
    migrate,
    dismiss,
  }
}

import { fsExists } from '~/utils/bd-api'
import { getFolderName } from '~/utils/path'
import { useNotification } from '~/composables/useNotification'
import type { Project, ProjectSortMode } from '~/utils/favorites-helpers'

// Retry once after 500ms to handle transient startup failures (Tauri backend not ready)
async function fsExistsWithRetry(path: string): Promise<boolean> {
  const exists = await fsExists(path)
  if (exists) return true
  await new Promise(r => setTimeout(r, 500))
  return fsExists(path)
}

// Re-export for backward compatibility
export type { Project as Favorite, Project, ProjectSortMode as FavoritesSortMode, ProjectSortMode }

// Normalize path by stripping trailing slashes for consistent comparison
function normalizePath(p: string): string {
  return p.replace(/\/+$/, '')
}

// Shared state across all components
const projects = ref<Project[]>([])
const sortMode = ref<ProjectSortMode>('alpha')
const hasReordered = ref(false)
let isInitialized = false
let isValidating = false
let watcherRegistered = false

function initSortModeFromStorage() {
  if (import.meta.client) {
    const stored = localStorage.getItem('beads:favoritesSortMode')
    if (stored === 'alpha' || stored === 'alpha-desc' || stored === 'manual') {
      sortMode.value = stored
    }
  }
}

function initFromStorage() {
  if (import.meta.client && !isInitialized) {
    // Read from existing localStorage key (kept for backward compatibility)
    const stored = localStorage.getItem('beads:favorites')
    if (stored) {
      try {
        const parsed = JSON.parse(stored) as Project[]
        // Deduplicate by normalized path (keep first occurrence)
        const seen = new Set<string>()
        const deduped = parsed.filter((proj) => {
          const key = normalizePath(proj.path)
          if (seen.has(key)) return false
          seen.add(key)
          return true
        })
        projects.value = deduped

        // Validate project paths exist asynchronously
        if (parsed.length > 0 && !isValidating) {
          isValidating = true
          Promise.all(
            parsed.map(async (proj) => ({
              ...proj,
              exists: await fsExistsWithRetry(proj.path),
            }))
          ).then((results) => {
            const validProjects = results.filter((f) => f.exists)
            const invalidCount = results.length - validProjects.length

            if (invalidCount > 0) {
              const invalidNames = results
                .filter(f => !f.exists)
                .map(f => f.name)
                .join(', ')
              const { warning } = useNotification()
              warning(
                `${invalidCount} projet${invalidCount > 1 ? 's' : ''} supprimé${invalidCount > 1 ? 's' : ''}`,
                `Chemin${invalidCount > 1 ? 's' : ''} inaccessible${invalidCount > 1 ? 's' : ''} : ${invalidNames}`
              )
              // Remove invalid projects
              projects.value = validProjects.map(({ exists, ...proj }) => proj)
              localStorage.setItem('beads:favorites', JSON.stringify(projects.value))
              console.warn(`[useProjects] Removed ${invalidCount} projects with invalid paths`)
            }
            isValidating = false
          }).catch(() => {
            isValidating = false
          })
        }
      } catch {
        projects.value = []
      }
    }
    initSortModeFromStorage()
    isInitialized = true
  }
}

export function useProjects() {
  // Initialize from localStorage
  initFromStorage()

  // Watch for changes and persist to localStorage (register only once)
  if (import.meta.client && !watcherRegistered) {
    watcherRegistered = true
    watch(
      projects,
      (newValue) => {
        localStorage.setItem('beads:favorites', JSON.stringify(newValue))
      },
      { deep: true }
    )
  }

  const sortedProjects = computed<Project[]>(() => {
    if (sortMode.value === 'alpha') {
      return [...projects.value].sort((a, b) => a.name.localeCompare(b.name))
    }
    if (sortMode.value === 'alpha-desc') {
      return [...projects.value].sort((a, b) => b.name.localeCompare(a.name))
    }
    return projects.value
  })

  const addProject = (path: string, name?: string) => {
    const normalized = normalizePath(path)
    // Don't add duplicates (compare normalized paths)
    if (projects.value.some((f) => normalizePath(f.path) === normalized)) {
      return false
    }

    // Extract folder name from path if no name provided
    const folderName = name || getFolderName(path)

    // Use array reassignment instead of push() for guaranteed reactivity
    projects.value = [...projects.value, {
      path: normalized,
      name: folderName,
      addedAt: new Date().toISOString(),
    }]
    return true
  }

  const removeProject = (path: string) => {
    const normalized = normalizePath(path)
    const index = projects.value.findIndex((f) => normalizePath(f.path) === normalized)
    if (index !== -1) {
      // Use array reassignment (filter) instead of splice for guaranteed reactivity
      projects.value = projects.value.filter((_, i) => i !== index)
      return true
    }
    return false
  }

  const isProject = (path: string) => {
    const normalized = normalizePath(path)
    return projects.value.some((f) => normalizePath(f.path) === normalized)
  }

  const renameProject = (path: string, newName: string) => {
    const project = projects.value.find((f) => f.path === path)
    if (project) {
      project.name = newName
      return true
    }
    return false
  }

  const reorderProjects = (newOrder: Project[]) => {
    projects.value = newOrder
    sortMode.value = 'manual'
    hasReordered.value = true
    localStorage.setItem('beads:favoritesSortMode', 'manual')
  }

  const setSortMode = (mode: ProjectSortMode) => {
    sortMode.value = mode
    localStorage.setItem('beads:favoritesSortMode', mode)
  }

  const resetSortOrder = () => {
    sortMode.value = 'alpha'
    hasReordered.value = false
    localStorage.setItem('beads:favoritesSortMode', 'alpha')
  }

  return {
    // New names
    projects: readonly(projects),
    sortedProjects,
    sortMode: readonly(sortMode),
    hasReordered: readonly(hasReordered),
    addProject,
    removeProject,
    isProject,
    renameProject,
    reorderProjects,
    setSortMode,
    resetSortOrder,
    // Backward-compatible aliases
    favorites: readonly(projects),
    sortedFavorites: sortedProjects,
    addFavorite: addProject,
    removeFavorite: removeProject,
    isFavorite: isProject,
    renameFavorite: renameProject,
    reorderFavorites: reorderProjects,
  }
}

/** @deprecated Use useProjects instead */
export const useFavorites = useProjects

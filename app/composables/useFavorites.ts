import { fsExists } from '~/utils/bd-api'
import { getFolderName } from '~/utils/path'
import { useNotification } from '~/composables/useNotification'

// Retry once after 500ms to handle transient startup failures (Tauri backend not ready)
async function fsExistsWithRetry(path: string): Promise<boolean> {
  const exists = await fsExists(path)
  if (exists) return true
  await new Promise(r => setTimeout(r, 500))
  return fsExists(path)
}

export type FavoritesSortMode = 'alpha' | 'alpha-desc' | 'manual'

export interface Favorite {
  path: string
  name: string
  addedAt: string
}

// Normalize path by stripping trailing slashes for consistent comparison
function normalizePath(p: string): string {
  return p.replace(/\/+$/, '')
}

// Shared state across all components
const favorites = ref<Favorite[]>([])
const sortMode = ref<FavoritesSortMode>('alpha')
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
    const stored = localStorage.getItem('beads:favorites')
    if (stored) {
      try {
        const parsedFavorites = JSON.parse(stored) as Favorite[]
        // Deduplicate by normalized path (keep first occurrence)
        const seen = new Set<string>()
        const deduped = parsedFavorites.filter((fav) => {
          const key = normalizePath(fav.path)
          if (seen.has(key)) return false
          seen.add(key)
          return true
        })
        favorites.value = deduped

        // Validate favorites paths exist asynchronously
        if (parsedFavorites.length > 0 && !isValidating) {
          isValidating = true
          Promise.all(
            parsedFavorites.map(async (fav) => ({
              ...fav,
              exists: await fsExistsWithRetry(fav.path),
            }))
          ).then((results) => {
            const validFavorites = results.filter((f) => f.exists)
            const invalidCount = results.length - validFavorites.length

            if (invalidCount > 0) {
              const invalidNames = results
                .filter(f => !f.exists)
                .map(f => f.name)
                .join(', ')
              const { warning } = useNotification()
              warning(
                `${invalidCount} favori${invalidCount > 1 ? 's' : ''} supprimé${invalidCount > 1 ? 's' : ''}`,
                `Chemin${invalidCount > 1 ? 's' : ''} inaccessible${invalidCount > 1 ? 's' : ''} : ${invalidNames}`
              )
              // Remove invalid favorites
              favorites.value = validFavorites.map(({ exists, ...fav }) => fav)
              localStorage.setItem('beads:favorites', JSON.stringify(favorites.value))
              console.warn(`[useFavorites] Removed ${invalidCount} favorites with invalid paths`)
            }
            isValidating = false
          }).catch(() => {
            isValidating = false
          })
        }
      } catch {
        favorites.value = []
      }
    }
    initSortModeFromStorage()
    isInitialized = true
  }
}

export function useFavorites() {
  // Initialize from localStorage
  initFromStorage()

  // Watch for changes and persist to localStorage (register only once)
  if (import.meta.client && !watcherRegistered) {
    watcherRegistered = true
    watch(
      favorites,
      (newValue) => {
        localStorage.setItem('beads:favorites', JSON.stringify(newValue))
      },
      { deep: true }
    )
  }

  const sortedFavorites = computed<Favorite[]>(() => {
    if (sortMode.value === 'alpha') {
      return [...favorites.value].sort((a, b) => a.name.localeCompare(b.name))
    }
    if (sortMode.value === 'alpha-desc') {
      return [...favorites.value].sort((a, b) => b.name.localeCompare(a.name))
    }
    return favorites.value
  })

  const addFavorite = (path: string, name?: string) => {
    const normalized = normalizePath(path)
    // Don't add duplicates (compare normalized paths)
    if (favorites.value.some((f) => normalizePath(f.path) === normalized)) {
      return false
    }

    // Extract folder name from path if no name provided
    const folderName = name || getFolderName(path)

    // Use array reassignment instead of push() for guaranteed reactivity
    favorites.value = [...favorites.value, {
      path: normalized,
      name: folderName,
      addedAt: new Date().toISOString(),
    }]
    return true
  }

  const removeFavorite = (path: string) => {
    const normalized = normalizePath(path)
    const index = favorites.value.findIndex((f) => normalizePath(f.path) === normalized)
    if (index !== -1) {
      // Use array reassignment (filter) instead of splice for guaranteed reactivity
      favorites.value = favorites.value.filter((_, i) => i !== index)
      return true
    }
    return false
  }

  const isFavorite = (path: string) => {
    const normalized = normalizePath(path)
    return favorites.value.some((f) => normalizePath(f.path) === normalized)
  }

  const renameFavorite = (path: string, newName: string) => {
    const favorite = favorites.value.find((f) => f.path === path)
    if (favorite) {
      favorite.name = newName
      return true
    }
    return false
  }

  const reorderFavorites = (newOrder: Favorite[]) => {
    favorites.value = newOrder
    sortMode.value = 'manual'
    hasReordered.value = true
    localStorage.setItem('beads:favoritesSortMode', 'manual')
  }

  const setSortMode = (mode: FavoritesSortMode) => {
    sortMode.value = mode
    localStorage.setItem('beads:favoritesSortMode', mode)
  }

  const resetSortOrder = () => {
    sortMode.value = 'alpha'
    hasReordered.value = false
    localStorage.setItem('beads:favoritesSortMode', 'alpha')
  }

  return {
    favorites: readonly(favorites),
    sortedFavorites,
    sortMode: readonly(sortMode),
    hasReordered: readonly(hasReordered),
    addFavorite,
    removeFavorite,
    isFavorite,
    renameFavorite,
    reorderFavorites,
    setSortMode,
    resetSortOrder,
  }
}

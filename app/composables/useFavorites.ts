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

export interface Favorite {
  path: string
  name: string
  addedAt: string
}

// Shared state across all components
const favorites = ref<Favorite[]>([])
let isInitialized = false
let isValidating = false

function initFromStorage() {
  if (import.meta.client && !isInitialized) {
    const stored = localStorage.getItem('beads:favorites')
    if (stored) {
      try {
        const parsedFavorites = JSON.parse(stored) as Favorite[]
        favorites.value = parsedFavorites

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
    isInitialized = true
  }
}

export function useFavorites() {
  // Initialize from localStorage
  initFromStorage()

  // Watch for changes and persist to localStorage
  if (import.meta.client) {
    watch(
      favorites,
      (newValue) => {
        localStorage.setItem('beads:favorites', JSON.stringify(newValue))
      },
      { deep: true }
    )
  }

  const addFavorite = (path: string, name?: string) => {
    // Don't add duplicates
    if (favorites.value.some((f) => f.path === path)) {
      return false
    }

    // Extract folder name from path if no name provided
    const folderName = name || getFolderName(path)

    // Use array reassignment instead of push() for guaranteed reactivity
    favorites.value = [...favorites.value, {
      path,
      name: folderName,
      addedAt: new Date().toISOString(),
    }]
    return true
  }

  const removeFavorite = (path: string) => {
    const index = favorites.value.findIndex((f) => f.path === path)
    if (index !== -1) {
      favorites.value.splice(index, 1)
      return true
    }
    return false
  }

  const isFavorite = (path: string) => {
    return favorites.value.some((f) => f.path === path)
  }

  const renameFavorite = (path: string, newName: string) => {
    const favorite = favorites.value.find((f) => f.path === path)
    if (favorite) {
      favorite.name = newName
      return true
    }
    return false
  }

  return {
    favorites: readonly(favorites),
    addFavorite,
    removeFavorite,
    isFavorite,
    renameFavorite,
  }
}

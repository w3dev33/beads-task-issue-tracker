/**
 * Pure helper functions for favorites management.
 * Extracted from useFavorites composable for testability.
 */
import { getFolderName } from '~/utils/path'

export interface Favorite {
  path: string
  name: string
  addedAt: string
}

export type FavoritesSortMode = 'alpha' | 'alpha-desc' | 'manual'

/**
 * Normalize path by stripping trailing slashes for consistent comparison.
 */
export function normalizePath(p: string): string {
  return p.replace(/\/+$/, '')
}

/**
 * Deduplicate favorites by normalized path, keeping first occurrence.
 */
export function deduplicateFavorites(favorites: Favorite[]): Favorite[] {
  const seen = new Set<string>()
  return favorites.filter((fav) => {
    const key = normalizePath(fav.path)
    if (seen.has(key)) return false
    seen.add(key)
    return true
  })
}

/**
 * Sort favorites according to the given mode.
 */
export function sortFavorites(favorites: Favorite[], mode: FavoritesSortMode): Favorite[] {
  if (mode === 'alpha') {
    return [...favorites].sort((a, b) => a.name.localeCompare(b.name))
  }
  if (mode === 'alpha-desc') {
    return [...favorites].sort((a, b) => b.name.localeCompare(a.name))
  }
  return favorites
}

/**
 * Check if a path is already in the favorites list (normalized comparison).
 */
export function isFavorite(favorites: Favorite[], path: string): boolean {
  const normalized = normalizePath(path)
  return favorites.some((f) => normalizePath(f.path) === normalized)
}

/**
 * Create a new Favorite entry from a path and optional name.
 */
export function createFavoriteEntry(path: string, name?: string): Favorite {
  const normalized = normalizePath(path)
  return {
    path: normalized,
    name: name || getFolderName(path),
    addedAt: new Date().toISOString(),
  }
}

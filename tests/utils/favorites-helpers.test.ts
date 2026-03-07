import { describe, it, expect } from 'vitest'
import {
  normalizePath,
  deduplicateFavorites,
  sortFavorites,
  isFavorite,
  createFavoriteEntry,
  type Favorite,
} from '~/utils/favorites-helpers'

function makeFav(overrides: Partial<Favorite> = {}): Favorite {
  return {
    path: '/home/dev/project',
    name: 'project',
    addedAt: '2025-01-01T00:00:00Z',
    ...overrides,
  }
}

// ---------------------------------------------------------------------------
// normalizePath
// ---------------------------------------------------------------------------
describe('normalizePath', () => {
  it('strips trailing slashes', () => {
    expect(normalizePath('/home/dev/project/')).toBe('/home/dev/project')
  })

  it('strips multiple trailing slashes', () => {
    expect(normalizePath('/home/dev/project///')).toBe('/home/dev/project')
  })

  it('leaves paths without trailing slash unchanged', () => {
    expect(normalizePath('/home/dev/project')).toBe('/home/dev/project')
  })

  it('handles root path', () => {
    expect(normalizePath('/')).toBe('')
  })

  it('handles empty string', () => {
    expect(normalizePath('')).toBe('')
  })
})

// ---------------------------------------------------------------------------
// deduplicateFavorites
// ---------------------------------------------------------------------------
describe('deduplicateFavorites', () => {
  it('returns empty for empty input', () => {
    expect(deduplicateFavorites([])).toEqual([])
  })

  it('keeps unique favorites', () => {
    const favs = [makeFav({ path: '/a' }), makeFav({ path: '/b' })]
    expect(deduplicateFavorites(favs)).toHaveLength(2)
  })

  it('removes duplicates by normalized path', () => {
    const favs = [
      makeFav({ path: '/home/dev/project', name: 'first' }),
      makeFav({ path: '/home/dev/project/', name: 'second' }),
    ]
    const result = deduplicateFavorites(favs)
    expect(result).toHaveLength(1)
    expect(result[0]!.name).toBe('first')
  })

  it('keeps first occurrence of each duplicate', () => {
    const favs = [
      makeFav({ path: '/a', name: 'A1' }),
      makeFav({ path: '/b', name: 'B' }),
      makeFav({ path: '/a', name: 'A2' }),
    ]
    const result = deduplicateFavorites(favs)
    expect(result).toHaveLength(2)
    expect(result[0]!.name).toBe('A1')
  })
})

// ---------------------------------------------------------------------------
// sortFavorites
// ---------------------------------------------------------------------------
describe('sortFavorites', () => {
  const favs = [
    makeFav({ path: '/c', name: 'Charlie' }),
    makeFav({ path: '/a', name: 'Alpha' }),
    makeFav({ path: '/b', name: 'Bravo' }),
  ]

  it('sorts alphabetically ascending', () => {
    const result = sortFavorites(favs, 'alpha')
    expect(result.map(f => f.name)).toEqual(['Alpha', 'Bravo', 'Charlie'])
  })

  it('sorts alphabetically descending', () => {
    const result = sortFavorites(favs, 'alpha-desc')
    expect(result.map(f => f.name)).toEqual(['Charlie', 'Bravo', 'Alpha'])
  })

  it('returns as-is for manual mode', () => {
    const result = sortFavorites(favs, 'manual')
    expect(result.map(f => f.name)).toEqual(['Charlie', 'Alpha', 'Bravo'])
  })

  it('does not mutate the original array', () => {
    const copy = [...favs]
    sortFavorites(favs, 'alpha')
    expect(favs.map(f => f.name)).toEqual(copy.map(f => f.name))
  })

  it('handles empty array', () => {
    expect(sortFavorites([], 'alpha')).toEqual([])
  })
})

// ---------------------------------------------------------------------------
// isFavorite
// ---------------------------------------------------------------------------
describe('isFavorite', () => {
  const favs = [makeFav({ path: '/home/dev/project' }), makeFav({ path: '/other' })]

  it('returns true for existing path', () => {
    expect(isFavorite(favs, '/home/dev/project')).toBe(true)
  })

  it('returns true with trailing slash (normalized)', () => {
    expect(isFavorite(favs, '/home/dev/project/')).toBe(true)
  })

  it('returns false for unknown path', () => {
    expect(isFavorite(favs, '/not/a/favorite')).toBe(false)
  })

  it('returns false for empty list', () => {
    expect(isFavorite([], '/any')).toBe(false)
  })
})

// ---------------------------------------------------------------------------
// createFavoriteEntry
// ---------------------------------------------------------------------------
describe('createFavoriteEntry', () => {
  it('normalizes the path', () => {
    const entry = createFavoriteEntry('/home/dev/project/')
    expect(entry.path).toBe('/home/dev/project')
  })

  it('uses provided name', () => {
    const entry = createFavoriteEntry('/home/dev/project', 'My Project')
    expect(entry.name).toBe('My Project')
  })

  it('extracts folder name when no name provided', () => {
    const entry = createFavoriteEntry('/home/dev/my-app')
    expect(entry.name).toBe('my-app')
  })

  it('sets addedAt to a valid ISO date', () => {
    const entry = createFavoriteEntry('/some/path')
    expect(() => new Date(entry.addedAt)).not.toThrow()
    expect(new Date(entry.addedAt).getFullYear()).toBeGreaterThanOrEqual(2025)
  })
})

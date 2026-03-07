/**
 * Pure helper functions for project management.
 * Extracted from useProjects composable for testability.
 */
import { getFolderName } from '~/utils/path'

export interface Project {
  path: string
  name: string
  addedAt: string
}

/** @deprecated Use Project instead */
export type Favorite = Project

export type ProjectSortMode = 'alpha' | 'alpha-desc' | 'manual'

/** @deprecated Use ProjectSortMode instead */
export type FavoritesSortMode = ProjectSortMode

/**
 * Normalize path by stripping trailing slashes for consistent comparison.
 */
export function normalizePath(p: string): string {
  return p.replace(/\/+$/, '')
}

/**
 * Deduplicate projects by normalized path, keeping first occurrence.
 */
export function deduplicateProjects(projects: Project[]): Project[] {
  const seen = new Set<string>()
  return projects.filter((proj) => {
    const key = normalizePath(proj.path)
    if (seen.has(key)) return false
    seen.add(key)
    return true
  })
}

/** @deprecated Use deduplicateProjects instead */
export const deduplicateFavorites = deduplicateProjects

/**
 * Sort projects according to the given mode.
 */
export function sortProjects(projects: Project[], mode: ProjectSortMode): Project[] {
  if (mode === 'alpha') {
    return [...projects].sort((a, b) => a.name.localeCompare(b.name))
  }
  if (mode === 'alpha-desc') {
    return [...projects].sort((a, b) => b.name.localeCompare(a.name))
  }
  return projects
}

/** @deprecated Use sortProjects instead */
export const sortFavorites = sortProjects

/**
 * Check if a path is already in the projects list (normalized comparison).
 */
export function isProject(projects: Project[], path: string): boolean {
  const normalized = normalizePath(path)
  return projects.some((f) => normalizePath(f.path) === normalized)
}

/** @deprecated Use isProject instead */
export const isFavorite = isProject

/**
 * Create a new Project entry from a path and optional name.
 */
export function createProjectEntry(path: string, name?: string): Project {
  const normalized = normalizePath(path)
  return {
    path: normalized,
    name: name || getFolderName(path),
    addedAt: new Date().toISOString(),
  }
}

/** @deprecated Use createProjectEntry instead */
export const createFavoriteEntry = createProjectEntry

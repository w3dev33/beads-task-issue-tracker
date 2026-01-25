import { readdir, stat } from 'node:fs/promises'
import { join, resolve } from 'node:path'
import { homedir } from 'node:os'

export interface DirectoryEntry {
  name: string
  path: string
  isDirectory: boolean
  hasBeads: boolean
}

export default defineEventHandler(async (event) => {
  const query = getQuery(event)
  let requestedPath = query.path ? String(query.path) : homedir()

  // Handle special paths
  if (requestedPath === '~') {
    requestedPath = homedir()
  }

  const targetPath = resolve(requestedPath)

  try {
    const entries = await readdir(targetPath, { withFileTypes: true })

    const directories: DirectoryEntry[] = []

    for (const entry of entries) {
      // Skip hidden files/folders (starting with .)
      if (entry.name.startsWith('.')) continue

      if (entry.isDirectory()) {
        const fullPath = join(targetPath, entry.name)

        // Check if this directory contains a .beads folder
        let hasBeads = false
        try {
          const beadsPath = join(fullPath, '.beads')
          const beadsStat = await stat(beadsPath)
          hasBeads = beadsStat.isDirectory()
        } catch {
          // .beads doesn't exist
        }

        directories.push({
          name: entry.name,
          path: fullPath,
          isDirectory: true,
          hasBeads,
        })
      }
    }

    // Sort: beads projects first, then alphabetically
    directories.sort((a, b) => {
      if (a.hasBeads && !b.hasBeads) return -1
      if (!a.hasBeads && b.hasBeads) return 1
      return a.name.localeCompare(b.name)
    })

    // Check if current directory has .beads
    let currentHasBeads = false
    try {
      const beadsPath = join(targetPath, '.beads')
      const beadsStat = await stat(beadsPath)
      currentHasBeads = beadsStat.isDirectory()
    } catch {
      // .beads doesn't exist
    }

    return {
      currentPath: targetPath,
      hasBeads: currentHasBeads,
      entries: directories,
    }
  } catch (error) {
    throw createError({
      statusCode: 500,
      message: `Cannot read directory: ${error instanceof Error ? error.message : 'Unknown error'}`,
    })
  }
})

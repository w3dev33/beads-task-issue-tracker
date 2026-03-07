/**
 * Path utilities for cross-platform compatibility (Windows/macOS/Linux)
 * Preserves the native path separator format of the platform.
 */

/**
 * Split a path into its components (works with both / and \)
 */
export function splitPath(path: string): string[] {
  return path.split(/[/\\]/)
}

/**
 * Detect which separator is used in a path
 */
export function getPathSeparator(path: string): string {
  return path.includes('\\') ? '\\' : '/'
}

/**
 * Get the folder name from a path (last component)
 */
export function getFolderName(path: string): string {
  const parts = splitPath(path)
  return parts.pop() || path
}

/**
 * Get the parent path, preserving the original separator
 */
export function getParentPath(path: string): string {
  const separator = getPathSeparator(path)
  const parts = splitPath(path)
  parts.pop()
  return parts.join(separator) || separator
}

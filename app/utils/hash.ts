/**
 * Generate a consistent hash from a string using djb2 algorithm.
 * Returns an 8-character hex string for use in localStorage key namespacing.
 */
export function hashPath(path: string): string {
  let hash = 5381
  for (let i = 0; i < path.length; i++) {
    hash = ((hash << 5) + hash) + path.charCodeAt(i)
    hash = hash & hash // Convert to 32bit integer
  }
  return Math.abs(hash).toString(16).padStart(8, '0').slice(0, 8)
}

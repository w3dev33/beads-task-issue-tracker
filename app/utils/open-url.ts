/**
 * Utility to open URLs in the system browser
 * Uses Tauri shell.open() in desktop mode, window.open() in web mode
 */

// Check if running in Tauri
function isTauri(): boolean {
  return typeof window !== 'undefined' && (!!window.__TAURI__ || !!window.__TAURI_INTERNALS__)
}

/**
 * Validate that a URL uses a safe protocol (http or https only)
 */
export function isValidUrl(url: string): boolean {
  try {
    const parsed = new URL(url)
    return ['http:', 'https:'].includes(parsed.protocol)
  } catch {
    return false
  }
}

/**
 * Normalize a URL by adding https:// if it starts with www.
 */
export function normalizeUrl(url: string): string {
  if (url.startsWith('www.')) {
    return `https://${url}`
  }
  return url
}

/**
 * Open a URL in the system browser
 * In Tauri mode: uses shell.open() to open in default browser
 * In web mode: uses window.open() with security attributes
 */
export async function openUrl(url: string): Promise<void> {
  const normalizedUrl = normalizeUrl(url)

  // Validate URL before opening
  if (!isValidUrl(normalizedUrl)) {
    console.warn('Attempted to open invalid URL:', url)
    return
  }

  if (isTauri()) {
    try {
      // Dynamic import to avoid issues in web mode
      const { open } = await import('@tauri-apps/plugin-shell')
      await open(normalizedUrl)
    } catch (error) {
      console.error('Failed to open URL with Tauri shell:', error)
      // Fallback to window.open if Tauri fails
      window.open(normalizedUrl, '_blank', 'noopener,noreferrer')
    }
  } else {
    window.open(normalizedUrl, '_blank', 'noopener,noreferrer')
  }
}

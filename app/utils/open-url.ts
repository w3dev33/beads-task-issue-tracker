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
 * Check if a path is a local file path (relative or absolute)
 */
export function isLocalPath(path: string): boolean {
  // Relative paths (screenshots/..., ./..., ../) or absolute paths starting with /
  return (
    path.startsWith('screenshots/') ||
    path.startsWith('./') ||
    path.startsWith('../') ||
    path.startsWith('/')
  )
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

/**
 * Open a local file with the system default application
 * @param filePath - Absolute path to the file
 */
export async function openImageFile(filePath: string): Promise<void> {
  if (!isTauri()) {
    console.warn('openImageFile is only available in Tauri mode')
    return
  }

  try {
    const { invoke } = await import('@tauri-apps/api/core')
    await invoke('open_image_file', { path: filePath })
  } catch (error) {
    console.error('Failed to open image file:', error)
  }
}

export interface TextData {
  content: string
}

export async function readTextFile(filePath: string): Promise<TextData | null> {
  if (!isTauri()) {
    console.warn('readTextFile is only available in Tauri mode')
    return null
  }

  try {
    const { invoke } = await import('@tauri-apps/api/core')
    const result = await invoke<{ content: string }>('read_text_file', { path: filePath })
    return { content: result.content }
  } catch (error) {
    console.error('Failed to read text file:', error)
    return null
  }
}

export async function writeTextFile(filePath: string, content: string): Promise<boolean> {
  if (!isTauri()) {
    console.warn('writeTextFile is only available in Tauri mode')
    return false
  }

  try {
    const { invoke } = await import('@tauri-apps/api/core')
    await invoke('write_text_file', { path: filePath, content })
    return true
  } catch (error) {
    console.error('Failed to write text file:', error)
    return false
  }
}

export interface ImageData {
  base64: string
  mimeType: string
}

export async function readImageFile(filePath: string): Promise<ImageData | null> {
  if (!isTauri()) {
    console.warn('readImageFile is only available in Tauri mode')
    return null
  }

  try {
    const { invoke } = await import('@tauri-apps/api/core')
    const result = await invoke<{ base64: string; mime_type: string }>('read_image_file', { path: filePath })
    return {
      base64: result.base64,
      mimeType: result.mime_type,
    }
  } catch (error) {
    console.error('Failed to read image file:', error)
    return null
  }
}

import { startWatching, stopWatching } from '~/utils/bd-api'

interface UseBeadsWatcherOptions {
  onChanged: () => Promise<void>
}

// Cooldown after processing to ignore self-triggered events.
// bd sync + poll write to .beads/, triggering the watcher again.
// This breaks the cascade: change → poll → sync writes → watcher → ignore (cooldown).
const SELF_TRIGGER_COOLDOWN_MS = 3_000

/**
 * Manages a native file watcher on the .beads/ directory via Tauri events.
 *
 * - Listens for `beads-changed` events emitted by the Rust backend
 * - 300ms frontend-side debounce to coalesce rapid event bursts
 * - Concurrency guard prevents overlapping data fetches
 * - 3s post-processing cooldown to ignore self-triggered events (sync writes)
 * - Filters events by project path (ignores events from other projects)
 *
 * Exposes `watcherActive` ref for useAdaptivePolling integration:
 * when true, adaptive polling disables the 1s mtime check loop and
 * falls back to a 30s safety-net interval.
 */
export function useBeadsWatcher(options: UseBeadsWatcherOptions) {
  const watcherActive = ref(false)
  const isProcessing = ref(false)

  let currentPath: string | null = null
  let unlisten: (() => void) | null = null
  let debounceTimer: ReturnType<typeof setTimeout> | null = null
  let lastProcessedAt = 0

  const handleEvent = (payload: { path: string }) => {
    // Ignore events for other projects
    if (currentPath && payload.path !== currentPath) return

    // Ignore events during cooldown (self-triggered by our own sync/poll writes)
    if (Date.now() - lastProcessedAt < SELF_TRIGGER_COOLDOWN_MS) return

    // Debounce: coalesce rapid events into one callback
    if (debounceTimer) clearTimeout(debounceTimer)
    debounceTimer = setTimeout(async () => {
      if (isProcessing.value) return
      isProcessing.value = true
      try {
        await options.onChanged()
      } catch {
        // Ignore errors — polling will catch up
      } finally {
        isProcessing.value = false
        lastProcessedAt = Date.now()
      }
    }, 300)
  }

  const startListening = async (path: string) => {
    currentPath = path

    // Start the Rust-side file watcher
    try {
      await startWatching(path)
    } catch (e) {
      console.warn('[useBeadsWatcher] Failed to start watcher:', e)
      watcherActive.value = false
      return
    }

    // Listen for Tauri events
    try {
      const { listen } = await import('@tauri-apps/api/event')
      const unlistenFn = await listen<{ path: string }>('beads-changed', (event) => {
        handleEvent(event.payload)
      })
      unlisten = unlistenFn
      watcherActive.value = true
    } catch (e) {
      console.warn('[useBeadsWatcher] Failed to listen for events:', e)
      watcherActive.value = false
    }
  }

  const stopListening = async () => {
    if (debounceTimer) {
      clearTimeout(debounceTimer)
      debounceTimer = null
    }

    if (unlisten) {
      unlisten()
      unlisten = null
    }

    try {
      await stopWatching()
    } catch {
      // Ignore stop errors
    }

    watcherActive.value = false
    currentPath = null
  }

  const switchProject = async (newPath: string) => {
    await stopListening()
    await startListening(newPath)
  }

  /** Call after any operation that writes to .beads/ (e.g. safety-net poll with bd sync) */
  const notifySelfWrite = () => {
    lastProcessedAt = Date.now()
  }

  return {
    watcherActive,
    startListening,
    stopListening,
    switchProject,
    notifySelfWrite,
  }
}

import { getExternalUrl, getProbeProjectName, logFrontend, startWatching, stopWatching } from '~/utils/bd-api'
import { usePipelineDiagnostics } from './usePipelineDiagnostics'

interface UseChangeDetectionOptions {
  onChanged: () => Promise<void>
}

// ============================================================================
// Shared constants (same for both backends)
// ============================================================================
const SELF_TRIGGER_COOLDOWN_MS = 3_000
const DEBOUNCE_MS = 300
const MAX_CONSECUTIVE_RERUNS = 5

/**
 * Queue-based handler: at most one `onChanged()` in flight with at most one
 * pending rerun. Events arriving during processing set a rerun flag instead
 * of spawning extra timers. Bounded by MAX_CONSECUTIVE_RERUNS to prevent
 * unbounded loops under sustained churn.
 */
export function createQueuedHandler(
  onChanged: () => Promise<void>,
  getSelfWriteCooldownActive: () => boolean,
  onProcessed: () => void,
) {
  const { recordWatcherTrigger, recordWatcherDebounce, recordWatcherRerun } = usePipelineDiagnostics()

  let inflight = false
  let pendingRerun = false
  let debounceTimer: ReturnType<typeof setTimeout> | null = null
  let consecutiveReruns = 0

  function schedule() {
    if (debounceTimer) clearTimeout(debounceTimer)
    recordWatcherDebounce()
    debounceTimer = setTimeout(() => run(), DEBOUNCE_MS)
  }

  async function run() {
    debounceTimer = null

    if (inflight) {
      pendingRerun = true
      return
    }

    inflight = true
    consecutiveReruns = 0

    try {
      // eslint-disable-next-line no-constant-condition
      while (true) {
        pendingRerun = false
        try {
          await onChanged()
        } catch {
          // Ignore — polling will catch up
        }
        onProcessed()
        consecutiveReruns++

        if (!pendingRerun || consecutiveReruns >= MAX_CONSECUTIVE_RERUNS) break
        recordWatcherRerun()
      }
    } finally {
      inflight = false
      consecutiveReruns = 0
    }
  }

  function trigger() {
    if (getSelfWriteCooldownActive()) {
      recordWatcherTrigger(false)
      return
    }
    recordWatcherTrigger(true)
    if (inflight) {
      pendingRerun = true
      return
    }
    schedule()
  }

  function cancel() {
    if (debounceTimer) {
      clearTimeout(debounceTimer)
      debounceTimer = null
    }
    pendingRerun = false
  }

  return { trigger, cancel }
}

// ============================================================================
// Native file watcher backend (direct mode)
// ============================================================================

function createWatcherBackend(options: UseChangeDetectionOptions) {
  const active = ref(false)

  let currentPath: string | null = null
  let unlisten: (() => void) | null = null
  let lastProcessedAt = 0

  const queue = createQueuedHandler(
    options.onChanged,
    () => Date.now() - lastProcessedAt < SELF_TRIGGER_COOLDOWN_MS,
    () => { lastProcessedAt = Date.now() },
  )

  const handleEvent = (payload: { path: string }) => {
    if (currentPath && payload.path !== currentPath) return
    queue.trigger()
  }

  const start = async (path: string) => {
    stop()
    currentPath = path

    try {
      await startWatching(path)
    } catch (e) {
      console.warn('[watcher] Failed to start:', e)
      active.value = false
      return
    }

    try {
      const { listen } = await import('@tauri-apps/api/event')
      const unlistenFn = await listen<{ path: string }>('beads-changed', (event) => {
        handleEvent(event.payload)
      })
      unlisten = unlistenFn
      active.value = true
    } catch (e) {
      console.warn('[watcher] Failed to listen:', e)
      active.value = false
    }
  }

  const stop = () => {
    queue.cancel()
    if (unlisten) {
      unlisten()
      unlisten = null
    }
    stopWatching().catch(() => {})
    active.value = false
    currentPath = null
  }

  const notifySelfWrite = () => {
    lastProcessedAt = Date.now()
  }

  return { active, start, stop, notifySelfWrite }
}

// ============================================================================
// SSE backend (probe mode) — kept for future dashboard use
// ============================================================================

function createSSEBackend(options: UseChangeDetectionOptions) {
  const active = ref(false)

  let eventSource: EventSource | null = null
  let lastProcessedAt = 0

  const queue = createQueuedHandler(
    options.onChanged,
    () => Date.now() - lastProcessedAt < SELF_TRIGGER_COOLDOWN_MS,
    () => { lastProcessedAt = Date.now() },
  )

  const start = async (beadsPath: string) => {
    stop()

    try {
      const projectName = await getProbeProjectName(beadsPath)
      if (!projectName) {
        logFrontend('warn', '[probe-sse] Could not resolve project name — SSE not started')
        return
      }

      const url = `${getExternalUrl()}/events/${encodeURIComponent(projectName)}`
      eventSource = new EventSource(url)

      eventSource.addEventListener('change', () => queue.trigger())
      eventSource.onopen = () => { active.value = true }
      eventSource.onerror = () => { active.value = false }
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e)
      logFrontend('warn', `[probe-sse] Failed to start: ${msg}`)
      active.value = false
    }
  }

  const stop = () => {
    queue.cancel()
    eventSource?.close()
    eventSource = null
    active.value = false
  }

  const notifySelfWrite = () => {
    lastProcessedAt = Date.now()
  }

  return { active, start, stop, notifySelfWrite }
}

// ============================================================================
// Unified composable — always uses native watcher
// ============================================================================

/**
 * Change detection composable using the native file watcher via Tauri events.
 *
 * The SSE backend (createSSEBackend) is kept as dead code for future dashboard use
 * but is not wired into this composable anymore — the client always reads locally.
 *
 * API:
 * - `startListening(beadsPath)` — starts the watcher
 * - `stopListening()` — stops the watcher
 * - `notifySelfWrite()` — arms cooldown so watcher ignores self-triggered events
 * - `active` ref — true when watcher is connected (for useAdaptivePolling)
 */
export function useChangeDetection(options: UseChangeDetectionOptions) {
  const watcher = createWatcherBackend(options)

  const startListening = async (beadsPath: string) => {
    await watcher.start(beadsPath)
  }

  const stopListening = async () => {
    watcher.stop()
  }

  const notifySelfWrite = () => {
    watcher.notifySelfWrite()
  }

  return {
    active: watcher.active,
    startListening,
    stopListening,
    notifySelfWrite,
  }
}

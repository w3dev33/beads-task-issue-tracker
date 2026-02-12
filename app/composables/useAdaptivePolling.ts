import { useWindowFocus, useIdle } from '@vueuse/core'

/**
 * Adaptive polling composable that adjusts poll interval based on window state.
 *
 * | State                    | Poll     | Check    | Rationale                              |
 * |--------------------------|----------|----------|----------------------------------------|
 * | Focused + active         | 5s       | 1s       | Fast mtime detection + periodic poll   |
 * | Focused + watcher active | 30s      | —        | Watcher handles detection, safety net  |
 * | Window blurred           | 30s      | —        | Visible on another monitor, not focused|
 * | Idle (2min no input)     | 60s      | —        | User away or just reading              |
 * | Hidden/minimized         | Paused   | —        | No point polling if not visible        |
 *
 * When a `checkFn` is provided, it runs on a fast 1s interval (active state only).
 * When `checkFn` returns true, `pollFn` is called immediately — no waiting for the
 * next poll cycle. This decouples cheap mtime detection from expensive data fetching.
 */

const INTERVAL_ACTIVE = 5_000        // 5 seconds — full poll fallback
const INTERVAL_WATCHER_SAFETY = 30_000 // 30 seconds — safety net when watcher is active
const INTERVAL_CHECK = 1_000         // 1 second — fast mtime check (active only)
const INTERVAL_BLURRED = 30_000      // 30 seconds
const INTERVAL_IDLE = 60_000         // 60 seconds
const IDLE_TIMEOUT = 120_000         // 2 minutes

interface AdaptivePollingOptions {
  /** Cheap check function (e.g., mtime stat). Returns true if pollFn should run. */
  checkFn?: () => Promise<boolean>
  /** Interval for checkFn in ms (default: 1000). Only used when window is focused + active. */
  checkInterval?: number
  /** When true, disables 1s mtime check loop and uses 30s safety-net interval instead. */
  watcherActive?: Ref<boolean>
}

export function useAdaptivePolling(pollFn: () => Promise<void>, options?: AdaptivePollingOptions) {
  const isFocused = useWindowFocus()
  const { idle } = useIdle(IDLE_TIMEOUT)

  let timer: ReturnType<typeof setTimeout> | null = null
  let checkTimer: ReturnType<typeof setTimeout> | null = null
  let running = false
  let polling = false // guard against concurrent polls

  // Track hidden state to detect return from minimized
  let wasHidden = false

  const checkInterval = options?.checkInterval ?? INTERVAL_CHECK

  const currentInterval = computed(() => {
    if (typeof document !== 'undefined' && document.hidden) return 0 // paused
    if (idle.value) return INTERVAL_IDLE
    if (!isFocused.value) return INTERVAL_BLURRED
    // When watcher is active, use longer safety-net interval (watcher handles detection)
    if (options?.watcherActive?.value) return INTERVAL_WATCHER_SAFETY
    return INTERVAL_ACTIVE
  })

  const isActive = () => currentInterval.value === INTERVAL_ACTIVE

  const runPoll = async () => {
    if (polling) return
    polling = true
    try {
      await pollFn()
    } catch {
      // Ignore poll errors
    } finally {
      polling = false
    }
  }

  // --- Fast change detection loop (only when active and no watcher) ---
  const scheduleCheck = () => {
    if (!running || !options?.checkFn) return
    if (checkTimer) clearTimeout(checkTimer)

    // Skip fast mtime check when watcher is active (watcher handles detection)
    if (options?.watcherActive?.value) return

    // Only run fast check when window is focused + active
    if (!isActive()) return

    checkTimer = setTimeout(async () => {
      if (!running || !isActive()) {
        scheduleCheck()
        return
      }
      try {
        const changed = await options.checkFn!()
        if (changed && running) {
          // Change detected — cancel pending poll timer and run immediately
          if (timer) { clearTimeout(timer); timer = null }
          await runPoll()
          scheduleNext() // Reset regular poll timer after immediate poll
        }
      } catch {
        // Ignore check errors
      }
      scheduleCheck()
    }, checkInterval)
  }

  const clearCheckTimer = () => {
    if (checkTimer) {
      clearTimeout(checkTimer)
      checkTimer = null
    }
  }

  // --- Regular poll timer (adaptive interval) ---
  const scheduleNext = () => {
    if (!running) return
    if (timer) clearTimeout(timer)

    const interval = currentInterval.value
    if (interval === 0) {
      // Paused — don't schedule, visibility handler will resume
      return
    }

    timer = setTimeout(async () => {
      if (!running) return
      await runPoll()
      scheduleNext()
    }, interval)
  }

  const handleVisibilityChange = () => {
    if (!running) return

    if (document.hidden) {
      // Going hidden — clear both timers
      wasHidden = true
      if (timer) { clearTimeout(timer); timer = null }
      clearCheckTimer()
    } else if (wasHidden) {
      // Returning from hidden — immediate poll + resume both loops
      wasHidden = false
      runPoll().finally(() => {
        scheduleNext()
        scheduleCheck()
      })
    }
  }

  // Watch focus transitions: poll immediately when returning from blurred
  watch(isFocused, (focused, wasFocused) => {
    if (!running) return
    if (focused && wasFocused === false) {
      // Window just gained focus — immediate poll + reschedule + start fast check
      runPoll().finally(() => {
        scheduleNext()
        scheduleCheck()
      })
    } else if (!focused) {
      // Blurred — stop fast check, reschedule poll with blurred interval
      clearCheckTimer()
      scheduleNext()
    }
  })

  // Watch idle transitions: reschedule when idle state changes
  watch(idle, (isIdle, wasIdlePrev) => {
    if (!running) return
    if (!isIdle && wasIdlePrev) {
      // User returned from idle — immediate poll + reschedule + start fast check
      runPoll().finally(() => {
        scheduleNext()
        scheduleCheck()
      })
    } else if (isIdle) {
      // User went idle — stop fast check, reschedule with idle interval
      clearCheckTimer()
      scheduleNext()
    }
  })

  const start = () => {
    if (running) return
    running = true
    wasHidden = false

    if (typeof document !== 'undefined') {
      document.addEventListener('visibilitychange', handleVisibilityChange)
    }

    scheduleNext()
    scheduleCheck() // Start fast check loop if checkFn provided
  }

  const stop = () => {
    running = false
    if (timer) { clearTimeout(timer); timer = null }
    clearCheckTimer()

    if (typeof document !== 'undefined') {
      document.removeEventListener('visibilitychange', handleVisibilityChange)
    }
  }

  return {
    start,
    stop,
    currentInterval,
  }
}

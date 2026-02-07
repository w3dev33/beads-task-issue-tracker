import { useWindowFocus, useIdle } from '@vueuse/core'

/**
 * Adaptive polling composable that adjusts poll interval based on window state.
 *
 * | State                    | Interval | Rationale                              |
 * |--------------------------|----------|----------------------------------------|
 * | Focused + active         | 5s       | User expects fresh data                |
 * | Window blurred           | 30s      | Visible on another monitor, not focused|
 * | Idle (2min no input)     | 60s      | User away or just reading              |
 * | Hidden/minimized         | Paused   | No point polling if not visible        |
 */

const INTERVAL_ACTIVE = 5_000   // 5 seconds
const INTERVAL_BLURRED = 30_000 // 30 seconds
const INTERVAL_IDLE = 60_000    // 60 seconds
const IDLE_TIMEOUT = 120_000    // 2 minutes

export function useAdaptivePolling(pollFn: () => Promise<void>) {
  const isFocused = useWindowFocus()
  const { idle } = useIdle(IDLE_TIMEOUT)

  let timer: ReturnType<typeof setTimeout> | null = null
  let running = false

  // Track hidden state to detect return from minimized
  let wasHidden = false

  const currentInterval = computed(() => {
    if (typeof document !== 'undefined' && document.hidden) return 0 // paused
    if (idle.value) return INTERVAL_IDLE
    if (!isFocused.value) return INTERVAL_BLURRED
    return INTERVAL_ACTIVE
  })

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
      try {
        await pollFn()
      } catch {
        // Ignore poll errors
      }
      scheduleNext()
    }, interval)
  }

  const handleVisibilityChange = () => {
    if (!running) return

    if (document.hidden) {
      // Going hidden — clear timer
      wasHidden = true
      if (timer) {
        clearTimeout(timer)
        timer = null
      }
    } else if (wasHidden) {
      // Returning from hidden — immediate poll + resume
      wasHidden = false
      pollFn().catch(() => {}).finally(() => scheduleNext())
    }
  }

  // Watch focus transitions: poll immediately when returning from blurred
  watch(isFocused, (focused, wasFocused) => {
    if (!running) return
    if (focused && wasFocused === false) {
      // Window just gained focus — immediate poll + reschedule
      pollFn().catch(() => {}).finally(() => scheduleNext())
    } else if (!focused) {
      // Reschedule with blurred interval
      scheduleNext()
    }
  })

  // Watch idle transitions: reschedule when idle state changes
  watch(idle, (isIdle, wasIdlePrev) => {
    if (!running) return
    if (!isIdle && wasIdlePrev) {
      // User returned from idle — immediate poll + reschedule with active interval
      pollFn().catch(() => {}).finally(() => scheduleNext())
    } else if (isIdle) {
      // User went idle — reschedule with idle interval
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
  }

  const stop = () => {
    running = false
    if (timer) {
      clearTimeout(timer)
      timer = null
    }

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

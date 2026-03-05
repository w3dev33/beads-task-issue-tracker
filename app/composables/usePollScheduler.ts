/**
 * Poll scheduler with backpressure gate.
 *
 * Merges all poll triggers (watcher, adaptive timer, manual) through a single
 * gate that enforces a minimum interval between expensive poll cycles.
 *
 * - `requestPoll()` — trigger from watcher or timer. Respects min interval.
 * - `requestImmediatePoll()` — manual refresh. Bypasses min interval.
 * - `stats` — lightweight instrumentation (skipped, deferred, executed counts).
 */

import { usePipelineDiagnostics } from './usePipelineDiagnostics'

const DEFAULT_MIN_INTERVAL_MS = 2_000

interface PollSchedulerOptions {
  /** Minimum ms between expensive poll cycles (default: 2000). */
  minInterval?: number
}

export function usePollScheduler(
  pollFn: () => Promise<void>,
  options?: PollSchedulerOptions,
) {
  const minInterval = options?.minInterval ?? DEFAULT_MIN_INTERVAL_MS
  const { recordPollRequest, recordPollDecision } = usePipelineDiagnostics()

  let lastPollEnd = 0
  let inflight = false
  let deferredTimer: ReturnType<typeof setTimeout> | null = null

  // Lightweight instrumentation (plain object — no reactivity needed for counters)
  const stats = {
    executed: 0,
    skipped: 0,
    deferred: 0,
  }

  const clearDeferred = () => {
    if (deferredTimer) {
      clearTimeout(deferredTimer)
      deferredTimer = null
    }
  }

  const runPoll = async () => {
    if (inflight) {
      stats.skipped++
      recordPollDecision('skipped')
      return
    }
    inflight = true
    try {
      await pollFn()
      stats.executed++
      recordPollDecision('executed')
    } finally {
      inflight = false
      lastPollEnd = Date.now()
    }
  }

  /**
   * Request a poll (from watcher or timer). Enforces min interval:
   * - If enough time has passed → runs immediately.
   * - If too soon → schedules a deferred poll at the end of the window.
   * - If already inflight or a deferred poll is pending → skips.
   */
  const requestPoll = () => {
    recordPollRequest('normal')

    if (inflight) {
      stats.skipped++
      recordPollDecision('skipped')
      return
    }

    const elapsed = Date.now() - lastPollEnd
    if (elapsed >= minInterval) {
      clearDeferred()
      runPoll()
      return
    }

    // Too soon — defer to end of cooldown window (if not already deferred)
    if (deferredTimer) {
      stats.skipped++
      recordPollDecision('skipped')
      return
    }

    stats.deferred++
    recordPollDecision('deferred')
    const remaining = minInterval - elapsed
    deferredTimer = setTimeout(() => {
      deferredTimer = null
      runPoll()
    }, remaining)
  }

  /**
   * Request an immediate poll (manual refresh). Bypasses min interval.
   * Still prevents concurrent polls.
   */
  const requestImmediatePoll = () => {
    recordPollRequest('immediate')
    clearDeferred()
    runPoll()
  }

  /** Cancel any pending deferred poll. */
  const cancel = () => {
    clearDeferred()
  }

  return {
    requestPoll,
    requestImmediatePoll,
    cancel,
    stats,
  }
}

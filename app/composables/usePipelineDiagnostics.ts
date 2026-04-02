/**
 * Pipeline diagnostics for watcher → poll pipeline.
 *
 * Tracks: watcher batches, emitted/suppressed events, poll start/finish/duration,
 * deferred runs, and self-write cooldown hits. All counters are plain numbers
 * (no Vue reactivity) to avoid observer overhead on hot paths.
 *
 * Exposes a compact snapshot for DebugPanel and support triage.
 * Console logging is rate-limited (at most once per LOG_INTERVAL_MS).
 */

const LOG_INTERVAL_MS = 5_000

interface PipelineCounters {
  // Watcher / change detection
  watcherTriggers: number
  watcherCooldownSkips: number
  watcherDebounces: number
  watcherReruns: number

  // Poll scheduler
  pollRequests: number
  pollExecuted: number
  pollSkipped: number
  pollDeferred: number
  pollImmediateRequests: number

  // Poll execution
  pollStarted: number
  pollFinished: number
  pollErrors: number
  pollTotalMs: number
  pollLastMs: number
  pollMaxMs: number

  // Mtime fast-check
  mtimeChecks: number
  mtimeHits: number
}

function createCounters(): PipelineCounters {
  return {
    watcherTriggers: 0,
    watcherCooldownSkips: 0,
    watcherDebounces: 0,
    watcherReruns: 0,
    pollRequests: 0,
    pollExecuted: 0,
    pollSkipped: 0,
    pollDeferred: 0,
    pollImmediateRequests: 0,
    pollStarted: 0,
    pollFinished: 0,
    pollErrors: 0,
    pollTotalMs: 0,
    pollLastMs: 0,
    pollMaxMs: 0,
    mtimeChecks: 0,
    mtimeHits: 0,
  }
}

// Singleton state
const counters = createCounters()
let lastLogAt = 0
let startedAt = Date.now()

/** Rate-limited console log (at most once per LOG_INTERVAL_MS). */
function maybeLog(tag: string, detail?: string) {
  const now = Date.now()
  if (now - lastLogAt < LOG_INTERVAL_MS) return
  lastLogAt = now
  const msg = detail ? `[diag][${tag}] ${detail}` : `[diag][${tag}]`
  console.debug(msg)
}

// ── Recording API (called from pipeline composables) ────────────────────

function recordWatcherTrigger(accepted: boolean) {
  if (accepted) {
    counters.watcherTriggers++
  } else {
    counters.watcherCooldownSkips++
  }
  maybeLog('watcher', `trigger accepted=${accepted} total=${counters.watcherTriggers} cooldownSkips=${counters.watcherCooldownSkips}`)
}

function recordWatcherDebounce() {
  counters.watcherDebounces++
}

function recordWatcherRerun() {
  counters.watcherReruns++
}

function recordPollRequest(kind: 'normal' | 'immediate') {
  counters.pollRequests++
  if (kind === 'immediate') counters.pollImmediateRequests++
}

function recordPollDecision(decision: 'executed' | 'skipped' | 'deferred') {
  counters[`poll${decision.charAt(0).toUpperCase() + decision.slice(1)}` as keyof PipelineCounters] =
    (counters[`poll${decision.charAt(0).toUpperCase() + decision.slice(1)}` as keyof PipelineCounters] as number) + 1
  maybeLog('scheduler', `decision=${decision} executed=${counters.pollExecuted} skipped=${counters.pollSkipped} deferred=${counters.pollDeferred}`)
}

function recordPollStart() {
  counters.pollStarted++
}

function recordPollFinish(durationMs: number, error: boolean) {
  counters.pollFinished++
  counters.pollTotalMs += durationMs
  counters.pollLastMs = durationMs
  if (durationMs > counters.pollMaxMs) counters.pollMaxMs = durationMs
  if (error) counters.pollErrors++
  maybeLog('poll', `duration=${durationMs}ms avg=${Math.round(counters.pollTotalMs / counters.pollFinished)}ms max=${counters.pollMaxMs}ms errors=${counters.pollErrors}`)
}

function recordMtimeCheck(hit: boolean) {
  counters.mtimeChecks++
  if (hit) counters.mtimeHits++
}

// ── Snapshot API (for DebugPanel / support triage) ──────────────────────

interface PipelineSnapshot {
  uptimeSeconds: number
  counters: PipelineCounters
  pollAvgMs: number
}

function snapshot(): PipelineSnapshot {
  const avgMs = counters.pollFinished > 0
    ? Math.round(counters.pollTotalMs / counters.pollFinished)
    : 0

  return {
    uptimeSeconds: Math.round((Date.now() - startedAt) / 1000),
    counters: { ...counters },
    pollAvgMs: avgMs,
  }
}

function reset() {
  Object.assign(counters, createCounters())
  startedAt = Date.now()
  lastLogAt = 0
}

// ── Composable ──────────────────────────────────────────────────────────

export function usePipelineDiagnostics() {
  return {
    // Recording
    recordWatcherTrigger,
    recordWatcherDebounce,
    recordWatcherRerun,
    recordPollRequest,
    recordPollDecision,
    recordPollStart,
    recordPollFinish,
    recordMtimeCheck,
    // Snapshot
    snapshot,
    reset,
  }
}

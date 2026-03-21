import { describe, it, expect, beforeEach } from 'vitest'
import { usePipelineDiagnostics } from '~/composables/usePipelineDiagnostics'

describe('usePipelineDiagnostics', () => {
  let diag: ReturnType<typeof usePipelineDiagnostics>

  beforeEach(() => {
    diag = usePipelineDiagnostics()
    diag.reset()
  })

  it('starts with zero counters', () => {
    const snap = diag.snapshot()
    expect(snap.counters.watcherTriggers).toBe(0)
    expect(snap.counters.pollExecuted).toBe(0)
    expect(snap.counters.mtimeChecks).toBe(0)
    expect(snap.pollAvgMs).toBe(0)
  })

  it('tracks watcher triggers', () => {
    diag.recordWatcherTrigger(true)
    diag.recordWatcherTrigger(true)
    diag.recordWatcherTrigger(false)

    const snap = diag.snapshot()
    expect(snap.counters.watcherTriggers).toBe(2)
    expect(snap.counters.watcherCooldownSkips).toBe(1)
  })

  it('tracks watcher debounces and reruns', () => {
    diag.recordWatcherDebounce()
    diag.recordWatcherDebounce()
    diag.recordWatcherRerun()

    const snap = diag.snapshot()
    expect(snap.counters.watcherDebounces).toBe(2)
    expect(snap.counters.watcherReruns).toBe(1)
  })

  it('tracks poll requests', () => {
    diag.recordPollRequest('normal')
    diag.recordPollRequest('normal')
    diag.recordPollRequest('immediate')

    const snap = diag.snapshot()
    expect(snap.counters.pollRequests).toBe(3)
    expect(snap.counters.pollImmediateRequests).toBe(1)
  })

  it('tracks poll decisions', () => {
    diag.recordPollDecision('executed')
    diag.recordPollDecision('skipped')
    diag.recordPollDecision('skipped')
    diag.recordPollDecision('deferred')

    const snap = diag.snapshot()
    expect(snap.counters.pollExecuted).toBe(1)
    expect(snap.counters.pollSkipped).toBe(2)
    expect(snap.counters.pollDeferred).toBe(1)
  })

  it('tracks poll execution timing', () => {
    diag.recordPollStart()
    diag.recordPollFinish(100, false)
    diag.recordPollStart()
    diag.recordPollFinish(300, false)

    const snap = diag.snapshot()
    expect(snap.counters.pollStarted).toBe(2)
    expect(snap.counters.pollFinished).toBe(2)
    expect(snap.counters.pollLastMs).toBe(300)
    expect(snap.counters.pollMaxMs).toBe(300)
    expect(snap.counters.pollTotalMs).toBe(400)
    expect(snap.pollAvgMs).toBe(200)
    expect(snap.counters.pollErrors).toBe(0)
  })

  it('tracks poll errors', () => {
    diag.recordPollStart()
    diag.recordPollFinish(50, true)

    const snap = diag.snapshot()
    expect(snap.counters.pollErrors).toBe(1)
    expect(snap.counters.pollFinished).toBe(1)
  })

  it('tracks mtime checks and hits', () => {
    diag.recordMtimeCheck(false)
    diag.recordMtimeCheck(false)
    diag.recordMtimeCheck(true)

    const snap = diag.snapshot()
    expect(snap.counters.mtimeChecks).toBe(3)
    expect(snap.counters.mtimeHits).toBe(1)
  })

  it('reset clears all counters', () => {
    diag.recordWatcherTrigger(true)
    diag.recordPollStart()
    diag.recordPollFinish(200, false)
    diag.recordMtimeCheck(true)

    diag.reset()
    const snap = diag.snapshot()
    expect(snap.counters.watcherTriggers).toBe(0)
    expect(snap.counters.pollStarted).toBe(0)
    expect(snap.counters.mtimeChecks).toBe(0)
    expect(snap.uptimeSeconds).toBe(0)
  })

  it('snapshot returns uptime', () => {
    const snap = diag.snapshot()
    expect(snap.uptimeSeconds).toBeGreaterThanOrEqual(0)
  })

  it('is a singleton — multiple calls share state', () => {
    const diag2 = usePipelineDiagnostics()
    diag.recordWatcherTrigger(true)

    const snap = diag2.snapshot()
    expect(snap.counters.watcherTriggers).toBe(1)
  })
})

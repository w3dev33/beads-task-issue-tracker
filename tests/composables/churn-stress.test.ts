import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { createQueuedHandler } from '~/composables/useChangeDetection'
import { usePollScheduler } from '~/composables/usePollScheduler'

describe('churn stress', () => {
  beforeEach(() => { vi.useFakeTimers() })
  afterEach(() => { vi.useRealTimers() })

  // ── helpers ──────────────────────────────────────────────────────────

  function setupHandler(opts: { onChangedMs?: number } = {}) {
    let resolvers: Array<() => void> = []
    const onChanged = vi.fn(() => {
      if (opts.onChangedMs != null) {
        return new Promise<void>((resolve) => { resolvers.push(resolve) })
      }
      return Promise.resolve()
    })
    const onProcessed = vi.fn()

    const handler = createQueuedHandler(onChanged, () => false, onProcessed)

    return {
      handler,
      onChanged,
      onProcessed,
      resolveLatest: () => {
        const r = resolvers.shift()
        r?.()
        return vi.advanceTimersByTimeAsync(0)
      },
    }
  }

  // ── (a) Sustained rapid triggers don't cause unbounded calls ────────

  it('sustained rapid triggers stay bounded', async () => {
    const { handler, onChanged } = setupHandler()

    // Fire 500 triggers over 30s in bursts: trigger, then advance enough
    // for some debounces to fire (mix of short and long gaps).
    for (let i = 0; i < 500; i++) {
      handler.trigger()
      // Every 5th trigger, advance past DEBOUNCE_MS so calls actually fire.
      // Other triggers advance only 10ms, resetting the debounce.
      if (i % 5 === 4) {
        await vi.advanceTimersByTimeAsync(350)
      } else {
        await vi.advanceTimersByTimeAsync(10)
      }
    }
    // Drain final debounce
    await vi.advanceTimersByTimeAsync(300)

    // 500 triggers, but debounce collapses groups of ~5 into 1 call.
    // Expect roughly 100 calls — well below 500.
    expect(onChanged.mock.calls.length).toBeLessThan(150)
    expect(onChanged.mock.calls.length).toBeGreaterThan(0)
  })

  // ── (b) Queue + scheduler pipeline stress ───────────────────────────

  it('queue + scheduler pipeline stays bounded', async () => {
    let inflightCount = 0
    let maxInflight = 0

    let pollResolvers: Array<() => void> = []
    const pollFn = vi.fn(() => new Promise<void>((resolve) => {
      inflightCount++
      maxInflight = Math.max(maxInflight, inflightCount)
      pollResolvers.push(() => {
        inflightCount--
        resolve()
      })
    }))

    const scheduler = usePollScheduler(pollFn, { minInterval: 2_000 })

    const onChanged = vi.fn(async () => {
      scheduler.requestPoll()
    })
    const onProcessed = vi.fn()
    const handler = createQueuedHandler(onChanged, () => false, onProcessed)

    // Fire 200 triggers over ~20s. Use bursts of 10 rapid triggers
    // followed by enough time for debounce to fire, so onChanged actually
    // calls requestPoll repeatedly.
    for (let i = 0; i < 200; i++) {
      handler.trigger()
      // Every 10th trigger, advance past debounce so onChanged fires
      if (i % 10 === 9) {
        await vi.advanceTimersByTimeAsync(350)
      } else {
        await vi.advanceTimersByTimeAsync(50)
      }
      // Resolve any pending poll so the pipeline keeps flowing
      while (pollResolvers.length > 0) {
        const r = pollResolvers.shift()
        r?.()
        await vi.advanceTimersByTimeAsync(0)
      }
    }

    // Drain remaining deferred timers
    await vi.advanceTimersByTimeAsync(3_000)
    while (pollResolvers.length > 0) {
      const r = pollResolvers.shift()
      r?.()
      await vi.advanceTimersByTimeAsync(0)
    }

    // With 2s min interval over ~20s, pollFn should execute ≤ ~12 times
    expect(scheduler.stats.executed).toBeLessThan(20)
    expect(scheduler.stats.executed).toBeGreaterThan(0)

    // Skipped + deferred should outnumber executed
    const nonExecuted = scheduler.stats.skipped + scheduler.stats.deferred
    expect(nonExecuted).toBeGreaterThan(scheduler.stats.executed)

    // No concurrent polls ever
    expect(maxInflight).toBeLessThanOrEqual(1)
  })

  // ── (c) Burst followed by quiet converges ───────────────────────────

  it('burst followed by quiet fully settles', async () => {
    let pollResolvers: Array<() => void> = []
    const pollFn = vi.fn(() => new Promise<void>((resolve) => {
      pollResolvers.push(resolve)
    }))

    const scheduler = usePollScheduler(pollFn, { minInterval: 2_000 })

    const onChanged = vi.fn(async () => {
      scheduler.requestPoll()
    })
    const onProcessed = vi.fn()
    const handler = createQueuedHandler(onChanged, () => false, onProcessed)

    // Burst: 50 rapid triggers in 100ms (every 2ms)
    for (let i = 0; i < 50; i++) {
      handler.trigger()
      await vi.advanceTimersByTimeAsync(2)
    }

    // Let debounce fire
    await vi.advanceTimersByTimeAsync(300)

    // Resolve all polls that were started
    while (pollResolvers.length > 0) {
      const r = pollResolvers.shift()
      r?.()
      await vi.advanceTimersByTimeAsync(0)
    }

    // Quiet period — advance 10s with no new triggers
    await vi.advanceTimersByTimeAsync(10_000)

    // Resolve any final deferred polls
    while (pollResolvers.length > 0) {
      const r = pollResolvers.shift()
      r?.()
      await vi.advanceTimersByTimeAsync(0)
    }

    // Pipeline should have settled: pollFn was called at least once
    expect(pollFn.mock.calls.length).toBeGreaterThan(0)

    // No more pending work — advancing more time should not trigger anything
    const callsAfterSettle = pollFn.mock.calls.length
    await vi.advanceTimersByTimeAsync(5_000)
    expect(pollFn.mock.calls.length).toBe(callsAfterSettle)

    // Last poll ran to completion (no unresolved promises)
    expect(pollResolvers).toHaveLength(0)
  })

  // ── (d) Continuous rerun bounding under churn ───────────────────────

  it('consecutive reruns are bounded by MAX_CONSECUTIVE_RERUNS', async () => {
    let resolvers: Array<() => void> = []
    const onChanged = vi.fn(() => new Promise<void>((resolve) => {
      resolvers.push(resolve)
    }))
    const onProcessed = vi.fn()

    const handler = createQueuedHandler(onChanged, () => false, onProcessed)

    // Initial trigger → debounce fires → first onChanged starts
    handler.trigger()
    await vi.advanceTimersByTimeAsync(300)
    expect(onChanged).toHaveBeenCalledTimes(1)

    // Keep firing triggers before each resolution to keep pendingRerun set.
    // MAX_CONSECUTIVE_RERUNS = 5, so the while loop should stop after 5
    // iterations (including the first run that resets the counter).
    for (let i = 0; i < 10; i++) {
      handler.trigger() // set pendingRerun
      const r = resolvers.shift()
      r?.()
      await vi.advanceTimersByTimeAsync(0)
    }

    // The handler should have stopped re-entering after at most 6 calls
    // (1 initial + up to 5 consecutive reruns = 6 total, but the loop
    // resets consecutiveReruns to 0 at the start so the initial run counts
    // as rerun #1 after increment → max 5+1=6).
    expect(onChanged.mock.calls.length).toBeLessThanOrEqual(6)
    expect(onChanged.mock.calls.length).toBeGreaterThanOrEqual(2)

    // After the handler stops, further triggers should schedule a fresh
    // debounce rather than immediately re-entering.
    const callsBefore = onChanged.mock.calls.length
    handler.trigger()
    await vi.advanceTimersByTimeAsync(0)
    // No immediate call — still debouncing
    expect(onChanged.mock.calls.length).toBe(callsBefore)

    // Resolve any remaining in-flight call
    while (resolvers.length > 0) {
      const r = resolvers.shift()
      r?.()
      await vi.advanceTimersByTimeAsync(0)
    }

    // After debounce, a new run starts fresh
    await vi.advanceTimersByTimeAsync(300)
    expect(onChanged.mock.calls.length).toBeGreaterThan(callsBefore)
  })
})

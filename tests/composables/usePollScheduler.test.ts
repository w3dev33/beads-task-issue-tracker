import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { usePollScheduler } from '~/composables/usePollScheduler'

describe('usePollScheduler', () => {
  beforeEach(() => { vi.useFakeTimers() })
  afterEach(() => { vi.useRealTimers() })

  function setup(opts: { minInterval?: number; pollMs?: number } = {}) {
    let resolvers: Array<() => void> = []
    const pollFn = vi.fn(() => {
      if (opts.pollMs) {
        return new Promise<void>((resolve) => { resolvers.push(resolve) })
      }
      return Promise.resolve()
    })

    const scheduler = usePollScheduler(pollFn, {
      minInterval: opts.minInterval ?? 100,
    })

    const resolveOnePoll = () => {
      const r = resolvers.shift()
      r?.()
    }

    return { pollFn, scheduler, resolveOnePoll }
  }

  it('executes first poll immediately', async () => {
    const { pollFn, scheduler } = setup()
    scheduler.requestPoll()
    await vi.advanceTimersByTimeAsync(0)
    expect(pollFn).toHaveBeenCalledTimes(1)
    expect(scheduler.stats.executed).toBe(1)
  })

  it('enforces min interval between polls', async () => {
    const { pollFn, scheduler } = setup({ minInterval: 100 })

    scheduler.requestPoll()
    await vi.advanceTimersByTimeAsync(0)
    expect(pollFn).toHaveBeenCalledTimes(1)

    // Request again immediately — should defer
    scheduler.requestPoll()
    await vi.advanceTimersByTimeAsync(0)
    expect(pollFn).toHaveBeenCalledTimes(1)
    expect(scheduler.stats.deferred).toBe(1)

    // Advance past min interval — deferred poll should fire
    await vi.advanceTimersByTimeAsync(100)
    expect(pollFn).toHaveBeenCalledTimes(2)
    expect(scheduler.stats.executed).toBe(2)
  })

  it('skips when a poll is already inflight', async () => {
    const { pollFn, scheduler, resolveOnePoll } = setup({ pollMs: 50 })

    scheduler.requestPoll()
    await vi.advanceTimersByTimeAsync(0)
    expect(pollFn).toHaveBeenCalledTimes(1)

    // Request while inflight — should skip
    scheduler.requestPoll()
    expect(scheduler.stats.skipped).toBe(1)

    // Finish inflight poll
    resolveOnePoll()
    await vi.advanceTimersByTimeAsync(0)
  })

  it('deduplicates rapid triggers into one deferred poll', async () => {
    const { pollFn, scheduler } = setup({ minInterval: 100 })

    scheduler.requestPoll()
    await vi.advanceTimersByTimeAsync(0)
    expect(pollFn).toHaveBeenCalledTimes(1)

    // Three rapid triggers — first defers, rest skip
    scheduler.requestPoll()
    scheduler.requestPoll()
    scheduler.requestPoll()
    expect(scheduler.stats.deferred).toBe(1)
    expect(scheduler.stats.skipped).toBe(2)

    await vi.advanceTimersByTimeAsync(100)
    expect(pollFn).toHaveBeenCalledTimes(2)
  })

  it('requestImmediatePoll bypasses min interval', async () => {
    const { pollFn, scheduler } = setup({ minInterval: 100 })

    scheduler.requestPoll()
    await vi.advanceTimersByTimeAsync(0)
    expect(pollFn).toHaveBeenCalledTimes(1)

    // Immediate poll right after — should bypass
    scheduler.requestImmediatePoll()
    await vi.advanceTimersByTimeAsync(0)
    expect(pollFn).toHaveBeenCalledTimes(2)
  })

  it('cancel clears pending deferred poll', async () => {
    const { pollFn, scheduler } = setup({ minInterval: 100 })

    scheduler.requestPoll()
    await vi.advanceTimersByTimeAsync(0)
    expect(pollFn).toHaveBeenCalledTimes(1)

    scheduler.requestPoll() // deferred
    scheduler.cancel()

    await vi.advanceTimersByTimeAsync(200)
    expect(pollFn).toHaveBeenCalledTimes(1) // no deferred poll ran
  })
})

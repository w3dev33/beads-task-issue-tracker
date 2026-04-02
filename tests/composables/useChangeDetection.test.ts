import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { createQueuedHandler } from '~/composables/useChangeDetection'

describe('createQueuedHandler', () => {
  beforeEach(() => { vi.useFakeTimers() })
  afterEach(() => { vi.useRealTimers() })

  function setup(opts: { onChangedMs?: number; cooldown?: boolean } = {}) {
    const calls: number[] = []
    let callId = 0
    let resolvers: Array<() => void> = []

    const onChanged = vi.fn(() => {
      const id = ++callId
      calls.push(id)
      if (opts.onChangedMs != null) {
        return new Promise<void>((resolve) => {
          resolvers.push(resolve)
        })
      }
      return Promise.resolve()
    })

    const cooldownActive = opts.cooldown ?? false
    const onProcessed = vi.fn()

    const handler = createQueuedHandler(
      onChanged,
      () => cooldownActive,
      onProcessed,
    )

    return {
      handler,
      onChanged,
      onProcessed,
      calls,
      resolveLatest: () => {
        const r = resolvers.shift()
        r?.()
        return vi.advanceTimersByTimeAsync(0)
      },
    }
  }

  it('debounces rapid triggers into a single call', async () => {
    const { handler, onChanged } = setup()

    handler.trigger()
    handler.trigger()
    handler.trigger()

    await vi.advanceTimersByTimeAsync(300)
    expect(onChanged).toHaveBeenCalledTimes(1)
  })

  it('at most one onChanged in flight', async () => {
    const { handler, onChanged, resolveLatest } = setup({ onChangedMs: 100 })

    handler.trigger()
    await vi.advanceTimersByTimeAsync(300) // debounce fires, onChanged starts
    expect(onChanged).toHaveBeenCalledTimes(1)

    // Trigger again while in-flight — should set pendingRerun, not start another
    handler.trigger()
    await vi.advanceTimersByTimeAsync(300)
    expect(onChanged).toHaveBeenCalledTimes(1) // still just 1

    // Resolve the first call — triggers the pending rerun
    await resolveLatest()
    expect(onChanged).toHaveBeenCalledTimes(2)
  })

  it('collapses multiple events during processing into one follow-up', async () => {
    const { handler, onChanged, resolveLatest } = setup({ onChangedMs: 100 })

    handler.trigger()
    await vi.advanceTimersByTimeAsync(300)
    expect(onChanged).toHaveBeenCalledTimes(1)

    // Fire many events while in-flight
    for (let i = 0; i < 10; i++) {
      handler.trigger()
    }
    await vi.advanceTimersByTimeAsync(300)
    expect(onChanged).toHaveBeenCalledTimes(1) // still 1

    // Resolve — exactly one follow-up
    await resolveLatest()
    expect(onChanged).toHaveBeenCalledTimes(2)

    // Resolve the follow-up — no more calls
    await resolveLatest()
    expect(onChanged).toHaveBeenCalledTimes(2)
  })

  it('respects self-write cooldown', async () => {
    const { handler, onChanged } = setup({ cooldown: true })

    handler.trigger()
    await vi.advanceTimersByTimeAsync(300)
    expect(onChanged).not.toHaveBeenCalled()
  })

  it('bounds consecutive reruns', async () => {
    // onChanged takes time, and events keep arriving, so pendingRerun is always set
    let resolvers: Array<() => void> = []
    const onChanged = vi.fn(() => new Promise<void>((resolve) => {
      resolvers.push(resolve)
    }))
    const onProcessed = vi.fn()

    const handler = createQueuedHandler(onChanged, () => false, onProcessed)

    handler.trigger()
    await vi.advanceTimersByTimeAsync(300)
    expect(onChanged).toHaveBeenCalledTimes(1)

    // Continuously fire events and resolve calls to test bounding
    for (let i = 0; i < 10; i++) {
      handler.trigger() // set pendingRerun
      const r = resolvers.shift()
      r?.()
      await vi.advanceTimersByTimeAsync(0)
    }

    // Should be bounded at MAX_CONSECUTIVE_RERUNS (5) + the initial = 6 max,
    // but since the initial counts as the first in the run loop, it's 5 total.
    expect(onChanged.mock.calls.length).toBeLessThanOrEqual(6)
    expect(onChanged.mock.calls.length).toBeGreaterThanOrEqual(2)
  })

  it('cancel stops pending debounce timer', async () => {
    const { handler, onChanged } = setup()

    handler.trigger()
    handler.cancel()
    await vi.advanceTimersByTimeAsync(300)
    expect(onChanged).not.toHaveBeenCalled()
  })

  it('calls onProcessed after each onChanged completes', async () => {
    const { handler, onProcessed } = setup()

    handler.trigger()
    await vi.advanceTimersByTimeAsync(300)
    expect(onProcessed).toHaveBeenCalledTimes(1)
  })

  it('handles onChanged errors gracefully and still reruns', async () => {
    let callCount = 0
    let resolvers: Array<(err?: Error) => void> = []
    const onChanged = vi.fn(() => new Promise<void>((resolve, reject) => {
      callCount++
      resolvers.push((err) => err ? reject(err) : resolve())
    }))
    const onProcessed = vi.fn()

    const handler = createQueuedHandler(onChanged, () => false, onProcessed)

    handler.trigger()
    await vi.advanceTimersByTimeAsync(300)

    // Set pending rerun, then reject the first call
    handler.trigger()
    const r = resolvers.shift()
    r?.(new Error('fail'))
    await vi.advanceTimersByTimeAsync(0)

    // Should have started the rerun despite the error
    expect(onChanged).toHaveBeenCalledTimes(2)
    expect(onProcessed).toHaveBeenCalledTimes(1) // first call still triggers onProcessed
  })
})

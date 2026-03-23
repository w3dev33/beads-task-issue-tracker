# .beads Churn Stress Test Runbook

## Overview

This document describes how to reproduce and validate that the app remains responsive under sustained `.beads` file write churn (e.g., concurrent writers, CI pipelines, rapid issue creation).

## Architecture Summary

The change detection pipeline has multiple layers of protection:

```
Filesystem events
  → Rust watcher (1s debounce, 2s min emit interval)
    → Frontend debounce (300ms)
      → Queued handler (single-flight + max 5 reruns)
        → Poll scheduler (2s backpressure gate)
          → Data fetch
```

## Key Constants

| Constant | Value | Location | Purpose |
|----------|-------|----------|---------|
| `WATCHER_DEBOUNCE_INTERVAL_MS` | 1000ms | `src-tauri/src/lib.rs` | Rust-side debounce of filesystem events |
| `WATCHER_MIN_EMIT_INTERVAL_MS` | 2000ms | `src-tauri/src/lib.rs` | Min time between emitting events to frontend (env-overridable) |
| `DEBOUNCE_MS` | 300ms | `app/composables/useChangeDetection.ts` | Frontend event debounce |
| `SELF_TRIGGER_COOLDOWN_MS` | 3000ms | `app/composables/useChangeDetection.ts` | Ignore self-writes for this long |
| `MAX_CONSECUTIVE_RERUNS` | 5 | `app/composables/useChangeDetection.ts` | Max sequential reruns per handler activation |
| `DEFAULT_MIN_INTERVAL_MS` | 2000ms | `app/composables/usePollScheduler.ts` | Backpressure gate min interval |
| `INTERVAL_ACTIVE` | 5000ms | `app/composables/useAdaptivePolling.ts` | Poll interval when active (no watcher) |
| `INTERVAL_WATCHER_SAFETY` | 30000ms | `app/composables/useAdaptivePolling.ts` | Safety-net poll when watcher is active |

## Reproduction

### Automated (CI / Local)

Run the vitest stress tests:

```bash
pnpm test -- tests/composables/churn-stress.test.ts
```

These tests simulate 500+ rapid trigger events over simulated time and verify:
- `onChanged` call count stays bounded (< 100 for 500 triggers)
- Poll execution count stays bounded (< 20 for 200 triggers over 20s)
- Pipeline converges after burst activity
- Consecutive reruns bounded by `MAX_CONSECUTIVE_RERUNS`

### Manual (with running app)

1. Start the app: `pnpm tauri:dev`
2. Open the pipeline diagnostics panel (Ctrl+Shift+D or console)
3. Run the churn script in another terminal:

```bash
# Default: 60s of writes every 50ms to .beads/
./scripts/churn-stress.sh

# Custom: 5 minutes at 20ms intervals against a specific directory
./scripts/churn-stress.sh /path/to/project/.beads 300 20
```

4. Observe the app — it should remain responsive throughout.

### Expected Metrics

Under default churn (20 writes/sec for 60s = ~1200 writes):

| Metric | Expected | Alarm threshold |
|--------|----------|-----------------|
| UI frame drops | < 5 | > 20 sustained |
| Poll executions | ~30 (≈ 60s / 2s gate) | > 60 |
| Watcher triggers (frontend) | ~60 (≈ 60s / 1s emit) | > 120 |
| Watcher cooldown skips | 0 (no self-writes) | > 10 |
| Consecutive rerun cap hits | 0 | > 5 |
| Poll avg duration | < 500ms | > 2000ms |

## Tuning Guide

### If the UI still freezes

1. **Increase `WATCHER_MIN_EMIT_INTERVAL_MS`**: Set environment variable before launching:
   ```bash
   WATCHER_MIN_EMIT_INTERVAL_MS=5000 pnpm tauri:dev
   ```
   This reduces the rate of events reaching the frontend. Min value: 250ms.

2. **Increase `DEFAULT_MIN_INTERVAL_MS`** in `usePollScheduler.ts`: Widens the backpressure gate. Try 5000ms.

3. **Decrease `MAX_CONSECUTIVE_RERUNS`** in `useChangeDetection.ts`: Reduces how many times the handler re-enters before yielding. Try 2-3.

### If changes feel slow to appear

1. **Decrease `WATCHER_MIN_EMIT_INTERVAL_MS`**: Set to 500ms for faster event delivery.
2. **Decrease `DEFAULT_MIN_INTERVAL_MS`**: Set to 1000ms for more frequent polls.
3. **Decrease `DEBOUNCE_MS`**: Set to 100ms for faster trigger response.

### Rollback

To revert all tuning to defaults, remove any environment variable overrides and restore the source constants:

```
WATCHER_DEBOUNCE_INTERVAL_MS = 1000
WATCHER_MIN_EMIT_INTERVAL_MS = 2000
DEBOUNCE_MS = 300
SELF_TRIGGER_COOLDOWN_MS = 3000
MAX_CONSECUTIVE_RERUNS = 5
DEFAULT_MIN_INTERVAL_MS = 2000
```

## Diagnostics

The pipeline diagnostics composable (`usePipelineDiagnostics`) provides real-time counters:

- `watcherTriggers` / `watcherCooldownSkips` — events reaching/filtered at frontend
- `watcherDebounces` / `watcherReruns` — debounce resets and rerun loops
- `pollRequests` / `pollExecuted` / `pollSkipped` / `pollDeferred` — scheduler decisions
- `pollAvgMs` / `pollMaxMs` — execution timing
- `mtimeChecks` / `mtimeHits` — fast change detection stats

Access via `usePipelineDiagnostics().snapshot()` in console or the diagnostics panel.

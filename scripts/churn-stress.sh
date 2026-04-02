#!/usr/bin/env bash
# churn-stress.sh — Generate controlled .beads file churn for stress testing
#
# Usage: ./scripts/churn-stress.sh [BEADS_DIR] [DURATION_SECS] [INTERVAL_MS]
#   BEADS_DIR     — path to .beads directory (default: .beads)
#   DURATION_SECS — how long to run (default: 60)
#   INTERVAL_MS   — ms between writes (default: 50)
#
# Creates a temporary churn file inside the .beads directory and writes to it
# at the specified interval. Cleans up on exit (SIGINT/SIGTERM).
#
# Expected behavior: the app should remain responsive throughout. Observe via
# the pipeline diagnostics panel (Ctrl+Shift+D) or console logs.
#
# Exit codes:
#   0 — completed successfully
#   1 — .beads directory not found

set -euo pipefail

BEADS_DIR="${1:-.beads}"
DURATION_SECS="${2:-60}"
INTERVAL_MS="${3:-50}"

if [[ ! -d "$BEADS_DIR" ]]; then
  echo "Error: .beads directory not found at '$BEADS_DIR'" >&2
  exit 1
fi

CHURN_FILE="$BEADS_DIR/.churn-stress-$$"
INTERVAL_S=$(awk "BEGIN {printf \"%.3f\", $INTERVAL_MS / 1000}")

cleanup() {
  rm -f "$CHURN_FILE"
}
trap cleanup EXIT

echo "churn-stress: writing to $CHURN_FILE every ${INTERVAL_MS}ms for ${DURATION_SECS}s"

writes=0
start_epoch=$(date +%s)
last_report=$start_epoch

while true; do
  now=$(date +%s)
  elapsed=$((now - start_epoch))

  if (( elapsed >= DURATION_SECS )); then
    break
  fi

  writes=$((writes + 1))
  echo "$(date -u +%Y-%m-%dT%H:%M:%S.%3NZ) write=$writes" > "$CHURN_FILE"

  if (( now - last_report >= 10 )); then
    if (( elapsed > 0 )); then
      rate=$(awk "BEGIN {printf \"%.1f\", $writes / $elapsed}")
    else
      rate="$writes"
    fi
    echo "  [${elapsed}s] writes=$writes rate=${rate}/s"
    last_report=$now
  fi

  sleep "$INTERVAL_S"
done

end_epoch=$(date +%s)
total_elapsed=$((end_epoch - start_epoch))
if (( total_elapsed > 0 )); then
  avg_rate=$(awk "BEGIN {printf \"%.1f\", $writes / $total_elapsed}")
else
  avg_rate="$writes"
fi

echo ""
echo "churn-stress: done"
echo "  total writes : $writes"
echo "  duration     : ${total_elapsed}s"
echo "  avg rate     : ${avg_rate}/s"

#!/usr/bin/env sh
set -eu

# Capture a focused snapshot when Copilot/Cursor shows ERR_CONNECTION_CLOSED.
#
# READ-ONLY: does not change router config.
#
# Usage:
#   sh scripts/openwrt/capture_copilot_error_snapshot.sh root@192.168.88.1 <request_id_or_note>
# Example:
#   sh scripts/openwrt/capture_copilot_error_snapshot.sh root@192.168.88.1 7fed1209-60bf-4718-8ffc-afe6b5053cec

TARGET="${1:-}"
NOTE="${2:-}"

if [ -z "$TARGET" ] || [ -z "$NOTE" ]; then
  echo "Usage: $0 root@192.168.88.1 <request_id_or_note>" >&2
  exit 2
fi

TS="$(date +%F_%H%M%S)"
OUTDIR="openwrt_audit/$TS"
mkdir -p "$OUTDIR"
echo "$TS" >"$OUTDIR/_timestamp.txt"
echo "$NOTE" >"$OUTDIR/_copilot_request_id_or_note.txt"

SSH_BASE="ssh -o ConnectTimeout=12 -o StrictHostKeyChecking=accept-new"

# 1) Quick monitor snapshot (counters + endpoint regression + filtered logread)
sh "$(dirname "$0")/monitor_stability.sh" "$TARGET" 6 --quick >/dev/null

# 2) Copy latest monitor cycle outputs into this folder for easy sharing
LATEST_MON=""
for d in $(ls -1t openwrt_audit 2>/dev/null | head -n 30); do
  if [ -f "openwrt_audit/$d/_monitor_run.txt" ]; then
    LATEST_MON="$d"
    break
  fi
done

if [ -n "$LATEST_MON" ] && [ -d "openwrt_audit/$LATEST_MON/cycle_1" ]; then
  cp -f "openwrt_audit/$LATEST_MON/cycle_1/04_counter_growth_6min.txt" "$OUTDIR/04_counter_growth_6min.txt" 2>/dev/null || true
  cp -f "openwrt_audit/$LATEST_MON/cycle_1/06_endpoint_regression_ipv4.txt" "$OUTDIR/06_endpoint_regression_ipv4.txt" 2>/dev/null || true
  cp -f "openwrt_audit/$LATEST_MON/cycle_1/07_logread_filtered_tail.txt" "$OUTDIR/07_logread_filtered_tail.txt" 2>/dev/null || true
  cp -f "openwrt_audit/$LATEST_MON/cycle_1/08_nft_key_counters.txt" "$OUTDIR/08_nft_key_counters.txt" 2>/dev/null || true
  cp -f "openwrt_audit/$LATEST_MON/cycle_1/08_nft_key_rules_raw.txt" "$OUTDIR/08_nft_key_rules_raw.txt" 2>/dev/null || true
fi

# 3) Extra: OpenClash log tail
$SSH_BASE "$TARGET" 'tail -n 220 /tmp/openclash.log 2>/dev/null || true' >"$OUTDIR/10_openclash_log_tail.txt" 2>/dev/null || true

echo "Wrote $OUTDIR"

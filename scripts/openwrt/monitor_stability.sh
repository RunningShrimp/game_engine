#!/usr/bin/env sh
set -eu

# Long-run stability monitor for Copilot/Cursor on OpenWrt.
#
# READ-ONLY: does not change router config, does not install/remove packages,
# and does not touch qdisc.
#
# What it does per cycle:
# - Runs the 6-minute nft/OpenClash counter sampler
# - Runs IPv4-only endpoint regression (GitHub + Copilot)
# - Checks router uptime before/after; if it decreases, treats as reboot and
#   exports pstore/ramoops logs into the same cycle folder.
#
# Usage:
#   sh scripts/openwrt/monitor_stability.sh root@192.168.88.1 [minutes]
#
# Examples:
#   sh scripts/openwrt/monitor_stability.sh root@192.168.88.1 60
#   sh scripts/openwrt/monitor_stability.sh root@192.168.88.1 180

TARGET="${1:-}"
MINUTES="${2:-60}"

if [ -z "$TARGET" ]; then
  echo "Usage: $0 root@192.168.88.1 [minutes]" >&2
  exit 2
fi

case "$MINUTES" in
  ''|*[!0-9]*)
    echo "minutes must be an integer (got: $MINUTES)" >&2
    exit 2
    ;;
esac

SSH_BASE="ssh -o ConnectTimeout=12 -o StrictHostKeyChecking=accept-new"
SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"

TS="$(date +%F_%H%M%S)"
OUTROOT="openwrt_audit/$TS"
mkdir -p "$OUTROOT"
echo "$TS" >"$OUTROOT/_timestamp.txt"

# Each cycle is approximately 6 minutes (counter sampler sleeps 2m twice).
CYCLES=$(( (MINUTES + 5) / 6 ))
if [ "$CYCLES" -lt 1 ]; then
  CYCLES=1
fi

echo "[monitor] target=$TARGET minutes=$MINUTES cycles=$CYCLES"
echo "[monitor] output=$OUTROOT"

i=1
while [ "$i" -le "$CYCLES" ]; do
  CYCLE_DIR="$OUTROOT/cycle_$i"
  mkdir -p "$CYCLE_DIR"

  echo "[cycle $i/$CYCLES] starting $(date -u '+%F %T UTC')"

  # record uptime before
  UPTIME_BEFORE="$($SSH_BASE "$TARGET" 'cut -d. -f1 /proc/uptime 2>/dev/null || echo 0' 2>/dev/null || echo 0)"
  echo "uptime_before_s=$UPTIME_BEFORE" >"$CYCLE_DIR/00_uptime.txt"

  # counter sampler (writes its own timestamped folder); copy into our cycle folder
  sh "$SCRIPT_DIR/counter_growth_6min.sh" "$TARGET" >/dev/null
  LATEST_SAMPLE_DIR="$(ls -1t openwrt_audit 2>/dev/null | head -n 1)"
  if [ -n "$LATEST_SAMPLE_DIR" ] && [ -f "openwrt_audit/$LATEST_SAMPLE_DIR/04_counter_growth_6min.txt" ]; then
    cp -f "openwrt_audit/$LATEST_SAMPLE_DIR/04_counter_growth_6min.txt" "$CYCLE_DIR/04_counter_growth_6min.txt"
  fi

  # endpoint regression
  sh "$SCRIPT_DIR/endpoint_regression_ipv4.sh" "$TARGET" >"$CYCLE_DIR/06_endpoint_regression_ipv4.txt" 2>&1 || true

  # record uptime after + detect reboot
  UPTIME_AFTER="$($SSH_BASE "$TARGET" 'cut -d. -f1 /proc/uptime 2>/dev/null || echo 0' 2>/dev/null || echo 0)"
  echo "uptime_after_s=$UPTIME_AFTER" >>"$CYCLE_DIR/00_uptime.txt"

  if [ "$UPTIME_AFTER" -gt 0 ] && [ "$UPTIME_BEFORE" -gt 0 ] && [ "$UPTIME_AFTER" -lt "$UPTIME_BEFORE" ]; then
    echo "[cycle $i] detected reboot (uptime decreased: $UPTIME_BEFORE -> $UPTIME_AFTER); exporting pstore" | tee "$CYCLE_DIR/05_reboot_detected.txt" >/dev/null
    $SSH_BASE "$TARGET" '
      echo "=== uptime"; uptime; echo
      echo "=== pstore list"; ls -l /sys/fs/pstore 2>/dev/null || true; echo
      for f in /sys/fs/pstore/dmesg-ramoops-0 /sys/fs/pstore/dmesg-ramoops-1 /sys/fs/pstore/console-ramoops-0; do
        echo "--- $(basename "$f") ---";
        if [ -e "$f" ]; then sed -n "1,360p" "$f"; else echo "(missing)"; fi
        echo
      done
    ' >"$CYCLE_DIR/05_pstore_reboot_logs.txt" 2>/dev/null || true
  fi

  echo "[cycle $i/$CYCLES] done $(date -u '+%F %T UTC')"
  i=$((i + 1))
done

echo "[monitor] complete: $OUTROOT"

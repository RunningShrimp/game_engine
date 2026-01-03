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
#   sh scripts/openwrt/monitor_stability.sh root@192.168.88.1 [minutes] [--quick]
#
# Examples:
#   sh scripts/openwrt/monitor_stability.sh root@192.168.88.1 60
#   sh scripts/openwrt/monitor_stability.sh root@192.168.88.1 180

TARGET="${1:-}"
MINUTES="${2:-60}"
MODE="${3:-}"

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

QUICK=0
if [ "$MODE" = "--quick" ] || [ "${QUICK:-0}" = "1" ]; then
  QUICK=1
fi

echo "[monitor] target=$TARGET minutes=$MINUTES cycles=$CYCLES"
echo "[monitor] output=$OUTROOT"

i=1
while [ "$i" -le "$CYCLES" ]; do
  CYCLE_DIR="$OUTROOT/cycle_$i"
  mkdir -p "$CYCLE_DIR"

  PREV_DIR=""
  if [ "$i" -gt 1 ]; then
    PREV_DIR="$OUTROOT/cycle_$((i - 1))"
  fi

  echo "[cycle $i/$CYCLES] starting $(date -u '+%F %T UTC')"

  # record uptime before
  UPTIME_BEFORE="$($SSH_BASE "$TARGET" 'cut -d. -f1 /proc/uptime 2>/dev/null || echo 0' 2>/dev/null || echo 0)"
  echo "uptime_before_s=$UPTIME_BEFORE" >"$CYCLE_DIR/00_uptime.txt"

  # counter sampler (writes its own timestamped folder); copy into our cycle folder
  if [ "$QUICK" -eq 1 ]; then
    SAMPLES=1 INTERVAL_SEC=0 sh "$SCRIPT_DIR/counter_growth_6min.sh" "$TARGET" >/dev/null
  else
    sh "$SCRIPT_DIR/counter_growth_6min.sh" "$TARGET" >/dev/null
  fi
  LATEST_SAMPLE_DIR="$(ls -1t openwrt_audit 2>/dev/null | head -n 1)"
  if [ -n "$LATEST_SAMPLE_DIR" ] && [ -f "openwrt_audit/$LATEST_SAMPLE_DIR/04_counter_growth_6min.txt" ]; then
    cp -f "openwrt_audit/$LATEST_SAMPLE_DIR/04_counter_growth_6min.txt" "$CYCLE_DIR/04_counter_growth_6min.txt"
  fi

  # endpoint regression
  sh "$SCRIPT_DIR/endpoint_regression_ipv4.sh" "$TARGET" >"$CYCLE_DIR/06_endpoint_regression_ipv4.txt" 2>&1 || true

  # filtered logs (help correlate disconnects with conntrack/Oops/panic/OpenClash)
  $SSH_BASE "$TARGET" '
    logread 2>/dev/null | tail -n 1200 | grep -i -E "(ct state invalid|nf_conntrack|conntrack|Connection reset|reset by peer|panic|Oops|Unable to handle kernel paging request|call trace|softirq|inet_diag|openclash)" | tail -n 300 || true
  ' >"$CYCLE_DIR/07_logread_filtered_tail.txt" 2>/dev/null || true

  # key nft counters snapshot + delta (OpenClash + ct invalid drop)
  $SSH_BASE "$TARGET" '
    set -e
    echo "ct_invalid_drop:"
    nft -a list chain inet fw4 accept_to_wan 2>/dev/null | grep -F "ct state invalid" || true
    echo
    echo "openclash_quic_reject:"
    nft -a list ruleset 2>/dev/null | grep -F "OpenClash QUIC REJECT" || true
    echo
    echo "openclash_dns_hijack:"
    nft -a list ruleset 2>/dev/null | grep -F "OpenClash DNS Hijack" || true
  ' >"$CYCLE_DIR/08_nft_key_rules_raw.txt" 2>/dev/null || true

  awk '
    function emit(k, p, b) { if (k != "") print k " packets=" p " bytes=" b }
    /^ct_invalid_drop:/ { section="ct_invalid_drop"; next }
    /^openclash_quic_reject:/ { section="openclash_quic_reject"; next }
    /^openclash_dns_hijack:/ { section="openclash_dns_hijack"; next }
    {
      if (section == "") next
      # Extract "counter packets N bytes M" occurrences
      p = b = ""
      if (match($0, /counter packets [0-9]+ bytes [0-9]+/)) {
        s = substr($0, RSTART, RLENGTH)
        gsub("counter packets ", "", s)
        split(s, a, " bytes ")
        p = a[1]
        b = a[2]
      } else {
        next
      }

      # Derive a stable key per rule line:
      # - for ct_invalid_drop: single key
      # - for openclash_*: prefer comment string
      k = section
      if (section ~ /^openclash_/) {
        if (index($0, "OpenClash QUIC REJECT") > 0) k = "openclash_quic_reject"
        else if (index($0, "OpenClash DNS Hijack") > 0) k = "openclash_dns_hijack"
      }
      emit(k, p, b)
    }
  ' "$CYCLE_DIR/08_nft_key_rules_raw.txt" >"$CYCLE_DIR/08_nft_key_counters.txt" 2>/dev/null || true

  if [ -n "$PREV_DIR" ] && [ -f "$PREV_DIR/08_nft_key_counters.txt" ] && [ -f "$CYCLE_DIR/08_nft_key_counters.txt" ]; then
    awk '
      function getv(line, key) {
        if (match(line, key "=[0-9]+")) return substr(line, RSTART + length(key) + 1, RLENGTH - length(key) - 1)
        return 0
      }
      FNR==NR {
        key=$1
        prev_p[key]=getv($0,"packets")
        prev_b[key]=getv($0,"bytes")
        next
      }
      {
        key=$1
        cur_p=getv($0,"packets")
        cur_b=getv($0,"bytes")
        dp=cur_p - (key in prev_p ? prev_p[key] : 0)
        db=cur_b - (key in prev_b ? prev_b[key] : 0)
        print key " d_packets=" dp " d_bytes=" db
      }
    ' "$PREV_DIR/08_nft_key_counters.txt" "$CYCLE_DIR/08_nft_key_counters.txt" >"$CYCLE_DIR/09_nft_key_counters_delta.txt" 2>/dev/null || true
  fi

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

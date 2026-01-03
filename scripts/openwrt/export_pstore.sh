#!/usr/bin/env sh
set -eu

# Export ramoops/pstore logs to a timestamped local folder.
# Usage: sh scripts/openwrt/export_pstore.sh root@192.168.88.1

TARGET="${1:-}"
if [ -z "$TARGET" ]; then
  echo "Usage: $0 root@192.168.88.1" >&2
  exit 2
fi

TS="$(date +%F_%H%M%S)"
OUTDIR="openwrt_audit/$TS"
mkdir -p "$OUTDIR"
echo "$TS" >"$OUTDIR/_timestamp.txt"

SSH_BASE="ssh -o ConnectTimeout=12 -o StrictHostKeyChecking=accept-new"

$SSH_BASE "$TARGET" '
  echo "=== uptime"
  uptime
  echo
  echo "=== pstore list"
  ls -l /sys/fs/pstore 2>/dev/null || true
  echo
  for f in /sys/fs/pstore/dmesg-ramoops-0 /sys/fs/pstore/dmesg-ramoops-1 /sys/fs/pstore/console-ramoops-0; do
    echo "--- $(basename "$f") ---"
    if [ -e "$f" ]; then
      sed -n "1,320p" "$f"
    else
      echo "(missing)"
    fi
    echo
  done
' >"$OUTDIR/05_pstore_reboot_logs.txt"

echo "Wrote $OUTDIR/05_pstore_reboot_logs.txt"
wc -l "$OUTDIR/05_pstore_reboot_logs.txt" >/dev/null || true

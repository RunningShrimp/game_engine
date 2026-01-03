#!/usr/bin/env sh
set -eu

# Package CAKE kernel panic evidence into a single tarball.
#
# This script is intentionally READ-ONLY: it does not change router config,
# does not install/remove packages, and does not apply any qdisc.
#
# Usage:
#   sh scripts/openwrt/package_cake_panic_report.sh root@192.168.88.1
#
# Output:
#   openwrt_audit/<timestamp>/cake_panic_report.tar.gz

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

# 1) Export ramoops/pstore logs (crash evidence)
sh "$(dirname "$0")/export_pstore.sh" "$TARGET" >/dev/null

# 2) Capture system/context info helpful for upstream triage
$SSH_BASE "$TARGET" '
  set -e
  echo "=== date"
  date
  echo

  echo "=== uname -a"
  uname -a
  echo

  echo "=== /etc/openwrt_release"
  cat /etc/openwrt_release 2>/dev/null || true
  echo

  echo "=== /etc/os-release"
  cat /etc/os-release 2>/dev/null || true
  echo

  echo "=== board"
  ubus call system board 2>/dev/null || true
  echo

  echo "=== uptime"
  uptime
  echo

  echo "=== memory"
  free -h 2>/dev/null || free 2>/dev/null || true
  echo

  echo "=== tc qdisc (pppoe-wan/wan/eth0/eth1/br-lan)"
  for dev in pppoe-wan wan eth0 eth1 br-lan; do
    echo "--- $dev"
    tc qdisc show dev "$dev" 2>/dev/null || true
  done
  echo

  echo "=== lsmod (cake/offload related excerpt)"
  lsmod 2>/dev/null | grep -E "^(sch_cake|sch_fq_codel|sch_htb|sch_tbf|act_|cls_|nft_|nf_|mtk_|pppoe)" || true
  echo

  echo "=== opkg list-installed (cake/offload related excerpt)"
  opkg list-installed 2>/dev/null | grep -E "^(kmod-sched-cake|tc|ip-full|kmod-nf-flow|kmod-nft-offload|kmod-mtk-hnat|sqm-scripts|luci-app-sqm|openclash)" || true
  echo

  echo "=== firewall defaults"
  uci -q show firewall.@defaults[0] 2>/dev/null || true
  echo

  echo "=== dhcp dnsmasq"
  uci -q show dhcp.@dnsmasq[0] 2>/dev/null || true
  echo

  echo "=== network wan"
  uci -q show network.wan 2>/dev/null || true
  echo

  echo "=== nft accept_to_wan excerpt"
  nft -a list chain inet fw4 accept_to_wan 2>/dev/null | sed -n "1,220p" || true
  echo

  echo "=== pstore list"
  ls -l /sys/fs/pstore 2>/dev/null || true
' >"$OUTDIR/07_system_context.txt" || true

# 3) Add local repro/notes templates
if [ -f docs/openwrt_cake_kernel_panic_bug_report.md ]; then
  cp -f docs/openwrt_cake_kernel_panic_bug_report.md "$OUTDIR/bug_report_template.md"
fi

if [ -f docs/openwrt_copilot_cursor_network_audit_2026-01-02.md ]; then
  cp -f docs/openwrt_copilot_cursor_network_audit_2026-01-02.md "$OUTDIR/audit_notes.md"
fi

cat >"$OUTDIR/README.txt" <<'EOF'
This folder packages evidence for a CAKE (sch_cake) kernel panic on OpenWrt.

Key files:
- 05_pstore_reboot_logs.txt   Crash logs exported from /sys/fs/pstore (ramoops)
- 07_system_context.txt       System/kernel/network context for triage
- bug_report_template.md      Fill-in template for upstream report (optional)
- audit_notes.md              Local audit notes / change log (optional)

Tip:
- If multiple crashes happened, attach multiple folders/exports.
EOF

# 4) Create a single tarball to attach to issues/threads
TARBALL="$OUTDIR/cake_panic_report.tar.gz"
(
  cd "$OUTDIR"
  tar -czf "cake_panic_report.tar.gz" \
    _timestamp.txt \
    README.txt \
    05_pstore_reboot_logs.txt \
    07_system_context.txt \
    bug_report_template.md \
    audit_notes.md \
    2>/dev/null || tar -czf "cake_panic_report.tar.gz" \
    _timestamp.txt \
    README.txt \
    05_pstore_reboot_logs.txt \
    07_system_context.txt
)

echo "Wrote $TARBALL"

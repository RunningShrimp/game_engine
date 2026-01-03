#!/usr/bin/env sh
set -eu

# Capture 3 snapshots (2 minutes apart) of key fw4/OpenClash counters.
# Usage: sh scripts/openwrt/counter_growth_6min.sh root@192.168.88.1

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

{
  i=1
  while [ $i -le 3 ]; do
    echo "===== SAMPLE $i $(date -u '+%F %T UTC') ====="
    $SSH_BASE "$TARGET" '
      echo "TS_EPOCH=$(date +%s)"
      echo "uptime=$(cut -d. -f1 /proc/uptime 2>/dev/null || true)s"
      echo "flow_offloading=$(uci -q get firewall.@defaults[0].flow_offloading || echo ?)"
      echo "flow_offloading_hw=$(uci -q get firewall.@defaults[0].flow_offloading_hw || echo ?)"
      echo "openclash_enable=$(uci -q get openclash.config.enable || echo ?)"
      echo "filter_aaaa=$(uci -q get dhcp.@dnsmasq[0].filter_aaaa || echo ?)"
      echo "conntrack_count=$(cat /proc/sys/net/netfilter/nf_conntrack_count 2>/dev/null || echo ?)"
      echo "conntrack_max=$(cat /proc/sys/net/netfilter/nf_conntrack_max 2>/dev/null || echo ?)"
      echo "--- ct_invalid_drop (accept_to_wan excerpt) ---"
      nft -a list chain inet fw4 accept_to_wan 2>/dev/null | sed -n "1,160p"
      echo "--- openclash_key_rules ---"
      nft -a list ruleset 2>/dev/null | grep -n -E "OpenClash QUIC REJECT|OpenClash DNS Hijack" || true
      echo "--- openclash_chains (head) ---"
      for c in openclash openclash_output openclash_mangle openclash_mangle_output; do
        echo "[chain $c]"
        nft -a list chain inet fw4 "$c" 2>/dev/null | sed -n "1,60p"
      done
    '
    echo

    if [ $i -lt 3 ]; then
      sleep 120
    fi

    i=$((i + 1))
  done
} >"$OUTDIR/04_counter_growth_6min.txt"

wc -l "$OUTDIR/04_counter_growth_6min.txt" >/dev/null || true

echo "Wrote $OUTDIR/04_counter_growth_6min.txt"

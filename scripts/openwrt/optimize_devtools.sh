#!/usr/bin/env sh
set -eu

# OpenWrt: stabilize network for dev tools (Copilot/Cursor)
# Target profile: single-WAN + OpenClash (transparent proxy).
# Applies: disable mwan3, disable flow offload, make DNS predictable.
# Run: sh optimize_devtools.sh root@192.168.88.1

TARGET="${1:-}"
if [ -z "$TARGET" ]; then
  echo "Usage: $0 root@192.168.88.1"
  exit 2
fi

ssh_run() {
  ssh -o ConnectTimeout=5 -o StrictHostKeyChecking=accept-new "$TARGET" "$@"
}

echo "[1/4] Checking reachability..."
ssh_run 'echo OK; uname -a' >/dev/null

echo "[1.5/4] Guardrail: ensure CAKE (sch_cake) is not installed/loaded..."
ssh_run '
  if lsmod 2>/dev/null | grep -q "^sch_cake"; then
    echo "sch_cake is loaded; attempting unload to avoid kernel panic";
    rmmod sch_cake 2>/dev/null || true;
  else
    echo "OK: sch_cake not loaded";
  fi
  if opkg list-installed 2>/dev/null | grep -q "^kmod-sched-cake "; then
    echo "kmod-sched-cake is installed; removing to avoid kernel panic";
    opkg remove kmod-sched-cake 2>/dev/null || true;
  else
    echo "OK: kmod-sched-cake not installed";
  fi
'

echo "[2/4] Disabling mwan3 (recommended when single-WAN)..."
ssh_run '/etc/init.d/mwan3 stop 2>/dev/null || true; /etc/init.d/mwan3 disable 2>/dev/null || true; echo "mwan3 status:"; /etc/init.d/mwan3 status 2>/dev/null || true'

echo "[3/4] Disabling firewall flow offload (OpenClash-friendly)..."
ssh_run "uci set firewall.@defaults[0].flow_offloading='0' && \
         uci set firewall.@defaults[0].flow_offloading_hw='0' && \
         \
         uci -q delete firewall.docker_nat.extra || true; \
         uci -q delete firewall.zerotier.reload || true; \
         uci -q delete firewall.socat.reload || true; \
         \
         if [ ! -e /var/etc/zerotier.include ] && [ -e /usr/share/nftables.d/table-post/20-zerotier.nft ]; then uci -q delete firewall.zerotier || true; fi; \
         if [ ! -e /var/etc/socat.include ] && [ -e /usr/share/nftables.d/table-post/20-socat.nft ]; then uci -q delete firewall.socat || true; fi; \
         \
         uci commit firewall && /etc/init.d/firewall restart"

echo "[3.5/4] Making DNS predictable (disable peerdns + dnsmasq noresolv)..."
ssh_run "uci set network.wan.peerdns='0' && uci -q delete network.wan.dns && uci add_list network.wan.dns='223.5.5.5' && uci add_list network.wan.dns='119.29.29.29' && uci commit network && \
         uci set dhcp.@dnsmasq[0].noresolv='1' && \
         uci set dhcp.@dnsmasq[0].filter_aaaa='1' && \
         uci commit dhcp && \
         (/etc/init.d/network restart || true) && /etc/init.d/dnsmasq restart"

echo "[3.8/4] Restarting OpenClash (if enabled)..."
ssh_run 'if [ "$(uci -q get openclash.config.enable 2>/dev/null || echo 0)" = "1" ]; then /etc/init.d/openclash restart || true; else echo "OpenClash not enabled, skip"; fi'

echo "[3.85/4] OpenClash devtools stability (avoid forced connection closes)..."
ssh_run '
  if [ "$(uci -q get openclash.config.enable 2>/dev/null || echo 0)" = "1" ]; then
    # OpenClash's streaming auto-select can run periodically and (when configured)
    # forcibly close connections, which surfaces as net::ERR_CONNECTION_CLOSED in browsers.
    # For dev tools stability, avoid forced closes and avoid touching OpenAI/Copilot flows.
    uci set openclash.config.stream_auto_select_close_con="0" 2>/dev/null || true
    uci set openclash.config.stream_auto_select_openai="0" 2>/dev/null || true

    # The periodic streaming auto-select tasks themselves can also trigger config writes
    # and OpenClash restarts, which will drop long-lived HTTP/2/WebSocket sessions.
    # For stability (Copilot/Cursor), disable these background tasks.
    uci set openclash.config.stream_auto_select="0" 2>/dev/null || true
    uci set openclash.config.auto_restart="0" 2>/dev/null || true
    uci set openclash.config.smart_auto_switch="0" 2>/dev/null || true
    uci set openclash.config.auto_smart_switch="0" 2>/dev/null || true
    uci commit openclash 2>/dev/null || true
    /etc/init.d/openclash restart >/dev/null 2>&1 || true
    echo "Applied: stream_auto_select=0, auto_restart=0, smart_auto_switch=0, auto_smart_switch=0, stream_auto_select_close_con=0, stream_auto_select_openai=0"
  else
    echo "OpenClash not enabled, skip"
  fi
'

echo "[3.9/4] Checking for noisy/broken diag kmods (non-invasive)..."
ssh_run 'if opkg list-installed 2>/dev/null | grep -q "^kmod-inet-diag "; then echo "NOTE: kmod-inet-diag is installed. If dmesg shows inet_diag Unknown symbol sock_diag_* spam, consider: opkg remove kmod-inet-diag"; else echo "OK: kmod-inet-diag not installed"; fi'

echo "[4/4] Quick sanity snapshots..."
ssh_run 'ip -4 rule show | head -n 30; echo; nft list flowtable inet fw4 ft 2>/dev/null || true; echo; nft -a list chain inet fw4 accept_to_wan 2>/dev/null | sed -n "1,120p" || true'

echo "Done. See docs/openwrt_copilot_cursor_network_audit_2026-01-02.md for rationale and follow-ups (SQM/CAKE, etc.)."

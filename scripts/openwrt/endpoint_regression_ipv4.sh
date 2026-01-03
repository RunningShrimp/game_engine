#!/usr/bin/env sh
set -eu

# Quick IPv4-only endpoint regression from the router.
# Usage: sh scripts/openwrt/endpoint_regression_ipv4.sh root@192.168.88.1

TARGET="${1:-}"
if [ -z "$TARGET" ]; then
  echo "Usage: $0 root@192.168.88.1" >&2
  exit 2
fi

SSH_BASE="ssh -o ConnectTimeout=12 -o StrictHostKeyChecking=accept-new"

for url in https://api.github.com https://api.githubcopilot.com; do
  ok=0
  fail=0
  i=1
  while [ $i -le 10 ]; do
    if $SSH_BASE "$TARGET" "curl -4 -I -m 8 -sS $url >/dev/null"; then
      ok=$((ok + 1))
    else
      fail=$((fail + 1))
    fi
    i=$((i + 1))
  done
  echo "$url ok=$ok fail=$fail"
done

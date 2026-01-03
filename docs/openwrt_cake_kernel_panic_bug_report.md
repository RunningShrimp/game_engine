# Bug report template: CAKE (sch_cake) triggers kernel panic on BeeconMini SEED AC5

## Summary

On OpenWrt 24.10.3 (mediatek/filogic, kernel 6.6.104) running on **BeeconMini SEED AC5**, loading and/or applying CAKE (`sch_cake`) can trigger a kernel Oops followed by a panic and automatic reboot.

This blocks SQM/CAKE deployment and causes instability (SSH drops, uptime resets).

## Device / software

- Device: BeeconMini SEED AC5
- OpenWrt: 24.10.3 r0-3f6fc3b7
- Target: mediatek/filogic
- Arch: aarch64_cortex-a53
- Kernel: 6.6.104

## How to reproduce (minimal)

1) Install CAKE kernel module and tc:

```sh
opkg update
opkg install tc-full kmod-sched-cake
```

2) Apply CAKE to WAN interface (`pppoe-wan`), e.g.:

```sh
tc qdisc replace dev pppoe-wan root cake diffserv4 nat ack-filter
```

3) Observe: within seconds to minutes, the router may Oops/panic and reboot.

## Expected

- CAKE qdisc attaches and stays active
- No kernel crash

## Actual

- Kernel Oops (paging request) and panic
- SSH disconnects with `Connection reset by peer`
- Uptime resets after reboot

## Crash evidence (ramoops)

- openwrt_audit/2026-01-02_215111/05_pstore_reboot_logs.txt
  - Oops/panic around netdevice unregister path (`unregister_netdevice_many_notify`), triggered by process `ip`
- openwrt_audit/2026-01-02_220730/05_pstore_reboot_logs.txt
  - Oops/panic with `sch_cake` present in module list; call trace in softirq/forwarding path (`__dev_queue_xmit`, `ip_forward`, `br_nf_hook_thresh`, etc)

## One-command evidence bundle

To collect crash logs + key system context into a single attachment:

```sh
sh scripts/openwrt/package_cake_panic_report.sh root@192.168.88.1
```

This produces `openwrt_audit/<timestamp>/cake_panic_report.tar.gz`.

## Notes

- This is reproducible on the current firmware/kernel combination.
- If you need more detail, request full pstore files (not truncated) and exact interface config.

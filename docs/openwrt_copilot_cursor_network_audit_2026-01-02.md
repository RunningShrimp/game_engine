# OpenWrt 网络优化审计（面向 Copilot / Cursor 稳定性）

审计对象：OpenWrt 24.10.3（BeeconMini SEED AC5，PPPoE）

数据来源：本地目录 `openwrt_audit/` 中通过 SSH 采集的配置与运行态快照。

辅助脚本（本仓库内，可重复运行采样/导出日志）：

- `sh scripts/openwrt/counter_growth_6min.sh root@192.168.88.1`
- `sh scripts/openwrt/export_pstore.sh root@192.168.88.1`
- `sh scripts/openwrt/endpoint_regression_ipv4.sh root@192.168.88.1`
- `sh scripts/openwrt/package_cake_panic_report.sh root@192.168.88.1`
- `sh scripts/openwrt/monitor_stability.sh root@192.168.88.1 60`

## 0. 本次已落地的变更（单 WAN + OpenClash）

- 已停用 `mwan3`（单 WAN 不需要多线路探测/策略路由，避免误判导致连接被切断）
- 已关闭 `flow_offloading` 与 `flow_offloading_hw`（OpenClash 透明代理 + 策略标记下更稳）
- 已固定 DNS：`network.wan.peerdns=0`，dnsmasq `noresolv=1`，上游仅使用 `223.5.5.5` / `119.29.29.29`
- 已启用并启动 OpenClash（核心进程已运行；透明代理规则由 nftables 下发）

## 1. 关键现状摘要

- WAN：`pppoe-wan`（IPv4 PPPoE），防火墙开启 `mtu_fix=1`
- 防火墙：`flow_offloading=0` 且 `flow_offloading_hw=0`（为 OpenClash 透明代理稳定性优先）
- NAT：`fullcone=2`（全锥形 NAT）
- 多拨/多 WAN：`mwan3` 已停用（单 WAN 场景避免策略路由误判/黑洞）
- QoS/SQM：出口使用默认 `fq_codel`；当前固件/内核下 **不要启用 CAKE（`sch_cake`）**（会触发 panic 重启）
- DNS：`network.wan.peerdns=0`，dnsmasq `noresolv=1`，并启用 `filter_aaaa=1`（规避异常 IPv6 路径导致的连通性问题）
- OpenClash：`openclash.config.enable='1'`（已启用；DNS 上游指向本地 127.0.0.1#7874）

## 2. 发现的问题（与 Copilot/Cursor 的 “net::ERR_CONNECTION_CLOSED” 高相关）

### P0：mwan3 造成的链路误判与路由抖动

- 日志中出现：`mwan3track: Check (ping) failed ... on interface wan (pppoe-wan)`
- 系统存在路由策略：
  - `fwmark ... blackhole / unreachable`
  - 一旦 mwan3 判定链路 down，可能会将部分流量标记为 unreachable，表现为连接被中断/立即失败

这类“连接被关闭/重置”在 VS Code Copilot、Cursor（长连接/WebSocket/HTTP2）上非常典型：网络并未完全断网，但特定连接会被路由策略切断。

### P0：转发流量存在大量 `ct state invalid` 丢弃计数

在 fw4 规则中：`ct state invalid ... drop (!fw4: Prevent NAT leakage)` 计数较高。

这通常由以下因素触发：
- 流量卸载（flow offload / hw offload）与复杂策略路由（mwan3）叠加
- 异步路由/回程路径变化导致 conntrack 认为“无效”

该丢弃会造成连接在中途被重置，表现为浏览器/扩展报 `ERR_CONNECTION_CLOSED`。

### P1：网络“优化”sysctl 配置重复/冲突，存在不可控性

`/etc/sysctl.conf` 与 `/etc/sysctl.d/*.conf` 存在大量重复与互相覆盖。
虽然当前运行态里多数关键 TCP 重试参数仍接近默认值，但这种配置状态非常不利于后续排障：
- 你很难确认某次重启后最终生效的是哪组参数
- 一旦某些“激进缩短重试/超时”的参数生效，会直接导致 TLS/HTTP2/WebSocket 更容易断

### P2：缺少 SQM/CAKE（没有 `tc`）

这不会“直接”导致连接被关闭，但会在高负载时产生 bufferbloat，让交互式/长连接更易抖动。
对 Copilot/Cursor 这类实时流量，SQM 通常是最有效的稳定性提升手段之一。

## 3. 建议的修复优先级（先稳定，再追求吞吐）

### 3.1 立即执行（低风险、对稳定性提升最大）

1) 若你当前只有一条 WAN（没有实际用到多 WAN 负载/故障切换）：**停用 mwan3**

```sh
/etc/init.d/mwan3 stop
/etc/init.d/mwan3 disable
```

验证：
- `mwan3 status` 应显示 stopped
- `ip rule` 中不再出现 mwan3 的 `fwmark ... blackhole/unreachable` 规则

2) 关闭“硬件流量卸载”（先排除不稳定因素）

```sh
uci set firewall.@defaults[0].flow_offloading_hw='0'
uci commit firewall
/etc/init.d/firewall restart
```

如果问题仍存在，可进一步临时关闭软件 flow offload：

```sh
uci set firewall.@defaults[0].flow_offloading='0'
uci commit firewall
/etc/init.d/firewall restart
```

验证：
- `nft list ruleset | grep -n "flowtable"`（flowtable 可能消失或不再被使用）
- 观察 `ct state invalid drop` 计数增长速度是否显著下降

### 3.2 次优先（把 DNS 路径变得可控）

避免使用 PPPoE peer 下发的 DNS（很多地区 8.8.8.8:53 会被干扰/阻断）：

```sh
uci set network.wan.peerdns='0'
uci -q delete network.wan.dns
uci add_list network.wan.dns='223.5.5.5'
uci add_list network.wan.dns='119.29.29.29'
uci commit network
/etc/init.d/network restart
```

如果你启用 OpenClash 并使用其 DNS 方案，则应以 OpenClash 的 DNS 设计为准（避免 dnsmasq 与 OpenClash DNS 互相打架）。

### 3.3 长期优化（降低延迟抖动：SQM/CAKE）

SQM/CAKE 通常是提升交互稳定性（尤其是上传打满时）的“终极手段”。

但：在当前固件/内核上，我们已经复现到 **CAKE（`sch_cake`）会触发 kernel Oops → panic → 自动重启**（详见 6.7 的 ramoops 证据，且不仅发生在“安装阶段”，也发生在运行阶段）。

因此：在根因明确或固件升级前，**不建议启用 CAKE / 安装 SQM 全家桶 / 使用 IFB ingress 整形**。

当前可用的“保守策略”是：

- 维持默认 `fq_codel`（系统默认的 qdisc 已经能缓解部分 bufferbloat）
- 等固件/内核修复后再上 SQM/CAKE

快速确认当前出口 qdisc：

```sh
tc qdisc show dev pppoe-wan
```

如果你此前安装过 `kmod-sched-cake`，建议卸载并确保模块未加载（避免误触发）：

```sh
opkg remove kmod-sched-cake
rmmod sch_cake 2>/dev/null || true
```

## 4. Copilot / Cursor 专项实施计划

### 4.1 目标

- 让 VS Code Copilot、Cursor 的 HTTPS / HTTP2 / WebSocket 长连接不再被中途切断
- 让 DNS/路由路径可预测、可复现

### 4.2 分阶段步骤（建议按顺序）

**阶段 A（消除“路由抖动/误判”）：**
- 停用 mwan3（或至少把 `track_method` 从 ping 改为更可靠的检测，并拉长 timeout/interval）

**阶段 B（消除“卸载 + conntrack invalid”）：**
- 关闭 `flow_offloading_hw`
- 若仍有大量 invalid drop，再考虑关闭 `flow_offloading`

**阶段 C（DNS 可控）：**
- 禁用 `peerdns`，明确指定 DNS（或部署 DoH/SmartDNS）

**阶段 D（延迟抖动治理）：**
- 部署 SQM/CAKE 并关闭 offload

### 4.3 验证方法（建议在 Mac 上执行）

- 基本连通：
  - `curl -I https://api.github.com`
  - `curl -I https://api.githubcopilot.com`
- 稳定性：重复运行以上命令 20~50 次，观察是否出现随机失败
- 长连接：使用 Cursor / Copilot 正常交互 10~20 分钟，看是否还会随机断连

## 6. 继续排障：建议补采的日志/计数器（用于定位“连接被关闭”的根因）

> 由于 OpenClash/防火墙重启期间可能出现 SSH 暂时不可用，优先建议用 LuCI 的 Web 终端（`ttyd`）执行下面命令：
> - 浏览器打开 `http://192.168.88.1:7681`（或 LuCI 内的终端入口）
> - 执行命令后把输出粘贴回来即可

### 6.1 OpenClash/Clash 日志

```sh
tail -n 300 /tmp/openclash.log
tail -n 200 /tmp/openclash_start.log 2>/dev/null || true
logread | grep -i -E 'openclash|clash' | tail -n 300
```

重点关注：
- `Error:` / `Warning:` / `DNS Hijack` / `TUN` / `proxy` / `dial` / `timeout` 相关行

### 6.2 防火墙计数器：QUIC、DNS 劫持、拒绝/丢包

```sh
# OpenClash 关键链（看 counter 是否快速增长）
for c in openclash openclash_output openclash_mangle openclash_mangle_output openclash_wan_input; do
  echo "===== $c"; nft -a list chain inet fw4 $c; echo
done

# 快速定位：QUIC REJECT / DNS Hijack 的命中计数
nft -a list ruleset | grep -n -E 'OpenClash QUIC REJECT|OpenClash DNS Hijack' | head -n 50

# fw4 转发丢包（ct invalid）是否仍在增长
nft -a list chain inet fw4 accept_to_wan | sed -n '1,160p'
nft -a list chain inet fw4 forward | sed -n '1,160p'
```

解释：
- 如果 `OpenClash QUIC REJECT` 计数持续增长，说明客户端在尝试 QUIC/HTTP3（UDP/443）。一般会回落到 TCP/443，但个别应用可能表现为“连接不稳定/被关闭”。
- 如果 `ct state invalid drop` 仍高速增长，优先排查：回程路径变化、某些策略分流误命中、以及 offload 是否真的关闭。

### 6.3 网络状态与 DNS 实际生效

```sh
ip -br addr
ip route show default
ip -4 rule show

uci -q get network.wan.peerdns
uci -q get network.wan.dns
uci -q get dhcp.@dnsmasq[0].noresolv

logread -e dnsmasq | grep 'using nameserver' | tail -n 20
```

### 6.4 SSH 不可用的快速定位（dropbear）

```sh
netstat -lntp | grep ':22'
logread -e dropbear | tail -n 80
```

如果 `:22` 没有监听，优先执行：

```sh
/etc/init.d/dropbear restart
sleep 1
netstat -lntp | grep ':22'
```

如果 `:22` 在监听但从 LAN 访问仍超时（不像“拒绝连接”那样立刻 RST），说明更可能是防火墙丢弃。可以加一条**显式放行 LAN->路由器 SSH** 的规则：

```sh
uci add firewall rule
uci set firewall.@rule[-1].name='Allow-SSH-LAN'
uci set firewall.@rule[-1].src='lan'
uci set firewall.@rule[-1].proto='tcp'
uci set firewall.@rule[-1].dest_port='22'
uci set firewall.@rule[-1].target='ACCEPT'
uci commit firewall
/etc/init.d/firewall restart
```

如果看到未监听或反复重启，建议先在 LuCI `System -> Startup` 重启 `dropbear`，再继续用 SSH 采集日志。

### 6.5 fw4 提示“option 不支持 / include 路径不存在”的清理（降噪）

你贴的这些信息里：
- `Section ... option 'extra'/'reload' is not supported by fw4`：代表旧版 fw3/iptables 时代的 UCI 字段，fw4/nftables 不再支持（会被忽略）。
- `Section zerotier specifies unreachable path '/var/etc/zerotier.include'`：代表 firewall include 指向的文件不存在，因此该 include 段会被忽略。
- `Automatically including '/usr/share/nftables.d/...'`：这是 fw4 正常行为（nftables 片段自动加载），不需要“修复”。

建议做法是：把不支持的选项删掉；并删除指向不存在文件的 legacy include 段（ZeroTier/Socat 现在通常由 `/usr/share/nftables.d/` 自动注入规则，不需要再靠 `/var/etc/*.include`）。

在路由器上（ttyd/LuCI 终端）执行：

```sh
# 先备份（可回滚）
uci export firewall > /root/firewall.bak.$(date +%F_%H%M%S)

# 1) DockerNAT：删掉 fw4 不支持的 extra（其余字段保留）
uci -q delete firewall.docker_nat.extra

# 2) ZeroTier/Socat：删掉 fw4 不支持的 reload
uci -q delete firewall.zerotier.reload
uci -q delete firewall.socat.reload

# 3) 如果 include 指向的文件不存在，删除该 include 段（推荐）
[ -e /var/etc/zerotier.include ] || uci -q delete firewall.zerotier
[ -e /var/etc/socat.include ] || uci -q delete firewall.socat

uci commit firewall
/etc/init.d/firewall restart
```

验证（可选）：

```sh
uci show firewall | grep -E 'docker_nat|zerotier|socat' || true
logread -e fw4 | tail -n 80
```

当前验证结果（2026-01-02）：
- `fw4 print` 未再输出 `not supported by fw4 / unreachable path / ignoring section` 相关告警（降噪完成）
- `/etc/config/firewall` 内已无 `option extra` / `option reload`
- `/var/etc/zerotier.include` 不存在，但对应 legacy `firewall.zerotier` include 段已移除，不会再触发 fw4 警告

同时观察到（用于 Copilot/Cursor 稳定性排障）：
- `OpenClash QUIC REJECT` 规则计数为 0（说明当前没有大量 UDP/443 被阻断导致回落异常）
- `OpenClash DNS Hijack` 计数正常增长（透明代理场景预期）
- `ct state invalid drop (!fw4: Prevent NAT leakage)` 计数在重启后处于较低水平（持续快速增长则需要回到 2.2/6.2 继续排查）

### 6.6 6 分钟增长率与端到端回归（建议在“实际用 Copilot/Cursor”时做）

为了确认“连接被关闭”是否仍由防火墙/conntrack 引起，建议用 6 分钟窗口看计数器增长率（每 2 分钟采样 1 次，共 3 次）。

当前采样结果（2026-01-02，见本地 `openwrt_audit/2026-01-02_213318/04_counter_growth_6min.txt`）：
- `ct state invalid drop`：148 → 150 → 152（4 分钟净增 +4，属于低水平）
- `OpenClash QUIC REJECT`：0 → 0 → 0（无 QUIC/UDP 443 拦截增长）
- `OpenClash DNS Hijack`：662 → 767 → 851（按预期增长）

同时从路由器本机对外做了快速回归（每个目标 10 次）：
- `https://api.github.com`：ok=10 fail=0
- `https://api.githubcopilot.com`：ok=10 fail=0

后续补采（用于对比不同时间窗口/负载下的稳定性）：

- `openwrt_audit/2026-01-02_221107/04_counter_growth_6min.txt`
- `openwrt_audit/2026-01-02_221107/06_endpoint_regression.txt`

补充：IPv6 路径稳定性

路由器具备 IPv6 地址，但对以下目标的 IPv6 连接测试失败（`curl -6` 直接报错）：
- `https://api.github.com`
- `https://api.githubcopilot.com`

这会导致客户端在拿到 AAAA 记录且“优先 IPv6”时出现连接失败/超时，表现为间歇性 `ERR_CONNECTION_CLOSED`/重试。
已采取的最小化缓解措施：
- `dnsmasq filter_aaaa=1`（对 LAN 返回 AAAA 为无效/过滤，从而强制走 IPv4 路径）

如果你确实需要 IPv6（例如访问 IPv6-only 站点），更正确的方向是：修复 IPv6 线路或让 OpenClash/规则完整支持 IPv6，再关闭 `filter_aaaa`。

如果 Copilot/Cursor 仍出现 `net::ERR_CONNECTION_CLOSED`：
- 优先抓取发生瞬间的 `logread`（`fw4` / `openclash`）与上述计数器 1 次快照
- 再把 Cursor/Copilot 的失败时间点对齐到 `openclash.log`（看是否存在 DNS fallback/上游超时/规则误命中）

### 6.7 SQM/CAKE 安装触发内核崩溃（ramoops 证据与规避）

现象：在执行 `opkg install`（涉及 SQM/CAKE/IFB 相关组件）或对出口应用 CAKE 的过程中，SSH 出现 `Connection reset by peer`，随后路由器 uptime 回到 1~3 分钟，符合“panic 后自动重启”的特征。

已从 `/sys/fs/pstore/*` 抽取到 ramoops 日志，见：

- `openwrt_audit/2026-01-02_215111/05_pstore_reboot_logs.txt`
- `openwrt_audit/2026-01-02_220730/05_pstore_reboot_logs.txt`

如果需要对外提交问题（给固件作者/驱动维护者），可使用模板：

- docs/openwrt_cake_kernel_panic_bug_report.md

关键点（摘要）：

- `kmodloader: loading kernel modules from /etc/modules.d/*` 之后出现模块加载/探测相关日志
- 随后触发 `Unable to handle kernel paging request ...`，最终 `Kernel panic - not syncing: Oops: Fatal exception in interrupt`
- Call trace 指向 `unregister_netdevice_many_notify -> rtnl_dellink`，触发进程为 `ip`（很像某脚本在做 `ip link del ...` 的清理动作）

补充：在另一次崩溃中，模块列表已包含 `sch_cake`，Call trace 发生在软中断/转发路径（`__dev_queue_xmit` 等），这意味着 **CAKE 本身/与当前驱动组合在运行态也不安全**。

建议规避策略：

1) 在根因明确前，不要“一步到位”安装 SQM 全套（尤其是 ingress 相关 IFB/脚本）。
2) 在当前固件/内核上，避免启用 CAKE：不要安装/加载 `kmod-sched-cake`，也不要对 `pppoe-wan` 应用 `tc qdisc ... cake`。
3) 若系统里存在会反复报错且无依赖的 `kmod-inet-diag`，建议卸载以降噪（我们已确认它没有其他包依赖）：

```sh
opkg remove kmod-inet-diag
```

4) 若仍需继续追根因：建议在触发 reboot 后第一时间再次导出 `/sys/fs/pstore/*`（避免被新崩溃覆盖），并同时记录当次安装/脚本执行的确切命令与时间点。

## 5. 结论

你当前的“网络优化”里，**最可能带来 Copilot/Cursor 断连的，是 mwan3 误判导致的策略路由切断，以及（可能叠加的）flow offload 导致的 conntrack invalid 丢包**。

优先按 3.1 执行两项变更，通常就能显著改善开发工具的连接稳定性；之后再做 DNS 与 SQM 的提升。

# 游戏引擎落地执行 TODO（与实施计划 2.0 对齐）

> 本 TODO 列表与 implementation_plan.md（2.0）一一对应，覆盖：workspace 基线修复、安全/unsafe 与密码学风险、async 边界收敛、性能/基准/剖析体系、以及既有 DDD/领域事件/事件溯源设计文档的落地任务。

文档版本：2.0
更新日期：2025-12-17

---

## P0：打通构建/测试基线 + 清除阻塞风险

### P0-1 Workspace 拓扑修复（阻塞项）

- [ ] 将 game_engine 加入 workspace members（根 Cargo.toml）
- [ ] 移除 workspace members 中的 "."（若根目录不是一个 package）
- [ ] 在 workspace.dependencies 补齐 tracing（满足 game_engine 的 tracing.workspace=true）
- [ ] 明确依赖继承策略：尽可能统一使用 workspace 版本（serde/tokio/wgpu/winit 等）
- [ ] 修复 game_engine 对 sibling crate 的 path
   - [ ] game_engine_simd → ../game_engine_simd
   - [ ] game_engine_hardware → ../game_engine_hardware
   - [ ] game_engine_performance → ../game_engine_performance

验收命令：

- [ ] 运行：cargo metadata（确认 packages 含 game_engine）
- [ ] 运行：cargo check -p game_engine

### P0-2 修复 game_engine_performance 测试编译失败

- [ ] 统一指标类型（f32/f64）策略并全局一致
- [ ] 修复 advanced_profiler 相关测试中的类型不匹配
- [ ] 修复 frame_analyzer 测试中对 start_frame().unwrap() 的错误调用
- [ ] 补充回归测试覆盖：
   - [ ] start_frame 行为契约（返回类型/错误策略）
   - [ ] 性能指标统计字段类型一致性

验收命令：

- [ ] 运行：cargo test -p game_engine_performance

### P0-3 收敛 async 边界与阻塞调用（先止血）

盘点与规范：

- [ ] 盘点所有 Handle::current().block_on / block_on / pollster::block_on 调用点（列出清单）
- [ ] 写出并放入 docs 的“同步/异步 API 边界规范”（最短可执行版）

止血改造（分批次）：

- [ ] platform 文件系统同步包装：runtime 内不得直接 block_on（改为 async 或 block_in_place/返回错误）
- [ ] editor 保存/加载：避免在 runtime 内 block_on
- [ ] profiling/storage 同步 API：避免在 runtime 内 block_on
- [ ] scene serialization 同步 API：避免在 runtime 内 block_on

验收命令：

- [ ] 运行：grep -R "Handle::current().block_on" game_engine/src（结果只剩白名单或 0）
- [ ] 运行：cargo test -p game_engine

### P0-4 安全与 unsafe 风险清零（必须）

#### P0-4a 移除窗口生命周期 transmute

- [ ] 替换 core/engine/initialization.rs 中的 std::mem::transmute
- [ ] 设计并实现 renderer 的 window 所有权模型（建议 Arc<Window> 或非 'static renderer）
- [ ] 添加回归测试/示例运行验证：窗口创建、渲染初始化、关闭释放

验收命令：

- [ ] 运行：grep -R "transmute" game_engine/src/core/engine/initialization.rs（结果为 0）

#### P0-4b 替换伪 ECDH 密钥交换

- [ ] 将 network/key_exchange.rs 的 SHA256 伪实现替换为真实密钥交换（建议 X25519 + HKDF）
- [ ] 增加 feature flag：secure_key_exchange 默认开启；insecure_key_exchange 仅用于 demo
- [ ] 单测覆盖：
   - [ ] 双方协商出的 session key 一致
   - [ ] 篡改消息无法通过校验
   - [ ] 重放/重复 nonce 检测（若协议支持）

#### P0-4c AES-GCM Nonce / HMAC Token 审计

- [ ] 明确 nonce 生成策略（同 key 下不复用）并补齐单测
- [ ] 明确 token 版本/过期/轮换策略并文档化

### P0-5 技术债清理（可维护性止血）

- [ ] 删除或迁移 engine.rs.backup（禁止在 src 保留备份文件）
- [ ] 处理 monitoring_legacy：
   - [ ] 选择策略：保留兼容层 + 明确弃用期限 / 迁移并删除 legacy
   - [ ] 统一 game_engine 与 game_engine_performance 的监控实现，避免重复
- [ ] 领域污染治理：将 implementation_plan 等非引擎域从 runtime domain 代码迁出（保留为 docs）

验收命令：

- [ ] 运行：find . -name "*.backup"（结果为 0）
- [ ] 运行：cargo test --workspace

---

## P1：性能与可观测性工程化（基准/剖析可落地）

### P1-1 统一可观测性：tracing + 指标 + profiling 接口

- [ ] 确定 logging 策略：tracing 为主，log 为兼容
- [ ] 关键路径添加 spans：frame loop / render submit / asset load / shader compile / network tick
- [ ] 指标定义（最小集合）：
   - [ ] 帧耗时（avg/p95/p99）
   - [ ] 资源队列长度与等待时间
   - [ ] shader 编译耗时与缓存命中率
   - [ ] network RTT/抖动（如适用）
- [ ] 让 game_engine_performance 提供统一对接层，避免重复实现

验收命令：

- [ ] 运行示例：cargo run -p game_engine --example hello_world（确认输出/trace 可用）

### P1-2 基准体系与回归闸门

- [ ] 确认并修复 game_engine 主 crate benches 可运行（math/ecs/physics/render/pathfinding）
- [ ] game_engine_simd：补齐 benches（或移除 bench 配置误导，改用 tests/criterion）
- [ ] 增加“基准运行脚本”并在文档中固定命令（scripts/）
- [ ] 定义回归阈值（先记录 baseline，再加 gate）：建议 <5%

### P1-3 异步资源/着色器队列优化（在可观测性之后）

- [ ] coroutine_loader：减少 sleep/poll，改为通知驱动；记录队列指标
- [ ] shader_async：记录编译队列与超时重试指标；对 spawn_blocking 并发度做配置化
- [ ] 资源加载/编译在关键帧路径的预算约束（预算超限报警）

---

## P2：DDD/领域事件/事件溯源（按既有设计落地）

### P2-1 聚合根边界与不变式

- [ ] 审查并修正 Scene 聚合根边界
- [ ] 审查并修正 GameEntity 聚合根边界
- [ ] 审查并修正 RenderScene 聚合根边界
- [ ] 审查并修正 PhysicsWorld 聚合根边界
- [ ] 审查并修正 AudioSource 聚合根边界
- [ ] 为每个聚合根补齐不变式校验 + 单元测试

### P2-2 错误处理与锁安全（safe_lock 替换）

- [ ] 全仓库替换 lock().unwrap() 等 panic 路径（仅保留明确白名单）
- [ ] 锁污染恢复策略落地 + 单测

### P2-3 领域事件系统（按设计文档）

- [ ] 实现类型安全的事件注册系统（registry + factory + 自动注册）
- [ ] 实现安全事件总线：最小持锁、支持批量、支持并行分发
- [ ] 聚合根事件集成：未提交事件队列、mark committed、版本字段
- [ ] 事件序列化/反序列化路径可用（用于持久化与重放）

### P2-4 事件溯源系统（按 improvement plan 阶段化落地）

- [ ] 命令完善：CreateEntityCommand
- [ ] 命令完善：DeleteEntityCommand
- [ ] 实现 UpdateEntityCommand
- [ ] 事件存储增强：批量、分页、版本控制
- [ ] 事件查询接口：过滤、排序、分页
- [ ] 事件重放：注册表集成 + 从快照恢复 + 错误处理
- [ ] 快照机制：触发策略（数量/时间/大小）、验证与恢复
- [ ] 性能监控与测试套件：单元/集成/性能测试

### P2-5 审计日志与版本控制

- [ ] 聚合版本控制：乐观锁与冲突检测
- [ ] 审计日志：结构、归档/清理、完整性保证（如需加密则按需求启用）

---

## 状态标记

- [ ] Todo
- [-] In Progress
- [x] Done

## 统一验收闸门（每个阶段都要过）

- [ ] cargo fmt --check
- [ ] cargo clippy --workspace --all-targets
- [ ] cargo test --workspace

- 性能相关改动需要性能测试验证

### 测试要求
- 单元测试覆盖率不低于90%
- 集成测试覆盖主要业务流程
- 性能测试确保无回归
- 压力测试验证系统稳定性

## 文档要求

每个任务完成后需要提供：
1. 设计文档
2. 实现文档
3. API文档
4. 使用指南
5. 测试报告

## 沟通计划

### 每周进度会议
- 时间: 每周五下午
- 参与者: 所有任务负责人
- 内容: 进度汇报、风险讨论、问题解决

### 里程碑评审
- 每个里程碑完成后进行评审
- 参与者: 项目团队、利益相关者
- 内容: 成果展示、质量评估、下阶段规划